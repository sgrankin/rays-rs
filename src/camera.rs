use cgmath::*;

use crate::geom::*;
use crate::util;

pub struct Camera<S> {
    origin: Point3<S>,
    lower_left: Point3<S>,
    horizontal: Vector3<S>,
    vertical: Vector3<S>,
    lens_radius: S,
    u: Vector3<S>,
    v: Vector3<S>,
    w: Vector3<S>,
}
impl<S> Camera<S>
where
    S: BaseFloat,
{
    pub fn new(
        origin: Point3<S>,
        target: Point3<S>,
        vup: Vector3<S>,
        fov: S,
        aspect: S,
        aperture: S,
        focus_dist: S,
    ) -> Camera<S> {
        let theta = fov * S::from(std::f64::consts::PI / 180.0).unwrap();

        let half_width = (theta / S::from(2).unwrap()).tan();
        let half_height = half_width / aspect;

        let w = (origin - target).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        Camera {
            origin,
            lower_left: origin - (u * half_width + v * half_height + w) * focus_dist,
            horizontal: u * (half_width + half_width) * focus_dist,
            vertical: v * (half_height + half_height) * focus_dist,
            lens_radius: aperture / S::from(2).unwrap(),
            u,
            v,
            w,
        }
    }

    pub fn get_ray(&self, s: S, t: S) -> Ray3<S> {
        let rd = util::random_in_unit_disk() * self.lens_radius;
        let offset = self.u * rd.x + self.v * rd.y;

        Ray3 {
            origin: self.origin + offset,
            direction: ((self.lower_left + self.horizontal * s + self.vertical * t)
                - (self.origin + offset))
                .normalize(),
        }
    }
}
