use crate::types::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Ray<S, P, V>
where
    S: cgmath::BaseNum,
    V: cgmath::VectorSpace<Scalar = S> + cgmath::Array,
    P: cgmath::EuclideanSpace<Scalar = S, Diff = V>,
{
    pub origin: P,
    pub direction: V,
    pub inv_d: V,
}

impl<S, P, V> Ray<S, P, V>
where
    S: cgmath::BaseFloat,
    V: cgmath::VectorSpace<Scalar = S>
        + cgmath::InnerSpace<Scalar = S>
        + cgmath::Array<Element = S>
        + cgmath::ElementWise,
    P: cgmath::EuclideanSpace<Scalar = S, Diff = V>,
{
    pub fn new(origin: P, direction: V) -> Self {
        let direction = direction.normalize();
        Ray {
            origin,
            direction: direction,
            inv_d: V::from_value(S::one()).div_element_wise(direction),
        }
    }
}

pub type Ray3f = Ray<Float, Point3f, Vector3f>;
