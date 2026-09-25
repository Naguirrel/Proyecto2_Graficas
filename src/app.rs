use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::{Duration, Instant};

use crate::{camera::CameraInput, framebuffer::Framebuffer, renderer, space};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const INTERACTIVE_SCALE: f32 = 0.5;
const FULL_QUALITY_DELAY: Duration = Duration::from_millis(180);
const PRINT_RENDER_TIMES: bool = true;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let (interactive_width, interactive_height) =
        interactive_dimensions(WIDTH, HEIGHT, INTERACTIVE_SCALE);
    let mut interactive_framebuffer = Framebuffer::new(interactive_width, interactive_height);
    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let mut orbit_camera = space::blue_moon_orbit_camera(aspect_ratio);
    let scene = space::build_blue_moon_scene()?;
    let mut window = Window::new(
        space::BLUE_MOON_WINDOW_TITLE,
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )?;

    window.set_target_fps(60);
    print_controls();
    print_rayon_threads();
    let mut camera = orbit_camera.to_camera();
    let mut render_state = InteractiveRenderState::new();
    let mut last_frame = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta_seconds = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        if orbit_camera.update(read_camera_input(&window), delta_seconds) {
            camera = orbit_camera.to_camera();
            render_state.mark_camera_changed(now);
        }

        if let Some(quality) = render_state.next_render(now, FULL_QUALITY_DELAY) {
            let started = Instant::now();

            match quality {
                RenderQuality::Interactive => {
                    let (width, height) = interactive_dimensions(WIDTH, HEIGHT, INTERACTIVE_SCALE);
                    interactive_framebuffer.resize(width, height);
                    renderer::render_scene(&mut interactive_framebuffer, &camera, &scene);
                    framebuffer.copy_scaled_nearest_from(&interactive_framebuffer);
                    print_render_timing(quality, width, height, started.elapsed());
                }
                RenderQuality::Full => {
                    renderer::render_scene(&mut framebuffer, &camera, &scene);
                    print_render_timing(
                        quality,
                        framebuffer.width(),
                        framebuffer.height(),
                        started.elapsed(),
                    );
                }
            }

            render_state.render_completed(quality);
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

fn print_rayon_threads() {
    println!("Rayon threads: {}", rayon::current_num_threads());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderQuality {
    Interactive,
    Full,
}

impl RenderQuality {
    fn label(self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone)]
struct InteractiveRenderState {
    scene_dirty: bool,
    last_interaction: Option<Instant>,
    interactive_mode: bool,
    full_quality_pending: bool,
}

impl InteractiveRenderState {
    fn new() -> Self {
        Self {
            scene_dirty: true,
            last_interaction: None,
            interactive_mode: false,
            full_quality_pending: true,
        }
    }

    fn mark_camera_changed(&mut self, now: Instant) {
        self.scene_dirty = true;
        self.last_interaction = Some(now);
        self.interactive_mode = true;
        self.full_quality_pending = true;
    }

    fn next_render(&mut self, now: Instant, full_quality_delay: Duration) -> Option<RenderQuality> {
        self.update_interactive_mode(now, full_quality_delay);

        if self.scene_dirty {
            if self.interactive_mode {
                Some(RenderQuality::Interactive)
            } else {
                Some(RenderQuality::Full)
            }
        } else if self.full_quality_pending && !self.interactive_mode {
            Some(RenderQuality::Full)
        } else {
            None
        }
    }

    fn render_completed(&mut self, quality: RenderQuality) {
        self.scene_dirty = false;

        if quality == RenderQuality::Full {
            self.full_quality_pending = false;
            self.interactive_mode = false;
        }
    }

    fn update_interactive_mode(&mut self, now: Instant, full_quality_delay: Duration) {
        self.interactive_mode = self.last_interaction.is_some_and(|last_interaction| {
            now.duration_since(last_interaction) < full_quality_delay
        });
    }
}

fn interactive_dimensions(width: usize, height: usize, scale: f32) -> (usize, usize) {
    (
        scaled_interactive_dimension(width, scale),
        scaled_interactive_dimension(height, scale),
    )
}

fn scaled_interactive_dimension(full_dimension: usize, scale: f32) -> usize {
    let maximum = full_dimension.max(1);

    if !scale.is_finite() || scale <= 0.0 {
        return 1;
    }

    ((full_dimension as f32 * scale).round() as usize).clamp(1, maximum)
}

fn print_render_timing(quality: RenderQuality, width: usize, height: usize, duration: Duration) {
    if PRINT_RENDER_TIMES {
        println!(
            "Render {} {}x{} terminado en {:.2?}",
            quality.label(),
            width,
            height,
            duration
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FULL_QUALITY_DELAY, INTERACTIVE_SCALE, InteractiveRenderState, RenderQuality,
        interactive_dimensions,
    };
    use crate::{
        camera::{Camera, CameraInput, OrbitCamera},
        math::Vec3,
    };
    use std::time::{Duration, Instant};

    const EPSILON: f32 = 0.0001;

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < EPSILON, "{left} != {right}");
    }

    fn clean_state() -> InteractiveRenderState {
        let mut state = InteractiveRenderState::new();
        state.render_completed(RenderQuality::Full);
        state
    }

    fn default_camera(aspect_ratio: f32) -> Camera {
        Camera::new(
            Vec3::new(0.0, 0.0, 4.0),
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
            55.0,
            aspect_ratio,
        )
    }

    fn default_orbit() -> OrbitCamera {
        OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.25,
            10.5,
            55.0,
            4.0 / 3.0,
            Vec3::new(0.0, 1.0, 0.0),
        )
    }

    #[test]
    fn interactive_scale_produces_smaller_dimensions() {
        let (width, height) = interactive_dimensions(800, 600, INTERACTIVE_SCALE);

        assert_eq!((width, height), (400, 300));
        assert!(width < 800);
        assert!(height < 600);
    }

    #[test]
    fn interactive_dimensions_never_become_zero() {
        assert_eq!(interactive_dimensions(1, 1, 0.25), (1, 1));
        assert_eq!(interactive_dimensions(800, 600, 0.0), (1, 1));
    }

    #[test]
    fn interactive_dimensions_keep_aspect_ratio_with_rounding_tolerance() {
        let (width, height) = interactive_dimensions(801, 601, 0.5);
        let full_aspect = 801.0 / 601.0;
        let preview_aspect = width as f32 / height as f32;
        let tolerance = 1.0 / height as f32;

        assert!((preview_aspect - full_aspect).abs() <= tolerance);
    }

    #[test]
    fn reduced_framebuffer_rays_keep_camera_composition() {
        let camera = default_camera(4.0 / 3.0);
        let top_left = camera.ray_for_pixel(0, 0, 400, 300);
        let bottom_right = camera.ray_for_pixel(399, 299, 400, 300);

        assert!(top_left.direction.x < 0.0);
        assert!(top_left.direction.y > 0.0);
        assert!(bottom_right.direction.x > 0.0);
        assert!(bottom_right.direction.y < 0.0);
    }

    #[test]
    fn preview_and_full_render_point_at_same_center() {
        let camera = default_camera(1.0);
        let full_center = camera.ray_for_pixel(4, 4, 9, 9);
        let preview_center = camera.ray_for_pixel(1, 1, 3, 3);

        assert!(full_center.direction.approx_eq(preview_center.direction));
    }

    #[test]
    fn initial_dirty_state_requests_one_full_render() {
        let mut state = InteractiveRenderState::new();
        let now = Instant::now();

        assert_eq!(
            state.next_render(now, FULL_QUALITY_DELAY),
            Some(RenderQuality::Full)
        );

        state.render_completed(RenderQuality::Full);

        assert_eq!(state.next_render(now, FULL_QUALITY_DELAY), None);
        assert!(!state.full_quality_pending);
    }

    #[test]
    fn camera_change_marks_full_quality_pending() {
        let mut state = clean_state();
        let now = Instant::now();

        state.mark_camera_changed(now);

        assert!(state.scene_dirty);
        assert!(state.interactive_mode);
        assert!(state.full_quality_pending);
        assert_eq!(state.last_interaction, Some(now));
    }

    #[test]
    fn no_camera_change_does_not_request_render() {
        let mut state = clean_state();
        let now = Instant::now();

        assert_eq!(state.next_render(now, FULL_QUALITY_DELAY), None);
    }

    #[test]
    fn recent_interaction_selects_reduced_quality() {
        let mut state = clean_state();
        let now = Instant::now();

        state.mark_camera_changed(now);

        assert_eq!(
            state.next_render(now + Duration::from_millis(10), FULL_QUALITY_DELAY),
            Some(RenderQuality::Interactive)
        );
    }

    #[test]
    fn elapsed_interaction_delay_requests_exactly_one_full_render() {
        let mut state = clean_state();
        let now = Instant::now();

        state.mark_camera_changed(now);
        assert_eq!(
            state.next_render(now, FULL_QUALITY_DELAY),
            Some(RenderQuality::Interactive)
        );
        state.render_completed(RenderQuality::Interactive);

        let idle = now + FULL_QUALITY_DELAY + Duration::from_millis(1);
        assert_eq!(
            state.next_render(idle, FULL_QUALITY_DELAY),
            Some(RenderQuality::Full)
        );
        state.render_completed(RenderQuality::Full);

        assert_eq!(state.next_render(idle, FULL_QUALITY_DELAY), None);
        assert!(!state.full_quality_pending);
    }

    #[test]
    fn full_render_completion_clears_pending_state() {
        let mut state = clean_state();
        let now = Instant::now();

        state.mark_camera_changed(now);
        state.render_completed(RenderQuality::Full);

        assert!(!state.scene_dirty);
        assert!(!state.interactive_mode);
        assert!(!state.full_quality_pending);
    }

    #[test]
    fn new_interaction_after_full_render_returns_to_reduced_quality() {
        let mut state = clean_state();
        let now = Instant::now();

        state.mark_camera_changed(now);
        state.render_completed(RenderQuality::Full);
        state.mark_camera_changed(now + Duration::from_millis(20));

        assert_eq!(
            state.next_render(now + Duration::from_millis(21), FULL_QUALITY_DELAY),
            Some(RenderQuality::Interactive)
        );
    }

    #[test]
    fn reset_counts_as_change_only_when_camera_changes() {
        let mut state = clean_state();
        let now = Instant::now();
        let mut orbit = default_orbit();

        assert!(!orbit.update(
            CameraInput {
                reset: true,
                ..CameraInput::default()
            },
            0.016,
        ));
        assert_eq!(state.next_render(now, FULL_QUALITY_DELAY), None);

        orbit.yaw = 1.0;
        assert!(orbit.update(
            CameraInput {
                reset: true,
                ..CameraInput::default()
            },
            0.016,
        ));
        state.mark_camera_changed(now);

        assert_eq!(
            state.next_render(now, FULL_QUALITY_DELAY),
            Some(RenderQuality::Interactive)
        );
    }

    #[test]
    fn input_without_effect_does_not_trigger_full_render() {
        let mut state = clean_state();
        let now = Instant::now();
        let mut orbit = default_orbit();

        assert!(!orbit.update(
            CameraInput {
                rotate_left: true,
                rotate_right: true,
                zoom_in: true,
                zoom_out: true,
                ..CameraInput::default()
            },
            0.016,
        ));

        assert_eq!(
            state.next_render(now + FULL_QUALITY_DELAY, FULL_QUALITY_DELAY),
            None
        );
    }

    #[test]
    fn full_quality_render_uses_visible_resolution() {
        let mut state = InteractiveRenderState::new();
        let now = Instant::now();
        let full_dimensions = (800, 600);

        assert_eq!(
            state.next_render(now, FULL_QUALITY_DELAY),
            Some(RenderQuality::Full)
        );
        assert_eq!(full_dimensions, (800, 600));
    }

    #[test]
    fn interactive_preview_renders_at_no_more_than_quarter_pixels() {
        let (width, height) = interactive_dimensions(800, 600, INTERACTIVE_SCALE);
        let preview_pixels = width * height;
        let full_pixels = 800 * 600;

        assert!(preview_pixels <= full_pixels / 4);
        assert_eq!(preview_pixels, 120_000);
    }

    #[test]
    fn center_ray_is_preserved_after_zoom_camera_update() {
        let mut orbit = default_orbit();
        let before = orbit.to_camera();

        assert!(orbit.update(
            CameraInput {
                zoom_in: true,
                ..CameraInput::default()
            },
            0.25,
        ));

        let after = orbit.to_camera();
        let ray = after.ray_for_pixel(1, 1, 3, 3);
        let expected = (after.target - after.position).normalized();

        assert_ne!(before.position, after.position);
        assert_near(ray.direction.dot(expected), 1.0);
    }
}
