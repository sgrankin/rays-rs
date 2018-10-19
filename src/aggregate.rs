use crate::geom::*;
use crate::prims::*;
use crate::types::*;

pub struct Aggregate {
    pub prims: Vec<Box<dyn Primitive + Sync>>,
}

impl Primitive for Aggregate {
    fn intersect(&self, r: Ray3f) -> Option<SurfaceInteraction<'_>> {
        // TODO: rewrite this faster. for loop & two if statements
        self.prims.iter().fold(None, |best, p| match (best, p.intersect(r)) {
            (None, int) => int,
            (Some(best), None) => Some(best),
            (Some(best), Some(int)) => {
                let t_int = (int.point - r.origin).magnitude();
                let t_best = (best.point - r.origin).magnitude();
                Some(iff!(t_int < t_best, int, best))
            }
        })
    }
}
