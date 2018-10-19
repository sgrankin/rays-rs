use crate::geom::*;
use crate::types::*;

pub trait Shape {
    fn intersect(&self, _: Ray3f) -> Option<(Point3f, Vector3f)>;
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
}
