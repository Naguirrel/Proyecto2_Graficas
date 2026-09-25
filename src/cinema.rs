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
const AISLE_HALF_WIDTH: f32 = 0.55;
const STEP_Z_START: f32 = -3.95;
const STEP_DEPTH: f32 = 0.92;
const STEP_TOP_OFFSET: f32 = 0.10;
const STEP_HEIGHT_RISE: f32 = 0.18;
const ACOUSTIC_PANEL_COLUMNS: usize = 5;
const SEAT_ROW_COUNT: usize = 4;
const SEAT_BLOCK_COUNT: usize = 2;
const SEATS_PER_BLOCK: usize = 3;
#[cfg(test)]
const SEATS_PER_ROW: usize = SEATS_PER_BLOCK * SEAT_BLOCK_COUNT;
#[cfg(test)]
const TOTAL_SEATS: usize = SEAT_ROW_COUNT * SEATS_PER_ROW;
const SEAT_ROW_SPACING: f32 = 1.12;
const SEAT_HORIZONTAL_SPACING: f32 = 0.78;
const SEAT_BLOCK_FIRST_CENTER_X: f32 = 1.52;
const SEAT_WIDTH: f32 = 0.56;
const SEAT_CUSHION_DEPTH: f32 = 0.48;
const SEAT_CUSHION_THICKNESS: f32 = 0.16;
const SEAT_BACK_THICKNESS: f32 = 0.14;
const SEAT_BACK_HEIGHT: f32 = 0.76;
const SEAT_ARM_WIDTH: f32 = 0.08;
const SEAT_ARM_HEIGHT: f32 = 0.32;
const SEAT_ARM_DEPTH: f32 = 0.56;
const SEAT_PLATFORM_CLEARANCE: f32 = 0.03;
const SEAT_FRONT_OFFSET: f32 = 0.18;
const SEAT_BACK_GAP: f32 = 0.02;
const PROJECTOR_BODY_MIN: Vec3 = Vec3::new(-3.08, 2.48, 2.30);
const PROJECTOR_BODY_MAX: Vec3 = Vec3::new(-2.22, 2.92, 3.05);
const PROJECTOR_LENS_MIN: Vec3 = Vec3::new(-2.78, 2.58, 2.16);
const PROJECTOR_LENS_MAX: Vec3 = Vec3::new(-2.52, 2.80, 2.26);
#[cfg(test)]
const PROJECTOR_LENS_APERTURE: Vec3 = Vec3::new(-2.65, 2.69, 2.15);
const PROJECTOR_PIECE_COUNT: usize = 9;
#[cfg(test)]
const PROJECTOR_MIN_PIECES: usize = 5;
#[cfg(test)]
const PROJECTOR_MAX_PIECES: usize = 12;
const AFTER_SHOW_DETAIL_CUBE_COUNT: usize = 28;
const POPCORN_KERNEL_COUNT: usize = 8;
#[cfg(test)]
const MIN_POPCORN_KERNELS: usize = 6;
#[cfg(test)]
const MAX_POPCORN_KERNELS: usize = 12;
const TICKET_COUNT: usize = 2;
const WRAPPER_COUNT: usize = 4;
#[cfg(test)]
const AISLE_CLEAR_HALF_WIDTH: f32 = 0.18;
#[cfg(test)]
const MAX_REASONABLE_CINEMA_CUBES: usize = 200;
#[cfg(test)]
const EXPECTED_CINEMA_CUBES: usize = 173;

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
    projector_indicator: usize,
    popcorn_kernel: usize,
    soda_spill: usize,
    paper_trash: usize,
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
    SeatCushion,
    SeatBack,
    SeatArm,
    ProjectorBody,
    ProjectorLens,
    ProjectorSupport,
    ProjectorDetail,
    ProjectorIndicator,
    PopcornBox,
    PopcornKernel,
    FallenCup,
    SodaSpill,
    DiscardedCan,
    Ticket,
    Wrapper,
    Opening,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectorPart {
    Body,
    Lens,
    Support,
    Detail,
    Indicator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AfterShowDetail {
    PopcornBox,
    PopcornKernel,
    FallenCup,
    SodaSpill,
    DiscardedCan,
    Ticket,
    Wrapper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeatBlock {
    Left,
    Right,
}

impl SeatBlock {
    const ALL: [Self; SEAT_BLOCK_COUNT] = [Self::Left, Self::Right];
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
struct SeatRowMetadata {
    row: usize,
    platform_y: f32,
    z_start: f32,
    seat_count: usize,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
struct SeatMetadata {
    row: usize,
    column: usize,
    block: SeatBlock,
    cushion_id: usize,
    back_id: usize,
    left_arm_id: usize,
    right_arm_id: usize,
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) struct ProjectorMetadata {
    pieces: Vec<(ProjectorPart, usize)>,
    lens_position: Vec3,
    screen_center: Vec3,
    body_material_id: usize,
    lens_material_id: usize,
    support_material_id: usize,
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) struct AfterShowMetadata {
    pieces: Vec<(AfterShowDetail, usize)>,
    popcorn_box_material_id: usize,
    popcorn_kernel_material_id: usize,
    cup_material_id: usize,
    spill_material_id: usize,
    can_material_id: usize,
    paper_material_id: usize,
}

#[derive(Debug, Clone, Copy)]
struct SeatPlacement {
    #[cfg(test)]
    row: usize,
    #[cfg(test)]
    column: usize,
    #[cfg(test)]
    block: SeatBlock,
    center_x: f32,
    platform_y: f32,
    z_start: f32,
}

#[derive(Debug, Default)]
pub(crate) struct CinemaBuildMetadata {
    elements: Vec<(CinemaElement, usize)>,
    #[cfg(test)]
    seat_rows: Vec<SeatRowMetadata>,
    #[cfg(test)]
    seats: Vec<SeatMetadata>,
    textured_material_count: usize,
    #[cfg(test)]
    seat_fabric_material_id: Option<usize>,
    #[cfg(test)]
    brushed_metal_material_id: Option<usize>,
    #[cfg(test)]
    dark_wall_material_id: Option<usize>,
    #[cfg(test)]
    transparent_plastic_material_id: Option<usize>,
    #[cfg(test)]
    projector: Option<ProjectorMetadata>,
    #[cfg(test)]
    after_show: Option<AfterShowMetadata>,
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

    fn seat_rows(&self) -> &[SeatRowMetadata] {
        &self.seat_rows
    }

    fn seats(&self) -> &[SeatMetadata] {
        &self.seats
    }

    fn seat_fabric_material_id(&self) -> Option<usize> {
        self.seat_fabric_material_id
    }

    fn brushed_metal_material_id(&self) -> Option<usize> {
        self.brushed_metal_material_id
    }

    fn dark_wall_material_id(&self) -> Option<usize> {
        self.dark_wall_material_id
    }

    fn transparent_plastic_material_id(&self) -> Option<usize> {
        self.transparent_plastic_material_id
    }

    fn projector(&self) -> Option<&ProjectorMetadata> {
        self.projector.as_ref()
    }

    fn after_show(&self) -> Option<&AfterShowMetadata> {
        self.after_show.as_ref()
    }
}

#[cfg(test)]
impl ProjectorMetadata {
    fn pieces(&self) -> &[(ProjectorPart, usize)] {
        &self.pieces
    }

    fn count(&self, part: ProjectorPart) -> usize {
        self.pieces
            .iter()
            .filter(|(registered, _)| *registered == part)
            .count()
    }

    fn cube_ids(&self) -> Vec<usize> {
        self.pieces.iter().map(|(_, cube_id)| *cube_id).collect()
    }

    fn cube_ids_for(&self, part: ProjectorPart) -> Vec<usize> {
        self.pieces
            .iter()
            .filter_map(|(registered, cube_id)| (*registered == part).then_some(*cube_id))
            .collect()
    }
}

#[cfg(test)]
impl AfterShowMetadata {
    fn pieces(&self) -> &[(AfterShowDetail, usize)] {
        &self.pieces
    }

    fn count(&self, detail: AfterShowDetail) -> usize {
        self.pieces
            .iter()
            .filter(|(registered, _)| *registered == detail)
            .count()
    }

    fn cube_ids(&self) -> Vec<usize> {
        self.pieces.iter().map(|(_, cube_id)| *cube_id).collect()
    }

    fn cube_ids_for(&self, detail: AfterShowDetail) -> Vec<usize> {
        self.pieces
            .iter()
            .filter_map(|(registered, cube_id)| (*registered == detail).then_some(*cube_id))
            .collect()
    }
}

#[cfg(test)]
impl SeatMetadata {
    fn cube_ids(self) -> [usize; 4] {
        [
            self.cushion_id,
            self.back_id,
            self.left_arm_id,
            self.right_arm_id,
        ]
    }
}

#[cfg(test)]
impl SeatRowMetadata {
    fn rear_z(self) -> f32 {
        self.z_start + STEP_DEPTH
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
    add_seating(&mut scene, &mut metadata, materials)?;
    add_exit(&mut scene, &mut metadata, materials)?;
    add_projector(&mut scene, &mut metadata, materials)?;
    add_after_show_details(&mut scene, &mut metadata, materials)?;
    configure_cinema_lighting(&mut scene);

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
    let projector_indicator = scene.add_material(Material::new(
        Color::new(0.28, 0.70, 1.0),
        0.45,
        36.0,
        0.08,
        0.0,
        1.0,
        Color::new(0.04, 0.16, 0.32),
    ))?;
    let popcorn_kernel = scene.add_material(Material::new(
        Color::new(1.0, 0.88, 0.56),
        0.10,
        10.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let soda_spill = scene.add_material(Material::new(
        Color::new(0.18, 0.045, 0.018),
        0.48,
        44.0,
        0.22,
        0.10,
        1.33,
        Color::BLACK,
    ))?;
    let paper_trash = scene.add_material(Material::new(
        Color::new(0.94, 0.80, 0.50),
        0.14,
        12.0,
        0.01,
        0.0,
        1.0,
        Color::BLACK,
    ))?;

    #[cfg(test)]
    {
        metadata.seat_fabric_material_id = Some(seat_fabric);
        metadata.brushed_metal_material_id = Some(brushed_metal);
        metadata.dark_wall_material_id = Some(dark_wall);
        metadata.transparent_plastic_material_id = Some(transparent_plastic);
    }

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
        projector_indicator,
        popcorn_kernel,
        soda_spill,
        paper_trash,
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
        Vec3::new(-AISLE_HALF_WIDTH, FLOOR_Y + 0.015, -5.25),
        Vec3::new(AISLE_HALF_WIDTH, FLOOR_Y + 0.105, BACK_Z - 0.35),
        materials.aisle_carpet,
    )?;

    for step in 0..STEP_COUNT {
        let z_start = step_z_start(step);
        let z_end = z_start + STEP_DEPTH;
        let top_y = step_top_y(step);

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

fn add_seating(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
) -> Result<(), CinemaBuildError> {
    for row in 0..SEAT_ROW_COUNT {
        add_seat_row(scene, metadata, materials, row)?;
    }

    Ok(())
}

fn add_seat_row(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
    row: usize,
) -> Result<(), CinemaBuildError> {
    let platform_y = step_top_y(row);
    let z_start = step_z_start(row);

    for block in SeatBlock::ALL {
        for column in 0..SEATS_PER_BLOCK {
            add_seat(
                scene,
                metadata,
                materials,
                SeatPlacement {
                    #[cfg(test)]
                    row,
                    #[cfg(test)]
                    column,
                    #[cfg(test)]
                    block,
                    center_x: seat_center_x(block, column),
                    platform_y,
                    z_start,
                },
            )?;
        }
    }

    #[cfg(test)]
    {
        metadata.seat_rows.push(SeatRowMetadata {
            row,
            platform_y,
            z_start,
            seat_count: SEATS_PER_ROW,
        });
    }

    Ok(())
}

fn add_seat(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
    placement: SeatPlacement,
) -> Result<(), CinemaBuildError> {
    let half_width = SEAT_WIDTH * 0.5;
    let arm_outer_half_width = half_width + SEAT_ARM_WIDTH;
    let cushion_min_y = placement.platform_y + SEAT_PLATFORM_CLEARANCE;
    let cushion_max_y = cushion_min_y + SEAT_CUSHION_THICKNESS;
    let cushion_min_z = placement.z_start + SEAT_FRONT_OFFSET;
    let cushion_max_z = cushion_min_z + SEAT_CUSHION_DEPTH;
    let back_min_z = cushion_max_z + SEAT_BACK_GAP;
    let back_max_z = back_min_z + SEAT_BACK_THICKNESS;
    let back_min_y = cushion_min_y + SEAT_CUSHION_THICKNESS * 0.45;
    let back_max_y = placement.platform_y + SEAT_BACK_HEIGHT;
    let arm_min_y = cushion_min_y;
    let arm_max_y = placement.platform_y + SEAT_ARM_HEIGHT;
    let arm_min_z = cushion_min_z - SEAT_BACK_GAP;
    let arm_max_z = arm_min_z + SEAT_ARM_DEPTH;

    let cushion_id = add_cube_id(
        scene,
        metadata,
        CinemaElement::SeatCushion,
        Vec3::new(
            placement.center_x - half_width,
            cushion_min_y,
            cushion_min_z,
        ),
        Vec3::new(
            placement.center_x + half_width,
            cushion_max_y,
            cushion_max_z,
        ),
        materials.seat_fabric,
    )?;
    let back_id = add_cube_id(
        scene,
        metadata,
        CinemaElement::SeatBack,
        Vec3::new(placement.center_x - half_width, back_min_y, back_min_z),
        Vec3::new(placement.center_x + half_width, back_max_y, back_max_z),
        materials.seat_fabric,
    )?;
    let left_arm_id = add_cube_id(
        scene,
        metadata,
        CinemaElement::SeatArm,
        Vec3::new(
            placement.center_x - arm_outer_half_width,
            arm_min_y,
            arm_min_z,
        ),
        Vec3::new(placement.center_x - half_width, arm_max_y, arm_max_z),
        materials.brushed_metal,
    )?;
    let right_arm_id = add_cube_id(
        scene,
        metadata,
        CinemaElement::SeatArm,
        Vec3::new(placement.center_x + half_width, arm_min_y, arm_min_z),
        Vec3::new(
            placement.center_x + arm_outer_half_width,
            arm_max_y,
            arm_max_z,
        ),
        materials.brushed_metal,
    )?;

    #[cfg(test)]
    {
        metadata.seats.push(SeatMetadata {
            row: placement.row,
            column: placement.column,
            block: placement.block,
            cushion_id,
            back_id,
            left_arm_id,
            right_arm_id,
        });
    }

    #[cfg(not(test))]
    {
        let _ = (cushion_id, back_id, left_arm_id, right_arm_id);
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

fn add_projector(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
) -> Result<(), CinemaBuildError> {
    let mut pieces = Vec::with_capacity(PROJECTOR_PIECE_COUNT);

    let body_id = add_projector_cube(
        scene,
        metadata,
        ProjectorPartRuntime::Body,
        PROJECTOR_BODY_MIN,
        PROJECTOR_BODY_MAX,
        materials.brushed_metal,
    )?;
    register_projector_piece(&mut pieces, ProjectorPart::Body, body_id);

    let lens_id = add_projector_cube(
        scene,
        metadata,
        ProjectorPartRuntime::Lens,
        PROJECTOR_LENS_MIN,
        PROJECTOR_LENS_MAX,
        materials.transparent_plastic,
    )?;
    register_projector_piece(&mut pieces, ProjectorPart::Lens, lens_id);

    let front_frame_id = add_projector_cube(
        scene,
        metadata,
        ProjectorPartRuntime::Detail,
        Vec3::new(-2.90, 2.53, 2.22),
        Vec3::new(-2.40, 2.85, 2.29),
        materials.brushed_metal,
    )?;
    register_projector_piece(&mut pieces, ProjectorPart::Detail, front_frame_id);

    let rear_panel_id = add_projector_cube(
        scene,
        metadata,
        ProjectorPartRuntime::Detail,
        Vec3::new(-3.02, 2.56, 3.07),
        Vec3::new(-2.28, 2.84, 3.13),
        materials.dark_wall,
    )?;
    register_projector_piece(&mut pieces, ProjectorPart::Detail, rear_panel_id);

    let left_vent_id = add_projector_cube(
        scene,
        metadata,
        ProjectorPartRuntime::Detail,
        Vec3::new(-3.12, 2.57, 2.52),
        Vec3::new(-3.06, 2.83, 2.88),
        materials.dark_wall,
    )?;
    register_projector_piece(&mut pieces, ProjectorPart::Detail, left_vent_id);

    let right_vent_id = add_projector_cube(
        scene,
        metadata,
        ProjectorPartRuntime::Detail,
        Vec3::new(-2.24, 2.57, 2.52),
        Vec3::new(-2.18, 2.83, 2.88),
        materials.dark_wall,
    )?;
    register_projector_piece(&mut pieces, ProjectorPart::Detail, right_vent_id);

    let support_id = add_projector_cube(
        scene,
        metadata,
        ProjectorPartRuntime::Support,
        Vec3::new(-2.72, 2.92, 2.24),
        Vec3::new(-2.58, 3.19, 2.38),
        materials.brushed_metal,
    )?;
    register_projector_piece(&mut pieces, ProjectorPart::Support, support_id);

    let ceiling_plate_id = add_projector_cube(
        scene,
        metadata,
        ProjectorPartRuntime::Support,
        Vec3::new(-2.98, 3.19, 1.98),
        Vec3::new(-2.36, 3.24, 2.34),
        materials.brushed_metal,
    )?;
    register_projector_piece(&mut pieces, ProjectorPart::Support, ceiling_plate_id);

    let indicator_id = add_projector_cube(
        scene,
        metadata,
        ProjectorPartRuntime::Indicator,
        Vec3::new(-2.38, 2.65, 2.20),
        Vec3::new(-2.30, 2.73, 2.27),
        materials.projector_indicator,
    )?;
    register_projector_piece(&mut pieces, ProjectorPart::Indicator, indicator_id);

    #[cfg(test)]
    {
        debug_assert_eq!(pieces.len(), PROJECTOR_PIECE_COUNT);
        metadata.projector = Some(ProjectorMetadata {
            pieces,
            lens_position: PROJECTOR_LENS_APERTURE,
            screen_center: screen_center(),
            body_material_id: materials.brushed_metal,
            lens_material_id: materials.transparent_plastic,
            support_material_id: materials.brushed_metal,
        });
    }

    Ok(())
}

fn add_after_show_details(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
) -> Result<(), CinemaBuildError> {
    let mut pieces = Vec::with_capacity(AFTER_SHOW_DETAIL_CUBE_COUNT);

    add_popcorn_box(scene, metadata, materials, &mut pieces)?;
    add_scattered_popcorn(scene, metadata, materials, &mut pieces)?;
    add_fallen_cup_and_spill(scene, metadata, materials, &mut pieces)?;
    add_discarded_can(scene, metadata, materials, &mut pieces)?;
    add_tickets_and_wrappers(scene, metadata, materials, &mut pieces)?;

    #[cfg(test)]
    {
        debug_assert_eq!(pieces.len(), AFTER_SHOW_DETAIL_CUBE_COUNT);
        metadata.after_show = Some(AfterShowMetadata {
            pieces,
            popcorn_box_material_id: materials.popcorn_cardboard,
            popcorn_kernel_material_id: materials.popcorn_kernel,
            cup_material_id: materials.transparent_plastic,
            spill_material_id: materials.soda_spill,
            can_material_id: materials.brushed_metal,
            paper_material_id: materials.paper_trash,
        });
    }

    Ok(())
}

fn add_popcorn_box(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
    pieces: &mut Vec<(AfterShowDetail, usize)>,
) -> Result<(), CinemaBuildError> {
    let platform_y = step_top_y(1);
    let base_y = platform_y + 0.012;
    let box_pieces = [
        (
            Vec3::new(0.82, base_y, -2.48),
            Vec3::new(1.10, base_y + 0.045, -2.08),
        ),
        (
            Vec3::new(0.80, base_y + 0.045, -2.46),
            Vec3::new(0.86, base_y + 0.31, -2.11),
        ),
        (
            Vec3::new(1.08, base_y + 0.045, -2.44),
            Vec3::new(1.14, base_y + 0.27, -2.09),
        ),
        (
            Vec3::new(0.85, base_y + 0.045, -2.12),
            Vec3::new(1.11, base_y + 0.29, -2.05),
        ),
        (
            Vec3::new(0.86, base_y + 0.045, -2.53),
            Vec3::new(1.06, base_y + 0.16, -2.47),
        ),
    ];

    for (min, max) in box_pieces {
        add_after_show_cube(
            scene,
            metadata,
            pieces,
            AfterShowDetail::PopcornBox,
            min,
            max,
            materials.popcorn_cardboard,
        )?;
    }

    Ok(())
}

fn add_scattered_popcorn(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
    pieces: &mut Vec<(AfterShowDetail, usize)>,
) -> Result<(), CinemaBuildError> {
    let platform_y = step_top_y(1);
    let popcorn_pieces: [(f32, f32, f32, f32, f32); POPCORN_KERNEL_COUNT] = [
        (0.76, -2.55, 0.055, 0.045, 0.060),
        (0.92, -2.65, 0.045, 0.050, 0.040),
        (1.03, -2.58, 0.060, 0.045, 0.055),
        (0.88, -2.35, 0.050, 0.055, 0.045),
        (1.07, -2.32, 0.050, 0.040, 0.060),
        (0.72, -2.22, 0.045, 0.045, 0.050),
        (1.00, -2.18, 0.055, 0.050, 0.050),
        (0.84, -2.06, 0.050, 0.040, 0.045),
    ];

    for (x, z, width, height, depth) in popcorn_pieces {
        let min = Vec3::new(x, platform_y + 0.014, z);
        add_after_show_cube(
            scene,
            metadata,
            pieces,
            AfterShowDetail::PopcornKernel,
            min,
            min + Vec3::new(width, height, depth),
            materials.popcorn_kernel,
        )?;
    }

    Ok(())
}

fn add_fallen_cup_and_spill(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
    pieces: &mut Vec<(AfterShowDetail, usize)>,
) -> Result<(), CinemaBuildError> {
    let row_top = step_top_y(0);
    let cup_y = row_top + 0.015;
    let cup_pieces = [
        (
            Vec3::new(-1.12, cup_y, -3.45),
            Vec3::new(-0.77, cup_y + 0.13, -3.22),
        ),
        (
            Vec3::new(-1.14, cup_y + 0.01, -3.49),
            Vec3::new(-1.08, cup_y + 0.16, -3.18),
        ),
        (
            Vec3::new(-0.80, cup_y + 0.02, -3.41),
            Vec3::new(-0.72, cup_y + 0.11, -3.26),
        ),
    ];

    for (min, max) in cup_pieces {
        add_after_show_cube(
            scene,
            metadata,
            pieces,
            AfterShowDetail::FallenCup,
            min,
            max,
            materials.transparent_plastic,
        )?;
    }

    let aisle_top = FLOOR_Y + 0.105;
    let spill_y = aisle_top + 0.007;
    let spill_pieces = [
        (
            Vec3::new(-0.68, spill_y, -3.52),
            Vec3::new(-0.30, spill_y + 0.012, -3.26),
        ),
        (
            Vec3::new(-0.58, spill_y + 0.004, -3.30),
            Vec3::new(-0.22, spill_y + 0.016, -3.08),
        ),
        (
            Vec3::new(-0.45, spill_y + 0.002, -3.61),
            Vec3::new(-0.25, spill_y + 0.014, -3.47),
        ),
    ];

    for (min, max) in spill_pieces {
        add_after_show_cube(
            scene,
            metadata,
            pieces,
            AfterShowDetail::SodaSpill,
            min,
            max,
            materials.soda_spill,
        )?;
    }

    Ok(())
}

fn add_discarded_can(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
    pieces: &mut Vec<(AfterShowDetail, usize)>,
) -> Result<(), CinemaBuildError> {
    let platform_y = step_top_y(2);
    let can_y = platform_y + 0.012;
    let can_pieces = [
        (
            Vec3::new(0.84, can_y, -1.43),
            Vec3::new(1.10, can_y + 0.11, -1.22),
        ),
        (
            Vec3::new(0.80, can_y + 0.01, -1.40),
            Vec3::new(0.86, can_y + 0.13, -1.25),
        ),
        (
            Vec3::new(1.08, can_y + 0.01, -1.39),
            Vec3::new(1.14, can_y + 0.12, -1.26),
        ),
    ];

    for (min, max) in can_pieces {
        add_after_show_cube(
            scene,
            metadata,
            pieces,
            AfterShowDetail::DiscardedCan,
            min,
            max,
            materials.brushed_metal,
        )?;
    }

    Ok(())
}

fn add_tickets_and_wrappers(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    materials: CinemaMaterials,
    pieces: &mut Vec<(AfterShowDetail, usize)>,
) -> Result<(), CinemaBuildError> {
    let aisle_y = FLOOR_Y + 0.112;
    let rear_ticket = (
        Vec3::new(0.28, aisle_y, 2.76),
        Vec3::new(0.58, aisle_y + 0.014, 3.04),
    );
    let step_y = step_top_y(3) + 0.012;
    let step_ticket = (
        Vec3::new(-1.12, step_y, -0.48),
        Vec3::new(-0.84, step_y + 0.014, -0.24),
    );

    let ticket_pieces: [(Vec3, Vec3); TICKET_COUNT] = [rear_ticket, step_ticket];

    for (min, max) in ticket_pieces {
        add_after_show_cube(
            scene,
            metadata,
            pieces,
            AfterShowDetail::Ticket,
            min,
            max,
            materials.paper_trash,
        )?;
    }

    let wrapper_pieces: [(Vec3, Vec3); WRAPPER_COUNT] = [
        (
            Vec3::new(-1.08, step_top_y(2) + 0.012, -1.86),
            Vec3::new(-0.82, step_top_y(2) + 0.030, -1.68),
        ),
        (
            Vec3::new(-0.54, aisle_y + 0.002, 3.70),
            Vec3::new(-0.30, aisle_y + 0.020, 3.91),
        ),
        (
            Vec3::new(1.00, FLOOR_Y + 0.012, -4.58),
            Vec3::new(1.26, FLOOR_Y + 0.030, -4.38),
        ),
        (
            Vec3::new(-2.18, FLOOR_Y + 0.012, 3.18),
            Vec3::new(-1.92, FLOOR_Y + 0.030, 3.40),
        ),
    ];

    for (min, max) in wrapper_pieces {
        add_after_show_cube(
            scene,
            metadata,
            pieces,
            AfterShowDetail::Wrapper,
            min,
            max,
            materials.paper_trash,
        )?;
    }

    Ok(())
}

impl AfterShowDetail {
    fn element(self) -> CinemaElement {
        match self {
            Self::PopcornBox => CinemaElement::PopcornBox,
            Self::PopcornKernel => CinemaElement::PopcornKernel,
            Self::FallenCup => CinemaElement::FallenCup,
            Self::SodaSpill => CinemaElement::SodaSpill,
            Self::DiscardedCan => CinemaElement::DiscardedCan,
            Self::Ticket => CinemaElement::Ticket,
            Self::Wrapper => CinemaElement::Wrapper,
        }
    }
}

fn add_after_show_cube(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    pieces: &mut Vec<(AfterShowDetail, usize)>,
    detail: AfterShowDetail,
    first_corner: Vec3,
    second_corner: Vec3,
    material_id: usize,
) -> Result<(), CinemaBuildError> {
    let cube_id = add_cube_id(
        scene,
        metadata,
        detail.element(),
        first_corner,
        second_corner,
        material_id,
    )?;
    pieces.push((detail, cube_id));

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum ProjectorPartRuntime {
    Body,
    Lens,
    Support,
    Detail,
    Indicator,
}

impl ProjectorPartRuntime {
    fn element(self) -> CinemaElement {
        match self {
            Self::Body => CinemaElement::ProjectorBody,
            Self::Lens => CinemaElement::ProjectorLens,
            Self::Support => CinemaElement::ProjectorSupport,
            Self::Detail => CinemaElement::ProjectorDetail,
            Self::Indicator => CinemaElement::ProjectorIndicator,
        }
    }
}

fn add_projector_cube(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    part: ProjectorPartRuntime,
    first_corner: Vec3,
    second_corner: Vec3,
    material_id: usize,
) -> Result<usize, CinemaBuildError> {
    add_cube_id(
        scene,
        metadata,
        part.element(),
        first_corner,
        second_corner,
        material_id,
    )
}

fn register_projector_piece(
    pieces: &mut Vec<(ProjectorPart, usize)>,
    part: ProjectorPart,
    cube_id: usize,
) {
    pieces.push((part, cube_id));
}

#[cfg(test)]
fn screen_center() -> Vec3 {
    Vec3::new(0.0, SCREEN_BOTTOM_Y + SCREEN_HEIGHT * 0.5, SCREEN_Z)
}

fn configure_cinema_lighting(scene: &mut Scene) {
    scene.add_light(PointLight::new(
        Vec3::new(0.0, 1.55, -5.25),
        Color::new(0.50, 0.66, 1.0),
        10.2,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(0.0, 0.42, -3.65),
        Color::new(0.34, 0.46, 0.94),
        2.4,
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

fn step_z_start(step: usize) -> f32 {
    STEP_Z_START + step as f32 * SEAT_ROW_SPACING
}

fn step_top_y(step: usize) -> f32 {
    FLOOR_Y + STEP_TOP_OFFSET + step as f32 * STEP_HEIGHT_RISE
}

fn seat_center_x(block: SeatBlock, column: usize) -> f32 {
    let distance_from_aisle = SEAT_BLOCK_FIRST_CENTER_X + column as f32 * SEAT_HORIZONTAL_SPACING;

    match block {
        SeatBlock::Left => -distance_from_aisle,
        SeatBlock::Right => distance_from_aisle,
    }
}

fn add_cube(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    element: CinemaElement,
    first_corner: Vec3,
    second_corner: Vec3,
    material_id: usize,
) -> Result<(), CinemaBuildError> {
    add_cube_id(
        scene,
        metadata,
        element,
        first_corner,
        second_corner,
        material_id,
    )
    .map(|_| ())
}

fn add_cube_id(
    scene: &mut Scene,
    metadata: &mut CinemaBuildMetadata,
    element: CinemaElement,
    first_corner: Vec3,
    second_corner: Vec3,
    material_id: usize,
) -> Result<usize, CinemaBuildError> {
    let cube_id = scene.object_count();

    scene.add_cube(Cube::new(first_corner, second_corner, material_id))?;
    metadata.elements.push((element, cube_id));

    Ok(cube_id)
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
        ACOUSTIC_PANEL_COLUMNS, AFTER_SHOW_DETAIL_CUBE_COUNT, AISLE_CLEAR_HALF_WIDTH,
        AISLE_HALF_WIDTH, AfterShowDetail, BACK_Z, CinemaElement, EXPECTED_CINEMA_CUBES, FLOOR_Y,
        FRONT_Z, HALF_ROOM_WIDTH, MAX_POPCORN_KERNELS, MAX_REASONABLE_CINEMA_CUBES,
        MIN_POPCORN_KERNELS, PROJECTOR_MAX_PIECES, PROJECTOR_MIN_PIECES, ProjectorPart, ROOM_DEPTH,
        ROOM_HEIGHT, ROOM_WIDTH, SCREEN_Z, SEAT_BACK_HEIGHT, SEAT_BLOCK_COUNT, SEAT_ROW_COUNT,
        SEATS_PER_BLOCK, SEATS_PER_ROW, STEP_COUNT, TICKET_COUNT, TOTAL_SEATS, WRAPPER_COUNT,
        build_cinema_scene_with_metadata,
    };
    use crate::{
        camera::Camera,
        color::Color,
        cube::Cube,
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

    fn cube_dimensions(cube: &Cube) -> Vec3 {
        cube.max - cube.min
    }

    fn cube_center(cube: &Cube) -> Vec3 {
        (cube.min + cube.max) * 0.5
    }

    fn cube(scene: &crate::scene::Scene, object_id: usize) -> &Cube {
        scene.objects()[object_id]
            .as_cube()
            .expect("cinema metadata should reference cube objects")
    }

    fn cube_opt(scene: &crate::scene::Scene, object_id: usize) -> Option<&Cube> {
        scene
            .objects()
            .get(object_id)
            .and_then(|object| object.as_cube())
    }

    fn assert_positive_finite_dimensions(cube: &Cube) {
        let dimensions = cube_dimensions(cube);

        assert!(dimensions.x.is_finite());
        assert!(dimensions.y.is_finite());
        assert!(dimensions.z.is_finite());
        assert!(dimensions.x > 0.0);
        assert!(dimensions.y > 0.0);
        assert!(dimensions.z > 0.0);
    }

    fn required_material_id(material_id: Option<usize>) -> usize {
        assert!(material_id.is_some());
        match material_id {
            Some(material_id) => material_id,
            None => usize::MAX,
        }
    }

    fn required_projector(metadata: &super::CinemaBuildMetadata) -> &super::ProjectorMetadata {
        let projector = metadata.projector();
        assert!(projector.is_some());
        match projector {
            Some(projector) => projector,
            None => unreachable!("projector metadata was asserted as present"),
        }
    }

    fn required_after_show(metadata: &super::CinemaBuildMetadata) -> &super::AfterShowMetadata {
        let after_show = metadata.after_show();
        assert!(after_show.is_some());
        match after_show {
            Some(after_show) => after_show,
            None => unreachable!("after-show metadata was asserted as present"),
        }
    }

    fn cubes_overlap(left: &Cube, right: &Cube) -> bool {
        left.min.x < right.max.x
            && left.max.x > right.min.x
            && left.min.y < right.max.y
            && left.max.y > right.min.y
            && left.min.z < right.max.z
            && left.max.z > right.min.z
    }

    fn assert_finite_vec3(vector: Vec3) {
        assert!(vector.x.is_finite());
        assert!(vector.y.is_finite());
        assert!(vector.z.is_finite());
    }

    fn average_center(scene: &crate::scene::Scene, cube_ids: &[usize]) -> Vec3 {
        assert!(!cube_ids.is_empty());
        let mut center = Vec3::ZERO;

        for cube_id in cube_ids {
            center += cube_center(cube(scene, *cube_id));
        }

        center / cube_ids.len() as f32
    }

    #[test]
    fn build_cinema_scene_returns_valid_scene_with_geometry() {
        let (scene, metadata) = built_scene();

        assert_eq!(scene.object_count(), EXPECTED_CINEMA_CUBES);
        assert_eq!(scene.cube_count(), EXPECTED_CINEMA_CUBES);
        assert_eq!(scene.sphere_count(), 0);
        assert_eq!(scene.oriented_box_count(), 0);
        assert_eq!(scene.cylinder_count(), 0);
        assert_eq!(metadata.count(CinemaElement::Floor), 1);
        assert_eq!(metadata.count(CinemaElement::Screen), 1);
        assert_eq!(metadata.count(CinemaElement::LeftWall), 1);
        assert_eq!(metadata.count(CinemaElement::RightWall), 1);
        assert!(metadata.count(CinemaElement::Opening) >= 1);
        assert_eq!(metadata.count(CinemaElement::Step), STEP_COUNT * 2);
        assert_eq!(metadata.count(CinemaElement::SeatCushion), TOTAL_SEATS);
        assert_eq!(metadata.count(CinemaElement::SeatBack), TOTAL_SEATS);
        assert_eq!(metadata.count(CinemaElement::SeatArm), TOTAL_SEATS * 2);
        assert_eq!(
            metadata.count(CinemaElement::AcousticPanel),
            ACOUSTIC_PANEL_COLUMNS * 2
        );
        assert!(metadata.count(CinemaElement::Door) >= 1);
        assert_eq!(metadata.count(CinemaElement::ProjectorBody), 1);
        assert_eq!(metadata.count(CinemaElement::ProjectorLens), 1);
        assert!(metadata.count(CinemaElement::ProjectorSupport) >= 1);
        assert_eq!(metadata.count(CinemaElement::PopcornBox), 5);
        assert_eq!(metadata.count(CinemaElement::PopcornKernel), 8);
        assert_eq!(metadata.count(CinemaElement::FallenCup), 3);
        assert_eq!(metadata.count(CinemaElement::SodaSpill), 3);
        assert_eq!(metadata.count(CinemaElement::DiscardedCan), 3);
        assert_eq!(metadata.count(CinemaElement::Ticket), TICKET_COUNT);
        assert_eq!(metadata.count(CinemaElement::Wrapper), WRAPPER_COUNT);
    }

    #[test]
    fn cinema_contains_one_projector_with_required_parts() {
        let (_, metadata) = built_scene();
        let projector = required_projector(&metadata);

        assert_eq!(projector.count(ProjectorPart::Body), 1);
        assert_eq!(projector.count(ProjectorPart::Lens), 1);
        assert!(projector.count(ProjectorPart::Support) >= 1);
        assert!(projector.count(ProjectorPart::Detail) >= 1);
        assert!(projector.pieces().len() >= PROJECTOR_MIN_PIECES);
        assert!(projector.pieces().len() <= PROJECTOR_MAX_PIECES);
    }

    #[test]
    fn projector_pieces_are_valid_and_use_intended_materials() {
        let (scene, metadata) = built_scene();
        let projector = required_projector(&metadata);
        let metal_id = required_material_id(metadata.brushed_metal_material_id());
        let dark_id = required_material_id(metadata.dark_wall_material_id());
        let lens_id = required_material_id(metadata.transparent_plastic_material_id());

        assert_eq!(projector.body_material_id, metal_id);
        assert_eq!(projector.lens_material_id, lens_id);
        assert_eq!(projector.support_material_id, metal_id);

        for (part, cube_id) in projector.pieces() {
            assert!(*cube_id < scene.object_count());
            let cube = cube(&scene, *cube_id);
            assert_positive_finite_dimensions(cube);
            assert!(scene.material(cube.material_id).is_some());

            match part {
                ProjectorPart::Body | ProjectorPart::Support => {
                    assert_eq!(cube.material_id, metal_id);
                }
                ProjectorPart::Lens => {
                    assert_eq!(cube.material_id, lens_id);
                }
                ProjectorPart::Detail => {
                    assert!(cube.material_id == metal_id || cube.material_id == dark_id);
                }
                ProjectorPart::Indicator => {
                    assert_ne!(
                        scene
                            .material(cube.material_id)
                            .map(|material| material.emission),
                        Some(Color::BLACK)
                    );
                }
            }
        }

        assert_eq!(scene.textures().len(), metadata.textured_material_count());
    }

    #[test]
    fn projector_is_rear_elevated_and_outside_the_aisle() {
        let (scene, metadata) = built_scene();
        let projector = required_projector(&metadata);
        let rear_row_top = metadata
            .seat_rows()
            .iter()
            .map(|row| row.platform_y + SEAT_BACK_HEIGHT)
            .fold(FLOOR_Y, f32::max);

        for cube_id in projector.cube_ids() {
            let cube = cube(&scene, cube_id);

            assert!(cube.max.x <= -AISLE_HALF_WIDTH || cube.min.x >= AISLE_HALF_WIDTH);
            assert!(cube.min.x > -HALF_ROOM_WIDTH);
            assert!(cube.max.x < HALF_ROOM_WIDTH);
            assert!(cube.min.z > 0.75);
            assert!(cube.max.z < BACK_Z);
            assert!(cube.min.y > rear_row_top);
            assert!(cube.max.y < ROOM_HEIGHT);
        }
    }

    #[test]
    fn projector_does_not_intersect_seats_or_initial_camera() {
        let (scene, metadata) = built_scene();
        let projector = required_projector(&metadata);
        let projector_ids = projector.cube_ids();

        for projector_id in &projector_ids {
            let projector_cube = cube(&scene, *projector_id);
            assert!(!point_inside_cube(INITIAL_CAMERA_POSITION, projector_cube));

            for seat in metadata.seats() {
                for seat_cube_id in seat.cube_ids() {
                    assert!(
                        !cubes_overlap(projector_cube, cube(&scene, seat_cube_id)),
                        "projector cube {projector_id} overlaps seat cube {seat_cube_id}"
                    );
                }
            }
        }
    }

    #[test]
    fn projector_lens_is_separated_and_points_to_screen_center() {
        let (scene, metadata) = built_scene();
        let projector = required_projector(&metadata);
        let body_id = projector.cube_ids_for(ProjectorPart::Body)[0];
        let lens_id = projector.cube_ids_for(ProjectorPart::Lens)[0];
        let body = cube(&scene, body_id);
        let lens = cube(&scene, lens_id);
        let direction = (projector.screen_center - projector.lens_position).normalized();

        assert!(lens.max.z < body.min.z);
        assert_finite_vec3(projector.lens_position);
        assert_finite_vec3(projector.screen_center);
        assert_finite_vec3(direction);
        assert!((direction.length() - 1.0).abs() < 0.0001);
        assert!(direction.z < 0.0);
        assert!(direction.y < 0.0);
        assert!(direction.x > 0.0);

        let lens_forward = Vec3::new(0.0, 0.0, -1.0);
        assert!(direction.dot(lens_forward) > 0.92);
    }

    #[test]
    fn ray_from_projector_lens_hits_screen_first() {
        let (scene, metadata) = built_scene();
        let projector = required_projector(&metadata);
        let screen_id = metadata.cube_ids(CinemaElement::Screen)[0];
        let screen_material_id = cube(&scene, screen_id).material_id;
        let ray = Ray::new(
            projector.lens_position,
            projector.screen_center - projector.lens_position,
        );
        let hit = scene.intersect(&ray, 0.001, 100.0);

        assert!(hit.is_some());
        if let Some(hit) = hit {
            assert_eq!(hit.material_id, screen_material_id);
        }
    }

    #[test]
    fn cinema_lighting_keeps_cold_screen_fill_and_warm_accents() {
        let (scene, metadata) = built_scene();
        let screen_id = metadata.cube_ids(CinemaElement::Screen)[0];
        let screen_material = scene.material(cube(&scene, screen_id).material_id).unwrap();
        let cold_lights = scene
            .lights()
            .iter()
            .filter(|light| light.color.b > light.color.r && light.color.b >= light.color.g)
            .count();
        let warm_lights = scene
            .lights()
            .iter()
            .filter(|light| light.color.r >= light.color.g && light.color.g > light.color.b)
            .count();

        assert_ne!(screen_material.emission, Color::BLACK);
        assert!(cold_lights >= 2);
        assert!(warm_lights >= 2);
        assert!(scene.lights().len() <= 6);

        for light in scene.lights() {
            assert_finite_vec3(light.position);
            assert!(light.color.r.is_finite());
            assert!(light.color.g.is_finite());
            assert!(light.color.b.is_finite());
            assert!(light.intensity.is_finite());
            assert!(light.intensity >= 0.0);
        }
    }

    #[test]
    fn after_show_details_include_all_required_categories() {
        let (_, metadata) = built_scene();
        let after_show = required_after_show(&metadata);

        assert_eq!(after_show.pieces().len(), AFTER_SHOW_DETAIL_CUBE_COUNT);
        assert!(after_show.count(AfterShowDetail::PopcornBox) >= 4);
        assert_eq!(after_show.count(AfterShowDetail::FallenCup), 3);
        assert_eq!(after_show.count(AfterShowDetail::SodaSpill), 3);
        assert_eq!(after_show.count(AfterShowDetail::DiscardedCan), 3);
        assert_eq!(after_show.count(AfterShowDetail::Ticket), TICKET_COUNT);
        assert!(after_show.count(AfterShowDetail::Wrapper) >= 2);
    }

    #[test]
    fn popcorn_box_uses_textured_cardboard() {
        let (scene, metadata) = built_scene();
        let after_show = required_after_show(&metadata);
        let box_ids = after_show.cube_ids_for(AfterShowDetail::PopcornBox);

        assert!(!box_ids.is_empty());
        for cube_id in box_ids {
            let cube = cube(&scene, cube_id);
            assert_positive_finite_dimensions(cube);
            assert_eq!(cube.material_id, after_show.popcorn_box_material_id);
        }

        let cardboard = scene.material(after_show.popcorn_box_material_id).unwrap();
        assert!(cardboard.texture_id.is_some());
        assert!(scene.texture(cardboard.texture_id.unwrap()).is_some());
    }

    #[test]
    fn scattered_popcorn_is_grouped_near_box_and_reuses_material() {
        let (scene, metadata) = built_scene();
        let after_show = required_after_show(&metadata);
        let popcorn_ids = after_show.cube_ids_for(AfterShowDetail::PopcornKernel);
        let box_ids = after_show.cube_ids_for(AfterShowDetail::PopcornBox);
        let box_center = average_center(&scene, &box_ids);
        let mut centers = Vec::new();

        assert!(popcorn_ids.len() >= MIN_POPCORN_KERNELS);
        assert!(popcorn_ids.len() <= MAX_POPCORN_KERNELS);

        for cube_id in popcorn_ids {
            let cube = cube(&scene, cube_id);
            let center = cube_center(cube);

            assert_eq!(cube.material_id, after_show.popcorn_kernel_material_id);
            assert!((center - box_center).length() < 0.75);
            assert!(!centers.iter().any(|known: &Vec3| known.approx_eq(center)));
            centers.push(center);
        }
    }

    #[test]
    fn fallen_cup_uses_transparent_plastic_and_valid_refraction() {
        let (scene, metadata) = built_scene();
        let after_show = required_after_show(&metadata);
        let cup_ids = after_show.cube_ids_for(AfterShowDetail::FallenCup);
        let cup_material = scene.material(after_show.cup_material_id).unwrap();

        assert_eq!(cup_ids.len(), 3);
        assert!(cup_material.transparency > 0.7);
        assert!(cup_material.refractive_index >= 1.49);

        for cube_id in cup_ids {
            let cube = cube(&scene, cube_id);
            assert_eq!(cube.material_id, after_show.cup_material_id);
            assert_positive_finite_dimensions(cube);
        }
    }

    #[test]
    fn soda_spill_is_near_cup_slightly_above_floor_and_reflective() {
        let (scene, metadata) = built_scene();
        let after_show = required_after_show(&metadata);
        let spill_ids = after_show.cube_ids_for(AfterShowDetail::SodaSpill);
        let cup_ids = after_show.cube_ids_for(AfterShowDetail::FallenCup);
        let cup_center = average_center(&scene, &cup_ids);
        let spill_material = scene.material(after_show.spill_material_id).unwrap();

        assert_eq!(spill_ids.len(), 3);
        assert!((0.10..=0.40).contains(&spill_material.reflectivity));
        assert!(spill_material.transparency <= 0.20);

        for cube_id in spill_ids {
            let cube = cube(&scene, cube_id);

            assert_eq!(cube.material_id, after_show.spill_material_id);
            assert!(cube.min.y > FLOOR_Y);
            assert!(cube.min.y > FLOOR_Y + 0.10);
            assert!((cube_center(cube) - cup_center).length() < 0.85);
        }
    }

    #[test]
    fn discarded_can_uses_metal_material() {
        let (scene, metadata) = built_scene();
        let after_show = required_after_show(&metadata);
        let can_ids = after_show.cube_ids_for(AfterShowDetail::DiscardedCan);
        let metal_material = scene.material(after_show.can_material_id).unwrap();

        assert_eq!(can_ids.len(), 3);
        assert!(metal_material.reflectivity > 0.7);

        for cube_id in can_ids {
            let cube = cube(&scene, cube_id);
            assert_eq!(cube.material_id, after_show.can_material_id);
            assert_positive_finite_dimensions(cube);
        }
    }

    #[test]
    fn tickets_and_wrappers_are_distinct_paper_details() {
        let (scene, metadata) = built_scene();
        let after_show = required_after_show(&metadata);
        let ticket_ids = after_show.cube_ids_for(AfterShowDetail::Ticket);
        let wrapper_ids = after_show.cube_ids_for(AfterShowDetail::Wrapper);

        assert_eq!(ticket_ids.len(), TICKET_COUNT);
        assert_eq!(wrapper_ids.len(), WRAPPER_COUNT);

        for cube_id in ticket_ids.iter().chain(wrapper_ids.iter()) {
            let cube = cube(&scene, *cube_id);
            assert_eq!(cube.material_id, after_show.paper_material_id);
            assert_positive_finite_dimensions(cube);
        }

        let ticket_width = cube_dimensions(cube(&scene, ticket_ids[0])).x;
        let wrapper_width = cube_dimensions(cube(&scene, wrapper_ids[0])).x;
        assert!(ticket_width >= wrapper_width);
    }

    #[test]
    fn after_show_details_are_inside_room_and_above_floor() {
        let (scene, metadata) = built_scene();
        let after_show = required_after_show(&metadata);

        for cube_id in after_show.cube_ids() {
            let cube = cube(&scene, cube_id);

            assert_positive_finite_dimensions(cube);
            assert!(cube.min.y >= FLOOR_Y);
            assert!(cube.max.y < ROOM_HEIGHT);
            assert!(cube.min.x > -HALF_ROOM_WIDTH);
            assert!(cube.max.x < HALF_ROOM_WIDTH);
            assert!(cube.min.z > FRONT_Z);
            assert!(cube.max.z < BACK_Z);
            assert!(scene.material(cube.material_id).is_some());
        }
    }

    #[test]
    fn after_show_details_keep_central_aisle_passable_and_avoid_seats() {
        let (scene, metadata) = built_scene();
        let after_show = required_after_show(&metadata);

        for detail_id in after_show.cube_ids() {
            let detail = cube(&scene, detail_id);

            assert!(
                detail.max.x <= -AISLE_CLEAR_HALF_WIDTH || detail.min.x >= AISLE_CLEAR_HALF_WIDTH
            );

            for seat in metadata.seats() {
                for seat_cube_id in seat.cube_ids() {
                    assert!(
                        !cubes_overlap(detail, cube(&scene, seat_cube_id)),
                        "after-show detail cube {detail_id} overlaps seat cube {seat_cube_id}"
                    );
                }
            }
        }
    }

    #[test]
    fn after_show_details_preserve_projector_path_and_scene_budget() {
        let (scene, metadata) = built_scene();
        let projector = required_projector(&metadata);
        let screen_id = metadata.cube_ids(CinemaElement::Screen)[0];
        let screen_material_id = cube(&scene, screen_id).material_id;
        let ray = Ray::new(
            projector.lens_position,
            projector.screen_center - projector.lens_position,
        );
        let hit = scene.intersect(&ray, 0.001, 100.0);

        assert!(scene.cube_count() < MAX_REASONABLE_CINEMA_CUBES);
        assert!(hit.is_some());
        if let Some(hit) = hit {
            assert_eq!(hit.material_id, screen_material_id);
        }
    }

    #[test]
    fn after_show_materials_are_valid_without_extra_textures() {
        let (scene, metadata) = built_scene();
        let after_show = required_after_show(&metadata);

        for material_id in [
            after_show.popcorn_kernel_material_id,
            after_show.spill_material_id,
            after_show.paper_material_id,
        ] {
            let material = scene.material(material_id).unwrap();

            assert!(material.texture_id.is_none());
            assert!((0.0..=1.0).contains(&material.specular_strength));
            assert!((0.0..=1.0).contains(&material.reflectivity));
            assert!((0.0..=1.0).contains(&material.transparency));
            assert!(material.refractive_index >= 1.0);
            assert!(material.shininess.is_finite());
        }

        assert_eq!(scene.textures().len(), metadata.textured_material_count());
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
    fn cinema_has_expected_seating_layout() {
        let (_, metadata) = built_scene();
        let mut seats_by_row = [0; SEAT_ROW_COUNT];
        let mut left_block_seats = [0; SEAT_ROW_COUNT];
        let mut right_block_seats = [0; SEAT_ROW_COUNT];
        let mut columns_by_row = [[0; SEATS_PER_BLOCK]; SEAT_ROW_COUNT];

        assert_eq!(metadata.seat_rows().len(), SEAT_ROW_COUNT);
        assert_eq!(metadata.seats().len(), TOTAL_SEATS);
        assert_eq!(SEAT_BLOCK_COUNT, 2);

        for row_metadata in metadata.seat_rows() {
            assert!(row_metadata.row < SEAT_ROW_COUNT);
            assert_eq!(row_metadata.seat_count, SEATS_PER_ROW);
        }

        for seat in metadata.seats() {
            assert!(seat.row < SEAT_ROW_COUNT);
            assert!(seat.column < SEATS_PER_BLOCK);
            seats_by_row[seat.row] += 1;
            columns_by_row[seat.row][seat.column] += 1;

            match seat.block {
                super::SeatBlock::Left => left_block_seats[seat.row] += 1,
                super::SeatBlock::Right => right_block_seats[seat.row] += 1,
            }
        }

        for row in 0..SEAT_ROW_COUNT {
            assert_eq!(seats_by_row[row], SEATS_PER_ROW);
            assert_eq!(left_block_seats[row], SEATS_PER_BLOCK);
            assert_eq!(right_block_seats[row], SEATS_PER_BLOCK);

            for column_count in columns_by_row[row] {
                assert_eq!(column_count, SEAT_BLOCK_COUNT);
            }
        }
    }

    #[test]
    fn seat_parts_are_finite_sized_and_use_expected_materials() {
        let (scene, metadata) = built_scene();
        let fabric_id = required_material_id(metadata.seat_fabric_material_id());
        let metal_id = required_material_id(metadata.brushed_metal_material_id());

        for seat in metadata.seats() {
            for cube_id in seat.cube_ids() {
                assert!(cube_id < scene.object_count());

                if let Some(cube) = cube_opt(&scene, cube_id) {
                    assert_positive_finite_dimensions(cube);
                    assert!(scene.material(cube.material_id).is_some());
                }
            }

            if let Some(cushion) = cube_opt(&scene, seat.cushion_id) {
                assert_eq!(cushion.material_id, fabric_id);
                assert!(
                    scene
                        .material(cushion.material_id)
                        .and_then(|material| material.texture_id)
                        .is_some()
                );
            }

            if let Some(back) = cube_opt(&scene, seat.back_id) {
                assert_eq!(back.material_id, fabric_id);
                assert!(
                    scene
                        .material(back.material_id)
                        .and_then(|material| material.texture_id)
                        .is_some()
                );
            }

            if let Some(left_arm) = cube_opt(&scene, seat.left_arm_id) {
                assert_eq!(left_arm.material_id, metal_id);
            }

            if let Some(right_arm) = cube_opt(&scene, seat.right_arm_id) {
                assert_eq!(right_arm.material_id, metal_id);
            }
        }

        assert_eq!(scene.textures().len(), metadata.textured_material_count());
    }

    #[test]
    fn seat_rows_follow_steps_and_seats_face_screen() {
        let (scene, metadata) = built_scene();

        for row_pair in metadata.seat_rows().windows(2) {
            assert!(row_pair[1].platform_y > row_pair[0].platform_y);
            assert!(row_pair[1].z_start > row_pair[0].z_start);
        }

        for seat in metadata.seats() {
            let row_metadata = metadata
                .seat_rows()
                .iter()
                .find(|row_metadata| row_metadata.row == seat.row);
            assert!(row_metadata.is_some());

            if let (Some(row_metadata), Some(cushion), Some(back)) = (
                row_metadata,
                cube_opt(&scene, seat.cushion_id),
                cube_opt(&scene, seat.back_id),
            ) {
                assert!(cushion.min.y >= row_metadata.platform_y);
                assert!(back.min.z > cushion.max.z);
                assert!(back.max.z <= row_metadata.rear_z());
            }
        }
    }

    #[test]
    fn seating_preserves_aisle_bounds_and_screen_access() {
        let (scene, metadata) = built_scene();
        let camera = initial_camera(4.0 / 3.0);
        let screen_ids = metadata.cube_ids(CinemaElement::Screen);

        for seat in metadata.seats() {
            for cube_id in seat.cube_ids() {
                if let Some(cube) = cube_opt(&scene, cube_id) {
                    assert!(cube.max.x <= -AISLE_HALF_WIDTH || cube.min.x >= AISLE_HALF_WIDTH);
                    assert!(cube.min.x > -HALF_ROOM_WIDTH);
                    assert!(cube.max.x < HALF_ROOM_WIDTH);
                    assert!(cube.min.z > SCREEN_Z);
                }
            }
        }

        let screen_ray = Ray::new(camera.position, camera.target - camera.position);
        let screen_hit = scene.intersect(&screen_ray, 0.001, 100.0);
        let screen_material_id = screen_ids
            .first()
            .and_then(|screen_id| cube_opt(&scene, *screen_id))
            .map(|screen_cube| screen_cube.material_id);

        assert!(screen_hit.is_some());
        if let Some(screen_hit) = screen_hit {
            assert_eq!(Some(screen_hit.material_id), screen_material_id);
        }

        let aisle_origin = Vec3::new(0.0, 0.45, -1.0);
        let aisle_target = Vec3::new(0.0, 0.6, SCREEN_Z);
        let aisle_ray = Ray::new(aisle_origin, aisle_target - aisle_origin);
        let aisle_hit = scene.intersect(&aisle_ray, 0.001, 100.0);

        assert!(aisle_hit.is_some());
        if let Some(aisle_hit) = aisle_hit {
            assert_eq!(Some(aisle_hit.material_id), screen_material_id);
        }
    }

    #[test]
    fn ray_can_hit_a_textured_seat() {
        let (scene, metadata) = built_scene();
        let fabric_id = required_material_id(metadata.seat_fabric_material_id());
        let target_seat = metadata.seats().iter().find(|seat| {
            seat.row == 0 && seat.column == 1 && seat.block == super::SeatBlock::Right
        });

        assert!(target_seat.is_some());

        if let Some(target_seat) = target_seat
            && let Some(cushion) = cube_opt(&scene, target_seat.cushion_id)
        {
            let target = cube_center(cushion);
            let origin = target + Vec3::new(0.0, 0.32, -0.7);
            let ray = Ray::new(origin, target - origin);
            let hit = scene.intersect(&ray, 0.001, 10.0);

            assert!(hit.is_some());
            if let Some(hit) = hit {
                assert_eq!(hit.material_id, fabric_id);
            }
        }
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
            cube(&scene, screen_ids[0]).material_id
        );

        let floor_ray = Ray::new(
            camera.position,
            Vec3::new(-1.35, -1.21, 4.35) - camera.position,
        );
        let floor_hit = scene.intersect(&floor_ray, 0.001, 100.0).unwrap();
        assert_eq!(
            floor_hit.material_id,
            cube(&scene, floor_ids[0]).material_id
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

        assert_eq!(hit.material_id, cube(&scene, screen_cube_id).material_id);
        assert_ne!(color.to_u32(), Color::BLACK.to_u32());
    }
}
