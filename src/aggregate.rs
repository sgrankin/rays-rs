use crate::geom::*;
use crate::prims::*;
use crate::shape::*;
// use crate::types::*;

pub struct Aggregate {
    bvh: Box<BVH>,
}

impl<'a> Aggregate {
    pub fn new(prims: Vec<Box<dyn Primitive>>) -> Self {
        Aggregate { bvh: BVH::new(prims) }
    }
}

impl<'a> Primitive for Aggregate {
    fn intersect(&self, r: Ray3f) -> Option<SurfaceInteraction<'_>> {
        self.bvh.intersect(r)
        // // TODO: rewrite this faster. for loop & two if statements
        // self.prims.iter().fold(None, |best, p| match (best, p.intersect(r)) {
        //     (None, int) => int,
        //     (Some(best), None) => Some(best),
        //     (Some(best), Some(int)) => Some(iff!(int.t < best.t, int, best)),
        // })
    }
    fn bounding_box(&self) -> Option<AABB> {
        self.bvh.bounding_box()
        // self.prims.iter().fold(None, |res, p| match (res, p.bounding_box()) {
        //     (None, b) => b,
        //     (Some(b), None) => Some(b),
        //     (Some(b1), Some(b2)) => Some(b1.union(b2)),
        // })
    }
}
