use crate::{color::Color, framebuffer::Framebuffer};

pub fn render_background(framebuffer: &mut Framebuffer) {
    let ceiling = Color::new(0.02, 0.03, 0.06);
    let screen_glow = Color::new(0.20, 0.26, 0.34);
    let carpet = Color::new(0.22, 0.02, 0.04);

    framebuffer.clear(Color::BLACK);

    for y in 0..framebuffer.height() {
        let v = normalized_coordinate(y, framebuffer.height());

        for x in 0..framebuffer.width() {
            let u = normalized_coordinate(x, framebuffer.width());
            let center_distance = ((u - 0.5).abs() * 2.0).clamp(0.0, 1.0);
            let glow = 1.0 - center_distance;
            let vertical = ceiling.lerp(carpet, v);
            let color = vertical.lerp(screen_glow, glow * (1.0 - v) * 0.35);

            framebuffer.set_pixel(x, y, color);
        }
    }
}

fn normalized_coordinate(position: usize, size: usize) -> f32 {
    if size <= 1 {
        return 0.0;
    }

    position as f32 / (size - 1) as f32
}

#[cfg(test)]
mod tests {
    use super::render_background;
    use crate::{color::Color, framebuffer::Framebuffer};

    #[test]
    fn background_renderer_writes_visible_gradient() {
        let mut framebuffer = Framebuffer::new(4, 4);

        render_background(&mut framebuffer);

        assert_ne!(framebuffer.pixels()[0], Color::BLACK.to_u32());
        assert_ne!(framebuffer.pixels()[0], framebuffer.pixels()[15]);
    }
}
