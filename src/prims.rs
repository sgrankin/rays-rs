use std::cmp::Ordering;
use std::marker::PhantomData;

use crate::geom::*;
use crate::material::*;
use crate::shape::*;
use crate::types::*;

pub trait Primitive: Sync + Send {
    fn intersect(&self, _: Ray3f) -> Option<SurfaceInteraction<'_>>;
    fn bounding_box(&self) -> Option<AABB>;
}

pub struct SurfaceInteraction<'a> {
    pub prim: &'a dyn Primitive,
    pub point: Point3f,
    pub normal: Vector3f,
    pub material: &'a dyn Material,
    pub t: Float,
}

pub struct ShapePrimitive<S: Shape, M: Material> {
    pub shape: S,
    pub material: M,
    aabb: Option<AABB>,
    __: PhantomData<fn(_: ()) -> Float>,
}
impl<S: Shape, M: Material> ShapePrimitive<S, M> {
    pub fn new(shape: S, material: M) -> ShapePrimitive<S, M> {
        let aabb = shape.bounding_box();
        ShapePrimitive { shape, material, aabb, __: PhantomData }
    }
}

impl<S: Shape, M: Material> Primitive for ShapePrimitive<S, M> {
    fn intersect(&self, r: Ray3f) -> Option<SurfaceInteraction<'_>> {
        if !self.aabb.map(|b| b.intersect(r)).unwrap_or(true) {
            return None;
        }
        self.shape.intersect(r).map(|(point, normal)| SurfaceInteraction {
            point,
            normal,
            prim: self,
            material: &self.material,
            t: (point - r.origin).dot(r.direction),
        })
    }
    fn bounding_box(&self) -> Option<AABB> {
        self.aabb
    }
}

pub enum BVH {
    Leaf { aabb: AABB, prims: Vec<*const dyn Primitive> },
    Node { aabb: AABB, left: Box<BVH>, right: Box<BVH> },
}

unsafe impl Sync for BVH {}
unsafe impl Send for BVH {}

#[derive(Copy, Clone, Debug)]
struct PrimitiveInfo {
    prim: *const dyn Primitive,
    aabb: AABB,
    center: Point3f,
}

#[derive(Clone, Debug)]
struct SplitResult {
    cost: Float,
    left: Vec<PrimitiveInfo>,
    left_aabb: AABB,
    right: Vec<PrimitiveInfo>,
    right_aabb: AABB,
    bounds: AABB,
}

#[derive(Copy, Clone, Debug)]
struct BucketInfo {
    count: usize,
    aabb: AABB,
}

impl BVH {
    const OBJECT_SPLIT_BUCKETS: usize = 16;
    const MAX_PRIMITIVES_PER_NODE: usize = 4;

    unsafe fn new_leaf(aabb: AABB, prims: Vec<PrimitiveInfo>) -> Box<Self> {
        Box::new(BVH::Leaf { aabb, prims: prims.iter().map(|pi| pi.prim).collect() })
    }

    fn fold_aabb(aabs: &[AABB]) -> AABB {
        aabs.iter().fold(AABB::empty(), |r, b| r.union(b))
    }

    unsafe fn new_sorted(aabb: AABB, prims: Vec<PrimitiveInfo>) -> Box<Self> {
        if prims.len() == 0 {
            unimplemented!("empty BVH::new input")
        } else if prims.len() <= Self::MAX_PRIMITIVES_PER_NODE {
            return Self::new_leaf(aabb, prims);
        } else {
            let mut splits: Vec<SplitResult> =
                (0..3).map(|dim| Self::object_split(prims.clone(), dim)).flatten().collect();
            if splits.len() == 0 {
                return Self::new_leaf(aabb, prims);
            }
            let mut best = 0;
            for i in 1..splits.len() {
                if splits[i].cost < splits[best].cost {
                    best = i
                }
            }

            let split = splits.swap_remove(best);
            if split.cost >= prims.len() as Float {
                // BVH cost same as just checking everything, so don't bother with a node.
                return Self::new_leaf(aabb, prims);
            }
            let left = BVH::new_sorted(split.left_aabb, split.left);
            let right = BVH::new_sorted(split.right_aabb, split.right);
            Box::new(BVH::Node { aabb, left, right })
        }
    }

    unsafe fn object_split(mut prims: Vec<PrimitiveInfo>, dim: usize) -> Option<SplitResult> {
        let bounds = prims.iter().fold(AABB::empty(), |res, pi| res.union(&pi.aabb));

        let center_bounds = prims.iter().fold(AABB::empty(), |res, pi| res.union_p(&pi.center));
        if center_bounds.min[dim] == center_bounds.max[dim] {
            return None;
        }
        prims.sort_unstable_by(|left, right| {
            left.center[dim].partial_cmp(&right.center[dim]).unwrap_or(Ordering::Less)
        });

        let c_buckets = Self::OBJECT_SPLIT_BUCKETS;
        let pi_bucket = |pi: &PrimitiveInfo| {
            let i = (center_bounds.offset_p(&pi.center)[dim] * (c_buckets as Float)) as usize;
            clamp!(i, 0, c_buckets - 1)
        };

        let mut buckets = vec![BucketInfo { count: 0, aabb: AABB::empty() }; c_buckets];
        for pi in prims.iter() {
            let i = pi_bucket(pi);
            buckets[i].count += 1;
            buckets[i].aabb = buckets[i].aabb.union(&pi.aabb);
        }

        // cost for splitting at this bucket
        let mut cost = vec![0.0 as Float; c_buckets - 1];
        for i in 0..cost.len() {
            let aabb_left = buckets[..i + 1].iter().fold(AABB::empty(), |r, b| r.union(&b.aabb));
            let aabb_right = buckets[i + 1..].iter().fold(AABB::empty(), |r, b| r.union(&b.aabb));

            let count_left = buckets[..i + 1].iter().fold(0, |r, b| r + b.count);
            let count_right = buckets[i + 1..].iter().fold(0, |r, b| r + b.count);

            cost[i] = 1.0
                + (count_left as Float * aabb_left.surface_area()
                    + count_right as Float * aabb_right.surface_area())
                    / bounds.surface_area()
        }

        let mut min_cost = cost[0];
        let mut min_cost_bucket = 0;
        for i in 1..c_buckets - 1 {
            if cost[i] < min_cost {
                min_cost = cost[i];
                min_cost_bucket = i;
            }
        }

        let (pi_left, pi_right): (Vec<PrimitiveInfo>, Vec<PrimitiveInfo>) =
            prims.iter().partition(|pi| pi_bucket(pi) <= min_cost_bucket);
        let left_aabbs: Vec<AABB> = pi_left.iter().map(|pi| (pi.aabb)).collect();
        let left_aabb = Self::fold_aabb(&left_aabbs);
        let right_aabbs: Vec<AABB> = pi_right.iter().map(|pi| pi.aabb).collect();
        let right_aabb = Self::fold_aabb(&right_aabbs);

        Some(SplitResult {
            cost: min_cost,
            bounds,
            left: pi_left,
            left_aabb,
            right: pi_right,
            right_aabb,
        })
    }

    pub unsafe fn new(prims: &[Box<dyn Primitive>]) -> Box<Self> {
        let prims: Vec<PrimitiveInfo> = prims
            .iter()
            .map(|x| {
                let prim = &**x as *const dyn Primitive;
                let aabb = (*prim)
                    .bounding_box()
                    .unwrap_or_else(|| unimplemented!("No bounding box in BVH::new"));
                let center = aabb.center();
                PrimitiveInfo { prim, aabb, center }
            })
            .collect();
        let aabb = Self::fold_aabb(&prims.iter().map(|pi| pi.aabb).collect::<Vec<AABB>>());
        Self::new_sorted(aabb, prims)
    }
}

impl Primitive for BVH {
    fn intersect(&self, r: Ray3f) -> Option<SurfaceInteraction<'_>> {
        match self {
            BVH::Leaf { aabb, prims } if aabb.intersect(r) => {
                prims.iter().fold(None, |hit, prim| match (hit, unsafe { &**prim }.intersect(r)) {
                    (None, hit) | (hit, None) => hit,
                    (Some(best), Some(hit)) => Some(iff!(best.t < hit.t, best, hit)),
                })
            }
            BVH::Node { aabb, left, right } if aabb.intersect(r) => {
                match (left.intersect(r), right.intersect(r)) {
                    (None, None) => None,
                    (hit, None) | (None, hit) => hit,
                    (Some(hl), Some(hr)) => Some(iff!(hl.t < hr.t, hl, hr)),
                }
            }
            _ => None,
        }
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(*match self {
            BVH::Leaf { aabb, prims: _ } => aabb,
            BVH::Node { aabb, left: _, right: _ } => aabb,
        })
    }
}
