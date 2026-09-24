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

    pub fn resize(&mut self, width: usize, height: usize) {
        let pixel_count = width
            .checked_mul(height)
            .expect("framebuffer dimensions overflowed usize");

        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;
        self.pixels.resize(pixel_count, Color::BLACK.to_u32());
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

    pub fn copy_scaled_nearest_from(&mut self, source: &Framebuffer) {
        if self.width == 0 || self.height == 0 || source.width == 0 || source.height == 0 {
            return;
        }

        for y in 0..self.height {
            let source_y = scaled_index(y, source.height, self.height);
            let source_row = source_y * source.width;
            let output_row = y * self.width;

            for x in 0..self.width {
                let source_x = scaled_index(x, source.width, self.width);
                self.pixels[output_row + x] = source.pixels[source_row + source_x];
            }
        }
    }
}

fn scaled_index(output_index: usize, source_size: usize, output_size: usize) -> usize {
    if source_size == 0 || output_size == 0 {
        return 0;
    }

    let index = (output_index as u128 * source_size as u128 / output_size as u128) as usize;

    index.min(source_size - 1)
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

    #[test]
    fn resize_keeps_allocation_when_dimensions_are_unchanged() {
        let mut framebuffer = Framebuffer::new(4, 3);
        let capacity = framebuffer.pixels.capacity();

        framebuffer.resize(4, 3);

        assert_eq!(framebuffer.width(), 4);
        assert_eq!(framebuffer.height(), 3);
        assert_eq!(framebuffer.pixels.capacity(), capacity);
    }

    #[test]
    fn resize_updates_dimensions_and_pixel_count() {
        let mut framebuffer = Framebuffer::new(4, 3);

        framebuffer.resize(2, 5);

        assert_eq!(framebuffer.width(), 2);
        assert_eq!(framebuffer.height(), 5);
        assert_eq!(framebuffer.pixels().len(), 10);
    }

    #[test]
    fn nearest_neighbor_scaling_fills_entire_output() {
        let mut source = Framebuffer::new(1, 1);
        let mut output = Framebuffer::new(3, 5);
        let color = Color::rgb(0.25, 0.5, 0.75);
        source.set_pixel(0, 0, color);

        output.copy_scaled_nearest_from(&source);

        assert!(output.pixels().iter().all(|&pixel| pixel == color.to_u32()));
    }

    #[test]
    fn nearest_neighbor_scaling_preserves_corners() {
        let mut source = Framebuffer::new(2, 2);
        let mut output = Framebuffer::new(4, 4);
        let top_left = Color::rgb(1.0, 0.0, 0.0);
        let top_right = Color::rgb(0.0, 1.0, 0.0);
        let bottom_left = Color::rgb(0.0, 0.0, 1.0);
        let bottom_right = Color::rgb(1.0, 1.0, 1.0);

        source.set_pixel(0, 0, top_left);
        source.set_pixel(1, 0, top_right);
        source.set_pixel(0, 1, bottom_left);
        source.set_pixel(1, 1, bottom_right);

        output.copy_scaled_nearest_from(&source);

        assert_eq!(output.pixels()[0], top_left.to_u32());
        assert_eq!(output.pixels()[3], top_right.to_u32());
        assert_eq!(output.pixels()[12], bottom_left.to_u32());
        assert_eq!(output.pixels()[15], bottom_right.to_u32());
    }

    #[test]
    fn nearest_neighbor_scaling_supports_odd_dimensions() {
        let mut source = Framebuffer::new(3, 3);
        let mut output = Framebuffer::new(5, 7);
        let center = Color::rgb(0.4, 0.6, 0.8);

        source.set_pixel(1, 1, center);
        output.copy_scaled_nearest_from(&source);

        assert_eq!(output.pixels()[3 * output.width() + 2], center.to_u32());
        assert_eq!(output.width(), 5);
        assert_eq!(output.height(), 7);
        assert_eq!(output.pixels().len(), 35);
    }

    #[test]
    fn nearest_neighbor_scaling_handles_smaller_destination() {
        let mut source = Framebuffer::new(5, 5);
        let mut output = Framebuffer::new(3, 3);
        let color = Color::rgb(0.9, 0.2, 0.1);

        source.set_pixel(3, 3, color);
        output.copy_scaled_nearest_from(&source);

        assert_eq!(output.pixels()[2 * output.width() + 2], color.to_u32());
    }

    #[test]
    fn nearest_neighbor_scaling_keeps_visible_dimensions() {
        let source = Framebuffer::new(2, 3);
        let mut output = Framebuffer::new(7, 5);

        output.copy_scaled_nearest_from(&source);

        assert_eq!(output.width(), 7);
        assert_eq!(output.height(), 5);
        assert_eq!(output.pixels().len(), 35);
    }

    #[test]
    fn scaled_preview_pixels_remain_valid_minifb_colors() {
        let mut source = Framebuffer::new(2, 1);
        let mut output = Framebuffer::new(5, 3);

        source.set_pixel(0, 0, Color::rgb(1.0, 0.5, 0.0));
        source.set_pixel(1, 0, Color::rgb(0.0, 0.5, 1.0));
        output.copy_scaled_nearest_from(&source);

        assert!(output.pixels().iter().all(|&pixel| pixel <= 0x00ff_ffff));
    }
}
