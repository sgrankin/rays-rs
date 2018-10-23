use crate::geom::*;
use crate::prims::*;
use crate::shape::*;
// use crate::types::*;

pub struct Aggregate {
    _prims: Box<Vec<Box<dyn Primitive>>>,
    bvh: Box<BVH>,
}

impl<'a> Aggregate {
    pub fn new(prims: Vec<Box<dyn Primitive>>) -> Self {
        unsafe {
            let prims = Box::new(prims);
            let bvh = BVH::new(&prims);
            Aggregate { _prims: prims, bvh }
        }
    }
}

impl<'a> Primitive for Aggregate {
    fn intersect(&self, r: Ray3f) -> Option<SurfaceInteraction<'_>> {
        self.bvh.intersect(r)
    }
    fn bounding_box(&self) -> Option<AABB> {
        self.bvh.bounding_box()
    }
}
