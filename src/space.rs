use std::{error::Error, fmt};

use crate::{
    basis::{Basis3, Basis3Error},
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
pub const BLUE_MOON_PLANET_RADIUS: f32 = 1.25;
pub const COOKIE_PLANET_CENTER: Vec3 = Vec3::new(4.45, -0.30, -0.25);
pub const COOKIE_PLANET_RADIUS: f32 = 1.45;
pub const LEVEL_THREE_PLANET_CENTER: Vec3 = Vec3::new(-0.35, 0.05, 0.10);
pub const LEVEL_THREE_PLANET_RADIUS: f32 = 1.70;
pub const SPACE_SUN_DIRECTION: Vec3 = Vec3::new(-0.76, 0.54, -0.36);
pub const BLUE_MOON_LEVEL_PIG_COUNT: usize = 6;
pub const COOKIE_LEVEL_PIG_COUNT: usize = 2;
pub const GALAXY_SELECTOR_BLUE_MOON_CENTER: Vec3 = Vec3::new(-2.85, 0.26, 0.0);
pub const GALAXY_SELECTOR_BLUE_MOON_RADIUS: f32 = 0.98;
pub const GALAXY_SELECTOR_COOKIE_CENTER: Vec3 = Vec3::new(2.65, -0.20, -0.28);
pub const GALAXY_SELECTOR_COOKIE_RADIUS: f32 = 0.90;
pub const GALAXY_SELECTOR_LEVEL_THREE_CENTER: Vec3 = Vec3::new(0.0, 1.72, -0.46);
pub const GALAXY_SELECTOR_LEVEL_THREE_RADIUS: f32 = 0.78;
const SPACE_SKYBOX_WIDTH: usize = 960;
const SPACE_SKYBOX_HEIGHT: usize = 480;
const SPACE_SUN_U: f32 = 0.125;
const SPACE_SUN_V: f32 = 0.770;
const SKYBOX_ASTEROID_A_U: f32 = 0.675;
const SKYBOX_ASTEROID_A_V: f32 = 0.620;
const SKYBOX_ASTEROID_B_U: f32 = 0.835;
const SKYBOX_ASTEROID_B_V: f32 = 0.455;
const SKYBOX_ASTEROID_C_U: f32 = 0.425;
const SKYBOX_ASTEROID_C_V: f32 = 0.730;
const BLUE_MOON_PLANET_TEXTURE_PATH: &str = "assets/textures/blue_moon_planet.ppm";
const WOOD_BLOCK_TEXTURE_PATH: &str = "assets/textures/space_wood_block.ppm";
const STONE_BLOCK_TEXTURE_PATH: &str = "assets/textures/space_stone_block.ppm";
const TNT_CRATE_TEXTURE_PATH: &str = "assets/textures/tnt_crate.ppm";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneState {
    Galaxy,
    Planet(PlanetType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetType {
    BlueMoon,
    CookieWorld,
    AsteroidBelt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectorWorld {
    pub planet: PlanetType,
    pub center: Vec3,
    pub radius: f32,
    pub level_number: u8,
    pub locked: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceBuildError {
    Scene(SceneError),
    Texture(TextureError),
    Sphere(SphereError),
    Cylinder(CylinderError),
    Cone(ConeError),
    OrientedBox(OrientedBoxError),
    RadialFrame(RadialFrameError),
    Basis(Basis3Error),
}

#[derive(Debug, Clone, Copy)]
struct SpaceMaterials {
    moon: usize,
    moon_crater_floor: usize,
    moon_stone: usize,
    moon_stone_light: usize,
    moon_dirt: usize,
    gravity_field: usize,
    cookie_gravity_field: usize,
    level_three: usize,
    level_three_gravity_field: usize,
    selector_locked_moon: usize,
    selector_locked_cookie: usize,
    selector_locked_level_three: usize,
    asteroid: usize,
    cookie: usize,
    chocolate: usize,
    wood: usize,
    ice: usize,
    metal: usize,
    tnt: usize,
    tnt_crate: usize,
    candy_red: usize,
    candy_blue: usize,
    candy_yellow: usize,
    pig: usize,
    snout: usize,
    eye: usize,
    pupil: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SpaceSceneMetadata {
    pub planet_id: Option<usize>,
    pub cookie_planet_id: Option<usize>,
    pub cookie_gravity_field_id: Option<usize>,
    pub level_three_planet_id: Option<usize>,
    pub level_three_gravity_field_id: Option<usize>,
    pub blue_moon_crater_count: usize,
    pub blue_moon_stone_count: usize,
    pub blue_moon_dirt_base_parts: usize,
    pub blue_moon_stone_structure_parts: usize,
    pub blue_moon_pig_count: usize,
    pub blue_moon_second_structure_parts: usize,
    pub decorative_asteroid_count: usize,
    pub cookie_chocolate_chip_count: usize,
    pub candy_count: usize,
    pub pig_count: usize,
    pub cookie_pig_count: usize,
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
            Self::Basis(error) => write!(formatter, "basis build error: {error:?}"),
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

impl From<Basis3Error> for SpaceBuildError {
    fn from(error: Basis3Error) -> Self {
        Self::Basis(error)
    }
}

pub fn build_blue_moon_scene() -> Result<Scene, SpaceBuildError> {
    build_blue_moon_scene_with_metadata().map(|built| built.0)
}

pub(crate) fn build_blue_moon_scene_with_metadata()
-> Result<(Scene, SpaceSceneMetadata), SpaceBuildError> {
    let (mut scene, materials) = base_space_scene_with_skybox(blue_moon_skybox()?)?;
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
    let (mut scene, materials) = base_space_scene_with_skybox(cookie_world_skybox()?)?;
    let mut metadata = SpaceSceneMetadata::default();

    add_cookie_level(&mut scene, &mut metadata, materials)?;
    add_space_lighting(&mut scene);
    add_cookie_level_lighting(&mut scene);

    Ok((scene, metadata))
}

pub fn build_galaxy_selector_scene() -> Result<Scene, SpaceBuildError> {
    let (mut scene, materials) = base_space_scene_with_skybox(space_menu_skybox()?)?;

    scene.set_ambient_light(Color::new(0.300, 0.340, 0.405));
    add_selector_worlds(&mut scene, materials)?;
    add_selector_lighting(&mut scene);

    Ok(scene)
}

pub fn build_level_three_scene() -> Result<Scene, SpaceBuildError> {
    build_level_three_scene_with_metadata().map(|built| built.0)
}

pub(crate) fn build_level_three_scene_with_metadata()
-> Result<(Scene, SpaceSceneMetadata), SpaceBuildError> {
    let (mut scene, materials) = base_space_scene_with_skybox(level_three_skybox()?)?;
    let mut metadata = SpaceSceneMetadata::default();

    add_level_three_placeholder(&mut scene, &mut metadata, materials)?;
    add_space_lighting(&mut scene);
    add_level_three_lighting(&mut scene);

    Ok((scene, metadata))
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
        BLUE_MOON_PLANET_CENTER,
        0.0,
        0.0,
        8.2,
        50.0,
        aspect_ratio,
        Vec3::new(0.18, 1.0, 0.0),
    )
}

pub fn cookie_world_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        COOKIE_PLANET_CENTER,
        -0.30,
        0.16,
        7.5,
        55.0,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

pub fn level_three_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        LEVEL_THREE_PLANET_CENTER,
        0.22,
        0.18,
        6.9,
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

pub fn galaxy_selector_worlds() -> [SelectorWorld; 3] {
    [
        SelectorWorld {
            planet: PlanetType::BlueMoon,
            center: GALAXY_SELECTOR_BLUE_MOON_CENTER,
            radius: GALAXY_SELECTOR_BLUE_MOON_RADIUS,
            level_number: 1,
            locked: true,
        },
        SelectorWorld {
            planet: PlanetType::CookieWorld,
            center: GALAXY_SELECTOR_COOKIE_CENTER,
            radius: GALAXY_SELECTOR_COOKIE_RADIUS,
            level_number: 2,
            locked: true,
        },
        SelectorWorld {
            planet: PlanetType::AsteroidBelt,
            center: GALAXY_SELECTOR_LEVEL_THREE_CENTER,
            radius: GALAXY_SELECTOR_LEVEL_THREE_RADIUS,
            level_number: 3,
            locked: true,
        },
    ]
}

fn base_space_scene() -> Result<(Scene, SpaceMaterials), SpaceBuildError> {
    base_space_scene_with_skybox(space_menu_skybox()?)
}

fn base_space_scene_with_skybox(
    skybox: Skybox,
) -> Result<(Scene, SpaceMaterials), SpaceBuildError> {
    let mut scene = Scene::new();

    scene.set_ambient_light(Color::new(0.240, 0.280, 0.340));
    scene.set_skybox(skybox);
    let materials = register_space_materials(&mut scene)?;

    Ok((scene, materials))
}

pub fn space_menu_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(space_skybox_texture()?)
        .with_intensity(1.18)
        .with_horizontal_rotation(0.0))
}

pub fn blue_moon_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(space_skybox_texture()?)
        .with_intensity(1.24)
        .with_horizontal_rotation(0.08))
}

pub fn cookie_world_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(space_skybox_texture()?)
        .with_intensity(1.16)
        .with_horizontal_rotation(0.34))
}

pub fn level_three_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(space_skybox_texture()?)
        .with_intensity(1.08)
        .with_horizontal_rotation(0.66))
}

pub fn angry_birds_space_skybox() -> Result<Skybox, TextureError> {
    space_menu_skybox()
}

fn space_skybox_texture() -> Result<Texture, TextureError> {
    let mut pixels = Vec::with_capacity(SPACE_SKYBOX_WIDTH * SPACE_SKYBOX_HEIGHT);

    for y in 0..SPACE_SKYBOX_HEIGHT {
        let v = 1.0 - y as f32 / (SPACE_SKYBOX_HEIGHT - 1) as f32;

        for x in 0..SPACE_SKYBOX_WIDTH {
            let u = x as f32 / (SPACE_SKYBOX_WIDTH - 1) as f32;
            pixels.push(space_skybox_color(u, v));
        }
    }

    Texture::new(SPACE_SKYBOX_WIDTH, SPACE_SKYBOX_HEIGHT, pixels)
}

fn space_skybox_color(u: f32, v: f32) -> Color {
    let u = u.rem_euclid(1.0);
    let bottom = Color::new(0.010, 0.045, 0.130);
    let top = Color::new(0.020, 0.095, 0.230);
    let lateral = Color::new(0.012, 0.070, 0.190);
    let lateral_mix = smoothstep(0.0, 1.0, 0.5 + 0.5 * (u * std::f32::consts::TAU).cos());
    let mut color = bottom.lerp(top, smoothstep(0.0, 1.0, v));
    color = color.lerp(lateral, lateral_mix * 0.20);

    color = add_cartoon_cloud_layers(color, u, v);

    let sun_distance = wrapped_uv_distance(u, v, SPACE_SUN_U, SPACE_SUN_V, 1.18);
    let broad_halo = 1.0 - smoothstep(0.13, 0.46, sun_distance);
    let warm_halo = 1.0 - smoothstep(0.065, 0.290, sun_distance);
    color += Color::new(0.070, 0.260, 0.310) * (broad_halo * 0.78);
    color += Color::new(1.000, 0.600, 0.150) * (warm_halo * 0.42);

    color = add_cartoon_sun(color, u, v, sun_distance);

    let mist = smooth_noise(u * 7.0 + 1.7, v * 5.0 + 0.9) * 0.010;
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
    let mottling = smooth_noise(u * 18.0 + 2.0, v * 15.0 + 4.0) * 0.09;
    sun_color += Color::new(0.060, 0.025, 0.0) * mottling;

    let spot_a = 1.0 - smoothstep(0.014, 0.034, wrapped_uv_distance(u, v, 0.100, 0.790, 1.35));
    let spot_b = 1.0 - smoothstep(0.011, 0.028, wrapped_uv_distance(u, v, 0.145, 0.735, 1.35));
    let spot_c = 1.0 - smoothstep(0.009, 0.023, wrapped_uv_distance(u, v, 0.170, 0.812, 1.35));
    let spots = (spot_a * 0.34 + spot_b * 0.26 + spot_c * 0.22).clamp(0.0, 0.45);
    sun_color = sun_color.lerp(Color::new(0.900, 0.430, 0.090), spots);

    color.lerp(sun_color, disk)
}

fn add_cartoon_cloud_layers(color: Color, u: f32, v: f32) -> Color {
    let lower_back = cloud_band(u, v, 0.220, 0.070, 0.038, 3.0, 0.25);
    let lower_front = cloud_band(u, v, 0.120, 0.052, 0.030, 4.0, 1.10);
    let side = side_cloud(u.min(1.0 - u), v, 0.042, 0.52, 0.052, 4.6);

    let color = color.lerp(Color::new(0.030, 0.145, 0.295), lower_back * 0.52);
    let color = color.lerp(Color::new(0.020, 0.098, 0.245), lower_front * 0.68);

    color.lerp(Color::new(0.017, 0.096, 0.245), side * 0.44)
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
        + (u * (frequency * 2.0) * std::f32::consts::TAU + phase * 0.7).sin() * height * 0.45;
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

fn smooth_noise(x: f32, y: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let tx = smoothstep(0.0, 1.0, x - x0);
    let ty = smoothstep(0.0, 1.0, y - y0);
    let a = smooth_hash(x0, y0);
    let b = smooth_hash(x0 + 1.0, y0);
    let c = smooth_hash(x0, y0 + 1.0);
    let d = smooth_hash(x0 + 1.0, y0 + 1.0);
    let top = a + (b - a) * tx;
    let bottom = c + (d - c) * tx;

    top + (bottom - top) * ty
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
        materials.selector_locked_moon,
    )?)?;
    scene.add_sphere(Sphere::new(
        GALAXY_SELECTOR_COOKIE_CENTER,
        GALAXY_SELECTOR_COOKIE_RADIUS * 1.17,
        materials.cookie_gravity_field,
    )?)?;
    scene.add_sphere(Sphere::new(
        GALAXY_SELECTOR_COOKIE_CENTER,
        GALAXY_SELECTOR_COOKIE_RADIUS,
        materials.selector_locked_cookie,
    )?)?;
    scene.add_sphere(Sphere::new(
        GALAXY_SELECTOR_LEVEL_THREE_CENTER,
        GALAXY_SELECTOR_LEVEL_THREE_RADIUS * 1.16,
        materials.level_three_gravity_field,
    )?)?;
    scene.add_sphere(Sphere::new(
        GALAXY_SELECTOR_LEVEL_THREE_CENTER,
        GALAXY_SELECTOR_LEVEL_THREE_RADIUS,
        materials.selector_locked_level_three,
    )?)?;

    Ok(())
}

fn add_blue_moon_world(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    add_planet(scene, metadata, materials)?;
    add_blue_moon_surface_details(scene, metadata, materials)?;
    add_blue_moon_wood_structure(scene, metadata, materials)?;
    add_blue_moon_side_wood_tower(scene, metadata, materials)?;
    add_blue_moon_second_structure(scene, metadata, materials)?;

    Ok(())
}

fn register_space_materials(scene: &mut Scene) -> Result<SpaceMaterials, SpaceBuildError> {
    let moon_texture = scene.add_texture(Texture::from_ppm_file(BLUE_MOON_PLANET_TEXTURE_PATH)?);
    let wood_block_texture = scene.add_texture(Texture::from_ppm_file(WOOD_BLOCK_TEXTURE_PATH)?);
    let stone_block_texture = scene.add_texture(Texture::from_ppm_file(STONE_BLOCK_TEXTURE_PATH)?);
    let tnt_crate_texture = scene.add_texture(Texture::from_ppm_file(TNT_CRATE_TEXTURE_PATH)?);

    let moon = scene.add_material(
        Material::new(
            Color::new(0.72, 0.78, 0.70),
            0.20,
            28.0,
            0.04,
            0.0,
            1.0,
            Color::new(0.105, 0.120, 0.095),
        )
        .with_texture(moon_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    let moon_crater_floor = scene.add_material(Material::new(
        Color::new(0.20, 0.30, 0.29),
        0.18,
        18.0,
        0.01,
        0.0,
        1.0,
        Color::new(0.005, 0.010, 0.008),
    ))?;
    let moon_stone = scene.add_material(
        Material::new(
            Color::new(0.42, 0.53, 0.49),
            0.20,
            22.0,
            0.02,
            0.0,
            1.0,
            Color::BLACK,
        )
        .with_texture(stone_block_texture, Vec2::new(1.55, 1.55), WrapMode::Repeat),
    )?;
    let moon_stone_light = scene.add_material(Material::new(
        Color::new(0.58, 0.68, 0.60),
        0.22,
        26.0,
        0.025,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let moon_dirt = scene.add_material(Material::new(
        Color::new(0.28, 0.12, 0.07),
        0.18,
        16.0,
        0.01,
        0.0,
        1.0,
        Color::new(0.012, 0.004, 0.001),
    ))?;
    let gravity_field = scene.add_material(Material::new(
        Color::new(0.76, 0.96, 1.0),
        0.16,
        44.0,
        0.025,
        0.992,
        1.005,
        Color::new(0.105, 0.240, 0.380),
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
    let level_three = scene.add_material(Material::new(
        Color::new(0.44, 0.30, 0.74),
        0.26,
        32.0,
        0.08,
        0.0,
        1.0,
        Color::new(0.012, 0.0, 0.028),
    ))?;
    let level_three_gravity_field = scene.add_material(Material::new(
        Color::new(0.78, 0.42, 1.0),
        0.10,
        48.0,
        0.012,
        0.982,
        1.005,
        Color::new(0.060, 0.018, 0.110),
    ))?;
    let selector_locked_moon = scene.add_material(Material::new(
        Color::new(0.18, 0.24, 0.34),
        0.12,
        16.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let selector_locked_cookie = scene.add_material(Material::new(
        Color::new(0.27, 0.16, 0.07),
        0.12,
        16.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let selector_locked_level_three = scene.add_material(Material::new(
        Color::new(0.17, 0.10, 0.29),
        0.14,
        18.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let asteroid = scene.add_material(Material::new(
        Color::new(0.36, 0.18, 0.10),
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
    let wood = scene.add_material(
        Material::new(
            Color::new(0.58, 0.34, 0.16),
            0.25,
            24.0,
            0.02,
            0.0,
            1.0,
            Color::BLACK,
        )
        .with_texture(wood_block_texture, Vec2::new(2.4, 1.1), WrapMode::Repeat),
    )?;
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
    let tnt_crate = scene.add_material(
        Material::new(
            Color::WHITE,
            0.18,
            20.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.05, 0.03, 0.015),
        )
        .with_texture(tnt_crate_texture, Vec2::new(1.0, 1.0), WrapMode::Clamp),
    )?;

    Ok(SpaceMaterials {
        moon,
        moon_crater_floor,
        moon_stone,
        moon_stone_light,
        moon_dirt,
        gravity_field,
        cookie_gravity_field,
        level_three,
        level_three_gravity_field,
        selector_locked_moon,
        selector_locked_cookie,
        selector_locked_level_three,
        asteroid,
        cookie,
        chocolate,
        wood,
        ice,
        metal,
        tnt,
        tnt_crate,
        candy_red,
        candy_blue,
        candy_yellow,
        pig,
        snout,
        eye,
        pupil,
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

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct BlueMoonCrater {
    latitude: f32,
    longitude: f32,
    radius: f32,
}

#[derive(Debug, Clone, Copy)]
struct BlueMoonStone {
    latitude: f32,
    longitude: f32,
    radius: f32,
    light: bool,
}

const BLUE_MOON_CRATERS: [BlueMoonCrater; 5] = [
    BlueMoonCrater {
        latitude: 0.46,
        longitude: -0.32,
        radius: 0.145,
    },
    BlueMoonCrater {
        latitude: 0.24,
        longitude: 0.58,
        radius: 0.115,
    },
    BlueMoonCrater {
        latitude: -0.06,
        longitude: -0.82,
        radius: 0.135,
    },
    BlueMoonCrater {
        latitude: -0.34,
        longitude: 0.08,
        radius: 0.105,
    },
    BlueMoonCrater {
        latitude: -0.48,
        longitude: 0.96,
        radius: 0.095,
    },
];

const BLUE_MOON_STONES: [BlueMoonStone; 9] = [
    BlueMoonStone {
        latitude: 0.52,
        longitude: -0.13,
        radius: 0.042,
        light: true,
    },
    BlueMoonStone {
        latitude: 0.36,
        longitude: -0.48,
        radius: 0.034,
        light: false,
    },
    BlueMoonStone {
        latitude: 0.28,
        longitude: 0.76,
        radius: 0.030,
        light: true,
    },
    BlueMoonStone {
        latitude: 0.11,
        longitude: 0.42,
        radius: 0.026,
        light: false,
    },
    BlueMoonStone {
        latitude: -0.02,
        longitude: -1.02,
        radius: 0.038,
        light: false,
    },
    BlueMoonStone {
        latitude: -0.18,
        longitude: -0.64,
        radius: 0.030,
        light: true,
    },
    BlueMoonStone {
        latitude: -0.36,
        longitude: 0.29,
        radius: 0.032,
        light: false,
    },
    BlueMoonStone {
        latitude: -0.51,
        longitude: 0.74,
        radius: 0.027,
        light: true,
    },
    BlueMoonStone {
        latitude: -0.58,
        longitude: 1.14,
        radius: 0.035,
        light: false,
    },
];

fn add_blue_moon_surface_details(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    for crater in BLUE_MOON_CRATERS {
        let frame = RadialFrame::from_latitude_longitude(
            BLUE_MOON_PLANET_CENTER,
            BLUE_MOON_PLANET_RADIUS,
            crater.latitude,
            crater.longitude,
        )?;

        add_radial_cylinder(
            scene,
            frame,
            Vec3::new(0.0, 0.018, 0.0),
            crater.radius,
            0.014,
            materials.moon_crater_floor,
        )?;
        metadata.blue_moon_crater_count += 1;
    }

    for stone in BLUE_MOON_STONES {
        let frame = RadialFrame::from_latitude_longitude(
            BLUE_MOON_PLANET_CENTER,
            BLUE_MOON_PLANET_RADIUS,
            stone.latitude,
            stone.longitude,
        )?;
        let material = if stone.light {
            materials.moon_stone_light
        } else {
            materials.moon_stone
        };

        scene.add_sphere(Sphere::new(
            frame.position(0.0, stone.radius + 0.024, 0.0),
            stone.radius,
            material,
        )?)?;
        metadata.blue_moon_stone_count += 1;
    }

    Ok(())
}

fn add_blue_moon_wood_structure(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let frame = blue_moon_structure_frame()?;

    for (center, half_extents) in [
        (Vec3::new(0.00, 0.006, 0.00), Vec3::new(0.50, 0.072, 0.30)),
        (Vec3::new(-0.24, 0.012, -0.04), Vec3::new(0.23, 0.072, 0.23)),
        (Vec3::new(0.26, 0.012, 0.07), Vec3::new(0.24, 0.072, 0.20)),
    ] {
        add_radial_box(scene, frame, center, half_extents, materials.moon_dirt)?;
        metadata.blue_moon_dirt_base_parts += 1;
    }

    for (center, half_extents) in [
        (
            Vec3::new(-0.30, 0.250, -0.08),
            Vec3::new(0.045, 0.190, 0.045),
        ),
        (
            Vec3::new(0.26, 0.275, -0.08),
            Vec3::new(0.045, 0.215, 0.045),
        ),
        (
            Vec3::new(-0.02, 0.490, -0.08),
            Vec3::new(0.360, 0.050, 0.050),
        ),
        (
            Vec3::new(-0.02, 0.165, 0.12),
            Vec3::new(0.440, 0.040, 0.040),
        ),
    ] {
        add_radial_box(scene, frame, center, half_extents, materials.wood)?;
        metadata.wood_parts += 1;
    }

    for (center, half_extents) in [
        (
            Vec3::new(-0.02, 0.610, -0.08),
            Vec3::new(0.300, 0.055, 0.055),
        ),
        (
            Vec3::new(-0.24, 0.780, -0.08),
            Vec3::new(0.055, 0.170, 0.055),
        ),
        (
            Vec3::new(0.20, 0.790, -0.08),
            Vec3::new(0.055, 0.180, 0.055),
        ),
        (
            Vec3::new(-0.02, 0.975, -0.08),
            Vec3::new(0.300, 0.055, 0.055),
        ),
        (
            Vec3::new(-0.17, 1.170, -0.08),
            Vec3::new(0.055, 0.170, 0.055),
        ),
        (
            Vec3::new(0.13, 1.170, -0.08),
            Vec3::new(0.055, 0.170, 0.055),
        ),
    ] {
        add_radial_box(scene, frame, center, half_extents, materials.moon_stone)?;
        metadata.blue_moon_stone_structure_parts += 1;
    }

    // Each pig rests on the top face of a structure part:
    // (tangent_x, top of the supporting part, tangent_z, radius).
    for (tangent_x, support_top, tangent_z, radius) in [
        // Front wood beam: center y 0.165 + half height 0.040.
        (-0.02, 0.205, 0.12, 0.105),
        // First stone floor, between the stone pillars: 0.610 + 0.055.
        (-0.02, 0.665, -0.08, 0.095),
        // Second stone floor, between the top pillars: 0.975 + 0.055.
        (-0.02, 1.030, -0.08, 0.085),
    ] {
        let center = Vec3::new(tangent_x, support_top + radius, tangent_z);
        add_space_pig_at(scene, frame, center, radius, materials)?;
        metadata.blue_moon_pig_count += 1;
        metadata.pig_count += 1;
    }

    Ok(())
}

fn add_blue_moon_side_wood_tower(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let frame = blue_moon_side_tower_frame()?;
    let lower_beam_top = 0.225;
    let middle_beam_top = 0.705;

    add_radial_box(
        scene,
        frame,
        Vec3::new(0.00, -0.025, 0.00),
        Vec3::new(0.40, 0.105, 0.24),
        materials.moon_dirt,
    )?;
    metadata.blue_moon_dirt_base_parts += 1;

    for (center, half_extents) in [
        (Vec3::new(0.00, 0.185, 0.06), Vec3::new(0.36, 0.040, 0.040)),
        (
            Vec3::new(-0.28, 0.430, 0.02),
            Vec3::new(0.040, 0.245, 0.040),
        ),
        (Vec3::new(0.24, 0.430, 0.02), Vec3::new(0.040, 0.245, 0.040)),
        (Vec3::new(-0.02, 0.665, 0.02), Vec3::new(0.34, 0.040, 0.040)),
        (
            Vec3::new(-0.10, 0.825, -0.07),
            Vec3::new(0.035, 0.120, 0.035),
        ),
        (
            Vec3::new(0.11, 0.825, -0.07),
            Vec3::new(0.035, 0.120, 0.035),
        ),
        (
            Vec3::new(0.005, 0.965, -0.07),
            Vec3::new(0.140, 0.035, 0.035),
        ),
        (
            Vec3::new(-0.31, 0.955, 0.02),
            Vec3::new(0.040, 0.220, 0.040),
        ),
        (Vec3::new(0.31, 0.955, 0.02), Vec3::new(0.040, 0.220, 0.040)),
        (Vec3::new(0.00, 1.190, 0.02), Vec3::new(0.42, 0.045, 0.045)),
    ] {
        add_radial_box(scene, frame, center, half_extents, materials.wood)?;
        metadata.wood_parts += 1;
    }

    add_radial_box(
        scene,
        frame,
        Vec3::new(0.36, middle_beam_top + 0.060, 0.02),
        Vec3::new(0.070, 0.060, 0.050),
        materials.moon_stone,
    )?;
    metadata.blue_moon_stone_structure_parts += 1;

    for (center, radius) in [
        (Vec3::new(0.10, lower_beam_top + 0.095, 0.055), 0.095),
        (Vec3::new(-0.16, middle_beam_top + 0.085, 0.055), 0.085),
    ] {
        add_space_pig_at(scene, frame, center, radius, materials)?;
        metadata.blue_moon_pig_count += 1;
        metadata.pig_count += 1;
    }

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

fn add_level_three_placeholder(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let planet_id = scene.object_count();
    scene.add_sphere(Sphere::new(
        LEVEL_THREE_PLANET_CENTER,
        LEVEL_THREE_PLANET_RADIUS,
        materials.level_three,
    )?)?;
    metadata.level_three_planet_id = Some(planet_id);

    for (offset, radius, material_id) in [
        (Vec3::new(-2.20, 0.42, -0.32), 0.22, materials.asteroid),
        (Vec3::new(-1.55, -0.34, 0.54), 0.16, materials.metal),
        (Vec3::new(1.86, 0.30, -0.28), 0.20, materials.asteroid),
        (Vec3::new(2.34, -0.42, 0.36), 0.13, materials.ice),
        (Vec3::new(0.62, 1.78, -0.22), 0.15, materials.candy_blue),
    ] {
        scene.add_sphere(Sphere::new(
            LEVEL_THREE_PLANET_CENTER + offset,
            radius,
            material_id,
        )?)?;
        metadata.decorative_asteroid_count += 1;
    }

    let gravity_id = scene.object_count();
    scene.add_sphere(Sphere::new(
        LEVEL_THREE_PLANET_CENTER,
        LEVEL_THREE_PLANET_RADIUS * 1.18,
        materials.level_three_gravity_field,
    )?)?;
    metadata.level_three_gravity_field_id = Some(gravity_id);

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

fn add_level_three_lighting(scene: &mut Scene) {
    scene.add_light(PointLight::new(
        Vec3::new(-4.0, 2.6, 4.8),
        Color::new(0.72, 0.48, 1.0),
        8.6,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(2.8, -0.4, 4.2),
        Color::new(0.42, 0.88, 1.0),
        4.2,
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
    add_space_pig_at(
        scene,
        frame,
        Vec3::new(tangent_x, radius + 0.04, tangent_z),
        radius,
        materials,
    )
}

fn add_space_pig_at(
    scene: &mut Scene,
    frame: RadialFrame,
    local_center: Vec3,
    radius: f32,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    // Pig local axes follow the frame: X is right, Y is radial (away from the
    // ground) and Z is the direction the pig faces, parallel to the ground.
    scene.add_sphere(Sphere::new(
        frame.local_to_world(local_center),
        radius,
        materials.pig,
    )?)?;
    scene.add_cylinder(Cylinder::new(
        frame.local_to_world(local_center + Vec3::new(0.0, -radius * 0.03, radius * 0.76)),
        radius * 0.34,
        radius * 0.18,
        facing_basis(frame)?,
        materials.snout,
    )?)?;

    for eye_x in [-radius * 0.34, radius * 0.34] {
        scene.add_sphere(Sphere::new(
            frame.local_to_world(local_center + Vec3::new(eye_x, radius * 0.42, radius * 0.90)),
            radius * 0.16,
            materials.eye,
        )?)?;
        scene.add_sphere(Sphere::new(
            frame.local_to_world(local_center + Vec3::new(eye_x, radius * 0.42, radius * 1.04)),
            radius * 0.07,
            materials.pupil,
        )?)?;
    }

    for ear_x in [-radius * 0.44, radius * 0.44] {
        scene.add_cone(Cone::new(
            frame.local_to_world(local_center + Vec3::new(ear_x, radius * 0.88, 0.0)),
            radius * 0.14,
            radius * 0.18,
            frame.basis(),
            materials.pig,
        )?)?;
    }

    Ok(())
}

/// Second fortress of Luna Azul: wood and stone pillars with a TNT crate on
/// the ground, a stone floor holding one pig, and a hollow stone square with a
/// pillar on top. It stands on its own dirt base, separated from the first one.
fn add_blue_moon_second_structure(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let frame = blue_moon_second_structure_frame()?;
    // Top of the dirt base, where the ground-level pieces stand.
    let ground_top = 0.08;
    // Top of the first stone floor, where the pig rests.
    let first_floor_top = 0.43;
    let pig_radius = 0.105;

    for (center, half_extents, material_id) in [
        // Dirt base, thick enough to stay buried under its corners.
        (
            Vec3::new(0.00, -0.03, 0.00),
            Vec3::new(0.44, 0.11, 0.24),
            materials.moon_dirt,
        ),
        // Ground level: stone pillar, TNT crate and two wood pillars.
        (
            Vec3::new(-0.35, ground_top + 0.13, 0.00),
            Vec3::new(0.05, 0.13, 0.045),
            materials.moon_stone,
        ),
        (
            Vec3::new(-0.15, ground_top + 0.10, 0.00),
            Vec3::new(0.10, 0.10, 0.10),
            materials.tnt_crate,
        ),
        (
            Vec3::new(0.06, ground_top + 0.13, 0.00),
            Vec3::new(0.045, 0.13, 0.045),
            materials.wood,
        ),
        (
            Vec3::new(0.30, ground_top + 0.13, 0.00),
            Vec3::new(0.045, 0.13, 0.045),
            materials.wood,
        ),
        // First stone floor.
        (
            Vec3::new(-0.02, first_floor_top - 0.045, 0.00),
            Vec3::new(0.38, 0.045, 0.06),
            materials.moon_stone,
        ),
        // Stone pillars around the pig.
        (
            Vec3::new(-0.22, first_floor_top + 0.12, 0.00),
            Vec3::new(0.045, 0.12, 0.045),
            materials.moon_stone,
        ),
        (
            Vec3::new(0.18, first_floor_top + 0.12, 0.00),
            Vec3::new(0.045, 0.12, 0.045),
            materials.moon_stone,
        ),
        // Second stone floor.
        (
            Vec3::new(-0.02, 0.71, 0.00),
            Vec3::new(0.28, 0.04, 0.06),
            materials.moon_stone,
        ),
        // Hollow stone square: bottom, top, left and right sides.
        (
            Vec3::new(0.02, 0.775, 0.00),
            Vec3::new(0.13, 0.025, 0.05),
            materials.moon_stone,
        ),
        (
            Vec3::new(0.02, 0.985, 0.00),
            Vec3::new(0.13, 0.025, 0.05),
            materials.moon_stone,
        ),
        (
            Vec3::new(-0.085, 0.88, 0.00),
            Vec3::new(0.025, 0.08, 0.045),
            materials.moon_stone,
        ),
        (
            Vec3::new(0.125, 0.88, 0.00),
            Vec3::new(0.025, 0.08, 0.045),
            materials.moon_stone,
        ),
        // Pillar on top of the square.
        (
            Vec3::new(0.02, 1.13, 0.00),
            Vec3::new(0.04, 0.12, 0.04),
            materials.moon_stone,
        ),
    ] {
        add_radial_box(scene, frame, center, half_extents, material_id)?;
        metadata.blue_moon_second_structure_parts += 1;
        if material_id == materials.tnt_crate {
            metadata.tnt_parts += 1;
        }
    }

    add_space_pig_at(
        scene,
        frame,
        Vec3::new(-0.02, first_floor_top + pig_radius, 0.00),
        pig_radius,
        materials,
    )?;
    metadata.blue_moon_pig_count += 1;
    metadata.pig_count += 1;

    Ok(())
}

/// Basis whose local Y axis points along the frame tangent Z, so cylinders and
/// cones built with it lie parallel to the ground and point to the front.
fn facing_basis(frame: RadialFrame) -> Result<Basis3, SpaceBuildError> {
    let basis = frame.basis();

    Ok(Basis3::new(basis.right(), basis.forward(), -basis.up())?)
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

fn blue_moon_structure_frame() -> Result<RadialFrame, SpaceBuildError> {
    Ok(RadialFrame::from_latitude_longitude(
        BLUE_MOON_PLANET_CENTER,
        BLUE_MOON_PLANET_RADIUS,
        0.40,
        -0.18,
    )?)
}

fn blue_moon_second_structure_frame() -> Result<RadialFrame, SpaceBuildError> {
    Ok(RadialFrame::from_latitude_longitude(
        BLUE_MOON_PLANET_CENTER,
        BLUE_MOON_PLANET_RADIUS,
        0.60,
        0.90,
    )?)
}

fn blue_moon_side_tower_frame() -> Result<RadialFrame, SpaceBuildError> {
    Ok(RadialFrame::from_latitude_longitude(
        BLUE_MOON_PLANET_CENTER,
        BLUE_MOON_PLANET_RADIUS,
        0.40,
        -1.10,
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
        BLUE_MOON_LEVEL_PIG_COUNT, BLUE_MOON_PLANET_CENTER, BLUE_MOON_PLANET_RADIUS,
        BLUE_MOON_PLANET_TEXTURE_PATH, COOKIE_LEVEL_PIG_COUNT, COOKIE_PLANET_CENTER,
        COOKIE_PLANET_RADIUS, GALAXY_SELECTOR_BLUE_MOON_CENTER, GALAXY_SELECTOR_COOKIE_CENTER,
        GALAXY_SELECTOR_LEVEL_THREE_CENTER, LEVEL_THREE_PLANET_CENTER, LEVEL_THREE_PLANET_RADIUS,
        PlanetType, SKYBOX_ASTEROID_A_U, SKYBOX_ASTEROID_A_V, SPACE_SUN_DIRECTION, SPACE_SUN_U,
        SPACE_SUN_V, STONE_BLOCK_TEXTURE_PATH, SceneState, SpaceMaterials, TNT_CRATE_TEXTURE_PATH,
        WOOD_BLOCK_TEXTURE_PATH, add_radial_box, blue_moon_orbit_camera,
        blue_moon_second_structure_frame, blue_moon_side_tower_frame, blue_moon_skybox,
        blue_moon_structure_frame, build_blue_moon_scene_with_metadata,
        build_cookie_world_scene_with_metadata, build_galaxy_selector_scene,
        build_level_three_scene_with_metadata, build_space_levels_scene_with_metadata,
        cookie_level_frame, cookie_world_orbit_camera, cookie_world_skybox,
        galaxy_selector_orbit_camera, galaxy_selector_worlds, level_three_orbit_camera,
        level_three_skybox, register_space_materials, space_levels_orbit_camera, space_menu_skybox,
        space_skybox_color, space_skybox_texture, sun_light_position, wrapped_uv_distance,
    };
    use crate::{
        color::Color,
        material::Material,
        math::Vec3,
        oriented_box::OrientedBox,
        radial::RadialFrame,
        ray::Ray,
        scene::Scene,
        texture::{Texture, WrapMode},
    };

    fn color_delta(left: Color, right: Color) -> f32 {
        (left.r - right.r).abs() + (left.g - right.g).abs() + (left.b - right.b).abs()
    }

    fn luminance(color: Color) -> f32 {
        color.r * 0.2126 + color.g * 0.7152 + color.b * 0.0722
    }

    #[test]
    fn blue_moon_scene_builds_valid_scene() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();

        assert!(scene.object_count() > 0);
        assert!(scene.object_count() < 180);
        assert!(metadata.planet_id.is_some());
        assert!(metadata.cookie_planet_id.is_none());
        assert_eq!(metadata.pig_count, BLUE_MOON_LEVEL_PIG_COUNT);
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
        assert_ne!(
            SceneState::Planet(PlanetType::CookieWorld),
            SceneState::Planet(PlanetType::AsteroidBelt)
        );
    }

    #[test]
    fn galaxy_selector_declares_three_clickable_numbered_locked_worlds() {
        let worlds = galaxy_selector_worlds();

        assert_eq!(worlds.len(), 3);
        assert_eq!(worlds[0].planet, PlanetType::BlueMoon);
        assert_eq!(worlds[0].center, GALAXY_SELECTOR_BLUE_MOON_CENTER);
        assert_eq!(worlds[0].level_number, 1);
        assert!(worlds[0].locked);
        assert!(worlds[0].radius > 0.0);
        assert_eq!(worlds[1].planet, PlanetType::CookieWorld);
        assert_eq!(worlds[1].center, GALAXY_SELECTOR_COOKIE_CENTER);
        assert_eq!(worlds[1].level_number, 2);
        assert!(worlds[1].locked);
        assert!(worlds[1].radius > 0.0);
        assert_eq!(worlds[2].planet, PlanetType::AsteroidBelt);
        assert_eq!(worlds[2].center, GALAXY_SELECTOR_LEVEL_THREE_CENTER);
        assert_eq!(worlds[2].level_number, 3);
        assert!(worlds[2].locked);
        assert!(worlds[2].radius > 0.0);
    }

    #[test]
    fn galaxy_selector_scene_contains_only_selector_world_geometry() {
        let scene = build_galaxy_selector_scene().unwrap();

        assert_eq!(scene.sphere_count(), 6);
        assert_eq!(scene.object_count(), 6);
        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 3);
        assert!(scene.ambient_light().b > 0.39);
    }

    #[test]
    fn galaxy_selector_planet_bodies_use_dark_locked_materials() {
        let scene = build_galaxy_selector_scene().unwrap();

        for body_index in [1, 3, 5] {
            let body = scene.objects()[body_index].as_sphere().unwrap();
            let material = scene.material(body.material_id()).unwrap();

            assert!(material.albedo.r <= 0.30);
            assert!(material.albedo.g <= 0.30);
            assert!(material.albedo.b <= 0.36);
            assert_eq!(material.transparency, 0.0);
        }
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

        assert!(texture.width() >= 960);
        assert!(texture.height() >= 480);
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
    fn space_skybox_background_samples_change_smoothly() {
        let base = space_skybox_color(0.52, 0.52);
        let horizontal_neighbor = space_skybox_color(0.521, 0.52);
        let vertical_neighbor = space_skybox_color(0.52, 0.521);
        let left_seam = space_skybox_color(0.001, 0.52);
        let right_seam = space_skybox_color(0.999, 0.52);

        assert!(color_delta(base, horizontal_neighbor) < 0.015);
        assert!(color_delta(base, vertical_neighbor) < 0.015);
        assert!(color_delta(left_seam, right_seam) < 0.015);
    }

    #[test]
    fn generated_space_skybox_wraps_without_visible_vertical_seam() {
        let texture = space_skybox_texture().unwrap();
        let middle_y = texture.height() / 2;
        let left_edge = texture.pixel(0, middle_y).unwrap();
        let right_edge = texture.pixel(texture.width() - 1, middle_y).unwrap();

        assert!(color_delta(left_edge, right_edge) < 0.001);
    }

    #[test]
    fn spatial_scenes_use_separate_space_skybox_presets() {
        let selector = build_galaxy_selector_scene().unwrap();
        let (blue, _) = build_blue_moon_scene_with_metadata().unwrap();
        let (cookie, _) = build_cookie_world_scene_with_metadata().unwrap();
        let (level_three, _) = build_level_three_scene_with_metadata().unwrap();
        let (combined, _) = build_space_levels_scene_with_metadata().unwrap();
        let menu_preset = space_menu_skybox().unwrap();
        let blue_preset = blue_moon_skybox().unwrap();
        let cookie_preset = cookie_world_skybox().unwrap();
        let level_three_preset = level_three_skybox().unwrap();

        assert_eq!(menu_preset.texture().width(), super::SPACE_SKYBOX_WIDTH);
        assert_eq!(menu_preset.texture().height(), super::SPACE_SKYBOX_HEIGHT);
        assert_ne!(
            menu_preset.horizontal_rotation(),
            blue_preset.horizontal_rotation()
        );
        assert_ne!(
            blue_preset.horizontal_rotation(),
            cookie_preset.horizontal_rotation()
        );
        assert_ne!(
            cookie_preset.horizontal_rotation(),
            level_three_preset.horizontal_rotation()
        );

        for scene in [&selector, &blue, &cookie, &level_three, &combined] {
            let skybox = scene.skybox().unwrap();

            assert_eq!(skybox.texture().width(), super::SPACE_SKYBOX_WIDTH);
            assert_eq!(skybox.texture().height(), super::SPACE_SKYBOX_HEIGHT);
            assert!(skybox.intensity() > 1.0);
        }

        assert_eq!(
            selector.skybox().unwrap().horizontal_rotation(),
            menu_preset.horizontal_rotation()
        );
        assert_eq!(
            blue.skybox().unwrap().horizontal_rotation(),
            blue_preset.horizontal_rotation()
        );
        assert_eq!(
            cookie.skybox().unwrap().horizontal_rotation(),
            cookie_preset.horizontal_rotation()
        );
        assert_eq!(
            level_three.skybox().unwrap().horizontal_rotation(),
            level_three_preset.horizontal_rotation()
        );
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

        assert_eq!(metadata.pig_count, BLUE_MOON_LEVEL_PIG_COUNT);
        assert_eq!(metadata.blue_moon_pig_count, BLUE_MOON_LEVEL_PIG_COUNT);
        assert_eq!(metadata.decorative_asteroid_count, 0);
        assert_eq!(metadata.wood_parts, 14);
        assert_eq!(metadata.ice_or_metal_parts, 0);
        assert_eq!(metadata.tnt_parts, 1);
        assert_eq!(metadata.blue_moon_crater_count, 5);
        assert_eq!(metadata.blue_moon_stone_count, 9);
        assert_eq!(metadata.blue_moon_dirt_base_parts, 4);
        assert_eq!(metadata.blue_moon_stone_structure_parts, 7);
        assert_eq!(metadata.blue_moon_second_structure_parts, 14);
        assert_eq!(
            scene.sphere_count(),
            1 + metadata.blue_moon_stone_count + metadata.blue_moon_pig_count * 5
        );
        assert_eq!(
            scene.cylinder_count(),
            metadata.blue_moon_crater_count + metadata.blue_moon_pig_count
        );
        assert_eq!(scene.cone_count(), metadata.blue_moon_pig_count * 2);
        assert_eq!(
            scene.oriented_box_count(),
            metadata.blue_moon_dirt_base_parts
                + metadata.wood_parts
                + metadata.blue_moon_stone_structure_parts
                + metadata.blue_moon_second_structure_parts
        );
    }

    #[test]
    fn blue_moon_scene_is_clean_for_manual_layout_pass() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();

        assert!(metadata.planet_id.is_some());
        assert!(metadata.cookie_planet_id.is_none());
        assert_eq!(metadata.pig_count, BLUE_MOON_LEVEL_PIG_COUNT);
        assert_eq!(
            scene.object_count(),
            1 + metadata.blue_moon_crater_count
                + metadata.blue_moon_stone_count
                + metadata.blue_moon_dirt_base_parts
                + metadata.wood_parts
                + metadata.blue_moon_stone_structure_parts
                + metadata.blue_moon_second_structure_parts
                + metadata.blue_moon_pig_count * 8,
            "the blue moon level should contain only the textured planet, surface details, both structures, and pigs"
        );
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
    fn blue_moon_planet_uses_beak_impact_texture_asset() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let planet = scene.objects()[metadata.planet_id.unwrap()]
            .as_sphere()
            .unwrap();
        let material = scene.material(planet.material_id()).unwrap();
        let texture_id = material.texture_id.unwrap();
        let texture = scene.texture(texture_id).unwrap();
        let top = texture
            .pixel(texture.width() / 2, texture.height() / 8)
            .unwrap();
        let middle = texture
            .pixel(texture.width() / 2, texture.height() / 2)
            .unwrap();
        let lower = texture
            .pixel(texture.width() / 2, texture.height() - texture.height() / 8)
            .unwrap();
        let north_pole = texture.pixel(texture.width() / 2, 0).unwrap();
        let south_pole = texture
            .pixel(texture.width() / 2, texture.height() - 1)
            .unwrap();

        assert_eq!(
            BLUE_MOON_PLANET_TEXTURE_PATH,
            "assets/textures/blue_moon_planet.ppm"
        );
        assert_eq!(texture.width(), 256);
        assert_eq!(texture.height(), 128);
        assert_eq!(material.uv_scale, crate::math::Vec2::new(1.0, 1.0));
        assert!(color_delta(top, middle) > 0.02);
        assert!(color_delta(middle, lower) > 0.02);
        assert!(luminance(north_pole) > 0.28);
        assert!(luminance(south_pole) > 0.22);
    }

    #[test]
    fn blue_moon_planet_keeps_main_body_and_small_stones() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let planet = scene.objects()[metadata.planet_id.unwrap()]
            .as_sphere()
            .unwrap();
        let mut surface_stones = 0;

        assert_eq!(planet.center(), BLUE_MOON_PLANET_CENTER);
        assert_eq!(planet.radius(), BLUE_MOON_PLANET_RADIUS);
        assert_eq!(
            scene.sphere_count(),
            1 + metadata.blue_moon_stone_count + metadata.blue_moon_pig_count * 5
        );

        for primitive in scene.objects() {
            let Some(stone) = primitive.as_sphere() else {
                continue;
            };

            if stone.center() == BLUE_MOON_PLANET_CENTER {
                continue;
            }

            let surface_clearance =
                (stone.center() - BLUE_MOON_PLANET_CENTER).length() - BLUE_MOON_PLANET_RADIUS;

            if surface_clearance < 0.080 {
                surface_stones += 1;
                assert!(stone.radius() <= 0.045);
                assert!(surface_clearance > stone.radius());
            }
        }

        assert_eq!(surface_stones, metadata.blue_moon_stone_count);
    }

    #[test]
    fn blue_moon_crater_floors_are_slightly_lifted_from_texture() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let SpaceMaterials {
            moon_crater_floor, ..
        } = register_space_materials(&mut Scene::new()).unwrap();
        let craters: Vec<_> = scene
            .cylinders()
            .filter(|cylinder| cylinder.material_id() == moon_crater_floor)
            .collect();

        assert_eq!(craters.len(), metadata.blue_moon_crater_count);

        for crater in craters {
            let surface_clearance =
                (crater.center() - BLUE_MOON_PLANET_CENTER).length() - BLUE_MOON_PLANET_RADIUS;

            assert!(crater.radius() >= 0.090);
            assert!(crater.radius() <= 0.150);
            assert!(crater.half_height() <= 0.015);
            assert!(surface_clearance > crater.half_height());
            assert!(surface_clearance < 0.025);
        }
    }

    fn structure_local_point(frame: RadialFrame, point: Vec3) -> Vec3 {
        frame
            .basis()
            .world_to_local_vector(point - frame.surface_point())
    }

    fn blue_moon_structure_frames() -> [RadialFrame; 3] {
        [
            blue_moon_structure_frame().unwrap(),
            blue_moon_second_structure_frame().unwrap(),
            blue_moon_side_tower_frame().unwrap(),
        ]
    }

    /// Index of the Luna Azul structure whose radial direction is closest to `point`.
    fn blue_moon_structure_index(point: Vec3) -> usize {
        let direction = (point - BLUE_MOON_PLANET_CENTER).normalized();
        let frames = blue_moon_structure_frames();

        frames
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| {
                direction
                    .dot(left.outward())
                    .total_cmp(&direction.dot(right.outward()))
            })
            .map(|(index, _)| index)
            .unwrap()
    }

    fn blue_moon_structure_boxes(scene: &Scene, index: usize) -> Vec<OrientedBox> {
        scene
            .oriented_boxes()
            .filter(|part| blue_moon_structure_index(part.center()) == index)
            .copied()
            .collect()
    }

    /// Largest gap between the projections of two boxes over the separating
    /// axis candidates. A positive value is a lower bound of their distance.
    fn oriented_box_separation(left: &OrientedBox, right: &OrientedBox) -> f32 {
        let left_axes = [
            left.orientation().right(),
            left.orientation().up(),
            left.orientation().forward(),
        ];
        let right_axes = [
            right.orientation().right(),
            right.orientation().up(),
            right.orientation().forward(),
        ];
        let mut candidates = Vec::new();
        candidates.extend(left_axes);
        candidates.extend(right_axes);
        for left_axis in left_axes {
            for right_axis in right_axes {
                let cross = left_axis.cross(right_axis);
                if cross.length() > 0.0001 {
                    candidates.push(cross.normalized());
                }
            }
        }
        let projected_radius = |part: &OrientedBox, axes: [Vec3; 3], axis: Vec3| {
            let half = part.half_extents();
            half.x * axes[0].dot(axis).abs()
                + half.y * axes[1].dot(axis).abs()
                + half.z * axes[2].dot(axis).abs()
        };

        candidates
            .into_iter()
            .map(|axis| {
                (right.center() - left.center()).dot(axis).abs()
                    - projected_radius(left, left_axes, axis)
                    - projected_radius(right, right_axes, axis)
            })
            .fold(f32::MIN, f32::max)
    }

    #[test]
    fn blue_moon_pigs_rest_on_top_of_structure_parts() {
        const CONTACT_TOLERANCE: f32 = 0.001;
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let SpaceMaterials { pig, .. } = register_space_materials(&mut Scene::new()).unwrap();
        let frames = blue_moon_structure_frames();
        let pig_bodies: Vec<_> = scene
            .objects()
            .iter()
            .filter_map(|primitive| primitive.as_sphere())
            .filter(|sphere| sphere.material_id() == pig)
            .collect();

        assert_eq!(pig_bodies.len(), metadata.blue_moon_pig_count);

        for body in pig_bodies {
            let index = blue_moon_structure_index(body.center());
            let frame = frames[index];
            let center = structure_local_point(frame, body.center());
            let bottom = center.y - body.radius();
            let mut supports = 0;

            for part in blue_moon_structure_boxes(&scene, index) {
                let part_center = structure_local_point(frame, part.center());
                let half_extents = part.half_extents();
                let closest = Vec3::new(
                    center.x.clamp(
                        part_center.x - half_extents.x,
                        part_center.x + half_extents.x,
                    ),
                    center.y.clamp(
                        part_center.y - half_extents.y,
                        part_center.y + half_extents.y,
                    ),
                    center.z.clamp(
                        part_center.z - half_extents.z,
                        part_center.z + half_extents.z,
                    ),
                );

                assert!(
                    (center - closest).length() >= body.radius() - CONTACT_TOLERANCE,
                    "pig at {center:?} should not overlap the part at {part_center:?}"
                );

                if (bottom - (part_center.y + half_extents.y)).abs() < CONTACT_TOLERANCE
                    && (center.x - part_center.x).abs() <= half_extents.x
                    && (center.z - part_center.z).abs() <= half_extents.z
                {
                    supports += 1;
                }
            }

            assert!(supports > 0, "pig at {center:?} should rest on a part");
        }
    }

    #[test]
    fn blue_moon_pigs_face_forward_parallel_to_ground() {
        const AXIS_TOLERANCE: f32 = 0.0001;
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();
        let SpaceMaterials {
            pig, snout, eye, ..
        } = register_space_materials(&mut Scene::new()).unwrap();
        let frames = blue_moon_structure_frames();
        let basis_at = |point: Vec3| frames[blue_moon_structure_index(point)].basis();
        let spheres_with = |material_id: usize| -> Vec<_> {
            scene
                .objects()
                .iter()
                .filter_map(|primitive| primitive.as_sphere())
                .filter(|sphere| sphere.material_id() == material_id)
                .collect()
        };
        let pig_bodies = spheres_with(pig);
        let snouts: Vec<_> = scene
            .cylinders()
            .filter(|cylinder| cylinder.material_id() == snout)
            .collect();
        let ears: Vec<_> = scene
            .cones()
            .filter(|cone| cone.material_id() == pig)
            .collect();

        assert_eq!(snouts.len(), BLUE_MOON_LEVEL_PIG_COUNT);
        assert_eq!(ears.len(), BLUE_MOON_LEVEL_PIG_COUNT * 2);

        for snout_part in snouts {
            let basis = basis_at(snout_part.center());
            let axis = snout_part.orientation().up();

            assert!(axis.dot(basis.up()).abs() < AXIS_TOLERANCE);
            assert!(axis.dot(basis.forward()) > 1.0 - AXIS_TOLERANCE);
        }

        for ear in ears {
            let basis = basis_at(ear.center());

            assert!(ear.orientation().up().dot(basis.up()) > 1.0 - AXIS_TOLERANCE);
        }

        for eye_ball in spheres_with(eye) {
            let body = pig_bodies
                .iter()
                .min_by(|left, right| {
                    let left_distance = (left.center() - eye_ball.center()).length();
                    let right_distance = (right.center() - eye_ball.center()).length();
                    left_distance.total_cmp(&right_distance)
                })
                .unwrap();
            let offset =
                basis_at(body.center()).world_to_local_vector(eye_ball.center() - body.center());

            assert!(offset.z > body.radius() * 0.5);
            assert!(offset.z > offset.y);
        }
    }

    #[test]
    fn blue_moon_wood_structures_sit_on_dirt_bases() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let mut dirt_parts = 0;
        let mut wood_parts = 0;
        let mut stone_parts = 0;

        assert_eq!(metadata.blue_moon_dirt_base_parts, 4);
        assert_eq!(metadata.wood_parts, 14);
        assert_eq!(metadata.blue_moon_stone_structure_parts, 7);

        for box_object in blue_moon_structure_boxes(&scene, 0)
            .into_iter()
            .chain(blue_moon_structure_boxes(&scene, 2))
        {
            let surface_clearance =
                (box_object.center() - BLUE_MOON_PLANET_CENTER).length() - BLUE_MOON_PLANET_RADIUS;
            let half_extents = box_object.half_extents();
            let material = scene.material(box_object.material_id()).unwrap();

            if material.albedo.r < 0.32 && material.albedo.g < 0.20 {
                dirt_parts += 1;
                assert!(surface_clearance < half_extents.y);
                assert!(surface_clearance + half_extents.y > 0.075);
                assert!(half_extents.x >= 0.20);
                assert!(half_extents.z >= 0.20);
            } else if material.albedo.r > material.albedo.g {
                wood_parts += 1;
                assert!(surface_clearance > half_extents.y);
                assert!(surface_clearance >= 0.160);
                assert!(half_extents.x <= 0.45);
                assert!(half_extents.z <= 0.06);
            } else {
                stone_parts += 1;
                assert!(surface_clearance > half_extents.y);
                assert!(surface_clearance >= 0.600);
                assert!(half_extents.x <= 0.30);
                assert!(half_extents.z <= 0.06);
            }
        }

        assert_eq!(dirt_parts, metadata.blue_moon_dirt_base_parts);
        assert_eq!(wood_parts, metadata.wood_parts);
        assert_eq!(stone_parts, metadata.blue_moon_stone_structure_parts);
    }

    #[test]
    fn blue_moon_side_tower_is_next_to_first_without_touching() {
        const MIN_GAP: f32 = 0.08;
        const MAX_GAP: f32 = 0.45;
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();
        let first = blue_moon_structure_boxes(&scene, 0);
        let side = blue_moon_structure_boxes(&scene, 2);

        assert!(first.len() > side.len() / 2);
        assert!(side.len() >= 10);

        let closest_gap = first
            .iter()
            .flat_map(|left| {
                side.iter()
                    .map(move |right| oriented_box_separation(left, right))
            })
            .fold(f32::MAX, f32::min);

        assert!(
            closest_gap >= MIN_GAP,
            "side tower touches the first tower: gap {closest_gap}"
        );
        assert!(
            closest_gap <= MAX_GAP,
            "side tower is too far from the first tower: gap {closest_gap}"
        );
    }

    #[test]
    fn blue_moon_second_structure_is_near_but_separate_from_first() {
        const MIN_GAP: f32 = 0.10;
        const MAX_GAP: f32 = 0.40;
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let first = blue_moon_structure_boxes(&scene, 0);
        let second = blue_moon_structure_boxes(&scene, 1);

        assert_eq!(second.len(), metadata.blue_moon_second_structure_parts);
        assert_eq!(
            first.len() + second.len() + blue_moon_structure_boxes(&scene, 2).len(),
            scene.oriented_box_count(),
            "every box belongs to one of the Luna Azul structures"
        );

        let closest_gap = first
            .iter()
            .flat_map(|left| {
                second
                    .iter()
                    .map(move |right| oriented_box_separation(left, right))
            })
            .fold(f32::MAX, f32::min);

        assert!(
            closest_gap >= MIN_GAP,
            "structures touch: gap {closest_gap}"
        );
        assert!(
            closest_gap <= MAX_GAP,
            "structures too far: gap {closest_gap}"
        );
    }

    #[test]
    fn blue_moon_second_structure_has_textured_tnt_on_its_dirt_base() {
        const CONTACT_TOLERANCE: f32 = 0.001;
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let SpaceMaterials {
            tnt_crate,
            moon_dirt,
            ..
        } = register_space_materials(&mut Scene::new()).unwrap();
        let frame = blue_moon_second_structure_frame().unwrap();
        let second = blue_moon_structure_boxes(&scene, 1);
        let crates: Vec<_> = second
            .iter()
            .filter(|part| part.material_id() == tnt_crate)
            .collect();
        let dirt = second
            .iter()
            .find(|part| part.material_id() == moon_dirt)
            .unwrap();
        let material = scene.material(tnt_crate).unwrap();

        assert_eq!(crates.len(), 1);
        assert_eq!(metadata.tnt_parts, 1);
        assert!(scene.texture(material.texture_id.unwrap()).is_some());

        let crate_center = structure_local_point(frame, crates[0].center());
        let dirt_top = structure_local_point(frame, dirt.center()).y + dirt.half_extents().y;

        assert!((crate_center.y - crates[0].half_extents().y - dirt_top).abs() < CONTACT_TOLERANCE);
    }

    #[test]
    fn tnt_crate_texture_loads_from_assets() {
        let texture = Texture::from_ppm_file(TNT_CRATE_TEXTURE_PATH).unwrap();

        assert!(texture.width() > 0);
        assert!(texture.height() > 0);
    }

    #[test]
    fn block_textures_load_from_assets() {
        for path in [WOOD_BLOCK_TEXTURE_PATH, STONE_BLOCK_TEXTURE_PATH] {
            let texture = Texture::from_ppm_file(path).unwrap();
            let first = texture.pixel(0, 0).unwrap();
            let middle = texture
                .pixel(texture.width() / 2, texture.height() / 2)
                .unwrap();

            assert_eq!(texture.width(), 32);
            assert_eq!(texture.height(), 32);
            assert_ne!(first, middle);
        }
    }

    #[test]
    fn wood_and_stone_materials_use_block_textures() {
        let mut scene = Scene::new();
        let SpaceMaterials {
            wood, moon_stone, ..
        } = register_space_materials(&mut scene).unwrap();
        let wood_material = scene.material(wood).unwrap();
        let stone_material = scene.material(moon_stone).unwrap();

        assert!(scene.texture(wood_material.texture_id.unwrap()).is_some());
        assert!(scene.texture(stone_material.texture_id.unwrap()).is_some());
        assert_eq!(wood_material.wrap_mode, WrapMode::Repeat);
        assert_eq!(stone_material.wrap_mode, WrapMode::Repeat);
        assert!(wood_material.uv_scale.u > 1.0);
        assert!(stone_material.uv_scale.u > 1.0);
    }

    #[test]
    fn radial_boxes_are_elevated_above_planet_surface() {
        let mut scene = Scene::new();
        let mut material_scene = Scene::new();
        let SpaceMaterials { wood, .. } = register_space_materials(&mut material_scene).unwrap();
        scene
            .add_material(Material::diffuse(crate::color::Color::WHITE))
            .unwrap();
        let frame = cookie_level_frame().unwrap();
        let half_extents = Vec3::new(0.1, 0.2, 0.1);
        let local_center = Vec3::new(0.0, half_extents.y + 0.05, 0.0);

        add_radial_box(&mut scene, frame, local_center, half_extents, 0).unwrap();

        let box_center = scene.objects()[0].as_oriented_box().unwrap().center();
        assert!((box_center - COOKIE_PLANET_CENTER).length() > COOKIE_PLANET_RADIUS);
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
    fn level_orbit_cameras_target_primary_planet_centers() {
        assert_eq!(
            blue_moon_orbit_camera(4.0 / 3.0).target,
            BLUE_MOON_PLANET_CENTER
        );
        assert_eq!(
            cookie_world_orbit_camera(4.0 / 3.0).target,
            COOKIE_PLANET_CENTER
        );
        assert_eq!(
            level_three_orbit_camera(4.0 / 3.0).target,
            LEVEL_THREE_PLANET_CENTER
        );
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
        assert!(scene.skybox().is_some());
    }

    #[test]
    fn level_three_scene_contains_valid_placeholder_world() {
        let (scene, metadata) = build_level_three_scene_with_metadata().unwrap();
        let planet = scene.objects()[metadata.level_three_planet_id.unwrap()]
            .as_sphere()
            .unwrap();
        let gravity = scene.objects()[metadata.level_three_gravity_field_id.unwrap()]
            .as_sphere()
            .unwrap();

        assert_eq!(planet.center(), LEVEL_THREE_PLANET_CENTER);
        assert_eq!(planet.radius(), LEVEL_THREE_PLANET_RADIUS);
        assert_eq!(gravity.center(), LEVEL_THREE_PLANET_CENTER);
        assert!(gravity.radius() > planet.radius());
        assert!(metadata.decorative_asteroid_count >= 5);
        assert!(scene.object_count() >= 7);
        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 5);
    }

    #[test]
    fn level_three_camera_is_outside_and_sees_placeholder() {
        let (scene, _) = build_level_three_scene_with_metadata().unwrap();
        let camera = level_three_orbit_camera(4.0 / 3.0).to_camera();
        let central_ray = Ray::new(camera.position, camera.target - camera.position);
        let hit = scene.intersect(&central_ray, 0.001, 100.0).unwrap();

        assert!(
            (camera.position - LEVEL_THREE_PLANET_CENTER).length()
                > LEVEL_THREE_PLANET_RADIUS * 1.30
        );
        assert!(hit.distance > 1.0);
    }

    #[test]
    fn space_levels_scene_contains_cookie_and_blue_moon_pigs() {
        let (_, metadata) = build_space_levels_scene_with_metadata().unwrap();

        assert_eq!(metadata.cookie_pig_count, COOKIE_LEVEL_PIG_COUNT);
        assert_eq!(metadata.blue_moon_pig_count, BLUE_MOON_LEVEL_PIG_COUNT);
        assert_eq!(
            metadata.pig_count,
            COOKIE_LEVEL_PIG_COUNT + BLUE_MOON_LEVEL_PIG_COUNT
        );
    }

    #[test]
    fn cookie_level_has_sweet_readable_details() {
        let (scene, metadata) = build_space_levels_scene_with_metadata().unwrap();

        assert!(metadata.cookie_chocolate_chip_count >= 8);
        assert!(metadata.candy_count >= 6);
        assert!(scene.sphere_count() >= 16);
        assert!(scene.cylinder_count() >= 8);
        assert!(scene.oriented_box_count() >= 7);
    }

    #[test]
    fn space_levels_scene_has_skybox_lights_and_scene_budget() {
        let (scene, _) = build_space_levels_scene_with_metadata().unwrap();

        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 7);
        assert!(scene.object_count() >= 40);
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
        let central_ray = Ray::new(camera.position, COOKIE_PLANET_CENTER - camera.position);
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
        let level_three_ray = Ray::new(
            camera.position,
            GALAXY_SELECTOR_LEVEL_THREE_CENTER - camera.position,
        );

        assert!(
            (camera.position - GALAXY_SELECTOR_BLUE_MOON_CENTER).length()
                > super::GALAXY_SELECTOR_BLUE_MOON_RADIUS * 1.30
        );
        assert!(
            (camera.position - GALAXY_SELECTOR_COOKIE_CENTER).length()
                > super::GALAXY_SELECTOR_COOKIE_RADIUS * 1.30
        );
        assert!(
            (camera.position - GALAXY_SELECTOR_LEVEL_THREE_CENTER).length()
                > super::GALAXY_SELECTOR_LEVEL_THREE_RADIUS * 1.30
        );
        assert!(scene.intersect(&blue_ray, 0.001, 100.0).is_some());
        assert!(scene.intersect(&cookie_ray, 0.001, 100.0).is_some());
        assert!(scene.intersect(&level_three_ray, 0.001, 100.0).is_some());
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
