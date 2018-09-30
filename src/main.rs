extern crate cgmath;
extern crate image;

use cgmath::*;
use std::error::Error;

mod shapes;
use self::shapes::*;

fn main() -> Result<(), Box<Error>> {
    let width = 200;
    let height = 100;

    let lower_left_corner = Vector3::new(-2.0, -1.0, -1f64);
    let origin = Point3::new(0.0, 0.0, 0.0);
    let vertical = Vector3::new(0.0, 2.0, 0.0);
    let horizontal = Vector3::new(4.0, 0.0, 0.0);

    let world = vec![
        Sphere { center: Point3::new(0f64, 0f64, -1f64), radius: 0.5f64 },
        Sphere { center: Point3::new(0f64, -100.5f64, -1f64), radius: 100f64 },
    ];

    let mut imgbuf = image::RgbImage::new(width, height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let u = f64::from(x) / f64::from(width);
        let v = (f64::from(height - y) as f64) / f64::from(height);
        let r = Ray3 {
            origin,
            direction: (lower_left_corner + u * horizontal + v * vertical).normalize(),
        };
        // TODO can't actually pass anything here—the algo doesn't support vectors, the vector impl doesn't return normals... and the entire lib is too complicated.
        // just start writing a collision helper! CGmath may be ok
        let col = color(&r, &world);
        *pixel =
            image::Rgb([(col[0] * 255.99) as u8, (col[1] * 255.99) as u8, (col[2] * 255.99) as u8]);
    }

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
