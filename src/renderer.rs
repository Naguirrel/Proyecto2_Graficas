use crate::{camera::Camera, color::Color, framebuffer::Framebuffer, math::Vec3};

pub fn render_background(framebuffer: &mut Framebuffer) {
    let aspect_ratio = framebuffer.width() as f32 / framebuffer.height().max(1) as f32;
    let camera = Camera::new(
        Vec3::new(0.0, 1.2, 4.0),
        Vec3::new(0.0, 0.8, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        aspect_ratio,
    );

    framebuffer.clear(Color::BLACK);

    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            let ray = camera.ray_for_pixel(x, y, framebuffer.width(), framebuffer.height());
            let color = ray_direction_to_color(ray.direction);

            framebuffer.set_pixel(x, y, color);
        }
    }
}

fn ray_direction_to_color(direction: Vec3) -> Color {
    Color::new(
        direction.x * 0.5 + 0.5,
        direction.y * 0.5 + 0.5,
        direction.z * 0.5 + 0.5,
    )
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
