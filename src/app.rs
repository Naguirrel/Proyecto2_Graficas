use minifb::{Key, Window, WindowOptions};

use crate::{framebuffer::Framebuffer, renderer};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window = Window::new(
        "Diorama Raytracing - Iluminacion",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )?;

    window.set_target_fps(60);
    renderer::render_background(&mut framebuffer);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(framebuffer.pixels(), WIDTH, HEIGHT)?;
    }

    Ok(())
}
