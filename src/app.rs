use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Instant;

use crate::{
    camera::{CameraInput, OrbitCamera},
    framebuffer::Framebuffer,
    math::Vec3,
    renderer,
};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const INITIAL_CAMERA_TARGET: Vec3 = Vec3::new(0.0, -0.25, 0.0);
const INITIAL_CAMERA_YAW: f32 = 0.60;
const INITIAL_CAMERA_PITCH: f32 = 0.40;
const INITIAL_CAMERA_DISTANCE: f32 = 7.30;
const INITIAL_CAMERA_FOV_DEGREES: f32 = 55.0;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let mut orbit_camera = OrbitCamera::new(
        INITIAL_CAMERA_TARGET,
        INITIAL_CAMERA_YAW,
        INITIAL_CAMERA_PITCH,
        INITIAL_CAMERA_DISTANCE,
        INITIAL_CAMERA_FOV_DEGREES,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    );
    let scene = renderer::sample_scene();
    let mut window = Window::new(
        "Diorama Raytracing - Camara orbital",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )?;

    window.set_target_fps(60);
    print_controls();
    let mut camera = orbit_camera.to_camera();
    renderer::render_scene(&mut framebuffer, &camera, &scene);
    let mut last_frame = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta_seconds = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        if orbit_camera.update(read_camera_input(&window), delta_seconds) {
            camera = orbit_camera.to_camera();
            renderer::render_scene(&mut framebuffer, &camera, &scene);
        }

        window.update_with_buffer(framebuffer.pixels(), WIDTH, HEIGHT)?;
    }

    Ok(())
}

fn read_camera_input(window: &Window) -> CameraInput {
    CameraInput {
        rotate_left: window.is_key_down(Key::A),
        rotate_right: window.is_key_down(Key::D),
        rotate_up: window.is_key_down(Key::W),
        rotate_down: window.is_key_down(Key::S),
        zoom_in: window.is_key_down(Key::Q),
        zoom_out: window.is_key_down(Key::E),
        reset: window.is_key_pressed(Key::R, KeyRepeat::No),
        scroll_zoom: window
            .get_scroll_wheel()
            .map(|(_, scroll_y)| scroll_y)
            .unwrap_or(0.0),
    }
}

fn print_controls() {
    println!("Controles:");
    println!("  W/S: inclinación");
    println!("  A/D: rotación");
    println!("  Q/E: zoom");
    println!("  R: reiniciar");
    println!("  Escape: salir");
}
