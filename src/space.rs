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
pub const BLUE_MOON_PLANET_CENTER: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const BLUE_MOON_PLANET_RADIUS: f32 = 2.2;
pub const LAUNCH_ASTEROID_CENTER: Vec3 = Vec3::new(-3.8, -0.25, 1.25);
pub const LAUNCH_ASTEROID_RADIUS: f32 = 0.62;
pub const BLUE_MOON_PIG_COUNT: usize = 3;
pub const BLUE_MOON_BIRD_COUNT: usize = 3;

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
8 4
255
3 5 18    4 7 24    8 12 36   5 8 24    12 16 42   4 8 28    2 4 18    8 10 26
5 8 28    10 16 45  3 6 20    180 205 255  8 14 40   5 8 26    35 70 140  3 5 18
2 4 16    4 7 22    18 28 66   6 10 30   4 7 24    120 160 255  5 8 24   2 4 18
1 2 10    2 4 16    4 7 22    3 5 18    5 8 24    2 4 14    4 6 20    1 2 10
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
    asteroid: usize,
    wood: usize,
    ice: usize,
    metal: usize,
    tnt: usize,
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
    pub crater_count: usize,
    pub pig_count: usize,
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
    let mut scene = Scene::new();
    let mut metadata = SpaceSceneMetadata::default();

    scene.set_ambient_light(Color::new(0.025, 0.03, 0.055));
    scene.set_skybox(
        Skybox::new(Texture::from_ppm_text(SPACE_SKYBOX_TEXTURE)?)
            .with_intensity(1.25)
            .with_horizontal_rotation(0.12),
    );
    let materials = register_space_materials(&mut scene)?;

    add_planet(&mut scene, &mut metadata, materials)?;
    add_enemy_construction(&mut scene, &mut metadata, materials)?;
    add_pigs(&mut scene, &mut metadata, materials)?;
    add_launcher(&mut scene, &mut metadata, materials)?;
    add_gravity_field(&mut scene, &mut metadata, materials)?;
    add_space_lighting(&mut scene);

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
        Color::new(0.30, 0.58, 1.0),
        0.12,
        64.0,
        0.08,
        0.82,
        1.05,
        Color::new(0.01, 0.035, 0.08),
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
        asteroid,
        wood,
        ice,
        metal,
        tnt,
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

fn add_space_lighting(scene: &mut Scene) {
    scene.add_light(PointLight::new(
        Vec3::new(-4.5, 4.5, 7.0),
        Color::new(0.62, 0.78, 1.0),
        7.5,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(3.2, 1.4, 4.0),
        Color::new(0.20, 0.40, 1.0),
        3.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(0.6, 2.4, 3.4),
        Color::new(1.0, 0.62, 0.28),
        2.2,
    ));
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
    }

    add_radial_box(
        scene,
        frame,
        Vec3::new(0.0, 0.80, 0.0),
        Vec3::new(0.24, 0.03, 0.04),
        materials.slingshot,
    )?;
    metadata.slingshot_parts += 1;

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

#[cfg(test)]
mod tests {
    use super::{
        BLUE_MOON_BIRD_COUNT, BLUE_MOON_PIG_COUNT, BLUE_MOON_PLANET_CENTER,
        BLUE_MOON_PLANET_RADIUS, LAUNCH_ASTEROID_CENTER, SpaceMaterials, add_radial_box,
        blue_moon_orbit_camera, build_blue_moon_scene_with_metadata, main_moon_frame,
        register_space_materials,
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
}
