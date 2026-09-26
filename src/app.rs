use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use std::time::{Duration, Instant};

use crate::{
    camera::{CameraInput, OrbitCamera},
    framebuffer::Framebuffer,
    math::Vec3,
    ray::Ray,
    renderer,
    scene::Scene,
    space::{self, PlanetType, SceneState, SelectorWorld},
};

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
    let mut scene_state = SceneState::Galaxy;
    let mut orbit_camera = space::galaxy_selector_orbit_camera(aspect_ratio);
    let mut scene = space::build_galaxy_selector_scene()?;
    let mut window = Window::new(
        space::SPACE_WORLDS_WINDOW_TITLE,
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
    let mut left_mouse_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta_seconds = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        if scene_state != SceneState::Galaxy && window.is_key_pressed(Key::Backspace, KeyRepeat::No)
        {
            scene_state = SceneState::Galaxy;
            scene = space::build_galaxy_selector_scene()?;
            orbit_camera = space::galaxy_selector_orbit_camera(aspect_ratio);
            camera = orbit_camera.to_camera();
            render_state.mark_scene_changed();
            println!("Regresando al selector de mundos");
        }

        if orbit_camera.update(read_camera_input(&window), delta_seconds) {
            camera = orbit_camera.to_camera();
            render_state.mark_camera_changed(now);
        }

        let left_mouse_down = window.get_mouse_down(MouseButton::Left);
        if scene_state == SceneState::Galaxy
            && left_mouse_down
            && !left_mouse_was_down
            && let Some((mouse_x, mouse_y)) = window.get_mouse_pos(MouseMode::Discard)
            && let Some((pixel_x, pixel_y)) =
                mouse_to_framebuffer_pixel(mouse_x, mouse_y, WIDTH, HEIGHT, WIDTH, HEIGHT)
        {
            let ray = camera.ray_for_pixel(pixel_x, pixel_y, WIDTH, HEIGHT);

            if let Some(planet) = pick_selector_world(&ray, &space::galaxy_selector_worlds()) {
                scene_state = SceneState::Planet(planet);
                scene = build_planet_scene(planet)?;
                orbit_camera = planet_orbit_camera(planet, aspect_ratio);
                camera = orbit_camera.to_camera();
                render_state.mark_scene_changed();
                println!("Mundo seleccionado: {}", planet_label(planet));
            }
        }
        left_mouse_was_down = left_mouse_down;

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
    println!("  Click izquierdo: seleccionar mundo en el selector");
    println!("  W/S: inclinación");
    println!("  A/D: rotación");
    println!("  Q/E: zoom");
    println!("  R: reiniciar");
    println!("  Backspace: volver al selector");
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

    fn mark_scene_changed(&mut self) {
        self.scene_dirty = true;
        self.last_interaction = None;
        self.interactive_mode = false;
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

fn build_planet_scene(planet: PlanetType) -> Result<Scene, space::SpaceBuildError> {
    match planet {
        PlanetType::BlueMoon => space::build_blue_moon_scene(),
        PlanetType::CookieWorld => space::build_cookie_world_scene(),
    }
}

fn planet_orbit_camera(planet: PlanetType, aspect_ratio: f32) -> OrbitCamera {
    match planet {
        PlanetType::BlueMoon => space::blue_moon_orbit_camera(aspect_ratio),
        PlanetType::CookieWorld => space::cookie_world_orbit_camera(aspect_ratio),
    }
}

fn planet_label(planet: PlanetType) -> &'static str {
    match planet {
        PlanetType::BlueMoon => "Luna Azul",
        PlanetType::CookieWorld => "Planeta galleta",
    }
}

fn mouse_to_framebuffer_pixel(
    mouse_x: f32,
    mouse_y: f32,
    window_width: usize,
    window_height: usize,
    framebuffer_width: usize,
    framebuffer_height: usize,
) -> Option<(usize, usize)> {
    if !mouse_x.is_finite()
        || !mouse_y.is_finite()
        || window_width == 0
        || window_height == 0
        || framebuffer_width == 0
        || framebuffer_height == 0
        || mouse_x < 0.0
        || mouse_y < 0.0
        || mouse_x >= window_width as f32
        || mouse_y >= window_height as f32
    {
        return None;
    }

    let pixel_x = ((mouse_x / window_width as f32) * framebuffer_width as f32).floor() as usize;
    let pixel_y = ((mouse_y / window_height as f32) * framebuffer_height as f32).floor() as usize;

    Some((
        pixel_x.min(framebuffer_width - 1),
        pixel_y.min(framebuffer_height - 1),
    ))
}

fn pick_selector_world(ray: &Ray, worlds: &[SelectorWorld]) -> Option<PlanetType> {
    worlds
        .iter()
        .filter_map(|world| {
            ray_sphere_distance(ray, world.center, world.radius)
                .map(|distance| (world.planet, distance))
        })
        .min_by(|(_, left), (_, right)| {
            left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(planet, _)| planet)
}

fn ray_sphere_distance(ray: &Ray, center: Vec3, radius: f32) -> Option<f32> {
    if !radius.is_finite()
        || radius <= 0.0
        || !is_finite_vec3(center)
        || !is_finite_vec3(ray.origin)
        || !is_finite_vec3(ray.direction)
        || ray.direction == Vec3::ZERO
    {
        return None;
    }

    let oc = ray.origin - center;
    let a = ray.direction.dot(ray.direction);
    let half_b = oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = half_b * half_b - a * c;

    if !a.is_finite()
        || a <= 0.0001
        || !half_b.is_finite()
        || !c.is_finite()
        || !discriminant.is_finite()
        || discriminant < 0.0
    {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let near = (-half_b - sqrt_discriminant) / a;
    let far = (-half_b + sqrt_discriminant) / a;

    if near.is_finite() && near >= 0.001 {
        Some(near)
    } else if far.is_finite() && far >= 0.001 {
        Some(far)
    } else {
        None
    }
}

fn is_finite_vec3(vector: Vec3) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{
        FULL_QUALITY_DELAY, INTERACTIVE_SCALE, InteractiveRenderState, RenderQuality,
        interactive_dimensions, mouse_to_framebuffer_pixel, pick_selector_world,
    };
    use crate::{
        camera::{Camera, CameraInput, OrbitCamera},
        math::Vec3,
        ray::Ray,
        space::{PlanetType, SelectorWorld, galaxy_selector_worlds},
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
    fn scene_change_requests_full_quality_render() {
        let mut state = clean_state();
        let now = Instant::now();

        state.mark_scene_changed();

        assert!(state.scene_dirty);
        assert!(!state.interactive_mode);
        assert!(state.full_quality_pending);
        assert_eq!(
            state.next_render(now, FULL_QUALITY_DELAY),
            Some(RenderQuality::Full)
        );
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
    fn mouse_coordinates_map_to_framebuffer_pixels() {
        assert_eq!(
            mouse_to_framebuffer_pixel(400.0, 300.0, 800, 600, 400, 300),
            Some((200, 150))
        );
        assert_eq!(
            mouse_to_framebuffer_pixel(799.9, 599.9, 800, 600, 400, 300),
            Some((399, 299))
        );
    }

    #[test]
    fn mouse_coordinates_outside_window_are_ignored() {
        assert_eq!(
            mouse_to_framebuffer_pixel(-1.0, 10.0, 800, 600, 800, 600),
            None
        );
        assert_eq!(
            mouse_to_framebuffer_pixel(10.0, 600.0, 800, 600, 800, 600),
            None
        );
        assert_eq!(
            mouse_to_framebuffer_pixel(f32::NAN, 10.0, 800, 600, 800, 600),
            None
        );
    }

    #[test]
    fn ray_picking_selects_blue_moon_when_ray_points_to_blue_selector() {
        let worlds = galaxy_selector_worlds();
        let blue = worlds
            .iter()
            .find(|world| world.planet == PlanetType::BlueMoon)
            .unwrap();
        let ray = Ray::new(
            Vec3::new(blue.center.x, blue.center.y, 8.0),
            blue.center - Vec3::new(blue.center.x, blue.center.y, 8.0),
        );

        assert_eq!(
            pick_selector_world(&ray, &worlds),
            Some(PlanetType::BlueMoon)
        );
    }

    #[test]
    fn ray_picking_selects_cookie_world_when_ray_points_to_cookie_selector() {
        let worlds = galaxy_selector_worlds();
        let cookie = worlds
            .iter()
            .find(|world| world.planet == PlanetType::CookieWorld)
            .unwrap();
        let ray = Ray::new(
            Vec3::new(cookie.center.x, cookie.center.y, 8.0),
            cookie.center - Vec3::new(cookie.center.x, cookie.center.y, 8.0),
        );

        assert_eq!(
            pick_selector_world(&ray, &worlds),
            Some(PlanetType::CookieWorld)
        );
    }

    #[test]
    fn ray_picking_returns_none_when_ray_misses_selector_worlds() {
        let ray = Ray::new(Vec3::new(0.0, 5.0, 8.0), Vec3::new(0.0, 1.0, 0.0));

        assert_eq!(pick_selector_world(&ray, &galaxy_selector_worlds()), None);
    }

    #[test]
    fn ray_picking_uses_nearest_selector_hit() {
        let worlds = [
            SelectorWorld {
                planet: PlanetType::CookieWorld,
                center: Vec3::new(0.0, 0.0, -4.0),
                radius: 1.0,
            },
            SelectorWorld {
                planet: PlanetType::BlueMoon,
                center: Vec3::new(0.0, 0.0, -2.0),
                radius: 1.0,
            },
        ];
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

        assert_eq!(
            pick_selector_world(&ray, &worlds),
            Some(PlanetType::BlueMoon)
        );
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
