extern crate cgmath;
extern crate image;

use cgmath::*;
use rand::random;
use rayon::prelude::*;
use std::error::Error;
use std::ops::*;

mod shapes;
use self::shapes::*;

fn main() -> Result<(), Box<Error>> {
    let width = 1440;
    let height = 900;
    let samples = 8;

    let world = vec![
        Sphere { center: Point3::new(0f64, 0f64, -1f64), radius: 0.5f64 },
        Sphere { center: Point3::new(0f64, -100.5f64, -1f64), radius: 100f64 },
    ];

    let c = Camera::new();
    let mut imgbuf = image::RgbImage::new(width, height);
    imgbuf
        .enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut image::Rgb<u8>)>>()
        .par_iter_mut()
        .for_each(|(x, y, pixel)| {
            let mut col = Vector3::zero();
            for _ in 0..samples {
                let u = (f64::from(*x) + random::<f64>()) / f64::from(width);
                let v = (f64::from(height - *y) + random::<f64>()) / f64::from(height);
                let r = c.get_ray(u, v);
                col += color(&r, &world);
            }
            col /= f64::from(samples);
            **pixel = image::Rgb([
                (col[0] * 255.99) as u8,
                (col[1] * 255.99) as u8,
                (col[2] * 255.99) as u8,
            ]);
        });

    image::ImageRgb8(imgbuf).save("out.png")?;
    Ok(())
}

fn color<S, T>(r: &Ray3<S>, s: &[T]) -> Vector3<S>
where
    S: BaseFloat,
    T: Intersectable<S>,
{
    match s.intersection(r) {
        Some(p) => p.normal.map(|x| x + S::one()) / S::from(2).unwrap(),

        None => {
            let t = (r.direction.y + S::one()) / S::from(2).unwrap();
            Vector3::new(S::one(), S::one(), S::one()) * (S::one() - t)
                + Vector3::new(S::from(0.5).unwrap(), S::from(0.7).unwrap(), S::one()) * t
        }
    }
}

struct Camera<S> {
    origin: Point3<S>,
    direction: Vector3<S>,
    image_distance: S,
    image_bounds: Vector2<S>,
}
impl<S> Camera<S>
where
    S: BaseFloat,
{
    fn new() -> Camera<S> {
        Camera {
            origin: Point3::origin(),
            direction: Vector3::unit_z().neg(),
            image_distance: S::one(),
            image_bounds: Vector2::new(S::from(3.2).unwrap(), S::from(2.0).unwrap()),
        }
    }
    fn get_ray(&self, u: S, v: S) -> Ray3<S> {
        // TODO: The math for the direction *depends* on direction only having a z component.
        // Figure out how to transform the screen bounds correctly into the direction's coordinate system.
        Ray3 {
            origin: self.origin,
            direction: (self.direction * self.image_distance
                + self.origin.to_vec()
                + Vector3::new(
                    self.image_bounds.x * (u - S::from(0.5).unwrap()),
                    self.image_bounds.y * (v - S::from(0.5).unwrap()),
                    S::zero(),
                )).normalize(),
        }
    }
}
