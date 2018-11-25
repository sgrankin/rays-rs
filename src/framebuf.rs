use crate::types::*;
use std::collections::BTreeMap;

pub struct FrameBuf {
    pub width: usize,
    pub height: usize,
    pixels: BTreeMap<(usize, usize), Vec<(Point2f, Vector3f)>>,
}

impl FrameBuf {
    pub fn new(width: usize, height: usize) -> FrameBuf {
        FrameBuf { width, height, pixels: BTreeMap::new() }
    }

    pub fn mk_image(&self) -> image::RgbImage {
        let mut buf = image::RgbImage::new(self.width as u32, self.height as u32);
        buf.enumerate_pixels_mut().for_each(|(x, y, p)| {
            let empty = vec![];
            // Box filter: ignore subpixel locations; add all with weight=1.0
            let pixels =
                self.pixels.get(&(x as usize, self.height - 1 - y as usize)).unwrap_or(&empty);
            let pixel = pixels.iter().fold(Vector3f::zero(), |sum, (_, p)| sum + p)
                / (pixels.len() as Float);
            *p = image::Rgb(FrameBuf::to_rgb(pixel));
        });
        buf
    }

    fn to_rgb(rgb: Vector3f) -> [u8; 3] {
        let v = rgb.map(|v| (v * 255.99) as u8);
        [v.x, v.y, v.z]
    }

    pub fn add_sample(&mut self, pixel: Point2u, subpixel: Point2f, rgb: Vector3f) {
        let k = (pixel.x, pixel.y);
        if !self.pixels.contains_key(&k) {
            self.pixels.insert(k, vec![]);
        }
        self.pixels.get_mut(&k).unwrap().push((subpixel, rgb));
    }

    pub fn enum_pixels(&self) -> Vec<Point2u> {
        (0..self.height).flat_map(|y| (0..self.width).map(move |x| Point2u::new(x, y))).collect()
    }
}
