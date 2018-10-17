mod aggregate;
mod camera;
mod geom;
mod material;
mod prims;
mod scene;
mod util;

use log::*;
use simple_logger;

use cgmath::*;
use image;
use rayon::prelude::*;
use std::error::Error;

use self::geom::*;
use self::prims::*;
use self::util::*;

struct Scene<S: BaseFloat> {
    pub aggregate: dyn Primitive<S>,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init()?;
    info!("starting");
    let width = 320; // 1920; // 960; //960;
    let height = 200; // 1200; // 600; // 600;
    let samples_per_pixel = 4;

    let world = scene::new_cover_scene();

    let from = Point3::new(12.0, 3.0, 3.0);
    let to = Point3::new(0.0, 0.0, -1.0);
    let c = camera::Camera::new(
        from,
        to,
        Vector3::unit_y(),
        55.0,
        f64::from(width) / f64::from(height),
        0.1,
        (to - from).magnitude(),
    );
    let mut imgbuf = image::RgbImage::new(width, height);
    imgbuf
        .enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut image::Rgb<u8>)>>()
        .par_iter_mut()
        // .iter_mut()
        .for_each(|(x, y, pixel)| {
            let mut col = Vector3::zero();
            for _ in 0..samples_per_pixel {
                let u = (f64::from(*x) + (random::<f64>())) / f64::from(width);
                let v = (f64::from(height - *y) + (random::<f64>())) / f64::from(height);
                let r = c.get_ray(u, v);
                col += color(r, &world);
            }
            col /= f64::from(samples_per_pixel);
            col = col.map(|x| x.sqrt()); // gamma correction
            **pixel = image::Rgb([
                (col[0] * 255.99) as u8,
                (col[1] * 255.99) as u8,
                (col[2] * 255.99) as u8,
            ]);
        });

    image::ImageRgb8(imgbuf).save("out.png")?;
    info!("done");
    Ok(())
}

fn color<S: BaseFloat>(r: Ray3<S>, world: &dyn Primitive<S>) -> Vector3<S> {
    let mut bounces = 0;
    let mut ray = r;
    let mut color = Vector3::from_value(S::zero());
    let mut throughput = Vector3::from_value(S::one());
    loop {
        match world.intersect(ray) {
            Some(ref hit) => {
                // Hit a thing!
                match hit.material.scatter(ray, hit.point, hit.normal) {
                    None => return Vector3::zero(),
                    Some((r, t)) => {
                        ray = r;
                        throughput.mul_assign_element_wise(t);
                        if bounces > 5 {
                            let p = S::max(throughput.x, S::max(throughput.y, throughput.z));
                            if random::<S>() > p {
                                return Vector3::zero();
                            }
                            throughput /= p;
                            bounces += 1;
                        }
                    }
                };
            }

            _ => {
                // No hit - eval against background radiation.
                break;
            }
        }
    }
    let t = (ray.direction.y + S::one()) / S::from(2).unwrap();
    color = Vector3::from_value(S::one()) * (S::one() - t)
        + Vector3::new(S::from(0.5).unwrap(), S::from(0.7).unwrap(), S::one()) * t;
    color.mul_element_wise(throughput)
}
