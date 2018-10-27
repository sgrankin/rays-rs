extern crate cgmath;
extern crate hdrsample;
extern crate image;
extern crate log;
extern crate rand;
extern crate rayon;
extern crate sdl2;
extern crate simple_logger;
extern crate tacho;

mod aggregate;
mod camera;
mod framebuf;
mod geom;
mod metrics;
#[macro_use]
mod macros;
mod material;
mod prims;
mod scene;
mod shape;
mod types;
mod util;

use log::info;
use rayon::prelude::*;
use std::error::Error;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time;

pub use self::aggregate::*;
pub use self::camera::*;
pub use self::geom::*;
pub use self::macros::*;
pub use self::prims::*;
pub use self::types::*;
pub use self::util::*;

// struct Scene {
//     pub aggregate: dyn Primitive,
// }

#[derive(Clone)]
struct Context {
    reporter: tacho::Reporter,

    time_per_ray: tacho::Timer,
    time_per_pass: tacho::Timer,
}

impl Context {
    fn new() -> Context {
        let (metrics, reporter) = tacho::new();
        let time_per_ray = metrics.timer_us("time_per_ray_us");
        let time_per_pass = metrics.timer_us("time_per_pass_us");

        Context { reporter, time_per_ray, time_per_pass }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init()?;
    let ctx = Context::new();

    let sdl_context = sdl2::init()?;
    let video = sdl_context.video()?;

    let width = 960; // 640; // 1920; // 960; //960;
    let height = 600; // 400; // 1200; // 600; // 600;
    let samples_per_pixel = 256;

    let last_display = video.display_bounds(video.num_video_displays()? - 1)?.center();

    let window = video
        .window("rays-rs", width, height)
        .position_centered()
        .position(last_display.x() - (width as i32) / 2, last_display.y() - (height as i32) / 2)
        // .opengl()
        .allow_highdpi()
        .build()?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 255));
    canvas.clear();
    canvas.present();

    // work around macos bug https://discourse.libsdl.org/t/macos-10-14-mojave-issues/25060/2
    event_pump.pump_events();
    canvas.window_mut().set_size(width, height)?;

    let world = scene::new_cover_scene();

    let from = Point3::new(12.0, 3.0, 3.0);
    let to = Point3::new(0.0, 0.0, -1.0);
    let c = Camera::new(
        from,
        to,
        Vector3f::unit_y(),
        /* fov */ 55.0,
        /* aspect */ (width as Float) / (height as Float),
        /* aperture */ 0.1,
        (to - from).magnitude(),
    );

    let (tx, rx) = sync_channel(100);
    thread::spawn({
        let ctx = ctx.clone();
        move || {
            info!("starting");
            let mut buf = framebuf::FrameBuf::new(width, height);
            for i in 0..samples_per_pixel {
                trace_into(&ctx, &mut buf, 1, &world, &c);
                tx.send(buf.mk_image()).unwrap();
                info!("frame {}/{}", i, samples_per_pixel);
            }
            info!("done!");
        }
    });

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        sdl2::pixels::PixelFormatEnum::RGB24,
        width,
        height,
    )?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        if let Ok(buf) = rx.try_recv() {
            let img = image::ImageRgb8(buf);
            texture.with_lock(None, |buf, _| {
                buf.copy_from_slice(&img.raw_pixels());
            })?;
            // img.save("out.png")?;
            info!("metrics:\n{}", metrics::string(&ctx.reporter.peek())?);
        }

        canvas.copy(&texture, None, None)?;
        canvas.present();
        thread::sleep(time::Duration::from_micros(1_000_000 / 60));
    }
    return Ok(());
}

fn trace_into(
    ctx: &Context,
    imgbuf: &mut framebuf::FrameBuf,
    samples_per_pixel: u32,
    scene: &Primitive,
    camera: &Camera,
) {
    let t_begin = time::Instant::now();
    let width = imgbuf.width as Float;
    let height = imgbuf.height as Float;
    imgbuf.pixels.par_iter_mut().for_each(|pixel| {
        // let t_begin = time::Instant::now();
        for _ in 0..samples_per_pixel {
            let u = (pixel.x as Float + random()) / width;
            let v = (height - pixel.y as Float + random()) / height;
            let r = camera.get_ray(u, v);
            let col = color(r, scene);
            let col = col.map(|x| x.sqrt()); // gamma correction
            pixel.add_sample(col)
        }
        // ctx.time_per_ray.record_since(t_begin);
    });
    ctx.time_per_pass.record_since(t_begin);
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
