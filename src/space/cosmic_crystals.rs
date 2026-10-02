//! Level four, Cristales Cosmicos: a purple crystal system traced from a
//! screenshot of the original level, with three birds at the slingshot and
//! four pigs among the crystals.
//!
//! The level lies in the XY plane like the picture: Y up and +Z towards the
//! default camera. `reference_point` maps a pixel of the 1600x1200 reference
//! screenshot to that plane, so the layout constants can be checked against
//! the picture. Crystals and gems are faceted `Crystal` primitives. The
//! planets and asteroids are balls of small cubes (see `voxel`); the two
//! glass planets are hollow balls of glass cubes.
//!
//! Shadow rays stop at every object, including the transparent gravity
//! bubbles and glass planets, so each closed region (outside, inside each
//! bubble and inside each glass planet) has its own lights.

use super::{
    ROCK_LUMP_OFFSET, SpaceBuildError, SpaceMaterials, add_slingshot, add_space_pig,
    add_voxel_rock, base_space_scene_with_skybox, basis_with_up, birds, smooth_hash, smoothstep,
    sphere_direction_from_uv, value_noise_3d,
    voxel::{self, VoxelBall, VoxelBody, VoxelBodyParts},
};
use crate::{
    basis::Basis3,
    camera::OrbitCamera,
    color::Color,
    cone::Cone,
    crystal::{Crystal, CrystalShape},
    cylinder::Cylinder,
    light::PointLight,
    material::Material,
    math::{Vec2, Vec3},
    oriented_box::OrientedBox,
    radial::RadialFrame,
    scene::Scene,
    skybox::Skybox,
    sphere::Sphere,
    texture::{Texture, TextureError, WrapMode},
};

const REFERENCE_PIXELS_PER_UNIT: f32 = 125.0;
const REFERENCE_CENTER_X: f32 = 800.0;
const REFERENCE_CENTER_Y: f32 = 600.0;

/// Point of the level plane under pixel `(x, y)` of the reference picture.
const fn reference_point(x: f32, y: f32) -> Vec3 {
    Vec3::new(
        (x - REFERENCE_CENTER_X) / REFERENCE_PIXELS_PER_UNIT,
        (REFERENCE_CENTER_Y - y) / REFERENCE_PIXELS_PER_UNIT,
        0.0,
    )
}

const fn reference_length(pixels: f32) -> f32 {
    pixels / REFERENCE_PIXELS_PER_UNIT
}

/// Bottom left planet with the slingshot. It is cut by the picture border,
/// so it is a little smaller and higher than in the reference.
pub const LEVEL_FOUR_LAUNCH_PLANET_CENTER: Vec3 = Vec3::new(-5.20, -4.40, 0.0);
pub const LEVEL_FOUR_LAUNCH_PLANET_RADIUS: f32 = 1.75;
pub const LEVEL_FOUR_CENTRAL_BUBBLE_CENTER: Vec3 = reference_point(752.0, 318.0);
pub const LEVEL_FOUR_CENTRAL_BUBBLE_RADIUS: f32 = reference_length(293.0);
pub const LEVEL_FOUR_CENTRAL_GLASS_CENTER: Vec3 = reference_point(752.0, 328.0);
pub const LEVEL_FOUR_CENTRAL_GLASS_RADIUS: f32 = reference_length(122.0);
pub const LEVEL_FOUR_RIGHT_PLANET_CENTER: Vec3 = reference_point(1445.0, 170.0);
pub const LEVEL_FOUR_RIGHT_PLANET_RADIUS: f32 = reference_length(213.0);
pub const LEVEL_FOUR_RIGHT_BUBBLE_CENTER: Vec3 = reference_point(1430.0, 195.0);
pub const LEVEL_FOUR_RIGHT_BUBBLE_RADIUS: f32 = reference_length(345.0);
pub const LEVEL_FOUR_LOWER_BUBBLE_CENTER: Vec3 = reference_point(1117.0, 800.0);
pub const LEVEL_FOUR_LOWER_BUBBLE_RADIUS: f32 = reference_length(195.0);
pub const LEVEL_FOUR_LOWER_GLASS_CENTER: Vec3 = reference_point(1117.0, 793.0);
pub const LEVEL_FOUR_LOWER_GLASS_RADIUS: f32 = reference_length(82.0);
pub const LEVEL_FOUR_CAMERA_TARGET: Vec3 = Vec3::new(0.45, 0.35, 0.0);

/// The tile covers a quarter turn horizontally and the whole sky
/// vertically, so it is twice as tall as wide to keep texels square.
const LEVEL_FOUR_SKYBOX_TILE_WIDTH: usize = 768;
const LEVEL_FOUR_SKYBOX_TILE_HEIGHT: usize = 1536;
/// Lighter violet ribbons across the upper sky: height, wave amplitude,
/// phase, half width and strength, in tile coordinates.
const LEVEL_FOUR_SKY_RIBBONS: [(f32, f32, f32, f32, f32); 3] = [
    (0.600, 0.030, 0.10, 0.030, 0.55),
    (0.548, 0.020, 0.55, 0.016, 0.38),
    (0.660, 0.028, 0.80, 0.022, 0.40),
];
const LEVEL_FOUR_SKYBOX_TILES: f32 = 4.0;
const CRYSTAL_PLANET_TEXTURE_WIDTH: usize = 512;
const CRYSTAL_PLANET_TEXTURE_HEIGHT: usize = 256;
/// Direction, from a planet center, of the middle of its swirl of rings.
const CRYSTAL_PLANET_SWIRL_POLE: Vec3 = Vec3::new(-0.42, 0.40, 0.81);
const CRYSTAL_PLANET_RING_FREQUENCY: f32 = 11.0;

/// Crystals are sunk this far (relative to their radius) below the surface
/// they grow from, so their flat base never shows.
const CRYSTAL_EMBED_RADII: f32 = 1.3;
/// Fraction of a crystal's length taken by its pointed tip.
const CRYSTAL_TIP_FRACTION: f32 = 0.30;
const CRYSTAL_SIDES: u32 = 6;
/// Embedded gems are octahedra: four sides and a point on each end.
const EMBEDDED_GEM_SIDES: u32 = 4;
/// Cut gems (the big diamond, the purple and the golden gem) are brilliants.
const CUT_GEM_SIDES: u32 = 8;

const PLANK_THICKNESS: f32 = reference_length(14.0);
const PLANK_DEPTH: f32 = 0.20;
const ROPE_RADIUS: f32 = 0.022;
const JOINT_RADIUS: f32 = reference_length(9.0);
const HANGING_BALL_RADIUS: f32 = reference_length(16.0);
const SMALL_STONE_HALF: f32 = reference_length(5.0);

/// Edge of the cubes that build the level four planets and asteroids. The
/// two glass planets are made of glass cubes.
const LEVEL_FOUR_CUBE_EDGE: f32 = 0.11;
/// The glass planets are a little rounder: their cubes are smaller.
const LEVEL_FOUR_GLASS_CUBE_EDGE: f32 = 0.095;
/// Every planet and asteroid is at least this many cubes across.
const LEVEL_FOUR_MIN_CUBES_ACROSS: f32 = 7.0;

const LEVEL_FOUR_SLINGSHOT_SCALE: f32 = 1.6;
const LEVEL_FOUR_SLINGSHOT_YAW_RADIANS: f32 = -0.35;
/// The loaded bird looks up and right, towards the central bubble.
const LEVEL_FOUR_BIRD_AIM: Vec3 = Vec3::new(0.58, 0.81, 0.0);
/// Where the other two birds wait on the launch planet, left of its crystals:
/// angle and depth in degrees, like the crystals.
const LEVEL_FOUR_WAITING_BIRDS: [(f32, f32); 2] = [(-40.0, 22.0), (-55.0, 26.0)];

pub const LEVEL_FOUR_PIG_COUNT: usize = 4;

/// Where a level four pig stands.
#[derive(Debug, Clone, Copy)]
enum PigSpot {
    /// On a planet of cubes: which planet, angle and depth in degrees.
    Planet(PigPlanet, f32, f32),
    /// Resting on the floating contraption: center pixel and up direction.
    Floating((f32, f32), Vec3),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PigPlanet {
    CentralGlass,
    Right,
    LowerGlass,
}

/// The four pigs: a medium one on the left of the central glass planet, a
/// big one under the right planet, a small one on top of the lower glass
/// planet and a small one on the floating contraption.
const LEVEL_FOUR_PIGS: [(PigSpot, f32); LEVEL_FOUR_PIG_COUNT] = [
    (PigSpot::Planet(PigPlanet::CentralGlass, 268.0, 14.0), 0.24),
    (PigSpot::Planet(PigPlanet::Right, 132.0, 18.0), 0.30),
    (PigSpot::Planet(PigPlanet::LowerGlass, 18.0, 16.0), 0.20),
    (
        PigSpot::Floating((407.0, 334.0), Vec3::new(0.35, 0.94, 0.0)),
        0.18,
    ),
];
/// Pigs sink this fraction of their radius into the ground.
const PIG_GROUND_SINK: f32 = 0.05;
/// Pigs look at the camera, turned a little towards the slingshot.
const PIG_LOOK: Vec3 = Vec3::new(-0.28, -0.12, 1.0);

/// Colors of the crystal materials, from light to dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrystalShade {
    Pale,
    Pink,
    Deep,
}

/// A crystal growing out of a sphere. Angles are in degrees: `angle` is where
/// it grows, clockwise from +Y as seen from the default camera, and `depth`
/// tilts that point towards the camera. The axis leans `lean` degrees
/// clockwise from the surface normal.
#[derive(Debug, Clone, Copy)]
struct CrystalSpec {
    angle: f32,
    depth: f32,
    lean: f32,
    length: f32,
    radius: f32,
    shade: CrystalShade,
}

const fn crystal(
    angle: f32,
    depth: f32,
    lean: f32,
    length: f32,
    radius: f32,
    shade: CrystalShade,
) -> CrystalSpec {
    CrystalSpec {
        angle,
        depth,
        lean,
        length,
        radius,
        shade,
    }
}

/// Small octahedral gem half sunk in a sphere: angle, depth (degrees) and
/// size.
type GemSpec = (f32, f32, f32);

use CrystalShade::{Deep, Pale, Pink};

const LAUNCH_PLANET_CRYSTALS: [CrystalSpec; 19] = [
    // Tall thin crystals on the top left.
    crystal(-17.0, 10.0, -12.0, 0.68, 0.14, Pink),
    crystal(-10.0, -15.0, -6.0, 0.88, 0.16, Deep),
    crystal(-4.0, 20.0, -2.0, 0.77, 0.15, Pale),
    crystal(2.0, -5.0, 4.0, 0.95, 0.18, Pink),
    crystal(7.0, 25.0, 8.0, 0.61, 0.12, Pale),
    crystal(-13.0, -30.0, -10.0, 0.55, 0.12, Deep),
    // Small cluster in the middle.
    crystal(14.0, 12.0, 0.0, 0.61, 0.14, Pink),
    crystal(18.0, -20.0, 4.0, 0.50, 0.12, Deep),
    crystal(21.0, 32.0, -5.0, 0.42, 0.11, Pale),
    // Crystals pointing right, under the big diamond.
    crystal(46.0, 15.0, 18.0, 1.04, 0.26, Pink),
    crystal(54.0, -20.0, 22.0, 0.94, 0.23, Deep),
    crystal(61.0, 25.0, 15.0, 1.10, 0.27, Pale),
    crystal(69.0, -5.0, 18.0, 0.99, 0.24, Pink),
    crystal(77.0, 20.0, 10.0, 0.77, 0.19, Deep),
    crystal(85.0, -25.0, 8.0, 0.66, 0.16, Pink),
    crystal(94.0, 10.0, 5.0, 0.50, 0.14, Pale),
    crystal(41.0, -35.0, 10.0, 0.66, 0.16, Pink),
    // Behind the planet, seen when the camera turns.
    crystal(-30.0, -60.0, -5.0, 0.65, 0.17, Pale),
    crystal(60.0, -62.0, 10.0, 0.75, 0.20, Pink),
];

const LAUNCH_PLANET_GEMS: [GemSpec; 18] = [
    (-72.0, 25.0, 0.08),
    (-62.0, 40.0, 0.07),
    (-55.0, 15.0, 0.09),
    (-46.0, 35.0, 0.08),
    (-38.0, 20.0, 0.07),
    (-30.0, 45.0, 0.09),
    (-24.0, 30.0, 0.07),
    (-8.0, 42.0, 0.08),
    (4.0, 38.0, 0.07),
    (12.0, 50.0, 0.08),
    (26.0, 45.0, 0.07),
    (34.0, 30.0, 0.08),
    (58.0, 40.0, 0.07),
    (72.0, 42.0, 0.08),
    (88.0, 35.0, 0.07),
    (100.0, 28.0, 0.08),
    (-66.0, 55.0, 0.07),
    (-18.0, 58.0, 0.08),
];

/// The big diamond the slingshot stands on: where it grows, its axis and
/// size.
const LAUNCH_DIAMOND_ANGLE: f32 = 32.0;
const LAUNCH_DIAMOND_AXIS_ANGLE: f32 = 22.0;
const LAUNCH_DIAMOND_DEPTH: f32 = 8.0;
const LAUNCH_DIAMOND_SHAPE: CrystalShape = CrystalShape {
    radius: 0.46,
    body_height: 0.10,
    tip_height: 0.26,
    tip_cut: 0.14,
    base_tip_height: 0.62,
    sides: CUT_GEM_SIDES,
};
/// How far the diamond's bottom point is sunk into the planet.
const LAUNCH_DIAMOND_EMBED: f32 = 0.30;

const CENTRAL_GLASS_CRYSTALS: [CrystalSpec; 21] = [
    // The two tall crystals on top and the cluster between them.
    crystal(25.0, 0.0, -6.0, 1.15, 0.28, Pink),
    crystal(-4.0, 5.0, -8.0, 0.95, 0.23, Pale),
    crystal(10.0, 25.0, 0.0, 0.55, 0.11, Deep),
    crystal(14.0, -25.0, 2.0, 0.70, 0.14, Pink),
    crystal(-24.0, 15.0, 0.0, 0.55, 0.13, Deep),
    crystal(0.0, -40.0, 0.0, 0.75, 0.18, Deep),
    crystal(30.0, -45.0, 0.0, 0.60, 0.15, Pale),
    // Broken chunks in front.
    crystal(-12.0, 40.0, 0.0, 0.35, 0.10, Pale),
    crystal(18.0, 45.0, 0.0, 0.30, 0.09, Pink),
    crystal(36.0, 38.0, 0.0, 0.30, 0.08, Deep),
    // Lower cluster on the right.
    crystal(45.0, 10.0, 5.0, 0.50, 0.13, Pink),
    crystal(55.0, -15.0, 10.0, 0.40, 0.11, Deep),
    crystal(63.0, 20.0, 12.0, 0.32, 0.09, Pale),
    crystal(73.0, 10.0, 5.0, 0.30, 0.07, Pink),
    crystal(85.0, -10.0, 0.0, 0.25, 0.06, Deep),
    // Top left.
    crystal(-36.0, -15.0, -6.0, 0.40, 0.10, Pink),
    crystal(-44.0, 20.0, -4.0, 0.28, 0.07, Pale),
    crystal(40.0, -30.0, 8.0, 0.45, 0.11, Pink),
    crystal(-18.0, -35.0, -4.0, 0.50, 0.12, Pink),
    // Towards the camera and behind, seen when the camera turns.
    crystal(5.0, 65.0, 0.0, 0.45, 0.12, Pink),
    crystal(20.0, -65.0, 0.0, 0.50, 0.13, Deep),
];

const CENTRAL_GLASS_GEMS: [GemSpec; 8] = [
    (-52.0, 20.0, 0.08),
    (-62.0, 30.0, 0.07),
    (-74.0, 15.0, 0.08),
    (-86.0, 28.0, 0.07),
    (-98.0, 18.0, 0.08),
    (-108.0, 30.0, 0.07),
    (-64.0, 50.0, 0.06),
    (-90.0, 48.0, 0.06),
];

const LOWER_GLASS_CRYSTALS: [CrystalSpec; 12] = [
    crystal(45.0, 10.0, 12.0, 0.60, 0.12, Pink),
    crystal(55.0, -15.0, 15.0, 0.75, 0.15, Deep),
    crystal(65.0, 20.0, 18.0, 0.85, 0.17, Pale),
    crystal(75.0, -5.0, 15.0, 0.90, 0.18, Pink),
    crystal(85.0, 25.0, 10.0, 0.70, 0.14, Deep),
    crystal(95.0, -20.0, 5.0, 0.55, 0.12, Pink),
    crystal(105.0, 10.0, 0.0, 0.35, 0.09, Pale),
    crystal(70.0, -35.0, 10.0, 0.50, 0.12, Pink),
    crystal(60.0, 40.0, 10.0, 0.40, 0.10, Pink),
    crystal(90.0, -40.0, 5.0, 0.45, 0.11, Deep),
    // Towards the camera and behind, seen when the camera turns.
    crystal(75.0, 62.0, 5.0, 0.45, 0.11, Pale),
    crystal(80.0, -62.0, 5.0, 0.50, 0.12, Pink),
];

const LOWER_GLASS_GEMS: [GemSpec; 7] = [
    (190.0, 20.0, 0.07),
    (205.0, 10.0, 0.06),
    (220.0, 25.0, 0.07),
    (235.0, 5.0, 0.06),
    (250.0, 20.0, 0.07),
    (262.0, 10.0, 0.06),
    (212.0, 45.0, 0.06),
];

const RIGHT_PLANET_CRYSTALS: [CrystalSpec; 30] = [
    // Upper mass, pointing left.
    crystal(252.0, 10.0, -8.0, 0.99, 0.32, Pink),
    crystal(260.0, -20.0, -5.0, 1.21, 0.36, Deep),
    crystal(266.0, 25.0, 0.0, 1.32, 0.35, Pale),
    crystal(272.0, -5.0, 5.0, 1.16, 0.32, Pink),
    crystal(279.0, 15.0, 10.0, 0.94, 0.29, Deep),
    crystal(286.0, -25.0, 12.0, 0.77, 0.26, Pink),
    crystal(294.0, 10.0, 15.0, 0.55, 0.20, Pale),
    crystal(258.0, 40.0, -5.0, 0.66, 0.22, Pink),
    crystal(270.0, -40.0, 0.0, 0.88, 0.29, Deep),
    // Short crystals around the golden gem.
    crystal(233.0, 22.0, 0.0, 0.33, 0.12, Pale),
    crystal(245.0, -15.0, 0.0, 0.39, 0.13, Pink),
    // Lower mass, pointing down and left.
    crystal(188.0, 10.0, -10.0, 0.88, 0.29, Pink),
    crystal(196.0, -20.0, -6.0, 1.10, 0.32, Pale),
    crystal(204.0, 25.0, -3.0, 1.21, 0.35, Deep),
    crystal(212.0, -5.0, 0.0, 1.32, 0.36, Pink),
    crystal(220.0, 20.0, 4.0, 1.10, 0.32, Pale),
    crystal(228.0, -25.0, 6.0, 0.88, 0.29, Pink),
    crystal(200.0, 45.0, 0.0, 0.66, 0.22, Pink),
    crystal(216.0, -45.0, 0.0, 0.77, 0.26, Deep),
    crystal(180.0, 15.0, -12.0, 0.61, 0.20, Pink),
    // Extra crystals that fill the gaps between the two masses.
    crystal(256.0, -8.0, -4.0, 0.95, 0.30, Pale),
    crystal(263.0, 12.0, 2.0, 1.25, 0.28, Pink),
    crystal(276.0, 32.0, 8.0, 0.80, 0.26, Pale),
    crystal(290.0, -12.0, 14.0, 0.70, 0.22, Pink),
    crystal(192.0, 30.0, -8.0, 0.95, 0.28, Deep),
    crystal(208.0, 8.0, -2.0, 1.30, 0.30, Pale),
    crystal(224.0, 34.0, 5.0, 0.85, 0.26, Pink),
    crystal(184.0, -30.0, -10.0, 0.75, 0.24, Pink),
    // Behind the planet, seen when the camera turns.
    crystal(215.0, -62.0, 0.0, 0.80, 0.24, Pale),
    crystal(275.0, -65.0, 0.0, 0.85, 0.25, Pink),
];

const RIGHT_PLANET_GEMS: [GemSpec; 16] = [
    (172.0, 30.0, 0.09),
    (184.0, 45.0, 0.08),
    (196.0, 35.0, 0.09),
    (226.0, 42.0, 0.08),
    (240.0, 35.0, 0.09),
    (254.0, 48.0, 0.08),
    (262.0, 30.0, 0.09),
    (276.0, 42.0, 0.08),
    (288.0, 30.0, 0.09),
    (298.0, 20.0, 0.08),
    (306.0, 38.0, 0.09),
    (316.0, 25.0, 0.08),
    (324.0, 45.0, 0.07),
    (210.0, 55.0, 0.07),
    (282.0, 58.0, 0.07),
    (160.0, 20.0, 0.07),
];

const RIGHT_PLANET_GOLD_GEM_ANGLE: f32 = 239.0;
const RIGHT_PLANET_GOLD_GEM_DEPTH: f32 = 30.0;
const GOLD_GEM_SHAPE: CrystalShape = CrystalShape {
    radius: 0.17,
    body_height: 0.03,
    tip_height: 0.11,
    tip_cut: 0.06,
    base_tip_height: 0.18,
    sides: CUT_GEM_SIDES,
};
const PURPLE_GEM_SHAPE: CrystalShape = CrystalShape {
    radius: 0.13,
    body_height: 0.025,
    tip_height: 0.09,
    tip_cut: 0.05,
    base_tip_height: 0.15,
    sides: CUT_GEM_SIDES,
};
/// The purple gem floats in the middle of the lower glass planet with its
/// table turned up, left and towards the camera.
const PURPLE_GEM_AXIS: Vec3 = Vec3::new(-0.35, 0.75, 0.55);

/// Floating wood and stone contraption on the left, with a stone ball on a
/// rope.
const FLOATING_BARS: [((f32, f32), (f32, f32)); 5] = [
    ((322.0, 389.0), (385.0, 356.0)),
    ((385.0, 356.0), (420.0, 369.0)),
    ((420.0, 369.0), (415.0, 415.0)),
    ((322.0, 389.0), (309.0, 424.0)),
    ((309.0, 424.0), (349.0, 452.0)),
];
const FLOATING_JOINTS: [(f32, f32); 4] = [
    (322.0, 389.0),
    (385.0, 356.0),
    (420.0, 369.0),
    (309.0, 424.0),
];
const FLOATING_PLATE: (f32, f32) = (358.0, 377.0);
const FLOATING_PLATE_RADIUS: f32 = reference_length(22.0);
const FLOATING_ROPE: [(f32, f32); 5] = [
    (365.0, 392.0),
    (385.0, 440.0),
    (430.0, 447.0),
    (467.0, 470.0),
    (472.0, 512.0),
];
const FLOATING_BALL: (f32, f32) = (475.0, 527.0);

/// Structure hanging under the central glass planet.
const CENTRAL_PLANKS: [((f32, f32), (f32, f32)); 3] = [
    ((650.0, 403.0), (663.0, 440.0)),
    ((682.0, 423.0), (632.0, 490.0)),
    ((684.0, 433.0), (698.0, 450.0)),
];
const CENTRAL_STONE_CUBES: [(f32, f32); 5] = [
    (632.0, 490.0),
    (773.0, 452.0),
    (858.0, 402.0),
    (875.0, 492.0),
    (689.0, 446.0),
];
/// Wooden frames with a stone ball inside: center, size in pixels and
/// rotation in degrees.
const CENTRAL_FRAMED_STONES: [((f32, f32), f32, f32); 3] = [
    ((806.0, 458.0), 40.0, -15.0),
    ((847.0, 431.0), 40.0, -20.0),
    ((870.0, 469.0), 40.0, -20.0),
];
const CENTRAL_DISH: (f32, f32) = (673.0, 468.0);
const CENTRAL_DISH_STICK: ((f32, f32), (f32, f32)) = ((688.0, 447.0), (677.0, 462.0));
const CENTRAL_TNT: (f32, f32) = (770.0, 360.0);
const CENTRAL_TNT_HALF: f32 = reference_length(24.0);

/// Scaffold on top of the lower glass planet.
const LOWER_PLANKS: [((f32, f32), (f32, f32)); 6] = [
    ((1047.0, 757.0), (987.0, 625.0)),
    ((990.0, 627.0), (1072.0, 600.0)),
    ((1020.0, 586.0), (1022.0, 612.0)),
    ((955.0, 650.0), (1000.0, 656.0)),
    ((990.0, 737.0), (1045.0, 748.0)),
    ((1037.0, 715.0), (1040.0, 742.0)),
];
const LOWER_JOINTS: [(f32, f32); 2] = [(1020.0, 580.0), (1077.0, 599.0)];
const LOWER_STONE_BLOCK: (f32, f32) = (975.0, 641.0);
const LOWER_STONE_CUBES: [(f32, f32); 2] = [(1055.0, 731.0), (1072.0, 728.0)];
const LOWER_TOP_ROPE: [(f32, f32); 3] = [(1026.0, 582.0), (1050.0, 594.0), (1072.0, 598.0)];
const LOWER_HANGING_ROPE: [(f32, f32); 2] = [(1079.0, 606.0), (1087.0, 652.0)];
const LOWER_BALL: (f32, f32) = (1090.0, 667.0);

/// Ladder of double planks and a metal rod left of the right planet.
const RIGHT_PLANKS: [((f32, f32), (f32, f32)); 4] = [
    ((1142.0, 338.0), (1075.0, 396.0)),
    ((1149.0, 346.0), (1082.0, 404.0)),
    ((1200.0, 418.0), (1130.0, 463.0)),
    ((1205.0, 427.0), (1135.0, 472.0)),
];
const RIGHT_ROD: ((f32, f32), (f32, f32)) = ((1075.0, 400.0), (1132.0, 468.0));
const RIGHT_ROD_RADIUS: f32 = reference_length(3.0);
const RIGHT_ROPE: [(f32, f32); 3] = [(1187.0, 375.0), (1150.0, 400.0), (1106.0, 436.0)];
const RIGHT_BALL: (f32, f32) = (1195.0, 362.0);
const RIGHT_BALL_RADIUS: f32 = reference_length(17.0);

/// Loose asteroids outside the bubbles: center pixel, radius in pixels and
/// whether it is made of ice.
const LOOSE_ASTEROIDS: [((f32, f32), f32, bool); 3] = [
    ((1025.0, 1050.0), 40.0, false),
    ((1560.0, 500.0), 30.0, true),
    ((225.0, 100.0), 28.0, true),
];

/// Lights, as offsets from the center of the region they light.
const LAUNCH_KEY_OFFSET: Vec3 = Vec3::new(-1.6, 3.2, 3.4);
const FLOATING_KEY_OFFSET: Vec3 = Vec3::new(-1.4, 1.6, 2.6);
const OUTSIDE_FILL_POSITION: Vec3 = Vec3::new(1.5, -1.0, 11.0);
const CENTRAL_KEY_OFFSET: Vec3 = Vec3::new(-1.25, 0.95, 1.20);
const CENTRAL_FILL_OFFSET: Vec3 = Vec3::new(1.00, -1.30, 1.15);
const CENTRAL_GLASS_LIGHT_OFFSET: Vec3 = Vec3::new(-0.35, 0.40, 0.55);
const RIGHT_KEY_OFFSET: Vec3 = Vec3::new(-1.60, 0.20, 1.90);
const RIGHT_FILL_OFFSET: Vec3 = Vec3::new(0.40, -1.90, 1.60);
const LOWER_KEY_OFFSET: Vec3 = Vec3::new(-0.60, 0.26, 1.25);
const LOWER_FILL_OFFSET: Vec3 = Vec3::new(0.95, -0.45, 0.95);
const LOWER_GLASS_LIGHT_OFFSET: Vec3 = Vec3::new(0.12, 0.28, 0.40);

/// Materials used only by level four. They are registered by the level four
/// builder, so the other scenes do not generate its procedural textures.
#[derive(Debug, Clone, Copy)]
struct LevelFourMaterials {
    launch_planet: usize,
    dark_planet: usize,
    crystal_pale: usize,
    crystal_pink: usize,
    crystal_deep: usize,
    glass: usize,
    bubble: usize,
    purple_gem: usize,
    gold_gem: usize,
    rope: usize,
    metal: usize,
    stone: usize,
    space_ice: usize,
}

impl LevelFourMaterials {
    fn crystal(self, shade: CrystalShade) -> usize {
        match shade {
            CrystalShade::Pale => self.crystal_pale,
            CrystalShade::Pink => self.crystal_pink,
            CrystalShade::Deep => self.crystal_deep,
        }
    }
}

/// Object ids and part counts of the level four scene, for tests.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct LevelFourMetadata {
    pub launch_planet: Option<VoxelBodyParts>,
    pub right_planet: Option<VoxelBodyParts>,
    pub central_bubble_id: Option<usize>,
    pub central_glass: Option<VoxelBodyParts>,
    pub right_bubble_id: Option<usize>,
    pub lower_bubble_id: Option<usize>,
    pub lower_glass: Option<VoxelBodyParts>,
    pub crystal_count: usize,
    pub embedded_gem_count: usize,
    pub cut_gem_count: usize,
    pub wood_parts: usize,
    pub stone_parts: usize,
    pub rope_parts: usize,
    pub metal_parts: usize,
    pub slingshot_parts: usize,
    pub asteroid_parts: usize,
    pub ice_chunk_parts: usize,
    pub tnt_parts: usize,
    pub bird_count: usize,
    pub bird_parts: usize,
    pub pig_count: usize,
    pub pig_parts: usize,
}

pub fn build_level_four_scene() -> Result<Scene, SpaceBuildError> {
    build_level_four_scene_with_metadata().map(|built| built.0)
}

pub(crate) fn build_level_four_scene_with_metadata()
-> Result<(Scene, LevelFourMetadata), SpaceBuildError> {
    let (mut scene, materials) = base_space_scene_with_skybox(level_four_skybox()?)?;
    let mut metadata = LevelFourMetadata::default();

    scene.set_ambient_light(Color::new(0.260, 0.215, 0.340));
    let level = register_level_four_materials(&mut scene)?;
    add_launch_planet(&mut scene, &mut metadata, materials, level)?;
    add_central_bubble(&mut scene, &mut metadata, materials, level)?;
    add_right_planet(&mut scene, &mut metadata, materials, level)?;
    add_lower_bubble(&mut scene, &mut metadata, materials, level)?;
    add_floating_contraption(&mut scene, &mut metadata, materials, level)?;
    add_loose_asteroids(&mut scene, &mut metadata, materials, level)?;
    add_level_four_pigs(&mut scene, &mut metadata, materials, level)?;
    add_level_four_lighting(&mut scene);

    scene.build_bvh();

    Ok((scene, metadata))
}

pub fn level_four_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        LEVEL_FOUR_CAMERA_TARGET,
        0.10,
        0.06,
        9.4,
        55.0,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

/// Level four background: deep purple space with lighter violet ribbons,
/// dots and four-pointed sparkles, generated as a seamless quarter-turn
/// tile.
pub fn level_four_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(level_four_skybox_texture()?)
        .with_intensity(1.0)
        .with_horizontal_rotation(0.0)
        .with_tiling(Vec2::new(LEVEL_FOUR_SKYBOX_TILES, 1.0)))
}

fn register_level_four_materials(scene: &mut Scene) -> Result<LevelFourMaterials, SpaceBuildError> {
    let planet_texture = scene.add_texture(crystal_planet_texture(
        CRYSTAL_PLANET_TEXTURE_WIDTH,
        CRYSTAL_PLANET_TEXTURE_HEIGHT,
    )?);
    let planet_material = |tint: Color, emission: Color| {
        Material::new(tint, 0.06, 12.0, 0.0, 0.0, 1.0, emission).with_texture(
            planet_texture,
            Vec2::new(1.0, 1.0),
            WrapMode::Repeat,
        )
    };
    let crystal_material = |albedo: Color, emission: Color| {
        Material::new(albedo, 0.85, 90.0, 0.07, 0.0, 1.0, emission)
    };

    let launch_planet = scene.add_material(planet_material(
        Color::new(0.90, 0.86, 0.90),
        Color::new(0.016, 0.012, 0.020),
    ))?;
    let dark_planet = scene.add_material(planet_material(
        Color::new(0.60, 0.56, 0.62),
        Color::new(0.012, 0.008, 0.016),
    ))?;
    let crystal_pale = scene.add_material(crystal_material(
        Color::new(0.96, 0.68, 0.94),
        Color::new(0.110, 0.055, 0.110),
    ))?;
    let crystal_pink = scene.add_material(crystal_material(
        Color::new(0.94, 0.52, 0.88),
        Color::new(0.110, 0.035, 0.105),
    ))?;
    let crystal_deep = scene.add_material(crystal_material(
        Color::new(0.76, 0.32, 0.74),
        Color::new(0.080, 0.018, 0.085),
    ))?;
    // Pink glass cubes of the glass planets: clearer and less refractive
    // than a glass ball, because every ray crosses two faces of each cube,
    // so what is inside still shows through and bends a little.
    let glass = scene.add_material(Material::new(
        Color::new(0.94, 0.80, 0.98),
        1.0,
        180.0,
        0.05,
        0.88,
        1.10,
        Color::new(0.070, 0.035, 0.080),
    ))?;
    // Light blue gravity bubbles, without refraction so nothing inside is
    // distorted.
    let bubble = scene.add_material(Material::new(
        Color::new(0.62, 0.86, 1.0),
        0.30,
        50.0,
        0.0,
        0.93,
        1.0,
        Color::new(0.030, 0.065, 0.105),
    ))?;
    let purple_gem = scene.add_material(Material::new(
        Color::new(0.68, 0.56, 1.0),
        1.0,
        140.0,
        0.18,
        0.0,
        1.0,
        Color::new(0.220, 0.150, 0.430),
    ))?;
    let gold_gem = scene.add_material(Material::new(
        Color::new(1.0, 0.80, 0.24),
        1.0,
        120.0,
        0.15,
        0.0,
        1.0,
        Color::new(0.480, 0.330, 0.050),
    ))?;
    let rope = scene.add_material(Material::new(
        Color::new(0.52, 0.33, 0.16),
        0.20,
        12.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.030, 0.016, 0.006),
    ))?;
    let metal = scene.add_material(Material::new(
        Color::new(0.80, 0.82, 0.86),
        0.90,
        90.0,
        0.22,
        0.0,
        1.0,
        Color::new(0.040, 0.040, 0.045),
    ))?;
    let stone = scene.add_material(Material::new(
        Color::new(0.60, 0.60, 0.62),
        0.25,
        26.0,
        0.02,
        0.0,
        1.0,
        Color::new(0.030, 0.028, 0.032),
    ))?;
    let space_ice = scene.add_material(Material::new(
        Color::new(0.80, 0.78, 1.0),
        0.80,
        70.0,
        0.06,
        0.35,
        1.20,
        Color::new(0.060, 0.055, 0.110),
    ))?;

    Ok(LevelFourMaterials {
        launch_planet,
        dark_planet,
        crystal_pale,
        crystal_pink,
        crystal_deep,
        glass,
        bubble,
        purple_gem,
        gold_gem,
        rope,
        metal,
        stone,
        space_ice,
    })
}

fn add_launch_planet(
    scene: &mut Scene,
    metadata: &mut LevelFourMetadata,
    materials: SpaceMaterials,
    level: LevelFourMaterials,
) -> Result<(), SpaceBuildError> {
    let center = LEVEL_FOUR_LAUNCH_PLANET_CENTER;
    let body = launch_planet_body(level.launch_planet)?;

    metadata.launch_planet = Some(voxel::add_voxel_body(scene, &body)?);
    metadata.crystal_count += add_crystals(scene, &body, &LAUNCH_PLANET_CRYSTALS, level)?;
    metadata.embedded_gem_count +=
        add_embedded_gems(scene, &body, &LAUNCH_PLANET_GEMS, level.crystal_pale)?;

    // The big diamond grows out of the planet with its table up, and the
    // slingshot stands on the table.
    let diamond_direction = surface_direction(LAUNCH_DIAMOND_ANGLE, LAUNCH_DIAMOND_DEPTH);
    let diamond = Crystal::new(
        center
            + diamond_direction
                * (body.surface_distance(center, diamond_direction) - LAUNCH_DIAMOND_EMBED
                    + LAUNCH_DIAMOND_SHAPE.base_tip_height),
        LAUNCH_DIAMOND_SHAPE,
        axis_basis(
            surface_direction(LAUNCH_DIAMOND_AXIS_ANGLE, LAUNCH_DIAMOND_DEPTH),
            0.2,
        )?,
        level.crystal_pink,
    )?;
    scene.add_crystal(diamond)?;
    metadata.cut_gem_count += 1;

    let table_normal = diamond.orientation().up();
    let table_frame = RadialFrame::from_normal(diamond.apex() - table_normal, 1.0, table_normal)?;
    let parts = add_slingshot(
        scene,
        table_frame,
        LEVEL_FOUR_SLINGSHOT_YAW_RADIANS,
        LEVEL_FOUR_SLINGSHOT_SCALE,
        materials,
    )?;
    metadata.slingshot_parts += parts.total();

    // One bird loaded in the slingshot and two waiting on the planet.
    let waiting = LEVEL_FOUR_WAITING_BIRDS.map(|(angle, depth)| surface_direction(angle, depth));
    let bird_materials = birds::register_bird_materials(scene, materials)?;
    metadata.bird_parts += birds::add_slingshot_birds(
        scene,
        birds::SlingshotBirds {
            frame: table_frame,
            yaw_radians: LEVEL_FOUR_SLINGSHOT_YAW_RADIANS,
            scale: LEVEL_FOUR_SLINGSHOT_SCALE,
            aim: LEVEL_FOUR_BIRD_AIM,
            ground: &body,
            waiting_directions: &waiting,
        },
        bird_materials,
    )?;
    metadata.bird_count += birds::BIRD_COUNT;

    Ok(())
}

fn add_central_bubble(
    scene: &mut Scene,
    metadata: &mut LevelFourMetadata,
    materials: SpaceMaterials,
    level: LevelFourMaterials,
) -> Result<(), SpaceBuildError> {
    let glass = central_glass_body(level.glass)?;

    metadata.central_glass = Some(voxel::add_voxel_body(scene, &glass)?);
    metadata.central_bubble_id = Some(scene.object_count());
    scene.add_sphere(Sphere::new(
        LEVEL_FOUR_CENTRAL_BUBBLE_CENTER,
        LEVEL_FOUR_CENTRAL_BUBBLE_RADIUS,
        level.bubble,
    )?)?;

    metadata.crystal_count += add_crystals(scene, &glass, &CENTRAL_GLASS_CRYSTALS, level)?;
    metadata.embedded_gem_count +=
        add_embedded_gems(scene, &glass, &CENTRAL_GLASS_GEMS, level.crystal_pale)?;

    // TNT crate inside the glass planet, a little turned.
    let tnt_orientation = Basis3::from_axis_angle(Vec3::new(0.25, 0.45, 1.0), -0.42)?;
    scene.add_oriented_box(OrientedBox::new(
        pixel(CENTRAL_TNT),
        Vec3::new(CENTRAL_TNT_HALF, CENTRAL_TNT_HALF, CENTRAL_TNT_HALF),
        tnt_orientation,
        materials.tnt_crate,
    )?)?;
    metadata.tnt_parts += 1;

    for (start, end) in CENTRAL_PLANKS {
        add_plank(scene, pixel(start), pixel(end), materials.wood)?;
        metadata.wood_parts += 1;
    }
    for center in CENTRAL_STONE_CUBES {
        add_stone_cube(scene, pixel(center), SMALL_STONE_HALF, level.stone)?;
        metadata.stone_parts += 1;
    }
    for (center, size, angle) in CENTRAL_FRAMED_STONES {
        let (wood, stone) = add_framed_stone(
            scene,
            pixel(center),
            reference_length(size),
            angle.to_radians(),
            materials.wood,
            level.stone,
        )?;
        metadata.wood_parts += wood;
        metadata.stone_parts += stone;
    }

    // Small metal dish on a stick, facing down and left.
    let (stick_start, stick_end) = CENTRAL_DISH_STICK;
    add_segment(
        scene,
        pixel(stick_start),
        pixel(stick_end),
        reference_length(2.5),
        level.metal,
    )?;
    scene.add_cone(Cone::new(
        pixel(CENTRAL_DISH),
        reference_length(13.0),
        reference_length(4.0),
        basis_with_up(Vec3::new(0.62, 0.78, 0.10))?,
        level.metal,
    )?)?;
    metadata.metal_parts += 2;

    Ok(())
}

fn add_right_planet(
    scene: &mut Scene,
    metadata: &mut LevelFourMetadata,
    materials: SpaceMaterials,
    level: LevelFourMaterials,
) -> Result<(), SpaceBuildError> {
    let center = LEVEL_FOUR_RIGHT_PLANET_CENTER;
    let body = right_planet_body(level.dark_planet)?;

    metadata.right_planet = Some(voxel::add_voxel_body(scene, &body)?);
    metadata.right_bubble_id = Some(scene.object_count());
    scene.add_sphere(Sphere::new(
        LEVEL_FOUR_RIGHT_BUBBLE_CENTER,
        LEVEL_FOUR_RIGHT_BUBBLE_RADIUS,
        level.bubble,
    )?)?;

    metadata.crystal_count += add_crystals(scene, &body, &RIGHT_PLANET_CRYSTALS, level)?;
    metadata.embedded_gem_count +=
        add_embedded_gems(scene, &body, &RIGHT_PLANET_GEMS, level.crystal_pale)?;

    let gold_direction =
        surface_direction(RIGHT_PLANET_GOLD_GEM_ANGLE, RIGHT_PLANET_GOLD_GEM_DEPTH);
    scene.add_crystal(Crystal::new(
        center
            + gold_direction
                * (body.surface_distance(center, gold_direction)
                    + GOLD_GEM_SHAPE.base_tip_height * 0.55),
        GOLD_GEM_SHAPE,
        axis_basis(gold_direction, 0.3)?,
        level.gold_gem,
    )?)?;
    metadata.cut_gem_count += 1;

    for (start, end) in RIGHT_PLANKS {
        add_plank(scene, pixel(start), pixel(end), materials.wood)?;
        metadata.wood_parts += 1;
    }
    let (rod_start, rod_end) = RIGHT_ROD;
    add_segment(
        scene,
        pixel(rod_start),
        pixel(rod_end),
        RIGHT_ROD_RADIUS,
        level.metal,
    )?;
    metadata.metal_parts += 1;
    metadata.rope_parts += add_rope(scene, &RIGHT_ROPE, level.rope)?;
    scene.add_sphere(Sphere::new(
        pixel(RIGHT_BALL),
        RIGHT_BALL_RADIUS,
        level.stone,
    )?)?;
    metadata.stone_parts += 1;

    Ok(())
}

fn add_lower_bubble(
    scene: &mut Scene,
    metadata: &mut LevelFourMetadata,
    materials: SpaceMaterials,
    level: LevelFourMaterials,
) -> Result<(), SpaceBuildError> {
    let glass_center = LEVEL_FOUR_LOWER_GLASS_CENTER;
    let glass = lower_glass_body(level.glass)?;

    metadata.lower_glass = Some(voxel::add_voxel_body(scene, &glass)?);
    metadata.lower_bubble_id = Some(scene.object_count());
    scene.add_sphere(Sphere::new(
        LEVEL_FOUR_LOWER_BUBBLE_CENTER,
        LEVEL_FOUR_LOWER_BUBBLE_RADIUS,
        level.bubble,
    )?)?;

    metadata.crystal_count += add_crystals(scene, &glass, &LOWER_GLASS_CRYSTALS, level)?;
    metadata.embedded_gem_count +=
        add_embedded_gems(scene, &glass, &LOWER_GLASS_GEMS, level.crystal_pale)?;

    // Purple cut gem floating in the middle of the glass planet.
    let gem_axis = PURPLE_GEM_AXIS.normalized();
    let gem_middle = (PURPLE_GEM_SHAPE.body_height + PURPLE_GEM_SHAPE.tip_height
        - PURPLE_GEM_SHAPE.tip_cut
        - PURPLE_GEM_SHAPE.base_tip_height)
        * 0.5;
    scene.add_crystal(Crystal::new(
        glass_center - gem_axis * gem_middle,
        PURPLE_GEM_SHAPE,
        axis_basis(gem_axis, 0.4)?,
        level.purple_gem,
    )?)?;
    metadata.cut_gem_count += 1;

    for (start, end) in LOWER_PLANKS {
        add_plank(scene, pixel(start), pixel(end), materials.wood)?;
        metadata.wood_parts += 1;
    }
    for center in LOWER_JOINTS {
        scene.add_sphere(Sphere::new(pixel(center), JOINT_RADIUS, level.stone)?)?;
        metadata.stone_parts += 1;
    }
    for center in LOWER_STONE_CUBES {
        add_stone_cube(scene, pixel(center), SMALL_STONE_HALF, level.stone)?;
        metadata.stone_parts += 1;
    }
    scene.add_oriented_box(OrientedBox::new(
        pixel(LOWER_STONE_BLOCK),
        Vec3::new(reference_length(11.0), reference_length(10.0), 0.14),
        Basis3::identity(),
        level.stone,
    )?)?;
    metadata.stone_parts += 1;

    metadata.rope_parts += add_rope(scene, &LOWER_TOP_ROPE, level.rope)?;
    metadata.rope_parts += add_rope(scene, &LOWER_HANGING_ROPE, level.rope)?;
    scene.add_sphere(Sphere::new(
        pixel(LOWER_BALL),
        HANGING_BALL_RADIUS,
        level.stone,
    )?)?;
    metadata.stone_parts += 1;

    Ok(())
}

fn add_floating_contraption(
    scene: &mut Scene,
    metadata: &mut LevelFourMetadata,
    materials: SpaceMaterials,
    level: LevelFourMaterials,
) -> Result<(), SpaceBuildError> {
    for (start, end) in FLOATING_BARS {
        add_plank(scene, pixel(start), pixel(end), materials.wood)?;
        metadata.wood_parts += 1;
    }
    for center in FLOATING_JOINTS {
        scene.add_sphere(Sphere::new(pixel(center), JOINT_RADIUS, level.stone)?)?;
        metadata.stone_parts += 1;
    }

    // Triangular stone plate with a corner pointing down: a three-sided
    // prism along the Z axis.
    let plate_half_depth = 0.08;
    let plate_basis = Basis3::new(
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(-1.0, 0.0, 0.0),
    )?;
    scene.add_crystal(Crystal::new(
        pixel(FLOATING_PLATE) - Vec3::new(0.0, 0.0, plate_half_depth),
        CrystalShape {
            radius: FLOATING_PLATE_RADIUS,
            body_height: plate_half_depth * 2.0,
            tip_height: 0.0,
            tip_cut: 0.0,
            base_tip_height: 0.0,
            sides: 3,
        },
        plate_basis,
        level.stone,
    )?)?;
    metadata.stone_parts += 1;

    metadata.rope_parts += add_rope(scene, &FLOATING_ROPE, level.rope)?;
    scene.add_sphere(Sphere::new(
        pixel(FLOATING_BALL),
        HANGING_BALL_RADIUS,
        level.stone,
    )?)?;
    metadata.stone_parts += 1;

    Ok(())
}

fn add_loose_asteroids(
    scene: &mut Scene,
    metadata: &mut LevelFourMetadata,
    materials: SpaceMaterials,
    level: LevelFourMaterials,
) -> Result<(), SpaceBuildError> {
    for (center, radius_pixels, icy) in LOOSE_ASTEROIDS {
        let center = pixel(center);
        let radius = reference_length(radius_pixels);

        // Rocks and chunks of space ice are lumpy balls of cubes.
        let (material_id, parts) = if icy {
            (level.space_ice, &mut metadata.ice_chunk_parts)
        } else {
            (materials.level_three_asteroid, &mut metadata.asteroid_parts)
        };
        *parts += add_voxel_rock(
            scene,
            center,
            radius,
            ROCK_LUMP_OFFSET,
            material_id,
            LEVEL_FOUR_CUBE_EDGE,
        )?
        .total();
    }

    Ok(())
}

/// The pigs of level four, standing on the top of the cubes of their planet
/// or on the floating contraption.
fn add_level_four_pigs(
    scene: &mut Scene,
    metadata: &mut LevelFourMetadata,
    materials: SpaceMaterials,
    level: LevelFourMaterials,
) -> Result<(), SpaceBuildError> {
    let first = scene.object_count();

    for (index, (center, up, radius)) in level_four_pig_placements(level)?.into_iter().enumerate() {
        let look = PIG_LOOK - up * PIG_LOOK.dot(up);
        let forward = look.normalized();
        let basis = Basis3::new(up.cross(forward), up, forward)?;
        add_space_pig(scene, center, radius, basis, materials)?;
        metadata.pig_count = index + 1;
    }
    metadata.pig_parts = scene.object_count() - first;

    Ok(())
}

/// Center, up direction and radius of every pig.
fn level_four_pig_placements(
    level: LevelFourMaterials,
) -> Result<[(Vec3, Vec3, f32); LEVEL_FOUR_PIG_COUNT], SpaceBuildError> {
    let central = central_glass_body(level.glass)?;
    let right = right_planet_body(level.dark_planet)?;
    let lower = lower_glass_body(level.glass)?;
    let mut placements = [(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), 0.0); LEVEL_FOUR_PIG_COUNT];

    for (placement, (spot, radius)) in placements.iter_mut().zip(LEVEL_FOUR_PIGS) {
        *placement = match spot {
            PigSpot::Planet(planet, angle, depth) => {
                let body = match planet {
                    PigPlanet::CentralGlass => &central,
                    PigPlanet::Right => &right,
                    PigPlanet::LowerGlass => &lower,
                };
                let up = surface_direction(angle, depth);
                let ground = body.surface_distance(body.origin(), up);
                (
                    body.origin() + up * (ground + radius * (1.0 - PIG_GROUND_SINK)),
                    up,
                    radius,
                )
            }
            PigSpot::Floating(center, up) => (pixel(center), up.normalized(), radius),
        };
    }

    Ok(placements)
}

/// Bottom left planet with the slingshot, made of cubes.
fn launch_planet_body(material_id: usize) -> Result<VoxelBody, SpaceBuildError> {
    VoxelBody::ball(
        VoxelBall::new(
            LEVEL_FOUR_LAUNCH_PLANET_CENTER,
            LEVEL_FOUR_LAUNCH_PLANET_RADIUS,
            material_id,
        ),
        LEVEL_FOUR_CUBE_EDGE,
        LEVEL_FOUR_MIN_CUBES_ACROSS,
    )
}

/// Dark planet on the upper right, made of cubes.
fn right_planet_body(material_id: usize) -> Result<VoxelBody, SpaceBuildError> {
    VoxelBody::ball(
        VoxelBall::new(
            LEVEL_FOUR_RIGHT_PLANET_CENTER,
            LEVEL_FOUR_RIGHT_PLANET_RADIUS,
            material_id,
        ),
        LEVEL_FOUR_CUBE_EDGE,
        LEVEL_FOUR_MIN_CUBES_ACROSS,
    )
}

/// Glass planet inside the central bubble: a hollow shell of glass cubes.
fn central_glass_body(material_id: usize) -> Result<VoxelBody, SpaceBuildError> {
    VoxelBody::ball(
        VoxelBall::new(
            LEVEL_FOUR_CENTRAL_GLASS_CENTER,
            LEVEL_FOUR_CENTRAL_GLASS_RADIUS,
            material_id,
        ),
        LEVEL_FOUR_GLASS_CUBE_EDGE,
        LEVEL_FOUR_MIN_CUBES_ACROSS,
    )
}

/// Glass planet inside the lower bubble: a hollow shell of glass cubes.
fn lower_glass_body(material_id: usize) -> Result<VoxelBody, SpaceBuildError> {
    VoxelBody::ball(
        VoxelBall::new(
            LEVEL_FOUR_LOWER_GLASS_CENTER,
            LEVEL_FOUR_LOWER_GLASS_RADIUS,
            material_id,
        ),
        LEVEL_FOUR_GLASS_CUBE_EDGE,
        LEVEL_FOUR_MIN_CUBES_ACROSS,
    )
}

fn add_level_four_lighting(scene: &mut Scene) {
    let warm = Color::new(1.0, 0.94, 0.90);
    let lilac = Color::new(0.86, 0.78, 1.0);
    let floating_center = pixel(FLOATING_PLATE);

    for (position, color, intensity) in [
        // Outside the bubbles.
        (
            LEVEL_FOUR_LAUNCH_PLANET_CENTER + LAUNCH_KEY_OFFSET,
            warm,
            11.0,
        ),
        (floating_center + FLOATING_KEY_OFFSET, warm, 9.0),
        (OUTSIDE_FILL_POSITION, lilac, 55.0),
        // Inside the central bubble, around the glass planet.
        (
            LEVEL_FOUR_CENTRAL_GLASS_CENTER + CENTRAL_KEY_OFFSET,
            warm,
            4.5,
        ),
        (
            LEVEL_FOUR_CENTRAL_GLASS_CENTER + CENTRAL_FILL_OFFSET,
            lilac,
            2.6,
        ),
        // Inside the central glass planet, for the TNT crate.
        (
            LEVEL_FOUR_CENTRAL_GLASS_CENTER + CENTRAL_GLASS_LIGHT_OFFSET,
            warm,
            1.3,
        ),
        // Inside the right bubble.
        (LEVEL_FOUR_RIGHT_PLANET_CENTER + RIGHT_KEY_OFFSET, warm, 2.0),
        (
            LEVEL_FOUR_RIGHT_PLANET_CENTER + RIGHT_FILL_OFFSET,
            lilac,
            3.2,
        ),
        // Inside the lower bubble and its glass planet.
        (LEVEL_FOUR_LOWER_GLASS_CENTER + LOWER_KEY_OFFSET, warm, 2.0),
        (
            LEVEL_FOUR_LOWER_GLASS_CENTER + LOWER_FILL_OFFSET,
            lilac,
            1.6,
        ),
        (
            LEVEL_FOUR_LOWER_GLASS_CENTER + LOWER_GLASS_LIGHT_OFFSET,
            warm,
            0.7,
        ),
    ] {
        scene.add_light(PointLight::new(position, color, intensity));
    }
}

/// Unit vector `angle_degrees` clockwise from +Y in the XY plane, tilted
/// `depth_degrees` towards +Z.
fn surface_direction(angle_degrees: f32, depth_degrees: f32) -> Vec3 {
    let (sin_angle, cos_angle) = angle_degrees.to_radians().sin_cos();
    let (sin_depth, cos_depth) = depth_degrees.to_radians().sin_cos();

    Vec3::new(sin_angle * cos_depth, cos_angle * cos_depth, sin_depth)
}

/// Basis with `up` along `axis`, turned `twist_radians` around it.
fn axis_basis(axis: Vec3, twist_radians: f32) -> Result<Basis3, SpaceBuildError> {
    let basis = basis_with_up(axis)?;
    let (sin_twist, cos_twist) = twist_radians.sin_cos();

    Ok(Basis3::new(
        basis.right() * cos_twist + basis.forward() * sin_twist,
        basis.up(),
        basis.forward() * cos_twist - basis.right() * sin_twist,
    )?)
}

fn pixel((x, y): (f32, f32)) -> Vec3 {
    reference_point(x, y)
}

/// Adds crystals growing out of a planet of cubes, sunk under the top of
/// its cubes. Returns how many were added.
fn add_crystals(
    scene: &mut Scene,
    body: &VoxelBody,
    specs: &[CrystalSpec],
    level: LevelFourMaterials,
) -> Result<usize, SpaceBuildError> {
    let center = body.origin();

    for (index, spec) in specs.iter().enumerate() {
        let normal = surface_direction(spec.angle, spec.depth);
        let axis = surface_direction(spec.angle + spec.lean, spec.depth);
        let embed = spec.radius * CRYSTAL_EMBED_RADII;
        let total = spec.length + embed;
        // Golden-angle twist, so neighbouring crystals show different facets.
        let twist = index as f32 * 2.399;

        scene.add_crystal(Crystal::new(
            center + normal * (body.surface_distance(center, normal) - embed),
            CrystalShape {
                radius: spec.radius,
                body_height: total * (1.0 - CRYSTAL_TIP_FRACTION),
                tip_height: total * CRYSTAL_TIP_FRACTION,
                tip_cut: 0.0,
                base_tip_height: 0.0,
                sides: CRYSTAL_SIDES,
            },
            axis_basis(axis, twist)?,
            level.crystal(spec.shade),
        )?)?;
    }

    Ok(specs.len())
}

/// Adds octahedral gems half sunk in the top of the cubes of a planet.
/// Returns how many were added.
fn add_embedded_gems(
    scene: &mut Scene,
    body: &VoxelBody,
    gems: &[GemSpec],
    material_id: usize,
) -> Result<usize, SpaceBuildError> {
    let center = body.origin();

    for (index, &(angle, depth, size)) in gems.iter().enumerate() {
        let normal = surface_direction(angle, depth);

        scene.add_crystal(Crystal::new(
            center + normal * (body.surface_distance(center, normal) - size * 0.15),
            CrystalShape {
                radius: size,
                body_height: 0.0,
                tip_height: size * 0.95,
                tip_cut: 0.0,
                base_tip_height: size * 0.95,
                sides: EMBEDDED_GEM_SIDES,
            },
            axis_basis(normal, 0.5 + index as f32 * 1.3)?,
            material_id,
        )?)?;
    }

    Ok(gems.len())
}

/// Wooden plank between two points of the level plane.
fn add_plank(
    scene: &mut Scene,
    start: Vec3,
    end: Vec3,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    let along = end - start;

    scene.add_oriented_box(OrientedBox::new(
        (start + end) * 0.5,
        Vec3::new(
            along.length() * 0.5,
            PLANK_THICKNESS * 0.5,
            PLANK_DEPTH * 0.5,
        ),
        plane_basis(along.y.atan2(along.x))?,
        material_id,
    )?)?;

    Ok(())
}

/// Basis turned `angle_radians` counterclockwise around +Z.
fn plane_basis(angle_radians: f32) -> Result<Basis3, SpaceBuildError> {
    let (sin_angle, cos_angle) = angle_radians.sin_cos();

    Ok(Basis3::new(
        Vec3::new(cos_angle, sin_angle, 0.0),
        Vec3::new(-sin_angle, cos_angle, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    )?)
}

fn add_stone_cube(
    scene: &mut Scene,
    center: Vec3,
    half: f32,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    scene.add_oriented_box(OrientedBox::new(
        center,
        Vec3::new(half, half, half),
        plane_basis(0.4)?,
        material_id,
    )?)?;

    Ok(())
}

/// Square wooden frame with a stone ball inside, turned `angle_radians`
/// counterclockwise. Returns the number of wood and stone parts.
fn add_framed_stone(
    scene: &mut Scene,
    center: Vec3,
    size: f32,
    angle_radians: f32,
    wood: usize,
    stone: usize,
) -> Result<(usize, usize), SpaceBuildError> {
    let basis = plane_basis(angle_radians)?;
    let half = size * 0.5;
    let bar = PLANK_THICKNESS * 0.9;
    let depth = PLANK_DEPTH * 0.5;

    for (offset, half_extents) in [
        (
            Vec3::new(0.0, half - bar * 0.5, 0.0),
            Vec3::new(half, bar * 0.5, depth),
        ),
        (
            Vec3::new(0.0, -half + bar * 0.5, 0.0),
            Vec3::new(half, bar * 0.5, depth),
        ),
        (
            Vec3::new(half - bar * 0.5, 0.0, 0.0),
            Vec3::new(bar * 0.5, half - bar, depth * 0.96),
        ),
        (
            Vec3::new(-half + bar * 0.5, 0.0, 0.0),
            Vec3::new(bar * 0.5, half - bar, depth * 0.96),
        ),
    ] {
        scene.add_oriented_box(OrientedBox::new(
            center + basis.local_to_world_vector(offset),
            half_extents,
            basis,
            wood,
        )?)?;
    }
    scene.add_sphere(Sphere::new(center, half - bar * 1.1, stone)?)?;

    Ok((4, 1))
}

/// Cylinder between two world points.
fn add_segment(
    scene: &mut Scene,
    start: Vec3,
    end: Vec3,
    radius: f32,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    let axis = end - start;

    scene.add_cylinder(Cylinder::new(
        (start + end) * 0.5,
        radius,
        axis.length() * 0.5,
        basis_with_up(axis)?,
        material_id,
    )?)?;

    Ok(())
}

/// Rope through the given reference pixels: a cylinder per stretch and a
/// small sphere on each bend. Returns the number of parts.
fn add_rope(
    scene: &mut Scene,
    points: &[(f32, f32)],
    material_id: usize,
) -> Result<usize, SpaceBuildError> {
    let mut parts = 0;

    for pair in points.windows(2) {
        add_segment(
            scene,
            pixel(pair[0]),
            pixel(pair[1]),
            ROPE_RADIUS,
            material_id,
        )?;
        parts += 1;
    }
    for &bend in points.iter().skip(1).take(points.len().saturating_sub(2)) {
        scene.add_sphere(Sphere::new(pixel(bend), ROPE_RADIUS, material_id)?)?;
        parts += 1;
    }

    Ok(parts)
}

/// Swirled gray planet texture in sphere UVs: rings around
/// `CRYSTAL_PLANET_SWIRL_POLE`, bent by noise, like the planets of the
/// original crystal levels. The selector uses a smaller copy.
pub(super) fn crystal_planet_texture(
    width: usize,
    height: usize,
) -> Result<Texture, SpaceBuildError> {
    let mut pixels = Vec::with_capacity(width * height);

    for y in 0..height {
        let v = 1.0 - y as f32 / (height.max(2) - 1) as f32;

        for x in 0..width {
            let u = x as f32 / (width.max(2) - 1) as f32;
            pixels.push(crystal_planet_color(sphere_direction_from_uv(u, v)));
        }
    }

    Ok(Texture::new(width, height, pixels)?)
}

fn crystal_planet_color(direction: Vec3) -> Color {
    let light = Color::new(0.47, 0.42, 0.44);
    let dark = Color::new(0.32, 0.27, 0.30);
    let pole = CRYSTAL_PLANET_SWIRL_POLE.normalized();
    let ring_angle = direction.dot(pole).clamp(-1.0, 1.0).acos();
    let warp = value_noise_3d(direction * 2.3 + Vec3::new(3.1, 1.7, 5.3)) * 2.0
        + value_noise_3d(direction * 5.1 + Vec3::new(7.2, 2.9, 0.4)) * 0.8;
    let rings = (ring_angle * CRYSTAL_PLANET_RING_FREQUENCY + warp * 2.4).sin();
    let band = smoothstep(0.18, 0.34, rings);
    let grain = value_noise_3d(direction * 9.0) * 0.05;

    light.lerp(dark, band) * (0.97 + grain)
}

fn level_four_skybox_texture() -> Result<Texture, TextureError> {
    let width = LEVEL_FOUR_SKYBOX_TILE_WIDTH;
    let height = LEVEL_FOUR_SKYBOX_TILE_HEIGHT;
    // The ribbons only depend on `u`, so their heights are found once per
    // column.
    let columns: Vec<_> = (0..width)
        .map(|x| sky_ribbon_heights(x as f32 / width as f32))
        .collect();
    let mut pixels = Vec::with_capacity(width * height);

    for y in 0..height {
        let v = 1.0 - y as f32 / (height - 1) as f32;

        for (x, ribbons) in columns.iter().enumerate() {
            pixels.push(sky_color(x as f32 / width as f32, v, ribbons));
        }
    }

    Texture::new(width, height, pixels)
}

/// Color of the level four background tile. `u` goes once around a quarter
/// turn (it wraps) and `v` from the bottom (0) to the top (1) of the sky.
#[cfg(test)]
fn level_four_skybox_color(u: f32, v: f32) -> Color {
    sky_color(u, v, &sky_ribbon_heights(u))
}

fn sky_ribbon_heights(u: f32) -> [f32; 3] {
    LEVEL_FOUR_SKY_RIBBONS.map(|(height, amplitude, phase, _, _)| {
        height
            + (std::f32::consts::TAU * (u + phase)).sin() * amplitude
            + (std::f32::consts::TAU * (u * 2.0 + phase * 3.0)).sin() * amplitude * 0.35
    })
}

/// Shapes are measured in degrees, so they look round on the sky.
fn sky_color(u: f32, v: f32, ribbon_heights: &[f32; 3]) -> Color {
    let u = u.rem_euclid(1.0);
    let bottom = Color::new(0.040, 0.040, 0.170);
    let middle = Color::new(0.150, 0.085, 0.360);
    let top = Color::new(0.330, 0.130, 0.480);
    let mut color = if v < 0.5 {
        bottom.lerp(middle, smoothstep(0.30, 0.50, v))
    } else {
        middle.lerp(top, smoothstep(0.50, 0.68, v))
    };

    for (&center, (_, _, _, width, strength)) in ribbon_heights.iter().zip(LEVEL_FOUR_SKY_RIBBONS) {
        let distance = (v - center).abs();
        if distance > width * 1.2 {
            continue;
        }
        let ribbon = 1.0 - smoothstep(width * 0.55, width, distance);
        let edge = (1.0 - smoothstep(0.0, 0.004, (distance - width * 0.72).abs())) * 0.35;

        color = color.lerp(Color::new(0.50, 0.30, 0.74), ribbon * strength);
        color += Color::new(0.20, 0.12, 0.26) * (edge * strength);
    }

    color += Color::new(0.92, 0.90, 1.0) * (sky_dots(u, v) + sky_sparkles(u, v));

    color.clamped()
}

/// Small round stars on a grid that wraps horizontally.
fn sky_dots(u: f32, v: f32) -> f32 {
    const CELLS_U: f32 = 90.0;
    const CELLS_V: f32 = 90.0;
    let grid_u = u * CELLS_U;
    let grid_v = v * CELLS_V;
    let cell_u = grid_u.floor().rem_euclid(CELLS_U);
    let cell_v = grid_v.floor();
    let chance = smooth_hash(cell_u + 3.0, cell_v + 11.0);

    if chance < 0.90 {
        return 0.0;
    }

    let center_u = 0.25 + smooth_hash(cell_u + 7.0, cell_v + 2.0) * 0.5;
    let center_v = 0.25 + smooth_hash(cell_u + 1.0, cell_v + 9.0) * 0.5;
    // One cell is 1 degree across and 2 degrees tall.
    let du = grid_u.fract() - center_u;
    let dv = (grid_v.fract() - center_v) * 2.0;
    let distance = (du * du + dv * dv).sqrt();
    let radius = 0.10 + smooth_hash(cell_u + 5.0, cell_v + 4.0) * 0.08;

    (1.0 - smoothstep(radius * 0.4, radius, distance)) * (0.45 + (chance - 0.90) * 5.0)
}

/// Four-pointed sparkles on a coarser grid that wraps horizontally.
fn sky_sparkles(u: f32, v: f32) -> f32 {
    const CELLS_U: f32 = 12.0;
    const CELLS_V: f32 = 20.0;
    let grid_u = u * CELLS_U;
    let grid_v = v * CELLS_V;
    let cell_u = grid_u.floor().rem_euclid(CELLS_U);
    let cell_v = grid_v.floor();
    let chance = smooth_hash(cell_u + 41.0, cell_v + 17.0);

    if chance < 0.55 {
        return 0.0;
    }

    // A cell is 7.5 by 9 degrees: measure in degrees so the arms are equal.
    let du = (grid_u.fract() - 0.5 - (smooth_hash(cell_u + 13.0, cell_v) - 0.5) * 0.4) * 7.5;
    let dv = (grid_v.fract() - 0.5 - (smooth_hash(cell_u, cell_v + 29.0) - 0.5) * 0.4) * 9.0;
    let size = 0.5 + smooth_hash(cell_u + 23.0, cell_v + 5.0) * 1.1;
    let arm = |along: f32, across: f32| {
        (1.0 - smoothstep(0.0, size, along.abs()))
            * (1.0
                - smoothstep(
                    0.0,
                    size * 0.10 * (1.0 - along.abs() / size).max(0.2),
                    across.abs(),
                ))
    };
    let core = 1.0 - smoothstep(0.0, size * 0.22, (du * du + dv * dv).sqrt());

    (arm(du, dv).max(arm(dv, du)) + core * 0.6).min(1.0) * (0.55 + (chance - 0.55) * 1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        CENTRAL_GLASS_LIGHT_OFFSET, CENTRAL_TNT, CENTRAL_TNT_HALF, LAUNCH_PLANET_GEMS,
        LEVEL_FOUR_CENTRAL_BUBBLE_CENTER, LEVEL_FOUR_CENTRAL_BUBBLE_RADIUS,
        LEVEL_FOUR_CENTRAL_GLASS_CENTER, LEVEL_FOUR_CENTRAL_GLASS_RADIUS, LEVEL_FOUR_CUBE_EDGE,
        LEVEL_FOUR_GLASS_CUBE_EDGE, LEVEL_FOUR_LAUNCH_PLANET_CENTER,
        LEVEL_FOUR_LAUNCH_PLANET_RADIUS, LEVEL_FOUR_LOWER_BUBBLE_CENTER,
        LEVEL_FOUR_LOWER_BUBBLE_RADIUS, LEVEL_FOUR_LOWER_GLASS_CENTER,
        LEVEL_FOUR_LOWER_GLASS_RADIUS, LEVEL_FOUR_MIN_CUBES_ACROSS, LEVEL_FOUR_PIG_COUNT,
        LEVEL_FOUR_PIGS, LEVEL_FOUR_RIGHT_BUBBLE_CENTER, LEVEL_FOUR_RIGHT_BUBBLE_RADIUS,
        LEVEL_FOUR_RIGHT_PLANET_CENTER, LEVEL_FOUR_RIGHT_PLANET_RADIUS, LOOSE_ASTEROIDS,
        LOWER_GLASS_LIGHT_OFFSET, PigPlanet, PigSpot, build_level_four_scene_with_metadata,
        central_glass_body, launch_planet_body, level_four_orbit_camera, level_four_pig_placements,
        level_four_skybox_color, lower_glass_body, pixel, reference_point,
        register_level_four_materials, right_planet_body, surface_direction,
    };
    use crate::{
        math::Vec3,
        scene::Scene,
        space::{register_space_materials, voxel::assert_voxel_ball},
    };

    #[test]
    fn reference_point_maps_the_picture_center_to_the_origin() {
        assert_eq!(reference_point(800.0, 600.0), Vec3::ZERO);
        assert_eq!(reference_point(925.0, 475.0), Vec3::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn level_four_scene_has_pigs_and_birds() {
        let (scene, metadata) = build_level_four_scene_with_metadata().unwrap();
        let shared = register_space_materials(&mut Scene::new()).unwrap();
        let pig_bodies = scene
            .objects()
            .iter()
            .filter_map(|object| object.as_sphere())
            .filter(|sphere| sphere.material_id() == shared.pig)
            .count();

        assert!(scene.skybox().is_some());
        assert!(scene.object_count() > 150);
        assert!(scene.curved_tetrahedron_count() == 0);
        assert_eq!(metadata.pig_count, LEVEL_FOUR_PIG_COUNT);
        assert_eq!(pig_bodies, LEVEL_FOUR_PIG_COUNT);
        // Every pig uses the shared rounded head and facial details.
        assert_eq!(
            metadata.pig_parts,
            LEVEL_FOUR_PIG_COUNT * super::super::pigs::PIG_PART_COUNT
        );
        assert_eq!(metadata.bird_count, 3);
        assert_eq!(metadata.bird_parts, 2 * 19 + 18);
        assert!(metadata.crystal_count >= 60);
        assert!(metadata.embedded_gem_count >= 40);
        assert_eq!(metadata.cut_gem_count, 3);
        assert_eq!(metadata.tnt_parts, 1);
        assert!(metadata.slingshot_parts > 0);
        assert!(metadata.wood_parts >= 20);
        assert!(metadata.rope_parts >= 8);
        // The stone plate and Lazer's body/belly also use triangular prisms.
        assert_eq!(
            scene.crystal_count(),
            metadata.crystal_count
                + metadata.embedded_gem_count
                + metadata.cut_gem_count
                + 1
                + super::super::birds::LAZER_PRISM_PARTS
        );
    }

    #[test]
    fn planet_pigs_stand_on_their_cubes_inside_their_bubbles() {
        let mut scene = Scene::new();
        register_space_materials(&mut scene).unwrap();
        let level = register_level_four_materials(&mut scene).unwrap();
        let placements = level_four_pig_placements(level).unwrap();

        for ((center, up, radius), (spot, _)) in placements.into_iter().zip(LEVEL_FOUR_PIGS) {
            assert!((up.length() - 1.0).abs() < 1.0e-4);
            let PigSpot::Planet(planet, _, _) = spot else {
                continue;
            };
            let (body, bubble_center, bubble_radius) = match planet {
                PigPlanet::CentralGlass => (
                    central_glass_body(level.glass).unwrap(),
                    LEVEL_FOUR_CENTRAL_BUBBLE_CENTER,
                    LEVEL_FOUR_CENTRAL_BUBBLE_RADIUS,
                ),
                PigPlanet::Right => (
                    right_planet_body(level.dark_planet).unwrap(),
                    LEVEL_FOUR_RIGHT_BUBBLE_CENTER,
                    LEVEL_FOUR_RIGHT_BUBBLE_RADIUS,
                ),
                PigPlanet::LowerGlass => (
                    lower_glass_body(level.glass).unwrap(),
                    LEVEL_FOUR_LOWER_BUBBLE_CENTER,
                    LEVEL_FOUR_LOWER_BUBBLE_RADIUS,
                ),
            };
            let ground = body.surface_distance(body.origin(), up);
            let height = (center - body.origin()).length();

            assert!(
                height > ground + radius * 0.9,
                "pig at {center:?} is buried"
            );
            assert!(height < ground + radius * 1.1, "pig at {center:?} floats");
            assert!((center - bubble_center).length() + radius < bubble_radius);
        }
    }

    #[test]
    fn glass_planets_sit_inside_their_gravity_bubbles() {
        for (glass_center, glass_radius, bubble_center, bubble_radius) in [
            (
                LEVEL_FOUR_CENTRAL_GLASS_CENTER,
                LEVEL_FOUR_CENTRAL_GLASS_RADIUS,
                LEVEL_FOUR_CENTRAL_BUBBLE_CENTER,
                LEVEL_FOUR_CENTRAL_BUBBLE_RADIUS,
            ),
            (
                LEVEL_FOUR_LOWER_GLASS_CENTER,
                LEVEL_FOUR_LOWER_GLASS_RADIUS,
                LEVEL_FOUR_LOWER_BUBBLE_CENTER,
                LEVEL_FOUR_LOWER_BUBBLE_RADIUS,
            ),
            (
                LEVEL_FOUR_RIGHT_PLANET_CENTER,
                LEVEL_FOUR_RIGHT_PLANET_RADIUS,
                LEVEL_FOUR_RIGHT_BUBBLE_CENTER,
                LEVEL_FOUR_RIGHT_BUBBLE_RADIUS,
            ),
        ] {
            assert!((glass_center - bubble_center).length() + glass_radius < bubble_radius);
        }
    }

    #[test]
    fn planets_are_balls_of_cubes_and_the_glass_planets_are_glass_cubes() {
        let (scene, metadata) = build_level_four_scene_with_metadata().unwrap();
        let launch = metadata.launch_planet.unwrap();
        let right = metadata.right_planet.unwrap();
        let central = metadata.central_glass.unwrap();
        let lower = metadata.lower_glass.unwrap();

        // Opaque planets: flat colored cubes from their swirled texture over
        // a dark core.
        for (parts, center, radius) in [
            (
                launch,
                LEVEL_FOUR_LAUNCH_PLANET_CENTER,
                LEVEL_FOUR_LAUNCH_PLANET_RADIUS,
            ),
            (
                right,
                LEVEL_FOUR_RIGHT_PLANET_CENTER,
                LEVEL_FOUR_RIGHT_PLANET_RADIUS,
            ),
        ] {
            let edge = assert_voxel_ball(&scene, parts, center, radius);

            assert!((edge - LEVEL_FOUR_CUBE_EDGE).abs() < 1.0e-5);
            assert_eq!(parts.core_count, 1);
            assert!(parts.palette_materials > 1);
            for id in parts.ids() {
                let material = scene.material(scene.objects()[id].material_id()).unwrap();
                assert!(material.texture_id.is_none());
                assert_eq!(material.transparency, 0.0);
            }
        }

        // Glass planets: hollow shells of clear glass cubes, all with the
        // same glass material.
        let glass_material = scene.objects()[central.first_id].material_id();
        let glass = scene.material(glass_material).unwrap();
        assert!(glass.transparency > 0.8);
        assert!(glass.refractive_index > 1.0);
        for (parts, center, radius) in [
            (
                central,
                LEVEL_FOUR_CENTRAL_GLASS_CENTER,
                LEVEL_FOUR_CENTRAL_GLASS_RADIUS,
            ),
            (
                lower,
                LEVEL_FOUR_LOWER_GLASS_CENTER,
                LEVEL_FOUR_LOWER_GLASS_RADIUS,
            ),
        ] {
            let edge = assert_voxel_ball(&scene, parts, center, radius);

            assert!((edge - LEVEL_FOUR_GLASS_CUBE_EDGE).abs() < 1.0e-5);
            assert!(radius * 2.0 / edge >= LEVEL_FOUR_MIN_CUBES_ACROSS);
            assert_eq!(parts.core_count, 0);
            assert_eq!(parts.palette_materials, 0);
            assert!(
                parts
                    .ids()
                    .all(|id| scene.objects()[id].material_id() == glass_material)
            );
        }
    }

    #[test]
    fn glass_cubes_leave_room_for_what_is_inside() {
        let (scene, metadata) = build_level_four_scene_with_metadata().unwrap();
        let inside_cube = |point: Vec3, ids: std::ops::Range<usize>| {
            ids.map(|id| scene.objects()[id].as_cube().unwrap())
                .any(|cube| {
                    point.x >= cube.min.x
                        && point.x <= cube.max.x
                        && point.y >= cube.min.y
                        && point.y <= cube.max.y
                        && point.z >= cube.min.z
                        && point.z <= cube.max.z
                })
        };
        let central = metadata.central_glass.unwrap().ids();
        let lower = metadata.lower_glass.unwrap().ids();
        // The TNT crate is a cube turned inside the central glass planet.
        let tnt = pixel(CENTRAL_TNT);
        let tnt_reach = CENTRAL_TNT_HALF * 3.0_f32.sqrt();

        for id in central.clone() {
            let cube = scene.objects()[id].as_cube().unwrap();
            let half = (cube.max - cube.min).length() * 0.5;
            assert!(((cube.min + cube.max) * 0.5 - tnt).length() > tnt_reach + half * 0.5);
        }
        // The lights inside the glass planets are in their hollow, not in a
        // cube, so they still light what is inside.
        assert!(!inside_cube(
            LEVEL_FOUR_CENTRAL_GLASS_CENTER + CENTRAL_GLASS_LIGHT_OFFSET,
            central
        ));
        assert!(!inside_cube(
            LEVEL_FOUR_LOWER_GLASS_CENTER + LOWER_GLASS_LIGHT_OFFSET,
            lower
        ));
    }

    #[test]
    fn loose_asteroids_and_ice_chunks_are_lumpy_balls_of_cubes() {
        let (scene, metadata) = build_level_four_scene_with_metadata().unwrap();
        let voxel_cubes = [
            metadata.launch_planet.unwrap(),
            metadata.right_planet.unwrap(),
            metadata.central_glass.unwrap(),
            metadata.lower_glass.unwrap(),
        ]
        .iter()
        .map(|parts| parts.cube_count)
        .sum::<usize>();
        let rocks = LOOSE_ASTEROIDS
            .iter()
            .filter(|asteroid| !asteroid.2)
            .count();
        let ice_chunks = LOOSE_ASTEROIDS.len() - rocks;

        let mut materials = Scene::new();
        register_space_materials(&mut materials).unwrap();
        let level = register_level_four_materials(&mut materials).unwrap();
        let ice_cubes = scene
            .cubes()
            .filter(|cube| cube.material_id == level.space_ice)
            .count();

        assert!(metadata.asteroid_parts > rocks * 20);
        assert!(metadata.ice_chunk_parts > ice_chunks * 20);
        // Rocks also have dark core spheres; ice is see-through, so it is a
        // shell of cubes with its own material and no core.
        assert_eq!(ice_cubes, metadata.ice_chunk_parts);
        assert!(scene.cube_count() > voxel_cubes + ice_cubes);
        assert!(
            scene.cube_count() <= voxel_cubes + metadata.asteroid_parts + metadata.ice_chunk_parts
        );
    }

    #[test]
    fn crystals_and_gems_grow_from_the_top_of_the_cubes() {
        let (scene, _) = build_level_four_scene_with_metadata().unwrap();
        let launch = launch_planet_body(0).unwrap();
        let center = LEVEL_FOUR_LAUNCH_PLANET_CENTER;

        for &(angle, depth, size) in &LAUNCH_PLANET_GEMS {
            let normal = surface_direction(angle, depth);
            let expected =
                center + normal * (launch.surface_distance(center, normal) - size * 0.15);

            assert!(
                scene
                    .crystals()
                    .any(|gem| (gem.base() - expected).length() < 1.0e-4),
                "no gem at {expected:?}"
            );
        }
    }

    #[test]
    fn every_closed_region_has_its_own_light() {
        let (scene, _) = build_level_four_scene_with_metadata().unwrap();
        let inside = |center: Vec3, radius: f32| {
            scene
                .lights()
                .iter()
                .filter(|light| (light.position - center).length() < radius)
                .count()
        };

        assert!(
            inside(
                LEVEL_FOUR_CENTRAL_GLASS_CENTER,
                LEVEL_FOUR_CENTRAL_GLASS_RADIUS
            ) >= 1
        );
        assert!(inside(LEVEL_FOUR_LOWER_GLASS_CENTER, LEVEL_FOUR_LOWER_GLASS_RADIUS) >= 1);
        assert!(
            inside(
                LEVEL_FOUR_CENTRAL_BUBBLE_CENTER,
                LEVEL_FOUR_CENTRAL_BUBBLE_RADIUS
            ) > inside(
                LEVEL_FOUR_CENTRAL_GLASS_CENTER,
                LEVEL_FOUR_CENTRAL_GLASS_RADIUS
            )
        );
        assert!(
            inside(
                LEVEL_FOUR_LOWER_BUBBLE_CENTER,
                LEVEL_FOUR_LOWER_BUBBLE_RADIUS
            ) > inside(LEVEL_FOUR_LOWER_GLASS_CENTER, LEVEL_FOUR_LOWER_GLASS_RADIUS)
        );
        assert!(
            inside(
                LEVEL_FOUR_RIGHT_BUBBLE_CENTER,
                LEVEL_FOUR_RIGHT_BUBBLE_RADIUS
            ) >= 2
        );
        assert!(
            scene
                .lights()
                .iter()
                .filter(|light| {
                    (light.position - LEVEL_FOUR_CENTRAL_BUBBLE_CENTER).length()
                        > LEVEL_FOUR_CENTRAL_BUBBLE_RADIUS
                        && (light.position - LEVEL_FOUR_RIGHT_BUBBLE_CENTER).length()
                            > LEVEL_FOUR_RIGHT_BUBBLE_RADIUS
                        && (light.position - LEVEL_FOUR_LOWER_BUBBLE_CENTER).length()
                            > LEVEL_FOUR_LOWER_BUBBLE_RADIUS
                })
                .count()
                >= 2
        );
    }

    #[test]
    fn bvh_matches_linear_search_from_the_default_camera() {
        let (scene, _) = build_level_four_scene_with_metadata().unwrap();
        let camera = level_four_orbit_camera(16.0 / 9.0).to_camera();

        for y in 0..18 {
            for x in 0..32 {
                let ray = camera.ray_for_pixel(x, y, 32, 18);
                let mut closest = 1_000.0;
                let mut expected = None;
                for object in scene.objects() {
                    if let Some(hit) = object.intersect(&ray, 0.001, closest) {
                        closest = hit.distance;
                        expected = Some(hit);
                    }
                }

                assert_eq!(scene.intersect(&ray, 0.001, 1_000.0), expected);
            }
        }
    }

    #[test]
    fn default_camera_looks_at_the_level_from_the_front() {
        let camera = level_four_orbit_camera(16.0 / 9.0).to_camera();
        let basis = camera.basis();

        assert!(camera.position.z > 9.0);
        assert!(basis.forward.z < -0.99);
    }

    #[test]
    fn skybox_is_purple_starred_and_wraps() {
        let low = level_four_skybox_color(0.3, 0.40);
        let high = level_four_skybox_color(0.3, 0.64);
        let brightest = (0..400)
            .map(|index| level_four_skybox_color(index as f32 / 400.0, 0.52))
            .map(|color| color.r + color.g + color.b)
            .fold(0.0_f32, f32::max);

        assert!(high.r > low.r && high.b > high.g);
        assert!(low.b > low.r);
        assert!(brightest > 1.5);
        for v in [0.35, 0.5, 0.6, 0.7] {
            let left = level_four_skybox_color(0.0, v);
            let right = level_four_skybox_color(1.0, v);
            assert!((left.r - right.r).abs() + (left.g - right.g).abs() < 0.02);
        }
    }
}
