use std::marker::PhantomData;

use crate::geom::*;
use crate::material::*;
use crate::shape::*;
use crate::types::*;

pub trait Primitive: Sync + Send {
    fn intersect(&self, _: Ray3f) -> Option<SurfaceInteraction<'_>>;
}

pub struct SurfaceInteraction<'a> {
    pub prim: &'a dyn Primitive,
    pub point: Point3f,
    pub normal: Vector3f,
    pub material: &'a dyn Material,
}

pub struct ShapePrimitive<S: Shape, M: Material> {
    pub shape: S,
    pub material: M,
    __: PhantomData<fn(_: ()) -> (Float)>,
}
impl<S: Shape, M: Material> ShapePrimitive<S, M> {
    pub fn new(shape: S, material: M) -> ShapePrimitive<S, M> {
        ShapePrimitive { shape, material, __: PhantomData }
    }
}

impl<S: Shape, M: Material> Primitive for ShapePrimitive<S, M> {
    fn intersect(&self, r: Ray3f) -> Option<SurfaceInteraction<'_>> {
        self.shape.intersect(r).map(|(point, normal)| SurfaceInteraction {
            point,
            normal,
            prim: self,
            material: &self.material,
        })
    }
}
