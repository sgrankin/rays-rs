use crate::geom::*;
use crate::types::*;
use crate::util;

pub struct Camera {
    origin: Point3f,
    lower_left: Point3f,
    horizontal: Vector3f,
    vertical: Vector3f,
    lens_radius: Float,
    u: Vector3f,
    v: Vector3f,
    w: Vector3f,
}
impl Camera {
    pub fn new(
        origin: Point3f,
        target: Point3f,
        vup: Vector3f,
        fov: Float,
        aspect: Float,
        aperture: Float,
        focus_dist: Float,
    ) -> Camera {
        let theta = fov * PI / 180.0;

        let half_width = (theta / 2.0).tan();
        let half_height = half_width / aspect;

        let w = (origin - target).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        Camera {
            origin,
            lower_left: origin - (u * half_width + v * half_height + w) * focus_dist,
            horizontal: u * (half_width + half_width) * focus_dist,
            vertical: v * (half_height + half_height) * focus_dist,
            lens_radius: aperture / 2.0,
            u,
            v,
            w,
        }
    }

    pub fn get_ray(&self, s: Float, t: Float) -> Ray3f {
        let rd = util::random_in_unit_disk() * self.lens_radius;
        let offset = self.u * rd.x + self.v * rd.y;

        Ray3f::new(
            self.origin + offset,
            (self.lower_left + self.horizontal * s + self.vertical * t) - (self.origin + offset),
        )
    }
}
