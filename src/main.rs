use image;
use log::info;
use rayon::prelude::*;
use simple_logger;
use std::error::Error;

use rays_rs::*;
// struct Scene {
//     pub aggregate: dyn Primitive,
// }

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init()?;
    info!("starting");
    let width = 1920; // 1920; // 960; //960;
    let height = 1200; // 1200; // 600; // 600;
    let samples_per_pixel = 3;

    let world = scene::new_cover_scene();

    let from = Point3::new(12.0, 3.0, 3.0);
    let to = Point3::new(0.0, 0.0, -1.0);
    let c = Camera::new(
        from,
        to,
        Vector3f::unit_y(),
        55.0,
        (width as Float) / (height as Float),
        0.1,
        (to - from).magnitude(),
    );
    let mut buf = framebuf::FrameBuf::new(width, height);
    trace_into(&mut buf, samples_per_pixel, &world, &c);
    image::ImageRgb8(buf.mk_image()).save("out.png")?;
    info!("done");
    Ok(())
}

fn trace_into(
    imgbuf: &mut framebuf::FrameBuf,
    samples_per_pixel: u32,
    scene: &Aggregate,
    camera: &Camera,
) {
    let width = imgbuf.width as Float;
    let height = imgbuf.height as Float;
    imgbuf
        .enum_pixels_mut()
        .collect::<Vec<(u32, u32, &mut framebuf::Pixel)>>()
        .par_iter_mut()
        .for_each(|(x, y, pixel)| {
            for _ in 0..samples_per_pixel {
                let u = (*x as Float + random()) / width;
                let v = (height - *y as Float + random()) / height;
                let r = camera.get_ray(u, v);
                let col = color(r, scene);
                let col = col.map(|x| x.sqrt()); // gamma correction
                pixel.add_sample(col)
            }
        });
}

fn color(r: Ray3f, world: &dyn Primitive) -> Vector3f {
    // with credit to https://computergraphics.stackexchange.com/questions/5152/progressive-path-tracing-with-explicit-light-sampling
    let mut bounces = 0;
    let mut ray = r;
    let mut throughput = Vector3f::from_value(1.0);
    while let Some(ref hit) = world.intersect(ray) {
        match hit.material.scatter(ray, hit.point, hit.normal) {
            None => return Vector3f::zero(), // absorbed
            Some((r, t)) => {
                ray = r;
                throughput.mul_assign_element_wise(t);
                if bounces > 5 {
                    // russian roulette
                    let p = max!(throughput.x, throughput.y, throughput.z);
                    if random() > p {
                        return Vector3f::zero(); // absorbed
                    }
                    throughput /= p;
                    bounces += 1;
                }
            }
        }
    }
    let t = (ray.direction.y + 1.0) / 2.0;
    let color = Vector3f::from_value(1.0) * (1.0 - t) + Vector3f::new(0.5, 0.7, 1.0) * t;
    color.mul_element_wise(throughput)
}
