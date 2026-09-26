use std::{error::Error, fmt};

use crate::{
    camera::OrbitCamera,
    color::Color,
    cone::{Cone, ConeError},
    cylinder::{Cylinder, CylinderError},
    light::PointLight,
    material::Material,
    math::{Vec2, Vec3},
    oriented_box::{OrientedBox, OrientedBoxError},
    radial::{RadialFrame, RadialFrameError},
    scene::{Scene, SceneError},
    skybox::Skybox,
    sphere::{Sphere, SphereError},
    texture::{Texture, TextureError, WrapMode},
};

pub const BLUE_MOON_WINDOW_TITLE: &str = "Angry Birds Space Diorama - Luna Azul";
pub const SPACE_WORLDS_WINDOW_TITLE: &str = "Angry Birds Space Diorama - Worlds";
pub const BLUE_MOON_PLANET_CENTER: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const BLUE_MOON_PLANET_RADIUS: f32 = 2.2;
pub const LAUNCH_ASTEROID_CENTER: Vec3 = Vec3::new(-3.8, -0.25, 1.25);
pub const LAUNCH_ASTEROID_RADIUS: f32 = 0.62;
pub const COOKIE_PLANET_CENTER: Vec3 = Vec3::new(4.45, -0.30, -0.25);
pub const COOKIE_PLANET_RADIUS: f32 = 1.45;
pub const SPACE_SUN_DIRECTION: Vec3 = Vec3::new(-0.76, 0.54, -0.36);
pub const BLUE_MOON_PIG_COUNT: usize = 3;
pub const BLUE_MOON_BIRD_COUNT: usize = 3;
pub const COOKIE_LEVEL_PIG_COUNT: usize = 2;
pub const GALAXY_SELECTOR_BLUE_MOON_CENTER: Vec3 = Vec3::new(-2.85, 0.26, 0.0);
pub const GALAXY_SELECTOR_BLUE_MOON_RADIUS: f32 = 0.98;
pub const GALAXY_SELECTOR_COOKIE_CENTER: Vec3 = Vec3::new(2.65, -0.20, -0.28);
pub const GALAXY_SELECTOR_COOKIE_RADIUS: f32 = 0.90;
const SPACE_SKYBOX_WIDTH: usize = 320;
const SPACE_SKYBOX_HEIGHT: usize = 160;
const SPACE_SUN_U: f32 = 0.125;
const SPACE_SUN_V: f32 = 0.770;
const SKYBOX_ASTEROID_A_U: f32 = 0.675;
const SKYBOX_ASTEROID_A_V: f32 = 0.620;
const SKYBOX_ASTEROID_B_U: f32 = 0.835;
const SKYBOX_ASTEROID_B_V: f32 = 0.455;
const SKYBOX_ASTEROID_C_U: f32 = 0.425;
const SKYBOX_ASTEROID_C_V: f32 = 0.730;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneState {
    Galaxy,
    Planet(PlanetType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetType {
    BlueMoon,
    CookieWorld,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectorWorld {
    pub planet: PlanetType,
    pub center: Vec3,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy)]
struct SkyboxAsteroid {
    center_u: f32,
    center_v: f32,
    radius: f32,
    stretch: f32,
    opacity: f32,
    seed: f32,
}

const BLUE_MOON_TEXTURE: &str = "\
P3
4 4
255
82 91 104   122 135 148  70 78 92    146 154 162
154 164 174  92 103 120   116 128 142  74 84 98
96 106 122   164 172 180  85 96 112    134 146 158
65 72 88     112 124 138  148 158 166  90 101 116
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceBuildError {
    Scene(SceneError),
    Texture(TextureError),
    Sphere(SphereError),
    Cylinder(CylinderError),
    Cone(ConeError),
    OrientedBox(OrientedBoxError),
    RadialFrame(RadialFrameError),
}

#[derive(Debug, Clone, Copy)]
struct SpaceMaterials {
    moon: usize,
    crater: usize,
    gravity_field: usize,
    cookie_gravity_field: usize,
    asteroid: usize,
    cookie: usize,
    chocolate: usize,
    wood: usize,
    ice: usize,
    metal: usize,
    tnt: usize,
    candy_red: usize,
    candy_blue: usize,
    candy_yellow: usize,
    pig: usize,
    snout: usize,
    eye: usize,
    pupil: usize,
    red_bird: usize,
    blue_bird: usize,
    yellow_bird: usize,
    beak: usize,
    slingshot: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SpaceSceneMetadata {
    pub planet_id: Option<usize>,
    pub gravity_field_id: Option<usize>,
    pub launch_asteroid_id: Option<usize>,
    pub cookie_planet_id: Option<usize>,
    pub cookie_gravity_field_id: Option<usize>,
    pub crater_count: usize,
    pub decorative_asteroid_count: usize,
    pub cookie_chocolate_chip_count: usize,
    pub candy_count: usize,
    pub pig_count: usize,
    pub cookie_pig_count: usize,
    pub bird_count: usize,
    pub slingshot_parts: usize,
    pub wood_parts: usize,
    pub ice_or_metal_parts: usize,
    pub tnt_parts: usize,
}

impl fmt::Display for SpaceBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scene(error) => write!(formatter, "scene build error: {error:?}"),
            Self::Texture(error) => write!(formatter, "texture build error: {error:?}"),
            Self::Sphere(error) => write!(formatter, "sphere build error: {error:?}"),
            Self::Cylinder(error) => write!(formatter, "cylinder build error: {error:?}"),
            Self::Cone(error) => write!(formatter, "cone build error: {error:?}"),
            Self::OrientedBox(error) => write!(formatter, "oriented box build error: {error:?}"),
            Self::RadialFrame(error) => write!(formatter, "radial frame build error: {error:?}"),
        }
    }
}

impl Error for SpaceBuildError {}

impl From<SceneError> for SpaceBuildError {
    fn from(error: SceneError) -> Self {
        Self::Scene(error)
    }
}

impl From<TextureError> for SpaceBuildError {
    fn from(error: TextureError) -> Self {
        Self::Texture(error)
    }
}

impl From<SphereError> for SpaceBuildError {
    fn from(error: SphereError) -> Self {
        Self::Sphere(error)
    }
}

impl From<CylinderError> for SpaceBuildError {
    fn from(error: CylinderError) -> Self {
        Self::Cylinder(error)
    }
}

impl From<ConeError> for SpaceBuildError {
    fn from(error: ConeError) -> Self {
        Self::Cone(error)
    }
}

impl From<OrientedBoxError> for SpaceBuildError {
    fn from(error: OrientedBoxError) -> Self {
        Self::OrientedBox(error)
    }
}

impl From<RadialFrameError> for SpaceBuildError {
    fn from(error: RadialFrameError) -> Self {
        Self::RadialFrame(error)
    }
}

pub fn build_blue_moon_scene() -> Result<Scene, SpaceBuildError> {
    build_blue_moon_scene_with_metadata().map(|built| built.0)
}

pub(crate) fn build_blue_moon_scene_with_metadata()
-> Result<(Scene, SpaceSceneMetadata), SpaceBuildError> {
    let (mut scene, materials) = base_space_scene()?;
    let mut metadata = SpaceSceneMetadata::default();

    add_blue_moon_world(&mut scene, &mut metadata, materials)?;
    add_space_lighting(&mut scene);

    Ok((scene, metadata))
}

pub fn build_cookie_world_scene() -> Result<Scene, SpaceBuildError> {
    build_cookie_world_scene_with_metadata().map(|built| built.0)
}

pub(crate) fn build_cookie_world_scene_with_metadata()
-> Result<(Scene, SpaceSceneMetadata), SpaceBuildError> {
    let (mut scene, materials) = base_space_scene()?;
    let mut metadata = SpaceSceneMetadata::default();

    add_cookie_level(&mut scene, &mut metadata, materials)?;
    add_space_lighting(&mut scene);
    add_cookie_level_lighting(&mut scene);

    Ok((scene, metadata))
}

pub fn build_galaxy_selector_scene() -> Result<Scene, SpaceBuildError> {
    let (mut scene, materials) = base_space_scene()?;

    scene.set_ambient_light(Color::new(0.300, 0.340, 0.405));
    add_selector_worlds(&mut scene, materials)?;
    add_selector_lighting(&mut scene);

    Ok(scene)
}

pub fn build_space_levels_scene() -> Result<Scene, SpaceBuildError> {
    build_space_levels_scene_with_metadata().map(|built| built.0)
}

pub(crate) fn build_space_levels_scene_with_metadata()
-> Result<(Scene, SpaceSceneMetadata), SpaceBuildError> {
    let (mut scene, materials) = base_space_scene()?;
    let mut metadata = SpaceSceneMetadata::default();

    add_blue_moon_world(&mut scene, &mut metadata, materials)?;
    add_cookie_level(&mut scene, &mut metadata, materials)?;
    add_space_lighting(&mut scene);
    add_cookie_level_lighting(&mut scene);

    Ok((scene, metadata))
}

pub fn blue_moon_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        Vec3::new(-0.75, 0.35, 0.70),
        -0.30,
        0.15,
        9.7,
        55.0,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

pub fn cookie_world_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        COOKIE_PLANET_CENTER + Vec3::new(-0.08, 0.18, 0.0),
        -0.30,
        0.16,
        7.5,
        55.0,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

pub fn galaxy_selector_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        Vec3::new(0.0, 0.0, -0.05),
        0.0,
        0.06,
        9.6,
        52.0,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

pub fn space_levels_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        Vec3::new(1.35, 0.20, 0.00),
        -0.14,
        0.12,
        13.8,
        58.0,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

pub fn galaxy_selector_worlds() -> [SelectorWorld; 2] {
    [
        SelectorWorld {
            planet: PlanetType::BlueMoon,
            center: GALAXY_SELECTOR_BLUE_MOON_CENTER,
            radius: GALAXY_SELECTOR_BLUE_MOON_RADIUS,
        },
        SelectorWorld {
            planet: PlanetType::CookieWorld,
            center: GALAXY_SELECTOR_COOKIE_CENTER,
            radius: GALAXY_SELECTOR_COOKIE_RADIUS,
        },
    ]
}

fn base_space_scene() -> Result<(Scene, SpaceMaterials), SpaceBuildError> {
    let mut scene = Scene::new();

    scene.set_ambient_light(Color::new(0.240, 0.280, 0.340));
    scene.set_skybox(angry_birds_space_skybox()?);
    let materials = register_space_materials(&mut scene)?;

    Ok((scene, materials))
}

fn angry_birds_space_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(space_skybox_texture()?)
        .with_intensity(1.18)
        .with_horizontal_rotation(0.0))
}

fn space_skybox_texture() -> Result<Texture, TextureError> {
    let mut pixels = Vec::with_capacity(SPACE_SKYBOX_WIDTH * SPACE_SKYBOX_HEIGHT);

    for y in 0..SPACE_SKYBOX_HEIGHT {
        let v = 1.0 - y as f32 / (SPACE_SKYBOX_HEIGHT - 1) as f32;

        for x in 0..SPACE_SKYBOX_WIDTH {
            let u = x as f32 / SPACE_SKYBOX_WIDTH as f32;
            pixels.push(space_skybox_color(u, v));
        }
    }

    Texture::new(SPACE_SKYBOX_WIDTH, SPACE_SKYBOX_HEIGHT, pixels)
}

fn space_skybox_color(u: f32, v: f32) -> Color {
    let bottom = Color::new(0.010, 0.045, 0.130);
    let top = Color::new(0.020, 0.095, 0.230);
    let lateral = Color::new(0.012, 0.070, 0.190);
    let mut color = bottom.lerp(top, smoothstep(0.0, 1.0, v));
    color = color.lerp(lateral, smoothstep(0.0, 1.0, 1.0 - u) * 0.26);

    color = add_cartoon_cloud_layers(color, u, v);

    let sun_distance = wrapped_uv_distance(u, v, SPACE_SUN_U, SPACE_SUN_V, 1.18);
    let broad_halo = 1.0 - smoothstep(0.13, 0.46, sun_distance);
    let warm_halo = 1.0 - smoothstep(0.065, 0.290, sun_distance);
    color += Color::new(0.070, 0.260, 0.310) * (broad_halo * 0.78);
    color += Color::new(1.000, 0.600, 0.150) * (warm_halo * 0.42);

    color = add_cartoon_sun(color, u, v, sun_distance);

    let mist = smooth_hash(u * 41.0 + 9.0, v * 29.0 + 5.0) * 0.010;
    color += Color::new(0.020, 0.070, 0.090) * mist;

    let star = star_strength(u, v);
    if star > 0.0 {
        color += Color::new(0.86, 0.96, 1.0) * (star * (1.0 - warm_halo * 0.65));
    }

    color = add_background_asteroids(color, u, v);

    color.clamped()
}

fn add_cartoon_sun(color: Color, u: f32, v: f32, sun_distance: f32) -> Color {
    let disk = 1.0 - smoothstep(0.066, 0.086, sun_distance);
    if disk <= 0.0 {
        return color;
    }

    let inner_glow = 1.0 - smoothstep(0.0, 0.078, sun_distance);
    let mut sun_color =
        Color::new(1.0, 0.720, 0.180).lerp(Color::new(1.0, 0.930, 0.520), inner_glow);
    let mottling = smooth_hash(u * 62.0 + 5.0, v * 51.0 + 17.0) * 0.12;
    sun_color += Color::new(0.060, 0.025, 0.0) * mottling;

    let spot_a = 1.0 - smoothstep(0.014, 0.034, wrapped_uv_distance(u, v, 0.100, 0.790, 1.35));
    let spot_b = 1.0 - smoothstep(0.011, 0.028, wrapped_uv_distance(u, v, 0.145, 0.735, 1.35));
    let spot_c = 1.0 - smoothstep(0.009, 0.023, wrapped_uv_distance(u, v, 0.170, 0.812, 1.35));
    let spots = (spot_a * 0.34 + spot_b * 0.26 + spot_c * 0.22).clamp(0.0, 0.45);
    sun_color = sun_color.lerp(Color::new(0.900, 0.430, 0.090), spots);

    color.lerp(sun_color, disk)
}

fn add_cartoon_cloud_layers(color: Color, u: f32, v: f32) -> Color {
    let lower_back = cloud_band(u, v, 0.220, 0.070, 0.038, 2.8, 0.25);
    let lower_front = cloud_band(u, v, 0.120, 0.052, 0.030, 4.0, 1.10);
    let left_side = side_cloud(u, v, 0.045, 0.55, 0.055, 5.2);
    let right_side = side_cloud(1.0 - u, v, 0.018, 0.45, 0.045, 3.5);

    let color = color.lerp(Color::new(0.030, 0.145, 0.295), lower_back * 0.52);
    let color = color.lerp(Color::new(0.020, 0.098, 0.245), lower_front * 0.68);
    let color = color.lerp(Color::new(0.020, 0.120, 0.290), left_side * 0.48);

    color.lerp(Color::new(0.014, 0.080, 0.210), right_side * 0.40)
}

fn cloud_band(
    u: f32,
    v: f32,
    base_v: f32,
    height: f32,
    softness: f32,
    frequency: f32,
    phase: f32,
) -> f32 {
    let wave = (u * frequency * std::f32::consts::TAU + phase).sin() * height
        + (u * (frequency * 1.8) * std::f32::consts::TAU + phase * 0.7).sin() * height * 0.45;
    let top_edge = base_v + wave;

    1.0 - smoothstep(top_edge - softness, top_edge + softness, v)
}

fn side_cloud(
    u_from_edge: f32,
    v: f32,
    width: f32,
    center_v: f32,
    softness: f32,
    frequency: f32,
) -> f32 {
    let edge = 1.0 - smoothstep(width, width + 0.16, u_from_edge);
    let vertical = 1.0
        - smoothstep(
            softness,
            softness + 0.30,
            (v - center_v - (u_from_edge * frequency).sin() * 0.05).abs(),
        );

    (edge * vertical * (1.0 - smoothstep(0.78, 1.0, v))).clamp(0.0, 1.0)
}

fn add_background_asteroids(color: Color, u: f32, v: f32) -> Color {
    [
        SkyboxAsteroid {
            center_u: SKYBOX_ASTEROID_A_U,
            center_v: SKYBOX_ASTEROID_A_V,
            radius: 0.054,
            stretch: 1.10,
            opacity: 0.80,
            seed: 3.0,
        },
        SkyboxAsteroid {
            center_u: SKYBOX_ASTEROID_B_U,
            center_v: SKYBOX_ASTEROID_B_V,
            radius: 0.036,
            stretch: 0.84,
            opacity: 0.58,
            seed: 7.0,
        },
        SkyboxAsteroid {
            center_u: SKYBOX_ASTEROID_C_U,
            center_v: SKYBOX_ASTEROID_C_V,
            radius: 0.030,
            stretch: 1.28,
            opacity: 0.46,
            seed: 13.0,
        },
    ]
    .into_iter()
    .fold(color, |color, asteroid| {
        add_background_asteroid(color, u, v, asteroid)
    })
}

fn add_background_asteroid(color: Color, u: f32, v: f32, asteroid: SkyboxAsteroid) -> Color {
    let du = ((u - asteroid.center_u + 0.5).rem_euclid(1.0) - 0.5) / asteroid.radius;
    let dv = (v - asteroid.center_v) / (asteroid.radius * asteroid.stretch);
    let angle = dv.atan2(du);
    let distance = (du * du + dv * dv).sqrt();
    let outline = 1.0
        + (angle * 3.0 + asteroid.seed).sin() * 0.16
        + (angle * 5.0 + asteroid.seed * 0.7).cos() * 0.11
        + (angle * 7.0 + asteroid.seed * 1.4).sin() * 0.07;
    let alpha = 1.0 - smoothstep(outline * 0.82, outline, distance);

    if alpha <= 0.0 {
        return color;
    }

    let highlight = (0.46 - du * 0.22 + dv * 0.18).clamp(0.0, 0.72);
    let mut asteroid_color =
        Color::new(0.225, 0.135, 0.430).lerp(Color::new(0.390, 0.255, 0.690), highlight);
    let crater_a = 1.0
        - smoothstep(
            0.08,
            0.24,
            ((du + 0.22).powi(2) + (dv - 0.08).powi(2)).sqrt(),
        );
    let crater_b = 1.0
        - smoothstep(
            0.06,
            0.18,
            ((du - 0.26).powi(2) + (dv + 0.20).powi(2)).sqrt(),
        );
    let crater_c = 1.0
        - smoothstep(
            0.05,
            0.16,
            ((du + 0.02).powi(2) + (dv + 0.30).powi(2)).sqrt(),
        );
    let crater = (crater_a * 0.32 + crater_b * 0.22 + crater_c * 0.18).clamp(0.0, 0.42);
    asteroid_color = asteroid_color.lerp(Color::new(0.110, 0.075, 0.250), crater);

    color.lerp(asteroid_color, alpha * asteroid.opacity)
}

fn wrapped_uv_distance(u: f32, v: f32, center_u: f32, center_v: f32, u_scale: f32) -> f32 {
    let du = ((u - center_u + 0.5).rem_euclid(1.0) - 0.5) * u_scale;
    let dv = v - center_v;
    (du * du + dv * dv).sqrt()
}

fn star_strength(u: f32, v: f32) -> f32 {
    let grid_x = u * 230.0;
    let grid_y = v * 126.0;
    let cell_x = grid_x.floor();
    let cell_y = grid_y.floor();
    let noise = smooth_hash(cell_x, cell_y);

    if noise > 0.978 {
        let local_x = grid_x.fract() - 0.5;
        let local_y = grid_y.fract() - 0.5;
        let distance = (local_x * local_x + local_y * local_y).sqrt();
        let radius = 0.14 + smooth_hash(cell_x + 17.0, cell_y + 31.0) * 0.09;
        let tint = smooth_hash(cell_x * 0.37 + 11.0, cell_y * 0.61 + 7.0);
        let core = 1.0 - smoothstep(radius * 0.35, radius, distance);
        let strength = ((noise - 0.978) / 0.022).clamp(0.0, 1.0);

        core * (0.42 + strength * (0.50 + tint * 0.15))
    } else {
        0.0
    }
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn smooth_hash(x: f32, y: f32) -> f32 {
    let value = (x * 12.9898 + y * 78.233).sin() * 43_758.547;
    value - value.floor()
}

fn add_selector_worlds(
    scene: &mut Scene,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    scene.add_sphere(Sphere::new(
        GALAXY_SELECTOR_BLUE_MOON_CENTER,
        GALAXY_SELECTOR_BLUE_MOON_RADIUS * 1.18,
        materials.gravity_field,
    )?)?;
    scene.add_sphere(Sphere::new(
        GALAXY_SELECTOR_BLUE_MOON_CENTER,
        GALAXY_SELECTOR_BLUE_MOON_RADIUS,
        materials.moon,
    )?)?;
    scene.add_sphere(Sphere::new(
        GALAXY_SELECTOR_COOKIE_CENTER,
        GALAXY_SELECTOR_COOKIE_RADIUS * 1.17,
        materials.cookie_gravity_field,
    )?)?;
    scene.add_sphere(Sphere::new(
        GALAXY_SELECTOR_COOKIE_CENTER,
        GALAXY_SELECTOR_COOKIE_RADIUS,
        materials.cookie,
    )?)?;

    Ok(())
}

fn add_blue_moon_world(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    add_planet(scene, metadata, materials)?;
    add_enemy_construction(scene, metadata, materials)?;
    add_pigs(scene, metadata, materials)?;
    add_launcher(scene, metadata, materials)?;
    add_blue_moon_debris(scene, metadata, materials)?;
    add_gravity_field(scene, metadata, materials)?;

    Ok(())
}

fn register_space_materials(scene: &mut Scene) -> Result<SpaceMaterials, SpaceBuildError> {
    let moon_texture = scene.add_texture(Texture::from_ppm_text(BLUE_MOON_TEXTURE)?);

    let moon = scene.add_material(
        Material::new(
            Color::new(0.68, 0.80, 1.0),
            0.20,
            28.0,
            0.04,
            0.0,
            1.0,
            Color::BLACK,
        )
        .with_texture(moon_texture, Vec2::new(4.0, 2.0), WrapMode::Repeat),
    )?;
    let crater = scene.add_material(Material::new(
        Color::new(0.28, 0.34, 0.46),
        0.12,
        12.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let gravity_field = scene.add_material(Material::new(
        Color::new(0.68, 0.90, 1.0),
        0.08,
        44.0,
        0.010,
        0.985,
        1.005,
        Color::new(0.020, 0.055, 0.105),
    ))?;
    let cookie_gravity_field = scene.add_material(Material::new(
        Color::new(1.0, 0.82, 0.38),
        0.08,
        42.0,
        0.010,
        0.985,
        1.005,
        Color::new(0.080, 0.040, 0.010),
    ))?;
    let asteroid = scene.add_material(Material::new(
        Color::new(0.52, 0.49, 0.45),
        0.18,
        16.0,
        0.02,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let cookie = scene.add_material(Material::new(
        Color::new(0.86, 0.56, 0.26),
        0.18,
        22.0,
        0.03,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let chocolate = scene.add_material(Material::new(
        Color::new(0.31, 0.14, 0.06),
        0.22,
        18.0,
        0.01,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let wood = scene.add_material(Material::new(
        Color::new(0.58, 0.34, 0.16),
        0.25,
        24.0,
        0.02,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let ice = scene.add_material(Material::new(
        Color::new(0.66, 0.92, 1.0),
        0.55,
        72.0,
        0.18,
        0.28,
        1.31,
        Color::BLACK,
    ))?;
    let metal = scene.add_material(Material::new(
        Color::new(0.78, 0.82, 0.86),
        0.75,
        96.0,
        0.48,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let tnt = scene.add_material(Material::new(
        Color::new(0.95, 0.08, 0.05),
        0.20,
        18.0,
        0.02,
        0.0,
        1.0,
        Color::new(0.08, 0.0, 0.0),
    ))?;
    let candy_red = scene.add_material(Material::new(
        Color::new(1.0, 0.10, 0.22),
        0.38,
        42.0,
        0.08,
        0.0,
        1.0,
        Color::new(0.04, 0.0, 0.01),
    ))?;
    let candy_blue = scene.add_material(Material::new(
        Color::new(0.06, 0.62, 1.0),
        0.38,
        42.0,
        0.08,
        0.0,
        1.0,
        Color::new(0.0, 0.02, 0.05),
    ))?;
    let candy_yellow = scene.add_material(Material::new(
        Color::new(1.0, 0.86, 0.12),
        0.32,
        36.0,
        0.05,
        0.0,
        1.0,
        Color::new(0.05, 0.035, 0.0),
    ))?;
    let pig = scene.add_material(Material::new(
        Color::new(0.44, 0.86, 0.32),
        0.25,
        24.0,
        0.02,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let snout = scene.add_material(Material::new(
        Color::new(0.64, 0.95, 0.48),
        0.25,
        18.0,
        0.02,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let eye = scene.add_material(Material::diffuse(Color::WHITE))?;
    let pupil = scene.add_material(Material::diffuse(Color::BLACK))?;
    let red_bird = scene.add_material(Material::new(
        Color::new(0.92, 0.08, 0.06),
        0.28,
        28.0,
        0.02,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let blue_bird = scene.add_material(Material::new(
        Color::new(0.08, 0.42, 0.95),
        0.26,
        28.0,
        0.02,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let yellow_bird = scene.add_material(Material::new(
        Color::new(1.0, 0.82, 0.08),
        0.24,
        22.0,
        0.02,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let beak = scene.add_material(Material::new(
        Color::new(1.0, 0.58, 0.05),
        0.18,
        18.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let slingshot = scene.add_material(Material::new(
        Color::new(0.36, 0.19, 0.08),
        0.22,
        18.0,
        0.01,
        0.0,
        1.0,
        Color::BLACK,
    ))?;

    Ok(SpaceMaterials {
        moon,
        crater,
        gravity_field,
        cookie_gravity_field,
        asteroid,
        cookie,
        chocolate,
        wood,
        ice,
        metal,
        tnt,
        candy_red,
        candy_blue,
        candy_yellow,
        pig,
        snout,
        eye,
        pupil,
        red_bird,
        blue_bird,
        yellow_bird,
        beak,
        slingshot,
    })
}

fn add_planet(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let planet_id = scene.object_count();
    scene.add_sphere(Sphere::new(
        BLUE_MOON_PLANET_CENTER,
        BLUE_MOON_PLANET_RADIUS,
        materials.moon,
    )?)?;
    metadata.planet_id = Some(planet_id);

    for (latitude, longitude, radius) in [
        (0.25, 1.38, 0.28),
        (-0.10, 1.62, 0.20),
        (0.48, 1.86, 0.17),
        (-0.42, 1.20, 0.24),
        (0.05, 2.10, 0.16),
        (0.62, 0.92, 0.14),
        (-0.66, 1.82, 0.18),
        (0.18, 0.74, 0.13),
        (-0.28, 2.48, 0.15),
        (0.72, 2.52, 0.12),
        (0.10, -2.55, 0.18),
        (-0.58, -2.10, 0.13),
        (0.34, -2.85, 0.11),
        (-0.72, -2.72, 0.10),
        (0.82, 0.42, 0.09),
        (-0.86, 0.58, 0.11),
        (0.42, 2.88, 0.08),
        (-0.08, 2.95, 0.09),
        (0.54, -1.72, 0.13),
        (-0.36, -1.48, 0.10),
    ] {
        let frame = RadialFrame::from_latitude_longitude(
            BLUE_MOON_PLANET_CENTER,
            BLUE_MOON_PLANET_RADIUS,
            latitude,
            longitude,
        )?;
        add_radial_cylinder(
            scene,
            frame,
            Vec3::new(0.0, 0.025, 0.0),
            radius,
            0.018,
            materials.crater,
        )?;
        metadata.crater_count += 1;
    }

    Ok(())
}

fn add_enemy_construction(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let frame = main_moon_frame()?;

    for (center, half_extents) in [
        (Vec3::new(-0.58, 0.24, -0.25), Vec3::new(0.08, 0.24, 0.08)),
        (Vec3::new(0.08, 0.44, -0.25), Vec3::new(0.08, 0.44, 0.08)),
        (Vec3::new(0.74, 0.68, -0.25), Vec3::new(0.08, 0.68, 0.08)),
        (Vec3::new(0.08, 0.98, -0.25), Vec3::new(0.76, 0.07, 0.07)),
        (Vec3::new(0.08, 0.36, -0.25), Vec3::new(0.74, 0.06, 0.06)),
        (Vec3::new(-0.72, 0.18, 0.42), Vec3::new(0.62, 0.06, 0.06)),
        (Vec3::new(-0.25, 0.68, -0.25), Vec3::new(0.05, 0.42, 0.05)),
        (Vec3::new(0.42, 0.88, -0.25), Vec3::new(0.05, 0.50, 0.05)),
        (Vec3::new(-0.58, 0.84, -0.25), Vec3::new(0.06, 0.24, 0.05)),
        (Vec3::new(0.74, 0.42, 0.08), Vec3::new(0.05, 0.18, 0.05)),
        (Vec3::new(0.26, 1.62, -0.25), Vec3::new(0.54, 0.05, 0.05)),
    ] {
        add_radial_box(scene, frame, center, half_extents, materials.wood)?;
        metadata.wood_parts += 1;
    }

    for (center, half_extents, material_id) in [
        (
            Vec3::new(-0.58, 0.55, -0.25),
            Vec3::new(0.20, 0.20, 0.18),
            materials.ice,
        ),
        (
            Vec3::new(0.08, 1.16, -0.25),
            Vec3::new(0.20, 0.12, 0.18),
            materials.ice,
        ),
        (
            Vec3::new(0.74, 1.42, -0.25),
            Vec3::new(0.18, 0.18, 0.18),
            materials.metal,
        ),
        (
            Vec3::new(-1.00, 0.28, 0.42),
            Vec3::new(0.18, 0.16, 0.18),
            materials.ice,
        ),
        (
            Vec3::new(0.42, 1.52, -0.25),
            Vec3::new(0.16, 0.11, 0.16),
            materials.ice,
        ),
        (
            Vec3::new(-0.18, 1.52, -0.25),
            Vec3::new(0.14, 0.10, 0.14),
            materials.metal,
        ),
        (
            Vec3::new(-0.70, 0.48, 0.42),
            Vec3::new(0.14, 0.10, 0.14),
            materials.metal,
        ),
    ] {
        add_radial_box(scene, frame, center, half_extents, material_id)?;
        metadata.ice_or_metal_parts += 1;
    }

    add_radial_box(
        scene,
        frame,
        Vec3::new(0.44, 0.20, 0.42),
        Vec3::new(0.18, 0.18, 0.18),
        materials.tnt,
    )?;
    metadata.tnt_parts += 1;

    add_radial_box(
        scene,
        frame,
        Vec3::new(0.76, 0.20, 0.18),
        Vec3::new(0.14, 0.14, 0.14),
        materials.tnt,
    )?;
    metadata.tnt_parts += 1;

    Ok(())
}

fn add_pigs(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let frame = main_moon_frame()?;

    add_space_pig(scene, frame, -0.58, -0.25, 0.22, materials)?;
    add_space_pig(scene, frame, 0.08, -0.25, 0.19, materials)?;
    add_space_pig(scene, frame, 0.46, 0.42, 0.18, materials)?;
    metadata.pig_count = BLUE_MOON_PIG_COUNT;

    Ok(())
}

fn add_launcher(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let asteroid_id = scene.object_count();
    scene.add_sphere(Sphere::new(
        LAUNCH_ASTEROID_CENTER,
        LAUNCH_ASTEROID_RADIUS,
        materials.asteroid,
    )?)?;
    metadata.launch_asteroid_id = Some(asteroid_id);

    let frame = RadialFrame::from_normal(
        LAUNCH_ASTEROID_CENTER,
        LAUNCH_ASTEROID_RADIUS,
        Vec3::new(0.10, 0.48, 1.0),
    )?;

    for local in [
        Vec3::new(-0.28, 0.020, -0.08),
        Vec3::new(0.18, 0.025, 0.18),
        Vec3::new(0.35, 0.020, -0.24),
    ] {
        add_radial_cylinder(scene, frame, local, 0.09, 0.018, materials.crater)?;
    }

    add_slingshot(scene, frame, metadata, materials)?;
    add_space_bird(
        scene,
        frame,
        Vec3::new(-0.03, 0.83, -0.03),
        0.17,
        materials.red_bird,
        materials,
    )?;
    add_space_bird(
        scene,
        frame,
        Vec3::new(-0.42, 0.20, 0.25),
        0.15,
        materials.blue_bird,
        materials,
    )?;
    add_space_bird(
        scene,
        frame,
        Vec3::new(-0.64, 0.18, -0.18),
        0.16,
        materials.yellow_bird,
        materials,
    )?;
    metadata.bird_count = BLUE_MOON_BIRD_COUNT;

    Ok(())
}

fn add_blue_moon_debris(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    for (center, radius, material_id) in [
        (Vec3::new(-2.85, 1.55, -0.85), 0.13, materials.asteroid),
        (Vec3::new(-3.15, -0.85, -1.35), 0.10, materials.asteroid),
        (Vec3::new(-1.85, 2.10, 1.15), 0.09, materials.crater),
        (Vec3::new(1.35, 1.80, -1.95), 0.11, materials.asteroid),
        (Vec3::new(2.55, -0.80, 1.50), 0.12, materials.asteroid),
        (Vec3::new(-2.35, -1.65, 1.65), 0.08, materials.crater),
    ] {
        scene.add_sphere(Sphere::new(center, radius, material_id)?)?;
        metadata.decorative_asteroid_count += 1;
    }

    Ok(())
}

fn add_gravity_field(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let gravity_id = scene.object_count();
    scene.add_sphere(Sphere::new(
        BLUE_MOON_PLANET_CENTER,
        BLUE_MOON_PLANET_RADIUS * 1.20,
        materials.gravity_field,
    )?)?;
    metadata.gravity_field_id = Some(gravity_id);

    Ok(())
}

fn add_cookie_level(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let planet_id = scene.object_count();
    scene.add_sphere(Sphere::new(
        COOKIE_PLANET_CENTER,
        COOKIE_PLANET_RADIUS,
        materials.cookie,
    )?)?;
    metadata.cookie_planet_id = Some(planet_id);

    add_cookie_chocolate_chips(scene, metadata, materials)?;
    add_cookie_enemy_construction(scene, metadata, materials)?;
    add_cookie_pigs(scene, metadata, materials)?;
    add_cookie_candy(scene, metadata, materials)?;

    let gravity_id = scene.object_count();
    scene.add_sphere(Sphere::new(
        COOKIE_PLANET_CENTER,
        COOKIE_PLANET_RADIUS * 1.19,
        materials.cookie_gravity_field,
    )?)?;
    metadata.cookie_gravity_field_id = Some(gravity_id);

    Ok(())
}

fn add_cookie_chocolate_chips(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    for (latitude, longitude, radius) in [
        (0.20, 1.46, 0.18),
        (-0.12, 1.78, 0.14),
        (0.46, 1.18, 0.12),
        (-0.48, 1.26, 0.13),
        (0.08, 2.20, 0.11),
        (0.60, 2.04, 0.10),
        (-0.64, 1.92, 0.09),
        (0.30, -2.70, 0.12),
        (-0.26, -2.42, 0.10),
    ] {
        let frame = RadialFrame::from_latitude_longitude(
            COOKIE_PLANET_CENTER,
            COOKIE_PLANET_RADIUS,
            latitude,
            longitude,
        )?;
        add_radial_cylinder(
            scene,
            frame,
            Vec3::new(0.0, 0.026, 0.0),
            radius,
            0.020,
            materials.chocolate,
        )?;
        metadata.cookie_chocolate_chip_count += 1;
    }

    Ok(())
}

fn add_cookie_enemy_construction(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let frame = cookie_level_frame()?;

    for (center, half_extents, material_id) in [
        (
            Vec3::new(-0.46, 0.22, -0.12),
            Vec3::new(0.06, 0.22, 0.06),
            materials.wood,
        ),
        (
            Vec3::new(0.18, 0.28, -0.12),
            Vec3::new(0.06, 0.28, 0.06),
            materials.wood,
        ),
        (
            Vec3::new(-0.14, 0.55, -0.12),
            Vec3::new(0.48, 0.05, 0.05),
            materials.wood,
        ),
        (
            Vec3::new(-0.14, 0.88, -0.12),
            Vec3::new(0.50, 0.05, 0.05),
            materials.metal,
        ),
        (
            Vec3::new(-0.46, 0.62, -0.12),
            Vec3::new(0.15, 0.13, 0.14),
            materials.ice,
        ),
        (
            Vec3::new(0.18, 0.68, -0.12),
            Vec3::new(0.14, 0.14, 0.14),
            materials.ice,
        ),
        (
            Vec3::new(0.48, 0.20, 0.22),
            Vec3::new(0.13, 0.13, 0.13),
            materials.tnt,
        ),
    ] {
        add_radial_box(scene, frame, center, half_extents, material_id)?;
        if material_id == materials.wood {
            metadata.wood_parts += 1;
        } else if material_id == materials.tnt {
            metadata.tnt_parts += 1;
        } else {
            metadata.ice_or_metal_parts += 1;
        }
    }

    Ok(())
}

fn add_cookie_pigs(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let frame = cookie_level_frame()?;

    add_space_pig(scene, frame, -0.46, -0.12, 0.16, materials)?;
    add_space_pig(scene, frame, 0.20, -0.12, 0.15, materials)?;
    metadata.pig_count += COOKIE_LEVEL_PIG_COUNT;
    metadata.cookie_pig_count = COOKIE_LEVEL_PIG_COUNT;

    Ok(())
}

fn add_cookie_candy(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    for (normal, local, radius, material_id) in [
        (
            Vec3::new(0.35, 0.35, 1.0),
            Vec3::new(-0.76, 0.18, 0.22),
            0.13,
            materials.candy_red,
        ),
        (
            Vec3::new(-0.15, 0.62, 1.0),
            Vec3::new(0.58, 0.16, -0.18),
            0.12,
            materials.candy_blue,
        ),
        (
            Vec3::new(0.82, -0.10, 0.55),
            Vec3::new(0.32, 0.20, 0.36),
            0.14,
            materials.candy_yellow,
        ),
    ] {
        let frame = RadialFrame::from_normal(COOKIE_PLANET_CENTER, COOKIE_PLANET_RADIUS, normal)?;
        scene.add_sphere(Sphere::new(
            frame.local_to_world(local),
            radius,
            material_id,
        )?)?;
        metadata.candy_count += 1;
    }

    let lollipop_frame = RadialFrame::from_normal(
        COOKIE_PLANET_CENTER,
        COOKIE_PLANET_RADIUS,
        Vec3::new(-0.45, 0.22, 1.0),
    )?;
    add_radial_cylinder(
        scene,
        lollipop_frame,
        Vec3::new(0.0, 0.22, 0.36),
        0.022,
        0.26,
        materials.ice,
    )?;
    scene.add_sphere(Sphere::new(
        lollipop_frame.position(0.0, 0.52, 0.36),
        0.18,
        materials.candy_red,
    )?)?;
    scene.add_sphere(Sphere::new(
        lollipop_frame.position(0.07, 0.58, 0.41),
        0.07,
        materials.candy_yellow,
    )?)?;
    metadata.candy_count += 2;
    metadata.ice_or_metal_parts += 1;

    let wrapped_frame = RadialFrame::from_normal(
        COOKIE_PLANET_CENTER,
        COOKIE_PLANET_RADIUS,
        Vec3::new(0.85, 0.40, 0.75),
    )?;
    add_radial_cylinder(
        scene,
        wrapped_frame,
        Vec3::new(0.0, 0.22, -0.30),
        0.12,
        0.055,
        materials.candy_blue,
    )?;
    for x in [-0.16, 0.16] {
        scene.add_cone(Cone::new(
            wrapped_frame.position(x, 0.22, -0.30),
            0.07,
            0.09,
            wrapped_frame.basis(),
            materials.candy_yellow,
        )?)?;
    }
    metadata.candy_count += 3;

    Ok(())
}

fn add_selector_lighting(scene: &mut Scene) {
    add_sun_key_light(scene, 66.0);
    scene.add_light(PointLight::new(
        Vec3::new(0.0, 2.8, 7.4),
        Color::new(0.62, 0.86, 1.0),
        18.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(4.4, 2.2, 5.2),
        Color::new(0.46, 0.74, 1.0),
        10.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(-2.6, -0.8, 4.0),
        Color::new(1.0, 0.78, 0.45),
        5.4,
    ));
}

fn add_space_lighting(scene: &mut Scene) {
    add_sun_key_light(scene, 82.0);
    scene.add_light(PointLight::new(
        Vec3::new(0.4, 3.0, 7.0),
        Color::new(0.58, 0.82, 1.0),
        22.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(5.0, 2.4, 5.4),
        Color::new(0.44, 0.68, 1.0),
        12.5,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(-2.4, -0.9, 4.8),
        Color::new(1.0, 0.78, 0.46),
        7.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(-5.2, 0.4, 1.6),
        Color::new(0.36, 0.78, 1.0),
        7.5,
    ));
}

fn add_cookie_level_lighting(scene: &mut Scene) {
    scene.add_light(PointLight::new(
        Vec3::new(5.9, 3.0, 5.4),
        Color::new(1.0, 0.70, 0.34),
        8.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(3.3, -0.2, 3.8),
        Color::new(0.72, 0.42, 1.0),
        3.4,
    ));
}

fn add_sun_key_light(scene: &mut Scene, intensity: f32) {
    scene.add_light(PointLight::new(
        sun_light_position(10.0),
        Color::new(1.0, 0.88, 0.58),
        intensity,
    ));
}

fn sun_light_position(distance: f32) -> Vec3 {
    SPACE_SUN_DIRECTION.normalized() * distance
}

fn add_space_pig(
    scene: &mut Scene,
    frame: RadialFrame,
    tangent_x: f32,
    tangent_z: f32,
    radius: f32,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    scene.add_sphere(Sphere::new(
        frame.position(tangent_x, radius + 0.04, tangent_z),
        radius,
        materials.pig,
    )?)?;
    scene.add_cylinder(Cylinder::new(
        frame.position(tangent_x, radius * 1.80, tangent_z - radius * 0.03),
        radius * 0.34,
        radius * 0.18,
        frame.basis(),
        materials.snout,
    )?)?;

    for eye_x in [-radius * 0.34, radius * 0.34] {
        scene.add_sphere(Sphere::new(
            frame.position(tangent_x + eye_x, radius * 1.94, tangent_z + radius * 0.42),
            radius * 0.16,
            materials.eye,
        )?)?;
        scene.add_sphere(Sphere::new(
            frame.position(tangent_x + eye_x, radius * 2.08, tangent_z + radius * 0.42),
            radius * 0.07,
            materials.pupil,
        )?)?;
    }

    for ear_x in [-radius * 0.48, radius * 0.48] {
        scene.add_cone(Cone::new(
            frame.position(tangent_x + ear_x, radius * 1.34, tangent_z + radius * 0.78),
            radius * 0.14,
            radius * 0.18,
            frame.basis(),
            materials.pig,
        )?)?;
    }

    Ok(())
}

fn add_slingshot(
    scene: &mut Scene,
    frame: RadialFrame,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    add_radial_box(
        scene,
        frame,
        Vec3::new(0.0, 0.08, -0.03),
        Vec3::new(0.36, 0.08, 0.12),
        materials.slingshot,
    )?;
    metadata.slingshot_parts += 1;

    for x in [-0.18, 0.18] {
        add_radial_cylinder(
            scene,
            frame,
            Vec3::new(x, 0.42, 0.0),
            0.055,
            0.38,
            materials.slingshot,
        )?;
        metadata.slingshot_parts += 1;

        scene.add_sphere(Sphere::new(
            frame.position(x, 0.82, 0.0),
            0.075,
            materials.slingshot,
        )?)?;
        metadata.slingshot_parts += 1;
    }

    add_radial_box(
        scene,
        frame,
        Vec3::new(0.0, 0.80, 0.0),
        Vec3::new(0.24, 0.03, 0.04),
        materials.slingshot,
    )?;
    metadata.slingshot_parts += 1;

    for (center, half_extents) in [
        (Vec3::new(-0.30, 0.31, -0.02), Vec3::new(0.05, 0.22, 0.04)),
        (Vec3::new(0.30, 0.31, -0.02), Vec3::new(0.05, 0.22, 0.04)),
        (Vec3::new(0.0, 0.74, 0.08), Vec3::new(0.30, 0.022, 0.025)),
    ] {
        add_radial_box(scene, frame, center, half_extents, materials.slingshot)?;
        metadata.slingshot_parts += 1;
    }

    Ok(())
}

fn add_space_bird(
    scene: &mut Scene,
    frame: RadialFrame,
    local_center: Vec3,
    radius: f32,
    body_material: usize,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    scene.add_sphere(Sphere::new(
        frame.local_to_world(local_center),
        radius,
        body_material,
    )?)?;

    for eye_x in [-radius * 0.28, radius * 0.28] {
        scene.add_sphere(Sphere::new(
            frame.local_to_world(local_center + Vec3::new(eye_x, radius * 0.78, radius * 0.28)),
            radius * 0.13,
            materials.eye,
        )?)?;
    }

    scene.add_cone(Cone::new(
        frame.local_to_world(local_center + Vec3::new(0.0, radius * 1.05, 0.0)),
        radius * 0.18,
        radius * 0.24,
        frame.basis(),
        materials.beak,
    )?)?;

    Ok(())
}

fn add_radial_box(
    scene: &mut Scene,
    frame: RadialFrame,
    local_center: Vec3,
    half_extents: Vec3,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    scene.add_oriented_box(OrientedBox::new(
        frame.local_to_world(local_center),
        half_extents,
        frame.basis(),
        material_id,
    )?)?;

    Ok(())
}

fn add_radial_cylinder(
    scene: &mut Scene,
    frame: RadialFrame,
    local_center: Vec3,
    radius: f32,
    half_height: f32,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    scene.add_cylinder(Cylinder::new(
        frame.local_to_world(local_center),
        radius,
        half_height,
        frame.basis(),
        material_id,
    )?)?;

    Ok(())
}

fn main_moon_frame() -> Result<RadialFrame, SpaceBuildError> {
    Ok(RadialFrame::from_latitude_longitude(
        BLUE_MOON_PLANET_CENTER,
        BLUE_MOON_PLANET_RADIUS,
        0.22,
        1.45,
    )?)
}

fn cookie_level_frame() -> Result<RadialFrame, SpaceBuildError> {
    Ok(RadialFrame::from_latitude_longitude(
        COOKIE_PLANET_CENTER,
        COOKIE_PLANET_RADIUS,
        0.20,
        1.50,
    )?)
}

#[cfg(test)]
mod tests {
    use super::{
        BLUE_MOON_BIRD_COUNT, BLUE_MOON_PIG_COUNT, BLUE_MOON_PLANET_CENTER,
        BLUE_MOON_PLANET_RADIUS, COOKIE_LEVEL_PIG_COUNT, COOKIE_PLANET_CENTER,
        COOKIE_PLANET_RADIUS, GALAXY_SELECTOR_BLUE_MOON_CENTER, GALAXY_SELECTOR_COOKIE_CENTER,
        LAUNCH_ASTEROID_CENTER, PlanetType, SKYBOX_ASTEROID_A_U, SKYBOX_ASTEROID_A_V,
        SPACE_SUN_DIRECTION, SPACE_SUN_U, SPACE_SUN_V, SceneState, SpaceMaterials, add_radial_box,
        angry_birds_space_skybox, blue_moon_orbit_camera, build_blue_moon_scene_with_metadata,
        build_cookie_world_scene_with_metadata, build_galaxy_selector_scene,
        build_space_levels_scene_with_metadata, cookie_world_orbit_camera,
        galaxy_selector_orbit_camera, galaxy_selector_worlds, main_moon_frame,
        register_space_materials, space_levels_orbit_camera, space_skybox_color,
        space_skybox_texture, sun_light_position, wrapped_uv_distance,
    };
    use crate::{material::Material, math::Vec3, ray::Ray, scene::Scene};

    #[test]
    fn blue_moon_scene_builds_valid_scene() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();

        assert!(scene.object_count() > 0);
        assert!(scene.object_count() < 180);
        assert!(metadata.planet_id.is_some());
        assert!(metadata.gravity_field_id.is_some());
        assert!(metadata.launch_asteroid_id.is_some());
    }

    #[test]
    fn scene_state_can_represent_selector_and_planets() {
        assert_eq!(SceneState::Galaxy, SceneState::Galaxy);
        assert_eq!(
            SceneState::Planet(PlanetType::BlueMoon),
            SceneState::Planet(PlanetType::BlueMoon)
        );
        assert_ne!(
            SceneState::Planet(PlanetType::BlueMoon),
            SceneState::Planet(PlanetType::CookieWorld)
        );
    }

    #[test]
    fn galaxy_selector_declares_two_clickable_worlds() {
        let worlds = galaxy_selector_worlds();

        assert_eq!(worlds.len(), 2);
        assert_eq!(worlds[0].planet, PlanetType::BlueMoon);
        assert_eq!(worlds[0].center, GALAXY_SELECTOR_BLUE_MOON_CENTER);
        assert!(worlds[0].radius > 0.0);
        assert_eq!(worlds[1].planet, PlanetType::CookieWorld);
        assert_eq!(worlds[1].center, GALAXY_SELECTOR_COOKIE_CENTER);
        assert!(worlds[1].radius > 0.0);
    }

    #[test]
    fn galaxy_selector_scene_contains_only_selector_world_geometry() {
        let scene = build_galaxy_selector_scene().unwrap();

        assert_eq!(scene.sphere_count(), 4);
        assert_eq!(scene.object_count(), 4);
        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 3);
        assert!(scene.ambient_light().b > 0.39);
    }

    #[test]
    fn space_skybox_is_dense_smooth_starred_and_sunlit() {
        let texture = space_skybox_texture().unwrap();
        let top = texture
            .pixel(texture.width() / 2, texture.height() / 12)
            .unwrap();
        let middle = texture
            .pixel(texture.width() / 2, texture.height() / 2)
            .unwrap();
        let bottom = texture
            .pixel(
                texture.width() / 2,
                texture.height() - texture.height() / 12,
            )
            .unwrap();
        let sun = space_skybox_color(SPACE_SUN_U, SPACE_SUN_V);
        let mut bright_pixels = 0;

        for y in 0..texture.height() {
            for x in 0..texture.width() {
                let color = texture.pixel(x, y).unwrap();
                let u = x as f32 / texture.width() as f32;
                let v = 1.0 - y as f32 / (texture.height() - 1) as f32;

                if wrapped_uv_distance(u, v, SPACE_SUN_U, SPACE_SUN_V, 1.18) > 0.08
                    && color.r > 0.62
                    && color.g > 0.68
                    && color.b > 0.74
                {
                    bright_pixels += 1;
                }
            }
        }

        assert!(texture.width() >= 256);
        assert!(texture.height() >= 128);
        assert_ne!(top, middle);
        assert_ne!(bottom, middle);
        assert!(sun.r > 0.95);
        assert!(sun.g > 0.85);
        assert!(sun.b > 0.12);
        assert!(sun.r > sun.b);
        assert!(bottom.b > 0.18);
        assert!(bright_pixels >= 12);
    }

    #[test]
    fn spatial_scenes_use_angry_birds_space_skybox() {
        let selector = build_galaxy_selector_scene().unwrap();
        let (blue, _) = build_blue_moon_scene_with_metadata().unwrap();
        let (cookie, _) = build_cookie_world_scene_with_metadata().unwrap();
        let (combined, _) = build_space_levels_scene_with_metadata().unwrap();
        let reference = angry_birds_space_skybox().unwrap();

        assert_eq!(reference.texture().width(), super::SPACE_SKYBOX_WIDTH);
        assert_eq!(reference.texture().height(), super::SPACE_SKYBOX_HEIGHT);
        assert!(reference.intensity() > 1.0);

        for scene in [&selector, &blue, &cookie, &combined] {
            let skybox = scene.skybox().unwrap();

            assert_eq!(skybox.texture().width(), super::SPACE_SKYBOX_WIDTH);
            assert_eq!(skybox.texture().height(), super::SPACE_SKYBOX_HEIGHT);
            assert_eq!(skybox.horizontal_rotation(), 0.0);
        }
    }

    #[test]
    fn angry_birds_space_skybox_has_cartoon_asteroids_and_cloud_layers() {
        let sun = space_skybox_color(SPACE_SUN_U, SPACE_SUN_V);
        let asteroid = space_skybox_color(SKYBOX_ASTEROID_A_U, SKYBOX_ASTEROID_A_V);
        let lower_cloud = space_skybox_color(0.50, 0.090);

        assert!(sun.r > 0.95);
        assert!(sun.g > 0.70);
        assert!(sun.b < sun.r);
        assert!(asteroid.b > 0.36);
        assert!(asteroid.r > asteroid.g);
        assert!(asteroid.b > asteroid.g * 2.0);
        assert!(lower_cloud.b > 0.20);
        assert!(lower_cloud.g > 0.08);
        assert!(lower_cloud.r < 0.08);
    }

    #[test]
    fn space_lighting_uses_visible_sun_direction_as_key() {
        let selector = build_galaxy_selector_scene().unwrap();
        let (blue, _) = build_blue_moon_scene_with_metadata().unwrap();
        let expected_direction = SPACE_SUN_DIRECTION.normalized();

        for scene in [&selector, &blue] {
            let key_light = scene.lights()[0];
            let actual_direction = key_light.position.normalized();

            assert!(actual_direction.dot(expected_direction) > 0.999);
            assert_eq!(key_light.position, sun_light_position(10.0));
            assert!(key_light.color.r > key_light.color.b);
            assert!(key_light.intensity >= 60.0);
        }
    }

    #[test]
    fn blue_moon_scene_contains_required_counts() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();

        assert_eq!(metadata.pig_count, BLUE_MOON_PIG_COUNT);
        assert_eq!(metadata.bird_count, BLUE_MOON_BIRD_COUNT);
        assert!(metadata.crater_count >= 10);
        assert!(metadata.decorative_asteroid_count >= 5);
        assert!(metadata.slingshot_parts >= 3);
        assert!(metadata.wood_parts >= 6);
        assert!(metadata.ice_or_metal_parts >= 4);
        assert!(metadata.tnt_parts >= 1);
        assert!(scene.sphere_count() >= 18);
        assert!(scene.cylinder_count() >= 17);
        assert!(scene.cone_count() >= 9);
        assert!(scene.oriented_box_count() >= 15);
    }

    #[test]
    fn blue_moon_scene_has_materials_textures_skybox_and_lights() {
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();

        assert!(scene.materials().len() >= 12);
        assert!(!scene.textures().is_empty());
        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 3);
    }

    #[test]
    fn all_primitive_materials_are_registered() {
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();

        for primitive in scene.objects() {
            assert!(scene.material(primitive.material_id()).is_some());
        }
    }

    #[test]
    fn textured_materials_reference_registered_textures() {
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();

        for material in scene.materials() {
            if let Some(texture_id) = material.texture_id {
                assert!(scene.texture(texture_id).is_some());
            }
        }
    }

    #[test]
    fn planet_and_gravity_field_are_spheres() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let planet = scene.objects()[metadata.planet_id.unwrap()]
            .as_sphere()
            .unwrap();
        let gravity = scene.objects()[metadata.gravity_field_id.unwrap()]
            .as_sphere()
            .unwrap();

        assert_eq!(planet.center(), BLUE_MOON_PLANET_CENTER);
        assert_eq!(planet.radius(), BLUE_MOON_PLANET_RADIUS);
        assert_eq!(gravity.center(), BLUE_MOON_PLANET_CENTER);
        assert!(gravity.radius() > planet.radius());

        let material = scene.material(gravity.material_id()).unwrap();
        assert!(material.transparency > 0.94);
        assert!(material.reflectivity < 0.04);
        assert!(material.emission.b > 0.10);
    }

    #[test]
    fn launch_asteroid_is_independent_and_visible_beside_planet() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let asteroid = scene.objects()[metadata.launch_asteroid_id.unwrap()]
            .as_sphere()
            .unwrap();

        assert_eq!(asteroid.center(), LAUNCH_ASTEROID_CENTER);
        assert!(asteroid.center().x < BLUE_MOON_PLANET_CENTER.x - BLUE_MOON_PLANET_RADIUS);
    }

    #[test]
    fn radial_boxes_are_elevated_above_planet_surface() {
        let mut scene = Scene::new();
        let mut material_scene = Scene::new();
        let SpaceMaterials { wood, .. } = register_space_materials(&mut material_scene).unwrap();
        scene
            .add_material(Material::diffuse(crate::color::Color::WHITE))
            .unwrap();
        let frame = main_moon_frame().unwrap();
        let half_extents = Vec3::new(0.1, 0.2, 0.1);
        let local_center = Vec3::new(0.0, half_extents.y + 0.05, 0.0);

        add_radial_box(&mut scene, frame, local_center, half_extents, 0).unwrap();

        let box_center = scene.objects()[0].as_oriented_box().unwrap().center();
        assert!((box_center - BLUE_MOON_PLANET_CENTER).length() > BLUE_MOON_PLANET_RADIUS);
        assert!(wood < material_scene.materials().len());
    }

    #[test]
    fn initial_camera_is_outside_geometry_and_sees_main_set() {
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();
        let camera = blue_moon_orbit_camera(4.0 / 3.0).to_camera();
        let central_ray = Ray::new(camera.position, camera.target - camera.position);
        let hit = scene.intersect(&central_ray, 0.001, 100.0).unwrap();

        assert!(camera.position.length() > BLUE_MOON_PLANET_RADIUS * 1.30);
        assert!(hit.distance > 1.0);
    }

    #[test]
    fn space_levels_scene_contains_two_main_planets() {
        let (scene, metadata) = build_space_levels_scene_with_metadata().unwrap();
        let moon = scene.objects()[metadata.planet_id.unwrap()]
            .as_sphere()
            .unwrap();
        let cookie = scene.objects()[metadata.cookie_planet_id.unwrap()]
            .as_sphere()
            .unwrap();

        assert_eq!(moon.center(), BLUE_MOON_PLANET_CENTER);
        assert_eq!(moon.radius(), BLUE_MOON_PLANET_RADIUS);
        assert_eq!(cookie.center(), COOKIE_PLANET_CENTER);
        assert_eq!(cookie.radius(), COOKIE_PLANET_RADIUS);
        assert!(metadata.gravity_field_id.is_some());
        assert!(metadata.cookie_gravity_field_id.is_some());
    }

    #[test]
    fn cookie_world_scene_contains_cookie_world_elements() {
        let (scene, metadata) = build_cookie_world_scene_with_metadata().unwrap();
        let cookie = scene.objects()[metadata.cookie_planet_id.unwrap()]
            .as_sphere()
            .unwrap();

        assert_eq!(cookie.center(), COOKIE_PLANET_CENTER);
        assert_eq!(cookie.radius(), COOKIE_PLANET_RADIUS);
        assert_eq!(metadata.cookie_pig_count, COOKIE_LEVEL_PIG_COUNT);
        assert_eq!(metadata.pig_count, COOKIE_LEVEL_PIG_COUNT);
        assert!(metadata.cookie_chocolate_chip_count >= 8);
        assert!(metadata.candy_count >= 6);
        assert!(metadata.wood_parts >= 3);
        assert!(metadata.ice_or_metal_parts >= 2);
        assert!(metadata.tnt_parts >= 1);
        assert!(metadata.cookie_gravity_field_id.is_some());
        assert!(metadata.planet_id.is_none());
        assert!(metadata.launch_asteroid_id.is_none());
        assert!(scene.skybox().is_some());
    }

    #[test]
    fn space_levels_scene_keeps_pigs_birds_and_launcher() {
        let (_, metadata) = build_space_levels_scene_with_metadata().unwrap();

        assert_eq!(metadata.bird_count, BLUE_MOON_BIRD_COUNT);
        assert_eq!(metadata.cookie_pig_count, COOKIE_LEVEL_PIG_COUNT);
        assert_eq!(
            metadata.pig_count,
            BLUE_MOON_PIG_COUNT + COOKIE_LEVEL_PIG_COUNT
        );
        assert!(metadata.launch_asteroid_id.is_some());
        assert!(metadata.slingshot_parts >= 7);
    }

    #[test]
    fn cookie_level_has_sweet_readable_details() {
        let (scene, metadata) = build_space_levels_scene_with_metadata().unwrap();

        assert!(metadata.cookie_chocolate_chip_count >= 8);
        assert!(metadata.candy_count >= 6);
        assert!(scene.sphere_count() >= 35);
        assert!(scene.cylinder_count() >= 28);
        assert!(scene.oriented_box_count() >= 26);
    }

    #[test]
    fn space_levels_scene_has_skybox_lights_and_scene_budget() {
        let (scene, _) = build_space_levels_scene_with_metadata().unwrap();

        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 7);
        assert!(scene.object_count() >= 125);
        assert!(scene.object_count() < 260);

        for primitive in scene.objects() {
            assert!(scene.material(primitive.material_id()).is_some());
        }
    }

    #[test]
    fn space_levels_scene_has_no_visible_emissive_light_panels() {
        let (scene, _) = build_space_levels_scene_with_metadata().unwrap();
        let bright_flat_panels = scene
            .objects()
            .iter()
            .filter_map(|object| object.as_oriented_box())
            .filter(|box_object| {
                let material = scene.material(box_object.material_id()).unwrap();
                let emission = material.emission;

                box_object.half_extents().y < 0.10
                    && box_object.half_extents().x > box_object.half_extents().z * 2.0
                    && emission.b > 0.20
            })
            .count();

        assert_eq!(bright_flat_panels, 0);
    }

    #[test]
    fn space_levels_initial_camera_is_outside_and_sees_a_world() {
        let (scene, _) = build_space_levels_scene_with_metadata().unwrap();
        let camera = space_levels_orbit_camera(4.0 / 3.0).to_camera();
        let central_ray = Ray::new(camera.position, camera.target - camera.position);
        let hit = scene.intersect(&central_ray, 0.001, 100.0).unwrap();

        assert!(
            (camera.position - BLUE_MOON_PLANET_CENTER).length() > BLUE_MOON_PLANET_RADIUS * 1.30
        );
        assert!((camera.position - COOKIE_PLANET_CENTER).length() > COOKIE_PLANET_RADIUS * 1.28);
        assert!(hit.distance > 1.0);
    }

    #[test]
    fn selector_camera_is_outside_and_can_see_selector_worlds() {
        let scene = build_galaxy_selector_scene().unwrap();
        let camera = galaxy_selector_orbit_camera(4.0 / 3.0).to_camera();
        let blue_ray = Ray::new(
            camera.position,
            GALAXY_SELECTOR_BLUE_MOON_CENTER - camera.position,
        );
        let cookie_ray = Ray::new(
            camera.position,
            GALAXY_SELECTOR_COOKIE_CENTER - camera.position,
        );

        assert!(
            (camera.position - GALAXY_SELECTOR_BLUE_MOON_CENTER).length()
                > super::GALAXY_SELECTOR_BLUE_MOON_RADIUS * 1.30
        );
        assert!(
            (camera.position - GALAXY_SELECTOR_COOKIE_CENTER).length()
                > super::GALAXY_SELECTOR_COOKIE_RADIUS * 1.30
        );
        assert!(scene.intersect(&blue_ray, 0.001, 100.0).is_some());
        assert!(scene.intersect(&cookie_ray, 0.001, 100.0).is_some());
    }

    #[test]
    fn cookie_world_camera_is_outside_and_sees_cookie_world() {
        let (scene, _) = build_cookie_world_scene_with_metadata().unwrap();
        let camera = cookie_world_orbit_camera(4.0 / 3.0).to_camera();
        let central_ray = Ray::new(camera.position, camera.target - camera.position);
        let hit = scene.intersect(&central_ray, 0.001, 100.0).unwrap();

        assert!((camera.position - COOKIE_PLANET_CENTER).length() > COOKIE_PLANET_RADIUS * 1.30);
        assert!(hit.distance > 1.0);
    }
}
