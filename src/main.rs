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
#[macro_use]
mod macros;
mod material;
mod metrics;
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

    let width = 640; // 960; // 640; // 1920; // 960; //960;
    let height = 400; // 600; // 400; // 1200; // 600; // 600;
    let samples_per_pixel = 256;
    let aspect_ratio = width as f64 / height as f64;

    let last_display = video.display_bounds(video.num_video_displays()? - 1)?.center();

    let mut winwidth = min!(width, 640) as i32;
    let mut winheight = (winwidth as f64 / aspect_ratio) as i32;

    let window = video
        .window("rays-rs", winwidth as u32, winheight as u32)
        .position_centered()
        .position(last_display.x() - (winwidth) / 2, last_display.y() - (winheight) / 2)
        .allow_highdpi()
        .resizable()
        .build()?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 255));
    canvas.clear();
    canvas.present();

    // work around macos bug https://discourse.libsdl.org/t/macos-10-14-mojave-issues/25060/2
    event_pump.pump_events();
    canvas.window_mut().set_size(winwidth as u32, winheight as u32)?;

    let world = scene::new_cover_scene();

    let from = Point3f::new(12.0, 3.0, 3.0);
    let to = Point3f::new(0.0, 0.0, -1.0);
    let c = Camera::new(
        from,
        to,
        Vector3f::unit_y(),
        /* fov */ 55.0,
        /* aperture */ 0.1,
        /*focus_dist */ (to - from).magnitude(),
        /* film_size */ Point2u::new(width, height),
    );

    let (tx, rx) = sync_channel(100);
    thread::spawn({
        let ctx = ctx.clone();
        move || {
            info!("starting");
            let mut buf = framebuf::FrameBuf::new(width, height);
            let mut i = 1;
            while i < samples_per_pixel {
                info!("tracing {} samples per pixel", i);
                trace_into(&ctx, &mut buf, i, &world, &c);
                tx.send(buf.mk_image()).unwrap();
                i *= 2;
            }
            info!("done!");
        }
    });

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        sdl2::pixels::PixelFormatEnum::RGB24,
        width as u32,
        height as u32,
    )?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                sdl2::event::Event::Window {
                    win_event:
                        sdl2::event::WindowEvent::Resized(mut new_winwidth, mut new_winheight),
                    ..
                } => {
                    if new_winwidth != winwidth {
                        new_winheight = (new_winwidth as f64 / aspect_ratio) as i32
                    } else {
                        new_winwidth = (new_winheight as f64 * aspect_ratio) as i32
                    }
                    winwidth = new_winwidth;
                    winheight = new_winheight;
                    canvas.window_mut().set_size(winwidth as u32, winheight as u32)?;
                }
                _ => {}
            }
        }
        if let Ok(buf) = rx.try_recv() {
            let img = image::ImageRgb8(buf);
            texture.with_lock(None, |buf, _| {
                buf.copy_from_slice(&img.raw_pixels());
            })?;
            img.save("out.png")?;
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
    samples_per_pixel: usize,
    scene: &Primitive,
    camera: &Camera,
) {
    let t_begin = time::Instant::now();
    let results: Vec<_> = imgbuf
        .enum_pixels()
        .par_iter()
        .flat_map(|pixel| {
            let rays = camera.get_rays(samples_per_pixel, *pixel);
            let res: Vec<_> = rays
                .iter()
                .map(|(r, offset)| {
                    let col = color(r, scene);
                    let col = col.map(|x| x.sqrt()); // gamma correction
                    (*pixel, *offset, col)
                })
                .collect();
            res
        })
        .collect();

    for (pixel, offset, col) in results {
        imgbuf.add_sample(pixel, offset, col);
    }

    ctx.time_per_pass.record_since(t_begin);
}

fn color(r: &Ray3f, world: &dyn Primitive) -> Vector3f {
    // with credit to https://computergraphics.stackexchange.com/questions/5152/progressive-path-tracing-with-explicit-light-sampling
    let mut bounces = 0;
    let mut ray = *r;
    let mut throughput = Vector3f::from_value(1.0);
    while let Some(ref hit) = world.intersect(ray) {
        match hit.material.scatter(ray, hit.point, hit.normal) {
            None => return Vector3f::zero(), // absorbed
            Some((r, t)) => {
                ray = r;
                throughput.mul_assign_element_wise(t);
                if bounces > 3 {
                    // russian roulette
                    let p = min!(max!(throughput.x, throughput.y, throughput.z), 0.95);
                    if random() > p {
                        return Vector3f::zero(); // absorbed
                    }
                    throughput /= p;
                }
                bounces += 1;
            }
        }
    }
    let t = (ray.direction.y + 1.0) / 2.0;
    let color = Vector3f::from_value(1.0) * (1.0 - t) + Vector3f::new(0.5, 0.7, 1.0) * t;
    color.mul_element_wise(throughput)
}
