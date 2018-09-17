extern crate cgmath;
extern crate image;

use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    let width = 200;
    let height = 100;

    let mut imgbuf = image::RgbImage::new(width, height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let col = [x as f32 / width as f32, (height - y) as f32 / height as f32, 0.2];
        *pixel =
            image::Rgb([(col[0] * 255.99) as u8, (col[1] * 255.99) as u8, (col[2] * 255.99) as u8]);
    }

    image::ImageRgb8(imgbuf).save("out.png")?;
    Ok(())
}
