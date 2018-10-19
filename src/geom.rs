use crate::types::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Ray<S, P, V>
where
    S: cgmath::BaseNum,
    V: cgmath::VectorSpace<Scalar = S>,
    P: cgmath::EuclideanSpace<Scalar = S, Diff = V>,
{
    pub origin: P,
    pub direction: V,
}

impl<S, P, V> Ray<S, P, V>
where
    S: cgmath::BaseFloat,
    V: cgmath::VectorSpace<Scalar = S> + cgmath::InnerSpace<Scalar = S>,
    P: cgmath::EuclideanSpace<Scalar = S, Diff = V>,
{
    pub fn new(origin: P, direction: V) -> Self {
        Ray { origin, direction: direction.normalize() }
    }
}

pub type Ray3f = Ray<Float, Point3f, Vector3f>;
