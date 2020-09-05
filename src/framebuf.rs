use crate::types::*;

#[derive(Copy, Clone)]
pub struct Pixel {
    rgb: Vector3f,
    count: usize,
}

impl Pixel {
    fn new() -> Pixel {
        Pixel { rgb: Vector3f::zero(), count: 0 }
    }
    /*
    TODO:
    - track variance
    - don't bother with subpixels tracking
    - add dual buffers (to start on the NLM approach)
    - ... and track which buffer a ray is destined for (will be needed for cache points?) as part of the enum_pixels output (a 'token' that's needed fo a call add_sample)
    */
    pub fn add_sample(&mut self, rgb: Vector3f) {
        self.rgb += rgb;
        self.count += 1;
    }

    pub fn to_rgb(&self) -> [u8; 3] {
        let mut rgb = self.rgb;
        rgb /= self.count as Float;
        let v = rgb.map(|v| (v * 255.99) as u8);
        [v.x, v.y, v.z]
    }

    pub fn x(&self) -> f32 {
        (self.rgb.x / self.count as Float) as f32
    }
    pub fn y(&self) -> f32 {
        (self.rgb.y / self.count as Float) as f32
    }
    pub fn z(&self) -> f32 {
        (self.rgb.z / self.count as Float) as f32
    }
}

pub struct FrameBuf {
    pub width: usize,
    pub height: usize,
    pixels: Vec<Pixel>,
}

impl FrameBuf {
    pub fn new(width: usize, height: usize) -> FrameBuf {
        FrameBuf { width, height, pixels: vec![Pixel::new(); height * width] }
    }

    // TODO: this should write into an f32 buffer instead
    pub fn to_rgb(&self) -> Vec<f32> {
        let mut buf = vec![0.0f32; self.width * self.height * 3];
        for y in 0..self.height {
            for x in 0..self.width {
                let p = self.pixels[y * self.width + x];
                buf[(y * self.width + x) * 3 + 0] = p.x();
                buf[(y * self.width + x) * 3 + 1] = p.y();
                buf[(y * self.width + x) * 3 + 2] = p.z();
            }
        }
        buf
    }

    pub fn mk_image(&self) -> image::RgbImage {
        let mut buf = image::RgbImage::new(self.width as u32, self.height as u32);
        buf.enumerate_pixels_mut().for_each(|(x, y, p)| {
            *p = image::Rgb(
                self.pixels[(x as usize + self.width * (self.height - 1 - y as usize))].to_rgb(),
            );
        });
        buf
    }

    pub fn add_sample(&mut self, pixel: Point2u, _subpixel: Point2f, rgb: Vector3f) {
        // box filter: ignore subpixel; weight is 1
        self.pixels[pixel.x + self.width * pixel.y].add_sample(rgb)
    }

    pub fn enum_pixels(&self) -> Vec<Point2u> {
        (0..self.height).flat_map(|y| (0..self.width).map(move |x| Point2u::new(x, y))).collect()
    }
}
