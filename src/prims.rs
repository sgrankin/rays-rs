use super::geom::*;
use super::material::*;

use cgmath::*;
use std::marker::PhantomData;

pub trait Primitive<S: BaseFloat> {
    fn intersect(&self, _: Ray3<S>) -> Option<SurfaceInteraction<'_, S>>;
}

pub struct SurfaceInteraction<'a, S: BaseFloat + 'a> {
    pub prim: &'a dyn Primitive<S>,
    pub point: Point3<S>,
    pub normal: Vector3<S>,
    pub material: &'a dyn Material<S>,
}

pub struct ShapePrimitive<S: BaseFloat, Sh: Shape<S>, M: Material<S>> {
    pub shape: Sh,
    pub material: M,
    __: PhantomData<fn(_: ()) -> (S)>,
}
impl<S: BaseFloat, Sh: Shape<S>, M: Material<S>> ShapePrimitive<S, Sh, M> {
    pub fn new(shape: Sh, material: M) -> ShapePrimitive<S, Sh, M> {
        ShapePrimitive { shape, material, __: PhantomData }
    }
}

impl<S: BaseFloat, Sh: Shape<S>, M: Material<S>> Primitive<S> for ShapePrimitive<S, Sh, M> {
    fn intersect(&self, r: Ray3<S>) -> Option<SurfaceInteraction<'_, S>> {
        self.shape.intersect(r).map(|(point, normal)| SurfaceInteraction {
            point,
            normal,
            prim: self,
            material: &self.material,
        })
    }
}
