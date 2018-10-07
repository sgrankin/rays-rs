use cgmath::*;
use crate::material::*;

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sphere<S: BaseFloat> {
    pub center: Point3<S>,
    pub radius: S,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ray<S, P, V>
where
    S: BaseNum,
    V: VectorSpace<Scalar = S>,
    P: EuclideanSpace<Scalar = S, Diff = V>,
{
    pub origin: P,
    pub direction: V,
}
// todo ray::new

pub type Ray3<S> = Ray<S, Point3<S>, Vector3<S>>;

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Contact<P: EuclideanSpace> {
    pub point: P,
    pub normal: P::Diff,
}

pub trait Intersectable<S>
where
    S: BaseNum,
{
    fn intersection(&self, _: &Ray3<S>) -> Option<Contact<Point3<S>>>;
}

impl<S> Intersectable<S> for Sphere<S>
where
    S: BaseFloat,
{
    fn intersection(&self, r: &Ray3<S>) -> Option<Contact<Point3<S>>> {
        // TODO credit collision crate for the source of the formula; rewrite for readability: this can be explained via geometry, but needs better names and a block comment.
        let s = self;
        let l = s.center - r.origin;
        let tca = l.dot(r.direction);
        if tca < S::zero() {
            return None;
        }
        let d2 = l.dot(l) - tca * tca;
        if d2 > s.radius * s.radius {
            return None;
        }
        let thc = (s.radius * s.radius - d2).sqrt();
        let p = r.origin + r.direction * (tca - thc);
        Some(Contact { point: p, normal: (p - s.center).normalize() })
    }
}

impl<'a, S: 'a, T> Intersectable<S> for &'a [T]
where
    S: BaseFloat,
    T: Intersectable<S>,
{
    fn intersection(&self, r: &Ray3<S>) -> Option<Contact<Point3<S>>> {
        self.iter().fold(None, |res, s| match (res, s.intersection(r)) {
            (Some(ref c1), Some(ref c2))
                if ((c1.point - r.origin).magnitude() < (c2.point - r.origin).magnitude()) =>
            {
                Some(*c1)
            }
            (None, res) => res,
            (res, _) => res,
        })
    }
}
