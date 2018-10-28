use crate::geom::*;
use crate::types::*;

pub trait Shape: Sync + Send {
    // TODO: &Ray3f to reduce possible copies
    fn intersect(&self, _: Ray3f) -> Option<(Point3f, Vector3f)>;
    fn bounding_box(&self) -> Option<AABB>;
}

pub struct Sphere {
    pub center: Point3f,
    pub radius: Float,
}

impl Shape for Sphere {
    fn intersect(&self, r: Ray3f) -> Option<(Point3f, Vector3f)> {
        // Due to floating point errors, advance ray up to avoid re-intersecting.
        // NOTE: this is insufficient for very oblique rays; see PBRT error-tracking for a better solution.
        let r = Ray3f::new(r.origin + r.direction * (0.000_001), r.direction);
        let r2 = self.radius * self.radius;
        let norm_dir = if self.radius > 0.0 { 1.0 } else { -1.0 };
        let l = self.center - r.origin;
        let p = if l.magnitude2() >= r2 {
            // origin is outside or on the surface
            let tca = l.dot(r.direction);
            if tca <= 0.0 {
                // heading away from the sphere
                return None;
            }
            let d2 = l.magnitude2() - tca * tca;
            if d2 >= r2 {
                return None;
            }
            let thc = (r2 - d2).sqrt();
            r.origin
                + r.direction
                    * (if tca != thc {
                        tca - thc
                    } else {
                        // we're on the surface going through the sphere, so we need to return the point on the other side.
                        // technically tca=thc;
                        tca + thc
                    })
        } else {
            // origin is in the sphere
            let tca = l.dot(r.direction);
            // if tca > 0: heading somewhat towards the center
            // else, heading away from center
            // In either case, correction is tca + thc; in the else case it's (thc - (-tca))
            let d2 = l.magnitude2() - tca * tca;
            let thc = (r2 - d2).sqrt();
            r.origin + r.direction * (tca + thc)
        };
        Some((p, (p - self.center).normalize() * norm_dir))
    }
    fn bounding_box(&self) -> Option<AABB> {
        let rad = Vector3f::from_value(self.radius);
        Some(AABB::new(self.center - rad, self.center + rad))
    }
}

#[derive(Copy, Clone)]
pub struct AABB {
    pub min: Point3f,
    pub max: Point3f,
}

use std::fmt;
impl fmt::Debug for AABB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "vol={:?} min={:?} max={:?}",
            (self.max[0] - self.min[0]) * (self.max[1] - self.min[1]) * (self.max[2] - self.min[2]),
            self.min,
            self.max,
        )
    }
}

impl AABB {
    pub fn empty() -> AABB {
        AABB { min: Point3f::from_value(FLOAT_MAX), max: Point3f::from_value(FLOAT_MIN) }
    }
    pub fn new(min: Point3f, max: Point3f) -> AABB {
        AABB { min, max }
    }

    pub fn intersect(&self, r: Ray3f) -> bool {
        let mut t_min = 0.000_001;
        let mut t_max = std::f64::MAX;

        for i in 0..3 {
            let mut t0 = (self.min[i] - r.origin[i]) * r.inv_d[i];
            let mut t1 = (self.max[i] - r.origin[i]) * r.inv_d[i];
            if r.inv_d[i] < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            t_min = iff!(t0 > t_min, t0, t_min);
            t_max = iff!(t1 < t_max, t1, t_max);
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub fn union(&self, other: &AABB) -> AABB {
        AABB {
            min: Point3f::new(
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ),
            max: Point3f::new(
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ),
        }
    }

    pub fn union_p(&self, other: &Point3f) -> AABB {
        AABB {
            min: Point3f::new(
                self.min[0].min(other[0]),
                self.min[1].min(other[1]),
                self.min[2].min(other[2]),
            ),
            max: Point3f::new(
                self.max[0].max(other[0]),
                self.max[1].max(other[1]),
                self.max[2].max(other[2]),
            ),
        }
    }

    pub fn offset_p(&self, p: &Point3f) -> Vector3f {
        let mut res = p - self.min;
        for i in 0..3 {
            if self.max[i] > self.min[i] {
                res[i] /= self.max[i] - self.min[i]
            };
        }
        res
    }

    pub fn diagonal(&self) -> Vector3f {
        self.max - self.min
    }

    pub fn surface_area(&self) -> Float {
        let d = self.diagonal();
        2.0 * (d.x * d.y + d.y * d.z + d.x * d.z)
    }

    pub fn center(&self) -> Point3f {
        Point3f::new(
            // TODO: this should be a vector op
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
            (self.min[2] + self.max[2]) / 2.0,
        )
    }
}
