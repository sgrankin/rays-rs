use cgmath::*;

use crate::geom::*;
use crate::prims::*;

pub struct Aggregate<S: BaseFloat> {
    pub prims: Vec<Box<dyn Primitive<S> + Sync>>,
}

impl<S: BaseFloat> Primitive<S> for Aggregate<S> {
    fn intersect(&self, r: Ray3<S>) -> Option<SurfaceInteraction<'_, S>> {
        // TODO: rewrite this faster. for loop & two if statements
        self.prims.iter().fold(None, |best, p| match (best, p.intersect(r)) {
            (None, int) => int,
            (Some(best), None) => Some(best),
            (Some(best), Some(int)) => {
                let t_int = (int.point - r.origin).magnitude();
                let t_best = (best.point - r.origin).magnitude();
                Some(if t_int < t_best { int } else { best })
            }
        })
    }
}
