extern crate cgmath;
extern crate collision;
extern crate image;

use cgmath::*;
use collision::*;
use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    let width = 200;
    let height = 100;

    let lower_left_corner = Vector3::new(-2.0, -1.0, -1f64);
    let origin = Point3::new(0.0, 0.0, 0.0);
    let vertical = Vector3::new(0.0, 2.0, 0.0);
    let horizontal = Vector3::new(4.0, 0.0, 0.0);

    let mut imgbuf = image::RgbImage::new(width, height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let u = f64::from(x) / f64::from(width);
        let v = (f64::from(height - y) as f64) / f64::from(height);
        let r = Ray::new(origin, (lower_left_corner + u * horizontal + v * vertical).normalize());
        let col = color(&r);
        *pixel =
            image::Rgb([(col[0] * 255.99) as u8, (col[1] * 255.99) as u8, (col[2] * 255.99) as u8]);
    }

    image::ImageRgb8(imgbuf).save("out.png")?;
    Ok(())
}

fn color(r: &Ray3<f64>) -> Vector3<f64> {
    let s = Sphere { center: Point3::new(0f64, 0f64, -1f64), radius: 0.5f64 };
    if s.intersection(r).is_some() {
        return Vector3::new(1f64, 0f64, 0f64);
    }
    let t = (r.direction.y + 1.0) * 0.5;
    (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0)
}
