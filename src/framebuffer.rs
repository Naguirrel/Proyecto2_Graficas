use crate::color::Color;

pub struct Framebuffer {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let pixel_count = width
            .checked_mul(height)
            .expect("framebuffer dimensions overflowed usize");

        Self {
            width,
            height,
            pixels: vec![Color::BLACK.to_u32(); pixel_count],
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

#[cfg(test)]
mod tests {
    use super::Framebuffer;
    use crate::color::Color;

    #[test]
    fn constructor_creates_expected_pixel_count() {
        let framebuffer = Framebuffer::new(4, 3);

        assert_eq!(framebuffer.pixels().len(), 12);
        assert!(
            framebuffer
                .pixels()
                .iter()
                .all(|&pixel| pixel == Color::BLACK.to_u32())
        );
    }

    #[test]
    fn reports_dimensions() {
        let framebuffer = Framebuffer::new(7, 5);

        assert_eq!(framebuffer.width(), 7);
        assert_eq!(framebuffer.height(), 5);
    }

    #[test]
    fn clear_updates_every_pixel() {
        let mut framebuffer = Framebuffer::new(3, 2);
        let color = Color::rgb(0.2, 0.4, 0.6);

        framebuffer.clear(color);

        assert!(
            framebuffer
                .pixels()
                .iter()
                .all(|&pixel| pixel == color.to_u32())
        );
    }

    #[test]
    fn set_pixel_updates_requested_position() {
        let mut framebuffer = Framebuffer::new(3, 2);
        let color = Color::WHITE;

        framebuffer.set_pixel(1, 1, color);

        assert_eq!(framebuffer.pixels()[4], color.to_u32());
        assert_eq!(framebuffer.pixels()[0], Color::BLACK.to_u32());
    }

    #[test]
    fn set_pixel_outside_bounds_is_ignored() {
        let mut framebuffer = Framebuffer::new(2, 2);
        let before = framebuffer.pixels().to_vec();

        framebuffer.set_pixel(2, 0, Color::WHITE);
        framebuffer.set_pixel(0, 2, Color::WHITE);

        assert_eq!(framebuffer.pixels(), before.as_slice());
    }
}
