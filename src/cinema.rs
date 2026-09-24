use std::{error::Error, fmt};

use crate::{
    color::Color,
    cube::Cube,
    light::PointLight,
    material::Material,
    math::{Vec2, Vec3},
    scene::{Scene, SceneError},
    skybox::Skybox,
    texture::{Texture, TextureError, WrapMode},
};

/// Sistema de coordenadas de la sala:
/// X crece de izquierda a derecha, Y es vertical y Z marca la profundidad.
/// La pantalla vive en el extremo Z negativo; la entrada abierta queda hacia Z positivo.
pub const ROOM_WIDTH: f32 = 8.0;
pub const ROOM_HEIGHT: f32 = 4.2;
pub const ROOM_DEPTH: f32 = 11.2;
pub const FLOOR_Y: f32 = -1.2;
pub const SCREEN_Z: f32 = -5.92;

const HALF_ROOM_WIDTH: f32 = ROOM_WIDTH * 0.5;
const FRONT_Z: f32 = -6.25;
const BACK_Z: f32 = FRONT_Z + ROOM_DEPTH;
const FLOOR_THICKNESS: f32 = 0.12;
const WALL_THICKNESS: f32 = 0.20;
const SCREEN_WIDTH: f32 = 5.8;
const SCREEN_HEIGHT: f32 = 2.65;
const SCREEN_BOTTOM_Y: f32 = -0.05;
const STEP_COUNT: usize = 6;
const ACOUSTIC_PANEL_COLUMNS: usize = 5;

const SEAT_FABRIC_TEXTURE_PATH: &str = "assets/textures/seat_fabric.ppm";
const THEATER_CARPET_TEXTURE_PATH: &str = "assets/textures/theater_carpet.ppm";
const BRUSHED_METAL_TEXTURE_PATH: &str = "assets/textures/brushed_metal.ppm";
const TRANSPARENT_PLASTIC_TEXTURE_PATH: &str = "assets/textures/transparent_plastic.ppm";
const POPCORN_CARDBOARD_TEXTURE_PATH: &str = "assets/textures/popcorn_cardboard.ppm";
const NIGHT_CINEMA_SKYBOX_TEXTURE_PATH: &str = "assets/textures/night_cinema_skybox.ppm";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CinemaBuildError {
    TextureLoad {
        path: &'static str,
        source: TextureError,
    },
    SkyboxLoad {
        path: &'static str,
        source: TextureError,
    },
    Scene(SceneError),
}

impl fmt::Display for CinemaBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TextureLoad { path, source } => {
                write!(formatter, "no se pudo cargar la textura {path}: {source:?}")
            }
            Self::SkyboxLoad { path, source } => {
                write!(formatter, "no se pudo cargar el skybox {path}: {source:?}")
            }
            Self::Scene(source) => write!(formatter, "no se pudo registrar la escena: {source:?}"),
        }
    }
}

impl Error for CinemaBuildError {}

impl From<SceneError> for CinemaBuildError {
    fn from(error: SceneError) -> Self {
        Self::Scene(error)
    }
}

#[derive(Debug, Clone, Copy)]
struct CinemaMaterials {
    seat_fabric: usize,
    theater_carpet: usize,
    brushed_metal: usize,
    transparent_plastic: usize,
    popcorn_cardboard: usize,
    wall: usize,
    dark_wall: usize,
    screen: usize,
    aisle_carpet: usize,
    exit_sign: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CinemaElement {
    Floor,
    FrontWall,
    LeftWall,
    RightWall,
    Ceiling,
    Screen,
    Aisle,
    Step,
    Door,
    ExitSign,
    AcousticPanel,
    Opening,
}

#[derive(Debug, Default)]
pub(crate) struct CinemaBuildMetadata {
    elements: Vec<(CinemaElement, usize)>,
    textured_material_count: usize,
}

#[cfg(test)]
impl CinemaBuildMetadata {
    pub(crate) fn count(&self, element: CinemaElement) -> usize {
        self.elements
            .iter()
            .filter(|(registered, _)| *registered == element)
            .count()
    }

    pub(crate) fn cube_ids(&self, element: CinemaElement) -> Vec<usize> {
        self.elements
            .iter()
            .filter_map(|(registered, cube_id)| (*registered == element).then_some(*cube_id))
            .collect()
    }

    pub(crate) fn textured_material_count(&self) -> usize {
        self.textured_material_count
    }
}

pub fn build_cinema_scene() -> Result<Scene, CinemaBuildError> {
    build_cinema_scene_internal().map(|built| built.0)
}

#[cfg(test)]
pub(crate) fn build_cinema_scene_with_metadata()
-> Result<(Scene, CinemaBuildMetadata), CinemaBuildError> {
    build_cinema_scene_internal()
}

fn build_cinema_scene_internal() -> Result<(Scene, CinemaBuildMetadata), CinemaBuildError> {
    let mut scene = Scene::new();
    let mut metadata = CinemaBuildMetadata::default();

    scene.set_ambient_light(Color::new(0.025, 0.023, 0.032));
    scene.set_skybox(load_skybox(NIGHT_CINEMA_SKYBOX_TEXTURE_PATH)?);
    let materials = register_materials(&mut scene, &mut metadata)?;

    add_floor(&mut scene, &mut metadata, materials)?;
    add_front_wall_and_screen(&mut scene, &mut metadata, materials)?;
    add_side_walls_and_panels(&mut scene, &mut metadata, materials)?;
    add_partial_ceiling(&mut scene, &mut metadata, materials)?;
    add_aisle_and_steps(&mut scene, &mut metadata, materials)?;
    add_exit(&mut scene, &mut metadata, materials)?;
    add_lights(&mut scene);

    Ok((scene, metadata))
}

fn register_materials(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
) -> Result<CinemaMaterials, CinemaBuildError> {
    let seat_texture = scene.add_texture(load_texture(SEAT_FABRIC_TEXTURE_PATH)?);
    let carpet_texture = scene.add_texture(load_texture(THEATER_CARPET_TEXTURE_PATH)?);
    let metal_texture = scene.add_texture(load_texture(BRUSHED_METAL_TEXTURE_PATH)?);
    let plastic_texture = scene.add_texture(load_texture(TRANSPARENT_PLASTIC_TEXTURE_PATH)?);
    let cardboard_texture = scene.add_texture(load_texture(POPCORN_CARDBOARD_TEXTURE_PATH)?);

    metadata.textured_material_count = 5;

    let seat_fabric = scene.add_material(
        Material::new(
            Color::new(0.78, 0.34, 0.34),
            0.22,
            18.0,
            0.03,
            0.0,
            1.0,
            Color::BLACK,
        )
        .with_texture(seat_texture, Vec2::new(3.0, 2.0), WrapMode::Repeat),
    )?;
    let theater_carpet = scene.add_material(
        Material::new(
            Color::new(0.42, 0.32, 0.56),
            0.08,
            8.0,
            0.01,
            0.0,
            1.0,
            Color::BLACK,
        )
        .with_texture(carpet_texture, Vec2::new(8.0, 9.0), WrapMode::Repeat),
    )?;
    let brushed_metal = scene.add_material(
        Material::new(
            Color::new(0.76, 0.76, 0.78),
            0.9,
            96.0,
            0.78,
            0.0,
            1.0,
            Color::BLACK,
        )
        .with_texture(metal_texture, Vec2::new(2.0, 1.0), WrapMode::Clamp),
    )?;
    let transparent_plastic = scene.add_material(
        Material::new(
            Color::new(0.72, 0.88, 1.0),
            0.55,
            64.0,
            0.35,
            0.72,
            1.49,
            Color::BLACK,
        )
        .with_texture(plastic_texture, Vec2::new(1.5, 1.5), WrapMode::Clamp),
    )?;
    let popcorn_cardboard = scene.add_material(
        Material::new(
            Color::new(1.0, 0.92, 0.72),
            0.18,
            14.0,
            0.02,
            0.0,
            1.0,
            Color::new(0.02, 0.015, 0.005),
        )
        .with_texture(cardboard_texture, Vec2::new(2.0, 2.0), WrapMode::Repeat),
    )?;
    let wall = scene.add_material(Material::new(
        Color::new(0.30, 0.27, 0.34),
        0.18,
        20.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let dark_wall = scene.add_material(Material::new(
        Color::new(0.12, 0.10, 0.16),
        0.12,
        12.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let screen = scene.add_material(Material::new(
        Color::new(0.86, 0.91, 1.0),
        0.25,
        28.0,
        0.02,
        0.0,
        1.0,
        Color::new(0.20, 0.26, 0.36),
    ))?;
    let aisle_carpet = scene.add_material(Material::new(
        Color::new(0.28, 0.20, 0.30),
        0.08,
        8.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let exit_sign = scene.add_material(Material::new(
        Color::new(0.12, 0.95, 0.46),
        0.12,
        20.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.0, 0.55, 0.18),
    ))?;

    Ok(CinemaMaterials {
        seat_fabric,
        theater_carpet,
        brushed_metal,
        transparent_plastic,
        popcorn_cardboard,
        wall,
        dark_wall,
        screen,
        aisle_carpet,
        exit_sign,
    })
}

fn add_floor(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
) -> Result<(), CinemaBuildError> {
    add_cube(
        scene,
        metadata,
        CinemaElement::Floor,
        Vec3::new(-HALF_ROOM_WIDTH, FLOOR_Y - FLOOR_THICKNESS, FRONT_Z),
        Vec3::new(HALF_ROOM_WIDTH, FLOOR_Y, BACK_Z),
        materials.theater_carpet,
    )
}

fn add_front_wall_and_screen(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
) -> Result<(), CinemaBuildError> {
    let wall_front = FRONT_Z - WALL_THICKNESS * 0.5;
    let wall_back = FRONT_Z + WALL_THICKNESS * 0.5;
    let screen_left = -SCREEN_WIDTH * 0.5;
    let screen_right = SCREEN_WIDTH * 0.5;
    let screen_top = SCREEN_BOTTOM_Y + SCREEN_HEIGHT;

    add_cube(
        scene,
        metadata,
        CinemaElement::FrontWall,
        Vec3::new(-HALF_ROOM_WIDTH, FLOOR_Y, wall_front),
        Vec3::new(screen_left - 0.25, ROOM_HEIGHT - 0.25, wall_back),
        materials.wall,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::FrontWall,
        Vec3::new(screen_right + 0.25, FLOOR_Y, wall_front),
        Vec3::new(HALF_ROOM_WIDTH, ROOM_HEIGHT - 0.25, wall_back),
        materials.wall,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::FrontWall,
        Vec3::new(screen_left - 0.25, FLOOR_Y, wall_front),
        Vec3::new(screen_right + 0.25, SCREEN_BOTTOM_Y - 0.22, wall_back),
        materials.dark_wall,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::FrontWall,
        Vec3::new(screen_left - 0.25, screen_top + 0.22, wall_front),
        Vec3::new(screen_right + 0.25, ROOM_HEIGHT - 0.25, wall_back),
        materials.dark_wall,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::Screen,
        Vec3::new(screen_left, SCREEN_BOTTOM_Y, SCREEN_Z - 0.03),
        Vec3::new(screen_right, screen_top, SCREEN_Z + 0.03),
        materials.screen,
    )
}

fn add_side_walls_and_panels(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
) -> Result<(), CinemaBuildError> {
    let left_outer = -HALF_ROOM_WIDTH - WALL_THICKNESS * 0.5;
    let left_inner = -HALF_ROOM_WIDTH + WALL_THICKNESS * 0.5;
    let right_inner = HALF_ROOM_WIDTH - WALL_THICKNESS * 0.5;
    let right_outer = HALF_ROOM_WIDTH + WALL_THICKNESS * 0.5;
    let wall_back = BACK_Z - 1.55;

    add_cube(
        scene,
        metadata,
        CinemaElement::LeftWall,
        Vec3::new(left_outer, FLOOR_Y, FRONT_Z),
        Vec3::new(left_inner, ROOM_HEIGHT - 0.8, wall_back),
        materials.wall,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::RightWall,
        Vec3::new(right_inner, FLOOR_Y, FRONT_Z),
        Vec3::new(right_outer, ROOM_HEIGHT - 0.8, wall_back),
        materials.wall,
    )?;

    for column in 0..ACOUSTIC_PANEL_COLUMNS {
        let z = -5.05 + column as f32 * 1.55;
        add_cube(
            scene,
            metadata,
            CinemaElement::AcousticPanel,
            Vec3::new(left_inner + 0.03, -0.18, z),
            Vec3::new(left_inner + 0.24, 2.25, z + 0.72),
            materials.seat_fabric,
        )?;
        add_cube(
            scene,
            metadata,
            CinemaElement::AcousticPanel,
            Vec3::new(right_inner - 0.24, -0.18, z),
            Vec3::new(right_inner - 0.03, 2.25, z + 0.72),
            materials.seat_fabric,
        )?;
    }

    Ok(())
}

fn add_partial_ceiling(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
) -> Result<(), CinemaBuildError> {
    let ceiling_bottom = ROOM_HEIGHT - 0.95;
    let ceiling_top = ROOM_HEIGHT - 0.72;

    add_cube(
        scene,
        metadata,
        CinemaElement::Ceiling,
        Vec3::new(-HALF_ROOM_WIDTH, ceiling_bottom, FRONT_Z),
        Vec3::new(HALF_ROOM_WIDTH, ceiling_top, -2.85),
        materials.dark_wall,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::Ceiling,
        Vec3::new(-HALF_ROOM_WIDTH, ceiling_bottom, -2.45),
        Vec3::new(-2.35, ceiling_top, 2.35),
        materials.dark_wall,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::Ceiling,
        Vec3::new(2.35, ceiling_bottom, -2.45),
        Vec3::new(HALF_ROOM_WIDTH, ceiling_top, 2.35),
        materials.dark_wall,
    )?;
    register_opening(metadata);

    Ok(())
}

fn add_aisle_and_steps(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
) -> Result<(), CinemaBuildError> {
    add_cube(
        scene,
        metadata,
        CinemaElement::Aisle,
        Vec3::new(-0.55, FLOOR_Y + 0.015, -5.25),
        Vec3::new(0.55, FLOOR_Y + 0.105, BACK_Z - 0.35),
        materials.aisle_carpet,
    )?;

    for step in 0..STEP_COUNT {
        let z_start = -3.95 + step as f32 * 1.12;
        let z_end = z_start + 0.92;
        let top_y = FLOOR_Y + 0.10 + step as f32 * 0.18;

        add_cube(
            scene,
            metadata,
            CinemaElement::Step,
            Vec3::new(-3.75, FLOOR_Y + 0.02, z_start),
            Vec3::new(-0.78, top_y, z_end),
            materials.theater_carpet,
        )?;
        add_cube(
            scene,
            metadata,
            CinemaElement::Step,
            Vec3::new(0.78, FLOOR_Y + 0.02, z_start),
            Vec3::new(3.75, top_y, z_end),
            materials.theater_carpet,
        )?;
    }

    Ok(())
}

fn add_exit(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
) -> Result<(), CinemaBuildError> {
    let frame_z_front = BACK_Z - 0.52;
    let frame_z_back = BACK_Z - 0.34;
    let door_left = 2.18;
    let door_right = 3.58;
    let door_top = 1.35;

    add_cube(
        scene,
        metadata,
        CinemaElement::Door,
        Vec3::new(door_left - 0.16, FLOOR_Y, frame_z_front),
        Vec3::new(door_left, door_top, frame_z_back),
        materials.brushed_metal,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::Door,
        Vec3::new(door_right, FLOOR_Y, frame_z_front),
        Vec3::new(door_right + 0.16, door_top, frame_z_back),
        materials.brushed_metal,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::Door,
        Vec3::new(door_left - 0.16, door_top, frame_z_front),
        Vec3::new(door_right + 0.16, door_top + 0.18, frame_z_back),
        materials.brushed_metal,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::Door,
        Vec3::new(door_left + 0.12, FLOOR_Y + 0.04, frame_z_back + 0.03),
        Vec3::new(door_right - 0.12, door_top - 0.08, frame_z_back + 0.12),
        materials.transparent_plastic,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::ExitSign,
        Vec3::new(door_left - 0.05, door_top + 0.34, frame_z_front - 0.02),
        Vec3::new(door_right + 0.05, door_top + 0.70, frame_z_back + 0.02),
        materials.exit_sign,
    )?;
    add_cube(
        scene,
        metadata,
        CinemaElement::Door,
        Vec3::new(door_left - 0.25, FLOOR_Y + 0.03, frame_z_front - 0.03),
        Vec3::new(door_right + 0.25, FLOOR_Y + 0.08, frame_z_back + 0.14),
        materials.popcorn_cardboard,
    )
}

fn add_lights(scene: &mut Scene) {
    scene.add_light(PointLight::new(
        Vec3::new(0.0, 1.45, -4.85),
        Color::new(0.42, 0.58, 1.0),
        9.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(-0.55, 0.55, -0.35),
        Color::new(1.0, 0.62, 0.34),
        2.2,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(0.55, 0.92, 2.75),
        Color::new(1.0, 0.58, 0.32),
        2.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(2.95, 1.80, 4.15),
        Color::new(0.35, 1.0, 0.55),
        3.5,
    ));
}

fn add_cube(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    element: CinemaElement,
    first_corner: Vec3,
    second_corner: Vec3,
    material_id: usize,
) -> Result<(), CinemaBuildError> {
    let cube_id = scene.cubes().len();

    scene.add_cube(Cube::new(first_corner, second_corner, material_id))?;
    metadata.elements.push((element, cube_id));

    Ok(())
}

fn register_opening(metadata: &mut CinemaBuildMetadata) {
    metadata.elements.push((CinemaElement::Opening, usize::MAX));
}

fn load_texture(path: &'static str) -> Result<Texture, CinemaBuildError> {
    Texture::from_ppm_file(path).map_err(|source| CinemaBuildError::TextureLoad { path, source })
}

fn load_skybox(path: &'static str) -> Result<Skybox, CinemaBuildError> {
    Skybox::from_ppm_file(path)
        .map(|skybox| skybox.with_intensity(1.15).with_horizontal_rotation(0.08))
        .map_err(|source| CinemaBuildError::SkyboxLoad { path, source })
}

#[cfg(test)]
mod tests {
    use super::{
        ACOUSTIC_PANEL_COLUMNS, CinemaElement, ROOM_DEPTH, ROOM_HEIGHT, ROOM_WIDTH, STEP_COUNT,
        build_cinema_scene_with_metadata,
    };
    use crate::{
        camera::Camera,
        color::Color,
        framebuffer::Framebuffer,
        math::Vec3,
        ray::Ray,
        renderer::{render_scene, trace_primary_ray},
    };

    const INITIAL_CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.70, -3.40);
    const INITIAL_CAMERA_POSITION: Vec3 = Vec3::new(0.0, 3.30, 6.78);

    fn built_scene() -> (crate::scene::Scene, super::CinemaBuildMetadata) {
        build_cinema_scene_with_metadata().expect("cinema scene should build from bundled assets")
    }

    fn initial_camera(aspect_ratio: f32) -> Camera {
        Camera::new(
            INITIAL_CAMERA_POSITION,
            INITIAL_CAMERA_TARGET,
            Vec3::new(0.0, 1.0, 0.0),
            55.0,
            aspect_ratio,
        )
    }

    fn point_inside_cube(point: Vec3, cube: &crate::cube::Cube) -> bool {
        point.x > cube.min.x
            && point.x < cube.max.x
            && point.y > cube.min.y
            && point.y < cube.max.y
            && point.z > cube.min.z
            && point.z < cube.max.z
    }

    #[test]
    fn build_cinema_scene_returns_valid_scene_with_geometry() {
        let (scene, metadata) = built_scene();

        assert!(!scene.cubes().is_empty());
        assert_eq!(metadata.count(CinemaElement::Floor), 1);
        assert_eq!(metadata.count(CinemaElement::Screen), 1);
        assert_eq!(metadata.count(CinemaElement::LeftWall), 1);
        assert_eq!(metadata.count(CinemaElement::RightWall), 1);
        assert!(metadata.count(CinemaElement::Opening) >= 1);
        assert_eq!(metadata.count(CinemaElement::Step), STEP_COUNT * 2);
        assert_eq!(
            metadata.count(CinemaElement::AcousticPanel),
            ACOUSTIC_PANEL_COLUMNS * 2
        );
        assert!(metadata.count(CinemaElement::Door) >= 1);
    }

    #[test]
    fn cinema_keeps_textured_materials_and_loaded_textures() {
        let (scene, metadata) = built_scene();
        let textured = scene
            .materials()
            .iter()
            .filter(|material| material.texture_id.is_some())
            .count();

        assert!(scene.materials().len() >= metadata.textured_material_count());
        assert_eq!(scene.textures().len(), 5);
        assert!(textured >= 5);
        assert!(scene.skybox().is_some());
    }

    #[test]
    fn cinema_has_emission_lights_and_valid_material_references() {
        let (scene, _) = built_scene();

        assert!(
            scene
                .materials()
                .iter()
                .any(|material| material.emission != Color::BLACK)
        );
        assert!(!scene.lights().is_empty());

        for cube in scene.cubes() {
            assert!(scene.material(cube.material_id).is_some());
        }

        for material in scene.materials() {
            if let Some(texture_id) = material.texture_id {
                assert!(scene.texture(texture_id).is_some());
            }
        }
    }

    #[test]
    fn cinema_cubes_have_finite_non_degenerate_dimensions() {
        let (scene, _) = built_scene();

        for cube in scene.cubes() {
            assert!(cube.min.x.is_finite());
            assert!(cube.min.y.is_finite());
            assert!(cube.min.z.is_finite());
            assert!(cube.max.x.is_finite());
            assert!(cube.max.y.is_finite());
            assert!(cube.max.z.is_finite());
            assert!(cube.max.x > cube.min.x);
            assert!(cube.max.y > cube.min.y);
            assert!(cube.max.z > cube.min.z);
        }
    }

    #[test]
    fn camera_rays_can_reach_screen_floor_and_skybox() {
        let (scene, metadata) = built_scene();
        let camera = initial_camera(4.0 / 3.0);
        let screen_ids = metadata.cube_ids(CinemaElement::Screen);
        let floor_ids = metadata.cube_ids(CinemaElement::Floor);

        let screen_ray = Ray::new(camera.position, camera.target - camera.position);
        let screen_hit = scene.intersect(&screen_ray, 0.001, 100.0).unwrap();
        assert_eq!(
            screen_hit.material_id,
            scene.cubes()[screen_ids[0]].material_id
        );

        let floor_ray = Ray::new(
            camera.position,
            Vec3::new(-1.35, -1.21, 4.35) - camera.position,
        );
        let floor_hit = scene.intersect(&floor_ray, 0.001, 100.0).unwrap();
        assert_eq!(
            floor_hit.material_id,
            scene.cubes()[floor_ids[0]].material_id
        );

        let opening_ray = Ray::new(Vec3::new(0.0, 0.2, 4.65), Vec3::new(0.0, 0.2, 1.0));
        assert!(scene.intersect(&opening_ray, 0.001, 100.0).is_none());
        assert!(scene.skybox().is_some());
    }

    #[test]
    fn small_framebuffer_contains_geometry_and_background() {
        let (scene, _) = built_scene();
        let mut framebuffer = Framebuffer::new(40, 30);
        let camera = initial_camera(framebuffer.width() as f32 / framebuffer.height() as f32);

        render_scene(&mut framebuffer, &camera, &scene);

        let mut contains_screen_like_geometry = false;
        let mut contains_background = false;
        for y in 0..framebuffer.height() {
            for x in 0..framebuffer.width() {
                let pixel = framebuffer.pixels()[y * framebuffer.width() + x];
                let ray = camera.ray_for_pixel(x, y, framebuffer.width(), framebuffer.height());
                if scene.intersect(&ray, 0.001, 100.0).is_some() {
                    contains_screen_like_geometry = true;
                } else {
                    contains_background = true;
                }
                assert!(pixel <= 0x00ff_ffff);
            }
        }

        assert!(contains_screen_like_geometry);
        assert!(contains_background);
        assert_eq!(framebuffer.width(), 40);
        assert_eq!(framebuffer.height(), 30);
        assert_eq!(framebuffer.pixels().len(), 40 * 30);
    }

    #[test]
    fn initial_camera_is_not_inside_geometry() {
        let (scene, _) = built_scene();

        assert!(
            scene
                .cubes()
                .iter()
                .all(|cube| !point_inside_cube(INITIAL_CAMERA_POSITION, cube))
        );
    }

    #[test]
    fn room_constants_describe_diorama_scale() {
        assert_eq!(ROOM_WIDTH, 8.0);
        assert_eq!(ROOM_HEIGHT, 4.2);
        assert_eq!(ROOM_DEPTH, 11.2);
    }

    #[test]
    fn trace_from_camera_center_hits_visible_screen_material() {
        let (scene, metadata) = built_scene();
        let camera = initial_camera(4.0 / 3.0);
        let ray = Ray::new(camera.position, camera.target - camera.position);
        let color = trace_primary_ray(&ray, &scene);
        let screen_cube_id = metadata.cube_ids(CinemaElement::Screen)[0];
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.material_id, scene.cubes()[screen_cube_id].material_id);
        assert_ne!(color.to_u32(), Color::BLACK.to_u32());
    }
}
