use crate::{color::Color, framebuffer::Framebuffer};

pub fn render_background(framebuffer: &mut Framebuffer) {
    let top = Color::new(0.03, 0.04, 0.07);
    let bottom = Color::new(0.18, 0.02, 0.03);

    for y in 0..framebuffer.height() {
        let t = y as f32 / (framebuffer.height() - 1) as f32;
        let color = top.lerp(bottom, t);

        for x in 0..framebuffer.width() {
            framebuffer.set_pixel(x, y, color);
        }
    }
}
