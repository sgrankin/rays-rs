pub use cgmath::{Array, ElementWise, EuclideanSpace, InnerSpace, MetricSpace, VectorSpace, Zero};
pub use cgmath::{Point2 as _Point2, Point3 as _Point3, Vector2 as _Vector2, Vector3 as _Vector3};

pub use std::f64::consts::PI;
pub use std::f64::MAX as FLOAT_MAX;
pub use std::f64::MIN as FLOAT_MIN;

pub type Float = f64;
pub type Vector2f = _Vector2<Float>;
pub type Vector3f = _Vector3<Float>;
pub type Point3f = _Point3<Float>;
pub type Point2f = _Point2<Float>;
