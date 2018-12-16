use crate::geom::*;
use crate::types::*;
use crate::util;

pub struct Camera {
    /// Center of the lens.
    origin: Point3f,
    /// Lower left corner of the transformed image plane.
    lower_left: Point3f,
    /// Horizontal edge of the transformed image plane.
    horizontal: Vector3f,
    /// Vertical edge of the transformed image plane.
    vertical: Vector3f,
    lens_radius: Float,
    u: Vector3f,
    v: Vector3f,
    w: Vector3f,
    film_size: Point2f,
    pixel_size: Point2f,
}
impl Camera {
    pub fn new(
        origin: Point3f,
        target: Point3f,
        up: Vector3f,
        fov: Float,
        aperture: Float,
        focus_dist: Float,
        film_size: Point2u,
    ) -> Camera {
        let theta = fov * PI / 180.0;

        let aspect_ratio = film_size.x as Float / film_size.y as Float;
        let half_width = (theta / 2.0).tan();
        let half_height = half_width / aspect_ratio;

        let w = (origin - target).normalize();
        let u = up.cross(w).normalize();
        let v = w.cross(u);
        let film_size = film_size.map(|v| v as Float);
        let pixel_size = film_size.map(|v| 1.0 / v);

        Camera {
            origin,
            lower_left: origin - (u * half_width + v * half_height + w) * focus_dist,
            horizontal: u * (half_width + half_width) * focus_dist,
            vertical: v * (half_height + half_height) * focus_dist,
            lens_radius: aperture / 2.0,
            u,
            v,
            w,
            film_size,
            pixel_size,
        }
    }

    pub fn get_rays(&self, n: usize, film_pos: Point2u) -> Vec<(Ray3f, Point2f)> {
        // scale film_pos to 0-1
        let film_pos = film_pos.map(|v| v as Float).div_element_wise(self.film_size);

        let mut lens_samples = util::stratified_samples_in_disk(n);
        util::shuffle(&mut lens_samples);
        let pixel_offsets = util::stratified_samples(n);
        lens_samples
            .iter()
            .zip(pixel_offsets)
            .map(|(lens_offset, pixel_offset)| {
                let lens_offset = lens_offset * self.lens_radius;
                let lens_pos = self.u * lens_offset.x + self.v * lens_offset.y;
                let origin = self.origin + lens_pos;
                (
                    Ray3f::new(
                        origin,
                        (self.lower_left
                            + self.horizontal * (film_pos.x + pixel_offset.x * self.pixel_size.x)
                            + self.vertical * (film_pos.y + pixel_offset.y * self.pixel_size.y))
                            - origin,
                    ),
                    pixel_offset,
                )
            })
            .collect()
    }
}
