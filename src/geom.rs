use crate::types::*;

#[derive(Copy, Clone, Debug)]
pub struct Ray3f {
    pub origin: Point3f,
    pub direction: Vector3f,
    pub inv_d: Vector3f,
}

impl Ray3f where {
    pub fn new(origin: Point3f, direction: Vector3f) -> Self {
        let direction = direction.normalize();
        Self {
            origin,
            direction: direction,
            inv_d: Vector3f::from_value(1.0).div_element_wise(direction),
        }
    }
}
