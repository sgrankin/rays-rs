extern crate image;

use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<Error>> {
    let width = 200;
    let height = 100;

    let mut imgbuf = image::RgbImage::new(width, height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = x as f32 / width as f32;
        let g = (height - y) as f32 / height as f32;
        let b = 0.2;
        *pixel = image::Rgb([(r * 255.99) as u8, (g * 255.99) as u8, (b * 255.99) as u8]);
    }

    image::ImageRgb8(imgbuf).save("out.png")?;
    Ok(())
}
