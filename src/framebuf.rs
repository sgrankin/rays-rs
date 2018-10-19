use crate::types::*;

#[derive(Copy, Clone)]
pub struct Pixel {
    rgb: Vector3f,
    samples: u32,
}

impl Pixel {
    fn new() -> Pixel {
        Pixel { rgb: Vector3f::zero(), samples: 0 }
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
    pixels: Vec<Pixel>,
}

pub struct EnumPixelsMut<'a> {
    pixels: std::slice::IterMut<'a, Pixel>,
    width: u32,
    x: u32,
    y: u32,
}

impl<'a> Iterator for EnumPixelsMut<'a> {
    type Item = (u32, u32, &'a mut Pixel);
    fn next(&mut self) -> Option<(u32, u32, &'a mut Pixel)> {
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        self.pixels.next().map(move |p| (x, y, p))
    }
}

impl FrameBuf {
    pub fn new(width: u32, height: u32) -> FrameBuf {
        FrameBuf { width, height, pixels: vec![Pixel::new(); (width * height) as usize] }
    }
    pub fn enum_pixels_mut(&mut self) -> EnumPixelsMut {
        EnumPixelsMut { pixels: self.pixels.iter_mut(), width: self.width, x: 0, y: 0 }
    }

    pub fn mk_image(&self) -> image::RgbImage {
        let mut buf = image::RgbImage::new(self.width, self.height);
        buf.enumerate_pixels_mut().for_each(|(x, y, p)| {
            *p = image::Rgb(self.pixels[(x + self.width * y) as usize].to_rgb());
        });
        buf
    }
}
