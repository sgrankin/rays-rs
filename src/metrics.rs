// Mostly copied from [1] and adjusted to write histograms in quantile format as described in [2]
// [1]: https://github.com/linkerd/tacho/blob/master/src/prometheus.rs
// [2]: https://github.com/prometheus/docs/blob/master/content/docs/instrumenting/exposition_formats.md

use hdrsample::Histogram;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;
use tacho::Report;
type Labels = BTreeMap<&'static str, String>;

pub fn string(report: &Report) -> Result<String, fmt::Error> {
    let mut out = String::with_capacity(8 * 1024);
    write(&mut out, report)?;
    Ok(out)
}

pub fn write<W>(out: &mut W, report: &Report) -> fmt::Result
where
    W: fmt::Write,
{
    for (k, v) in report.counters() {
        let name = FmtName::new(k.prefix(), k.name());
        write_metric(out, &name, &k.labels().into(), v)?;
    }

    for (k, v) in report.gauges() {
        let name = FmtName::new(k.prefix(), k.name());
        write_metric(out, &name, &k.labels().into(), v)?;
    }

    for (k, h) in report.stats() {
        let name = FmtName::new(k.prefix(), k.name());
        let labels = k.labels().into();
        let count = h.count();
        write_metric(out, &format_args!("{}_{}", name, "count"), &labels, &count)?;
        if count > 0 {
            write_histogram(out, &name, &labels, h.histogram())?;
            write_metric(out, &format_args!("{}_{}", name, "min"), &labels, &h.min())?;
            write_metric(out, &format_args!("{}_{}", name, "max"), &labels, &h.max())?;
            write_metric(out, &format_args!("{}_{}", name, "sum"), &labels, &h.sum())?;
        }
    }

    Ok(())
}

fn write_histogram<N: fmt::Display, W: fmt::Write>(
    out: &mut W, name: &N, labels: &FmtLabels, h: &Histogram<usize>,
) -> fmt::Result {
    for quantile in [0.5, 0.9, 0.99, 0.999, 0.9999].iter() {
        write_bucket(out, name, labels, *quantile, h.value_at_percentile(*quantile * 100.0))?;
    }
    Ok(())
}

fn write_bucket<N: fmt::Display, W: fmt::Write>(
    out: &mut W, name: &N, labels: &FmtLabels, quantile: f64, value: u64,
) -> fmt::Result {
    write_metric(
        out,
        &format_args!("{}", name),
        &labels.with_extra("quantile", format_args!("{}", quantile)),
        &value,
    )
}

fn write_metric<W: fmt::Write, N: fmt::Display, V: fmt::Display>(
    out: &mut W, name: &N, labels: &FmtLabels, v: &V,
) -> fmt::Result {
    writeln!(out, "{}{} {}", name, labels, v)
}

fn write_prefix<W: fmt::Write>(out: &mut W, prefix: Arc<tacho::Prefix>) -> fmt::Result {
    if let tacho::Prefix::Node { ref prefix, value } = *prefix {
        write_prefix(out, prefix.clone())?;
        write!(out, "{}:", value)?;
    }
    Ok(())
}

/// Formats a prefixed name.
struct FmtName<'a> {
    prefix: &'a Arc<tacho::Prefix>,
    name: &'a str,
}

impl<'a> FmtName<'a> {
    fn new(prefix: &'a Arc<tacho::Prefix>, name: &'a str) -> Self {
        FmtName { prefix, name }
    }
}

impl<'a> fmt::Display for FmtName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_prefix(f, self.prefix.clone())?;
        write!(f, "{}", self.name)?;
        Ok(())
    }
}

impl<'a> From<&'a Labels> for FmtLabels<'a> {
    fn from(base: &'a Labels) -> Self {
        FmtLabels { base, extra: None }
    }
}

/// Formats labels.
struct FmtLabels<'a> {
    /// Labels from the original Key.
    base: &'a Labels,
    /// An export-specific label (for buckets, etc).
    extra: Option<(&'static str, fmt::Arguments<'a>)>,
}

impl<'a> FmtLabels<'a> {
    fn is_empty(&self) -> bool {
        self.base.is_empty() && self.extra.is_none()
    }

    /// Creates a new FmtLabels sharing a common `base` with a new copy of `extra`.
    fn with_extra(&'a self, k: &'static str, v: fmt::Arguments<'a>) -> FmtLabels<'a> {
        FmtLabels { base: self.base, extra: Some((k, v)) }
    }
}

impl<'a> fmt::Display for FmtLabels<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        let mut first = true;
        write!(f, "{{")?;
        if let Some((k, v)) = self.extra {
            write!(f, "{}=\"{}\"", k, v)?;
            first = false;
        }
        for (k, v) in self.base.iter() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}=\"{}\"", k, v)?;
            first = false;
        }
        write!(f, "}}")?;

        Ok(())
    }
}
