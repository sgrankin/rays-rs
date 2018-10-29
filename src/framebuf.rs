use crate::types::*;

#[derive(Copy, Clone)]
pub struct Pixel {
    rgb: Vector3f,
    weights: Float,
}

impl Pixel {
    fn new() -> Pixel {
        Pixel { rgb: Vector3f::zero(), weights: 0.0 }
    }
    pub fn add_sample(&mut self, rgb: Vector3f, weight: Float) {
        self.rgb += rgb;
        self.weights += weight;
    }

    pub fn to_rgb(&self) -> [u8; 3] {
        let mut rgb = self.rgb;
        if self.weights > 0.0 {
            rgb /= self.weights
        }
        let v = rgb.map(|v| (v * 255.99) as u8);
        [v.x, v.y, v.z]
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

    pub fn mk_image(&self) -> image::RgbImage {
        let mut buf = image::RgbImage::new(self.width as u32, self.height as u32);
        buf.enumerate_pixels_mut().for_each(|(x, y, p)| {
            *p = image::Rgb(
                self.pixels[(x as usize + self.width * (self.height - 1 - y as usize))].to_rgb(),
            );
        });
        buf
    }
    pub fn add_sample(&mut self, pixel: Point2u, subpixel: Point2f, rgb: Vector3f) {
        // box filter: ignore subpixel; weight is 1
        self.pixels[pixel.x + self.width * pixel.y].add_sample(rgb, 1.0)
    }

    pub fn enum_pixels(&self) -> Vec<Point2u> {
        (0..self.height).flat_map(|y| (0..self.width).map(move |x| Point2u::new(x, y))).collect()
    }
}
