use num;
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
            t: (point - r.origin).dot(r.direction),
        })
    }
    fn bounding_box(&self) -> Option<AABB> {
        self.shape.bounding_box()
    }
}

// morton ordering for doubles; see M.Conor, P.Kumar
fn msdb(a: i16, b: i16) -> u64 {
    let mut i = 0;
    let mut n = a ^ b;
    while n > 0 {
        i += 1;
        n >>= 1;
    }
    i
}

fn xor_msb(a: Float, b: Float) -> u64 {
    let (a_exp, a_mant, _) = num::Float::integer_decode(a);
    let (b_exp, b_mant, _) = num::Float::integer_decode(b);
    if a_exp == b_exp {
        let z = msdb(a_mant, b_mant);
        return a_exp - z;
    }
    iff!(a_exp > b_exp, a_exp, b_exp)
}

fn point_compare(p: Point3f, q: Point3f) -> Ordering {
    let mut x = 0;
    let mut dim = 0;
    for d in 0..3 {
        let y = xor_msb(p[d], q[d]);
        if x < y {
            x = y;
            dim = d;
        }
    }
    p[dim].partial_cmp(&q[dim]).unwrap_or_else(|| unimplemented!("NaN in point_compare"))
}

pub enum BVH {
    Leaf { box_: AABB, prim: Box<dyn Primitive> },
    Node { box_: AABB, left: Box<BVH>, right: Box<BVH> },
}

impl BVH {
    fn new_sorted(mut prims: Vec<Box<dyn Primitive>>) -> Box<Self> {
        if prims.len() == 0 {
            unimplemented!("empty BVH::new input")
        } else if prims.len() == 1 {
            let prim = prims.pop().unwrap();
            Box::new(BVH::Leaf {
                box_: prim
                    .bounding_box()
                    .unwrap_or_else(|| unimplemented!("No bounding box in BVH::new")),
                prim: prim,
            })
        } else {
            let split = prims.len() / 2;
            let n = prims.len();
            let right = prims.split_off(split);
            let left = BVH::new_sorted(prims);
            let right = BVH::new_sorted(right);
            let box_ = left.bounding_box().unwrap().union(right.bounding_box().unwrap());
            println!("bvh node with n={:?} box={:?}", n, box_);
            Box::new(BVH::Node { box_, left: left, right: right })
        }
    }
    pub fn new(mut prims: Vec<Box<dyn Primitive>>) -> Box<Self> {
        // ensure all coordinates are positive by offsetting from min
        let box_ = prims
            .iter()
            .fold(None, |res, p| match (res, p.bounding_box()) {
                (None, b) | (b, None) => b,
                (Some(bl), Some(br)) => Some(bl.union(br)),
            }).unwrap();
        let min = box_.min;

        prims.sort_unstable_by(|left, right| match (left.bounding_box(), right.bounding_box()) {
            (None, _) | (_, None) => unimplemented!("No bounding box in BVH::new"),
            (Some(bleft), Some(bright)) => point_compare(
                Point3f::from_vec(bleft.center() - min),
                Point3f::from_vec(bright.center() - min),
            ),
        });
        BVH::new_sorted(prims)
    }
}

impl<'a> Primitive for BVH {
    fn intersect(&self, r: Ray3f) -> Option<SurfaceInteraction<'_>> {
        match self {
            BVH::Leaf { box_, prim } if box_.intersect(r) => prim.intersect(r),
            BVH::Node { box_, left, right } if box_.intersect(r) => {
                match (left.intersect(r), right.intersect(r)) {
                    (None, None) => None,
                    (Some(h), None) | (None, Some(h)) => Some(h),
                    (Some(hl), Some(hr)) => Some(iff!(hl.t < hr.t, hl, hr)),
                }
            }
            _ => None,
        }
    }
    fn bounding_box(&self) -> Option<AABB> {
        Some(*match self {
            BVH::Leaf { box_, prim: _ } => box_,
            BVH::Node { box_, left: _, right: _ } => box_,
        })
    }
}
