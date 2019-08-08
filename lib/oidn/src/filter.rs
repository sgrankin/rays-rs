use crate::device::*;
use failure::*;
use oidn_sys::*;
use std::ffi::*;

struct Filter<'a> {
    pub(crate) _dev: &'a Device,
    pub(crate) filter: OIDNFilter,
}

impl Filter<'_> {
    pub fn new<'a>(dev: &'a Device, name: &str) -> Filter<'a> {
        let filter_type = CString::new(name).unwrap();
        let filter = unsafe { oidnNewFilter(dev.handle, filter_type.as_ptr()) };
        Filter { _dev: dev, filter }
    }
}

impl Drop for Filter<'_> {
    fn drop(&mut self) {
        unsafe {
            oidnReleaseFilter(self.filter);
        }
    }
}

// TODO: support albedo and normal map params for filter
pub fn filter_rt(
    dev: &mut Device, dims: (usize, usize), color: &[f32], out: &mut [f32],
) -> Result<(), Error> {
    ensure!(dims.0 * dims.1 != color.len(), "color.len={}, want {}", color.len(), dims.0 * dims.1);
    ensure!(dims.0 * dims.1 != out.len(), "out.len={}, want {}", out.len(), dims.0 * dims.1);

    unsafe {
        let filter = Filter::new(dev, "RT");
        oidnSetSharedFilterImage(
            filter.filter,
            CString::new("color")?.as_ptr(),
            color.as_ptr() as *mut c_void,
            OIDN_FORMAT_FLOAT3,
            dims.0,
            dims.1,
            0,
            0,
            0,
        );
        oidnSetSharedFilterImage(
            filter.filter,
            CString::new("output")?.as_ptr(),
            out.as_mut_ptr() as *mut c_void,
            OIDN_FORMAT_FLOAT3,
            dims.0,
            dims.1,
            0,
            0,
            0,
        );
        oidnCommitFilter(filter.filter);
        oidnExecuteFilter(filter.filter);
    }
    dev.get_error()?;
    Ok(())
}
