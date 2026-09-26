use std::{error::Error, fmt};

use crate::{
    basis::Basis3,
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
pub const SPACE_LIGHT_PANEL_CENTER: Vec3 = Vec3::new(-2.35, 3.45, 3.15);
pub const BLUE_MOON_PIG_COUNT: usize = 3;
pub const BLUE_MOON_BIRD_COUNT: usize = 3;
pub const COOKIE_LEVEL_PIG_COUNT: usize = 2;

const BLUE_MOON_TEXTURE: &str = "\
P3
4 4
255
82 91 104   122 135 148  70 78 92    146 154 162
154 164 174  92 103 120   116 128 142  74 84 98
96 106 122   164 172 180  85 96 112    134 146 158
65 72 88     112 124 138  148 158 166  90 101 116
";

const SPACE_SKYBOX_TEXTURE: &str = "\
P3
16 8
255
5 8 24    6 9 27    7 10 30   7 11 32   8 12 35   8 12 36   7 11 34   6 10 31   6 9 28    5 8 25    6 9 27    7 10 31   8 12 35   7 11 33   6 9 29    5 8 24
6 10 30   8 12 36   8 13 39   95 118 170  9 14 42   10 15 45  9 14 43   8 13 39   7 11 35   7 11 34   8 12 36   9 14 42   90 112 164  8 13 39   7 11 33   6 10 30
7 11 34   9 14 42   10 16 48  9 15 45   10 17 50  11 18 54  135 160 215  10 16 49  9 15 45   8 13 40   8 13 38   9 15 44   10 16 48  9 14 43   8 13 38   7 11 34
6 10 31   8 13 39   9 15 45   10 17 51  11 18 55  10 17 52  9 15 47   9 14 43   8 13 40   8 12 37   98 120 174  8 13 39   9 14 43   8 13 40   7 11 35   6 10 31
5 8 25    7 11 34   8 13 39   9 15 46   10 16 50  9 15 47   8 13 42   8 12 38   7 11 35   7 10 32   7 11 34   8 12 37   9 14 42   8 12 38   6 10 32   5 8 25
4 7 22    6 10 30   7 11 35   8 13 40   8 14 43   8 13 42   7 12 38   7 11 35   115 138 190  6 10 31   6 10 31   7 11 34   8 12 37   7 11 34   6 9 28    4 7 22
3 6 19    5 8 25    6 10 30   7 11 34   7 12 36   6 11 34   6 10 32   6 9 29    5 8 27    5 8 25    6 9 27    95 116 168  7 10 31   6 9 28    4 7 23    3 6 19
2 4 14    3 6 19    4 7 22   5 8 25    5 9 27    5 8 26    4 8 24    4 7 22    4 6 20    3 6 19    4 6 20    5 8 24    5 8 25    4 7 22    3 5 17    2 4 14
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
    light_panel: usize,
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
    pub light_panel_id: Option<usize>,
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
    pub panel_light_count: usize,
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

pub fn build_space_levels_scene() -> Result<Scene, SpaceBuildError> {
    build_space_levels_scene_with_metadata().map(|built| built.0)
}

pub(crate) fn build_space_levels_scene_with_metadata()
-> Result<(Scene, SpaceSceneMetadata), SpaceBuildError> {
    let (mut scene, materials) = base_space_scene()?;
    let mut metadata = SpaceSceneMetadata::default();

    add_blue_moon_world(&mut scene, &mut metadata, materials)?;
    add_cookie_level(&mut scene, &mut metadata, materials)?;
    add_space_light_panel(&mut scene, &mut metadata, materials)?;
    add_space_lighting(&mut scene);
    add_cookie_level_lighting(&mut scene);
    add_panel_lighting(&mut scene, &mut metadata);

    Ok((scene, metadata))
}

pub fn blue_moon_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        Vec3::new(-0.75, 0.35, 0.70),
        -0.30,
        0.15,
        8.2,
        58.0,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

pub fn space_levels_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        Vec3::new(1.35, 0.20, 0.00),
        -0.14,
        0.12,
        12.2,
        62.0,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

fn base_space_scene() -> Result<(Scene, SpaceMaterials), SpaceBuildError> {
    let mut scene = Scene::new();

    scene.set_ambient_light(Color::new(0.095, 0.105, 0.145));
    scene.set_skybox(
        Skybox::new(Texture::from_ppm_text(SPACE_SKYBOX_TEXTURE)?)
            .with_intensity(0.82)
            .with_horizontal_rotation(0.12),
    );
    let materials = register_space_materials(&mut scene)?;

    Ok((scene, materials))
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
            Color::new(0.58, 0.65, 0.72),
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
        Color::new(0.18, 0.21, 0.27),
        0.12,
        12.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let gravity_field = scene.add_material(Material::new(
        Color::new(0.24, 0.50, 0.95),
        0.12,
        64.0,
        0.08,
        0.90,
        1.05,
        Color::new(0.005, 0.020, 0.045),
    ))?;
    let cookie_gravity_field = scene.add_material(Material::new(
        Color::new(1.0, 0.56, 0.14),
        0.12,
        60.0,
        0.06,
        0.91,
        1.05,
        Color::new(0.045, 0.020, 0.0),
    ))?;
    let light_panel = scene.add_material(Material::new(
        Color::new(0.78, 0.92, 1.0),
        0.62,
        80.0,
        0.08,
        0.0,
        1.0,
        Color::new(0.62, 0.80, 1.0),
    ))?;
    let asteroid = scene.add_material(Material::new(
        Color::new(0.42, 0.39, 0.36),
        0.18,
        16.0,
        0.02,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let cookie = scene.add_material(Material::new(
        Color::new(0.72, 0.45, 0.20),
        0.18,
        22.0,
        0.03,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let chocolate = scene.add_material(Material::new(
        Color::new(0.19, 0.08, 0.03),
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
        light_panel,
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
        BLUE_MOON_PLANET_RADIUS * 1.30,
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
        COOKIE_PLANET_RADIUS * 1.28,
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

fn add_space_light_panel(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let panel_id = scene.object_count();
    scene.add_oriented_box(OrientedBox::new(
        SPACE_LIGHT_PANEL_CENTER,
        Vec3::new(2.35, 0.045, 0.82),
        Basis3::identity(),
        materials.light_panel,
    )?)?;
    metadata.light_panel_id = Some(panel_id);

    for (center, half_extents) in [
        (
            SPACE_LIGHT_PANEL_CENTER + Vec3::new(0.0, 0.015, -0.88),
            Vec3::new(2.48, 0.040, 0.035),
        ),
        (
            SPACE_LIGHT_PANEL_CENTER + Vec3::new(0.0, 0.015, 0.88),
            Vec3::new(2.48, 0.040, 0.035),
        ),
        (
            SPACE_LIGHT_PANEL_CENTER + Vec3::new(-2.48, 0.015, 0.0),
            Vec3::new(0.035, 0.040, 0.88),
        ),
        (
            SPACE_LIGHT_PANEL_CENTER + Vec3::new(2.48, 0.015, 0.0),
            Vec3::new(0.035, 0.040, 0.88),
        ),
    ] {
        scene.add_oriented_box(OrientedBox::new(
            center,
            half_extents,
            Basis3::identity(),
            materials.metal,
        )?)?;
        metadata.ice_or_metal_parts += 1;
    }

    Ok(())
}

fn add_space_lighting(scene: &mut Scene) {
    scene.add_light(PointLight::new(
        Vec3::new(-4.5, 4.5, 7.0),
        Color::new(0.66, 0.82, 1.0),
        11.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(3.2, 1.4, 4.0),
        Color::new(0.28, 0.48, 1.0),
        4.4,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(0.6, 2.4, 3.4),
        Color::new(1.0, 0.62, 0.28),
        3.1,
    ));
}

fn add_cookie_level_lighting(scene: &mut Scene) {
    scene.add_light(PointLight::new(
        Vec3::new(5.8, 2.8, 5.0),
        Color::new(1.0, 0.70, 0.34),
        5.3,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(3.2, -0.4, 3.2),
        Color::new(0.80, 0.35, 1.0),
        2.4,
    ));
}

fn add_panel_lighting(scene: &mut Scene, metadata: &mut SpaceSceneMetadata) {
    for (position, color, intensity) in [
        (
            SPACE_LIGHT_PANEL_CENTER + Vec3::new(-1.30, -0.36, 0.30),
            Color::new(0.72, 0.88, 1.0),
            5.8,
        ),
        (
            SPACE_LIGHT_PANEL_CENTER + Vec3::new(1.15, -0.32, -0.20),
            Color::new(0.82, 0.94, 1.0),
            4.6,
        ),
    ] {
        scene.add_light(PointLight::new(position, color, intensity));
        metadata.panel_light_count += 1;
    }
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
        COOKIE_PLANET_RADIUS, LAUNCH_ASTEROID_CENTER, SPACE_LIGHT_PANEL_CENTER, SpaceMaterials,
        add_radial_box, blue_moon_orbit_camera, build_blue_moon_scene_with_metadata,
        build_space_levels_scene_with_metadata, main_moon_frame, register_space_materials,
        space_levels_orbit_camera,
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
        assert!(scene.oriented_box_count() >= 30);
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
    fn space_levels_scene_has_emissive_light_panel_and_panel_lights() {
        let (scene, metadata) = build_space_levels_scene_with_metadata().unwrap();
        let panel = scene.objects()[metadata.light_panel_id.unwrap()]
            .as_oriented_box()
            .unwrap();
        let material = scene.material(panel.material_id()).unwrap();

        assert_eq!(panel.center(), SPACE_LIGHT_PANEL_CENTER);
        assert!(panel.half_extents().x > panel.half_extents().z);
        assert!(panel.half_extents().y < 0.10);
        assert!(material.emission.b > 0.8);
        assert!(material.emission.r > 0.5);
        assert!(metadata.panel_light_count >= 2);
        assert!(scene.lights().len() >= metadata.panel_light_count);
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
}
