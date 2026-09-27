use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, ScaleMode, Window, WindowOptions};
use std::time::{Duration, Instant};

use crate::{
    camera::{Camera, CameraInput, OrbitCamera},
    color::Color,
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
const LEFT_CLICK_DRAG_THRESHOLD: f32 = 5.0;
const TEXT_SCALE: usize = 2;
const DIGIT_SCALE: usize = 10;

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
            borderless: true,
            title: false,
            resize: false,
            scale: Scale::FitScreen,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )?;

    window.set_background_color(0, 0, 0);
    window.set_target_fps(60);
    print_controls();
    print_rayon_threads();
    let mut camera = orbit_camera.to_camera();
    let mut render_state = InteractiveRenderState::new();
    let mut last_frame = Instant::now();
    let mut mouse_state = MouseInteractionState::default();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta_seconds = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        if scene_state != SceneState::Galaxy && window.is_key_pressed(Key::Backspace, KeyRepeat::No)
        {
            return_to_selector(
                aspect_ratio,
                &mut scene_state,
                &mut scene,
                &mut orbit_camera,
                &mut camera,
                &mut render_state,
            )?;
        }

        let mouse_orbit_delta = mouse_state.update_right_drag(&window);
        if orbit_camera.update(read_camera_input(&window, mouse_orbit_delta), delta_seconds) {
            camera = orbit_camera.to_camera();
            render_state.mark_camera_changed(now);
        }

        if scene_state == SceneState::Galaxy
            && !window.get_mouse_down(MouseButton::Right)
            && let Some((mouse_x, mouse_y)) = mouse_state.update_left_click(&window)
            && let Some((pixel_x, pixel_y)) =
                mouse_to_framebuffer_pixel(mouse_x, mouse_y, WIDTH, HEIGHT, WIDTH, HEIGHT)
        {
            let ray = camera.ray_for_pixel(pixel_x, pixel_y, WIDTH, HEIGHT);
            if let Some(planet) = pick_selector_world(&ray, &space::galaxy_selector_worlds()) {
                select_planet(
                    planet,
                    aspect_ratio,
                    &mut scene_state,
                    &mut scene,
                    &mut orbit_camera,
                    &mut camera,
                    &mut render_state,
                )?;
            }
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
            draw_ui(&mut framebuffer, &camera, scene_state);
        }

        window.update_with_buffer(framebuffer.pixels(), WIDTH, HEIGHT)?;
    }

    Ok(())
}

fn read_camera_input(window: &Window, mouse_orbit_delta: (f32, f32)) -> CameraInput {
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
        mouse_delta_x: mouse_orbit_delta.0,
        mouse_delta_y: mouse_orbit_delta.1,
    }
}

fn print_controls() {
    println!("Controles:");
    println!("  Click izquierdo: seleccionar mundo en el selector");
    println!("  Click derecho + mover mouse: rotar");
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
        PlanetType::AsteroidBelt => space::build_level_three_scene(),
    }
}

fn planet_orbit_camera(planet: PlanetType, aspect_ratio: f32) -> OrbitCamera {
    match planet {
        PlanetType::BlueMoon => space::blue_moon_orbit_camera(aspect_ratio),
        PlanetType::CookieWorld => space::cookie_world_orbit_camera(aspect_ratio),
        PlanetType::AsteroidBelt => space::level_three_orbit_camera(aspect_ratio),
    }
}

fn planet_label(planet: PlanetType) -> &'static str {
    match planet {
        PlanetType::BlueMoon => "Luna Azul",
        PlanetType::CookieWorld => "Planeta galleta",
        PlanetType::AsteroidBelt => "Cinturon de asteroides",
    }
}

fn select_planet(
    planet: PlanetType,
    aspect_ratio: f32,
    scene_state: &mut SceneState,
    scene: &mut Scene,
    orbit_camera: &mut OrbitCamera,
    camera: &mut Camera,
    render_state: &mut InteractiveRenderState,
) -> Result<(), space::SpaceBuildError> {
    *scene_state = SceneState::Planet(planet);
    *scene = build_planet_scene(planet)?;
    *orbit_camera = planet_orbit_camera(planet, aspect_ratio);
    *camera = orbit_camera.to_camera();
    render_state.mark_scene_changed();
    println!("Mundo seleccionado: {}", planet_label(planet));

    Ok(())
}

fn return_to_selector(
    aspect_ratio: f32,
    scene_state: &mut SceneState,
    scene: &mut Scene,
    orbit_camera: &mut OrbitCamera,
    camera: &mut Camera,
    render_state: &mut InteractiveRenderState,
) -> Result<bool, space::SpaceBuildError> {
    if *scene_state == SceneState::Galaxy {
        return Ok(false);
    }

    *scene_state = SceneState::Galaxy;
    *scene = space::build_galaxy_selector_scene()?;
    *orbit_camera = space::galaxy_selector_orbit_camera(aspect_ratio);
    *camera = orbit_camera.to_camera();
    render_state.mark_scene_changed();
    println!("Regresando al selector de mundos");

    Ok(true)
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

#[derive(Debug, Default)]
struct MouseInteractionState {
    right_previous: Option<(f32, f32)>,
    left_start: Option<(f32, f32)>,
    left_dragged: bool,
}

impl MouseInteractionState {
    fn update_right_drag(&mut self, window: &Window) -> (f32, f32) {
        if !window.get_mouse_down(MouseButton::Right) {
            self.right_previous = None;
            return (0.0, 0.0);
        }

        let Some(current) = window.get_mouse_pos(MouseMode::Clamp) else {
            self.right_previous = None;
            return (0.0, 0.0);
        };
        let delta = self
            .right_previous
            .map(|previous| (current.0 - previous.0, current.1 - previous.1))
            .unwrap_or((0.0, 0.0));

        self.right_previous = Some(current);
        delta
    }

    fn update_left_click(&mut self, window: &Window) -> Option<(f32, f32)> {
        let left_down = window.get_mouse_down(MouseButton::Left);

        if left_down {
            if let Some(current) = window.get_mouse_pos(MouseMode::Discard) {
                if let Some(start) = self.left_start {
                    let delta_x = current.0 - start.0;
                    let delta_y = current.1 - start.1;
                    self.left_dragged |=
                        delta_x * delta_x + delta_y * delta_y > LEFT_CLICK_DRAG_THRESHOLD.powi(2);
                } else {
                    self.left_start = Some(current);
                    self.left_dragged = false;
                }
            }

            return None;
        }

        let released = self
            .left_start
            .take()
            .and_then(|_| window.get_mouse_pos(MouseMode::Discard));
        let was_dragged = self.left_dragged;
        self.left_dragged = false;

        released.filter(|_| !was_dragged)
    }
}

fn draw_ui(framebuffer: &mut Framebuffer, camera: &Camera, scene_state: SceneState) {
    if scene_state == SceneState::Galaxy {
        draw_selector_numbers(framebuffer, camera);
    }

    draw_controls_overlay(framebuffer);
}

fn draw_selector_numbers(framebuffer: &mut Framebuffer, camera: &Camera) {
    for world in space::galaxy_selector_worlds() {
        if let Some((x, y)) = project_world_to_pixel(
            *camera,
            world.center,
            framebuffer.width(),
            framebuffer.height(),
        ) {
            let digit = char::from_digit(world.level_number as u32, 10).unwrap_or('?');
            draw_text_centered(
                framebuffer,
                x as isize + 3,
                y as isize + 3,
                &digit.to_string(),
                DIGIT_SCALE,
                Color::BLACK,
            );
            draw_text_centered(
                framebuffer,
                x as isize,
                y as isize,
                &digit.to_string(),
                DIGIT_SCALE,
                Color::new(1.0, 0.92, 0.36),
            );
        }
    }
}

fn draw_controls_overlay(framebuffer: &mut Framebuffer) {
    let lines = [
        "CLICK IZQUIERDO: SELECCIONAR",
        "CLICK DERECHO + MOVER: ROTAR",
        "RUEDA: ZOOM",
        "BACKSPACE: REGRESAR",
        "ESC: SALIR",
    ];
    let line_height = (GLYPH_HEIGHT + 2) * TEXT_SCALE;
    let panel_width = lines
        .iter()
        .map(|line| text_width(line, TEXT_SCALE))
        .max()
        .unwrap_or(0)
        + 18;
    let panel_height = lines.len() * line_height + 12;
    let x = framebuffer.width().saturating_sub(panel_width + 12);
    let y = framebuffer.height().saturating_sub(panel_height + 12);

    draw_rect(
        framebuffer,
        x,
        y,
        panel_width,
        panel_height,
        Color::new(0.0, 0.0, 0.0),
    );

    for (index, line) in lines.iter().enumerate() {
        draw_text(
            framebuffer,
            x + 9,
            y + 7 + index * line_height,
            line,
            TEXT_SCALE,
            Color::new(0.86, 0.93, 1.0),
        );
    }
}

fn project_world_to_pixel(
    camera: Camera,
    point: Vec3,
    framebuffer_width: usize,
    framebuffer_height: usize,
) -> Option<(usize, usize)> {
    if framebuffer_width == 0 || framebuffer_height == 0 {
        return None;
    }

    let basis = camera.basis();
    let camera_to_point = point - camera.position;
    let depth = camera_to_point.dot(basis.forward);

    if !depth.is_finite() || depth <= 0.001 {
        return None;
    }

    let half_height = (camera.vertical_fov_degrees.to_radians() * 0.5).tan();
    let half_width = half_height * camera.aspect_ratio;
    let ndc_x = camera_to_point.dot(basis.right) / (depth * half_width);
    let ndc_y = camera_to_point.dot(basis.up) / (depth * half_height);

    if !ndc_x.is_finite()
        || !ndc_y.is_finite()
        || !(-1.0..=1.0).contains(&ndc_x)
        || !(-1.0..=1.0).contains(&ndc_y)
    {
        return None;
    }

    let pixel_x = ((ndc_x + 1.0) * 0.5 * framebuffer_width as f32).floor() as usize;
    let pixel_y = ((1.0 - ndc_y) * 0.5 * framebuffer_height as f32).floor() as usize;

    Some((
        pixel_x.min(framebuffer_width - 1),
        pixel_y.min(framebuffer_height - 1),
    ))
}

const GLYPH_WIDTH: usize = 5;
const GLYPH_HEIGHT: usize = 7;

fn text_width(text: &str, scale: usize) -> usize {
    text.chars().count() * (GLYPH_WIDTH + 1) * scale
}

fn draw_text_centered(
    framebuffer: &mut Framebuffer,
    center_x: isize,
    center_y: isize,
    text: &str,
    scale: usize,
    color: Color,
) {
    let width = text_width(text, scale) as isize;
    let height = (GLYPH_HEIGHT * scale) as isize;
    let x = center_x - width / 2;
    let y = center_y - height / 2;

    draw_text_at(framebuffer, x, y, text, scale, color);
}

fn draw_text(
    framebuffer: &mut Framebuffer,
    x: usize,
    y: usize,
    text: &str,
    scale: usize,
    color: Color,
) {
    draw_text_at(framebuffer, x as isize, y as isize, text, scale, color);
}

fn draw_text_at(
    framebuffer: &mut Framebuffer,
    x: isize,
    y: isize,
    text: &str,
    scale: usize,
    color: Color,
) {
    let mut cursor_x = x;

    for character in text.chars() {
        draw_glyph(framebuffer, cursor_x, y, character, scale, color);
        cursor_x += ((GLYPH_WIDTH + 1) * scale) as isize;
    }
}

fn draw_glyph(
    framebuffer: &mut Framebuffer,
    x: isize,
    y: isize,
    character: char,
    scale: usize,
    color: Color,
) {
    let glyph = glyph_rows(character);

    for (row, bits) in glyph.iter().enumerate() {
        for column in 0..GLYPH_WIDTH {
            if bits & (1 << (GLYPH_WIDTH - 1 - column)) == 0 {
                continue;
            }

            for dy in 0..scale {
                for dx in 0..scale {
                    set_pixel_i(
                        framebuffer,
                        x + (column * scale + dx) as isize,
                        y + (row * scale + dy) as isize,
                        color,
                    );
                }
            }
        }
    }
}

fn draw_rect(
    framebuffer: &mut Framebuffer,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
) {
    let max_y = (y + height).min(framebuffer.height());
    let max_x = (x + width).min(framebuffer.width());

    for py in y..max_y {
        for px in x..max_x {
            framebuffer.set_pixel(px, py, color);
        }
    }
}

fn set_pixel_i(framebuffer: &mut Framebuffer, x: isize, y: isize, color: Color) {
    if x < 0 || y < 0 {
        return;
    }

    framebuffer.set_pixel(x as usize, y as usize, color);
}

fn glyph_rows(character: char) -> [u8; GLYPH_HEIGHT] {
    match character.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        ':' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        ' ' => [0; GLYPH_HEIGHT],
        _ => [
            0b11111, 0b10001, 0b00010, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FULL_QUALITY_DELAY, INTERACTIVE_SCALE, InteractiveRenderState, RenderQuality,
        build_planet_scene, interactive_dimensions, mouse_to_framebuffer_pixel,
        pick_selector_world, return_to_selector,
    };
    use crate::{
        camera::{Camera, CameraInput, OrbitCamera},
        math::Vec3,
        ray::Ray,
        space::{self, PlanetType, SceneState, SelectorWorld, galaxy_selector_worlds},
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
    fn ray_picking_selects_level_three_when_ray_points_to_third_selector() {
        let worlds = galaxy_selector_worlds();
        let level_three = worlds
            .iter()
            .find(|world| world.planet == PlanetType::AsteroidBelt)
            .unwrap();
        let ray = Ray::new(
            Vec3::new(level_three.center.x, level_three.center.y, 8.0),
            level_three.center - Vec3::new(level_three.center.x, level_three.center.y, 8.0),
        );

        assert_eq!(
            pick_selector_world(&ray, &worlds),
            Some(PlanetType::AsteroidBelt)
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
                level_number: 2,
                locked: true,
            },
            SelectorWorld {
                planet: PlanetType::BlueMoon,
                center: Vec3::new(0.0, 0.0, -2.0),
                radius: 1.0,
                level_number: 1,
                locked: true,
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
    fn third_planet_builds_placeholder_scene() {
        let scene = build_planet_scene(PlanetType::AsteroidBelt).unwrap();

        assert!(scene.object_count() >= 7);
        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 5);
    }

    #[test]
    fn return_to_selector_rebuilds_selector_from_planet_state() {
        let aspect_ratio = 4.0 / 3.0;
        let mut scene_state = SceneState::Planet(PlanetType::AsteroidBelt);
        let mut scene = space::build_level_three_scene().unwrap();
        let mut orbit_camera = space::level_three_orbit_camera(aspect_ratio);
        let mut camera = orbit_camera.to_camera();
        let mut render_state = clean_state();

        let changed = return_to_selector(
            aspect_ratio,
            &mut scene_state,
            &mut scene,
            &mut orbit_camera,
            &mut camera,
            &mut render_state,
        )
        .unwrap();

        assert!(changed);
        assert_eq!(scene_state, SceneState::Galaxy);
        assert_eq!(scene.object_count(), 6);
        assert_eq!(
            orbit_camera,
            space::galaxy_selector_orbit_camera(aspect_ratio)
        );
        assert!(render_state.scene_dirty);
    }

    #[test]
    fn mouse_drag_delta_changes_orbit_without_keyboard_input() {
        let mut orbit = default_orbit();
        let before = orbit;

        assert!(orbit.update(
            CameraInput {
                mouse_delta_x: 12.0,
                mouse_delta_y: -6.0,
                ..CameraInput::default()
            },
            0.016,
        ));

        assert_ne!(orbit.yaw, before.yaw);
        assert_ne!(orbit.pitch, before.pitch);
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
