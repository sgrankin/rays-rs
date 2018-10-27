use crate::types::*;

#[derive(Copy, Clone)]
pub struct Pixel {
    pub x: u32,
    pub y: u32,
    rgb: Vector3f,
    samples: u32,
}

impl Pixel {
    fn new(x: u32, y: u32) -> Pixel {
        Pixel { x, y, rgb: Vector3f::zero(), samples: 0 }
    }
    pub fn add_sample(&mut self, rgb: Vector3f) {
        self.rgb += rgb;
        self.samples += 1;
    }

    pub fn to_rgb(&self) -> [u8; 3] {
        let v = (self.rgb / self.samples as f64).map(|v| (v * 255.99) as u8);
        [v.x, v.y, v.z]
    }
}

pub struct FrameBuf {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Pixel>,
}

impl FrameBuf {
    pub fn new(width: u32, height: u32) -> FrameBuf {
        FrameBuf {
            width,
            height,
            pixels: (0..height).flat_map(|y| (0..width).map(move |x| Pixel::new(x, y))).collect(),
        }
    }

    pub fn mk_image(&self) -> image::RgbImage {
        let mut buf = image::RgbImage::new(self.width, self.height);
        buf.enumerate_pixels_mut().for_each(|(x, y, p)| {
            *p = image::Rgb(self.pixels[(x + self.width * y) as usize].to_rgb());
        });
        buf
    }
}
