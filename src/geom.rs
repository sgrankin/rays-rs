use cgmath::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Ray<S, P, V>
where
    S: BaseNum,
    V: VectorSpace<Scalar = S>,
    P: EuclideanSpace<Scalar = S, Diff = V>,
{
    pub origin: P,
    pub direction: V,
}
pub type Ray3<S> = Ray<S, Point3<S>, Vector3<S>>;
impl<S: BaseNum> Ray3<S> {
    pub fn new(origin: Point3<S>, direction: Vector3<S>) -> Ray3<S> {
        Ray3 { origin, direction }
    }
}

pub trait Shape<S: BaseFloat> {
    fn intersect(&self, _: Ray3<S>) -> Option<(Point3<S>, Vector3<S>)>;
}

pub struct Sphere<S: BaseFloat> {
    pub center: Point3<S>,
    pub radius: S,
}

impl<S: BaseFloat> Shape<S> for Sphere<S> {
    fn intersect(&self, r: Ray3<S>) -> Option<(Point3<S>, Vector3<S>)> {
        // Due to floating point errors, advance ray up to avoid re-intersecting.
        // NOTE: this is insufficient for very oblique rays; see PBRT error-tracking for a better solution.
        let r = Ray3::new(r.origin + r.direction * S::from(0.000_001).unwrap(), r.direction);
        let r2 = self.radius * self.radius;
        let norm_dir = if self.radius > S::zero() { S::one() } else { -S::one() };
        let l = self.center - r.origin;
        let p = if l.magnitude2() >= r2 {
            // origin is outside or on the surface
            let tca = l.dot(r.direction);
            if tca <= S::zero() {
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
