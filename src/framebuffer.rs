use crate::color::Color;

pub struct Framebuffer {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::BLACK.to_u32(); width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color.to_u32());
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        self.pixels[y * self.width + x] = color.to_u32();
    }
}
