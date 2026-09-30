use std::{error::Error, fmt};

use crate::{
    basis::{Basis3, Basis3Error},
    camera::OrbitCamera,
    color::Color,
    cone::{Cone, ConeError},
    crystal::CrystalError,
    curved_tetrahedron::{CurvedTetrahedron, CurvedTetrahedronError},
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

mod birds;
mod blue_moon;
mod cosmic_crystals;
mod level_one;
mod voxel;

pub use cosmic_crystals::{build_level_four_scene, level_four_orbit_camera, level_four_skybox};
pub use level_one::{
    LevelOneGame, build_level_one, build_level_one_scene, level_one_layout, level_one_orbit_camera,
};
pub use voxel::VoxelError;
use voxel::{VoxelBall, VoxelBody, VoxelBodyParts};

pub const BLUE_MOON_WINDOW_TITLE: &str = "Angry Birds Space Diorama - Luna Azul";
pub const SPACE_WORLDS_WINDOW_TITLE: &str = "Angry Birds Space Diorama - Worlds";
pub const BLUE_MOON_PLANET_CENTER: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const BLUE_MOON_PLANET_RADIUS: f32 = 1.45;
pub const COOKIE_PLANET_CENTER: Vec3 = Vec3::new(4.45, -0.30, -0.25);
pub const COOKIE_PLANET_RADIUS: f32 = 1.45;
/// Level two centerpiece: a big pig floating inside a gravity bubble.
pub const COOKIE_WORLD_PIG_CENTER: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const COOKIE_WORLD_PIG_RADIUS: f32 = 1.3;
pub const COOKIE_WORLD_BUBBLE_RADIUS: f32 = 3.2;
/// Tilt of the pig around the view axis (about 19 degrees), like the playful
/// tilt of the pig in the reference level.
const COOKIE_WORLD_PIG_ROLL_RADIANS: f32 = 0.33;
/// The level two background tile spans a quarter turn and repeats 4 times.
const COOKIE_WORLD_SKYBOX_TILES: f32 = 4.0;
const PIG_SKIN_TEXTURE_WIDTH: usize = 512;
const PIG_SKIN_TEXTURE_HEIGHT: usize = 256;
const PIG_SKIN_BASE_COLOR: Color = Color::new(0.44, 0.92, 0.28);
const PIG_SKIN_SPOT_COLOR: Color = Color::new(0.30, 0.70, 0.13);
/// Angular width of the soft border of the pig's skin spots, in radians.
const PIG_SKIN_SPOT_SOFTNESS: f32 = 0.035;
/// Lights inside the level two bubble: offset from the pig in pig radii,
/// color, and intensity for a pig of radius 1 (it grows with the pig's area).
const COOKIE_WORLD_INNER_LIGHTS: [(Vec3, Color, f32); 4] = [
    (
        Vec3::new(-1.30, 1.25, 1.35),
        Color::new(1.0, 0.95, 0.88),
        3.2,
    ),
    (Vec3::new(0.20, 0.30, 2.10), Color::new(1.0, 1.0, 0.95), 1.3),
    (
        Vec3::new(1.50, -0.55, 1.45),
        Color::new(0.95, 0.80, 1.0),
        2.2,
    ),
    (
        Vec3::new(0.35, 1.85, -1.25),
        Color::new(0.90, 0.95, 1.0),
        1.8,
    ),
];
/// Lights around the level two bubble, in the plane of its silhouette seen
/// from the initial camera, a little outside the bubble.
const COOKIE_WORLD_RIM_LIGHT_DIRECTIONS: [Vec3; 4] = [
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(0.0, -1.0, 0.0),
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(-1.0, 0.0, 0.0),
];
const COOKIE_WORLD_RIM_LIGHT_GAP: f32 = 0.65;
const COOKIE_WORLD_RIM_LIGHT_INTENSITY: f32 = 3.5;
/// Cookie planets around the level two bubble: two with a waffle horn and one
/// on the left side with the slingshot on top.
pub const COOKIE_WORLD_COOKIE_COUNT: usize = 3;
const COOKIE_WORLD_COOKIES: [CookiePlanet; COOKIE_WORLD_COOKIE_COUNT] = [
    // Upper right, a little behind the bubble.
    CookiePlanet {
        center: Vec3::new(4.57, 3.53, -1.39),
        radius: 1.05,
        seed: 11.0,
    },
    // Lower left, a little in front of the bubble.
    CookiePlanet {
        center: Vec3::new(-5.14, -2.95, 0.46),
        radius: 1.15,
        seed: 23.0,
    },
    // Left side, carries the slingshot.
    CookiePlanet {
        center: Vec3::new(-6.04, 1.09, -0.34),
        radius: 0.85,
        seed: 37.0,
    },
];
const COOKIE_WORLD_SLINGSHOT_COOKIE: usize = 2;
/// Each cookie has a warm key light from the upper left front, close enough
/// that it barely reaches the bubble.
const COOKIE_WORLD_COOKIE_KEY_DIRECTION: Vec3 = Vec3::new(-0.45, 0.65, 0.75);
const COOKIE_WORLD_COOKIE_KEY_DISTANCE: f32 = 2.3;
const COOKIE_WORLD_COOKIE_KEY_INTENSITY: f32 = 3.2;
/// Direction from the slingshot cookie's center to the slingshot, tilted a
/// little towards the pig.
const COOKIE_WORLD_SLINGSHOT_DIRECTION: Vec3 = Vec3::new(0.16, 1.0, 0.06);
/// The slingshot is larger than the one in Luna Azul so it reads from the
/// level camera.
const COOKIE_WORLD_SLINGSHOT_SCALE: f32 = 1.6;
/// How deep the slingshot trunk goes into the cookie.
const COOKIE_WORLD_SLINGSHOT_EMBED: f32 = 0.06;
/// Turn of the slingshot around its trunk: a quarter turn puts the fork in
/// the view plane and the rest shows it at an angle, like in Luna Azul.
const COOKIE_WORLD_SLINGSHOT_YAW_RADIANS: f32 = -std::f32::consts::FRAC_PI_2 + 0.35;
/// Lone waffle cones on the right side. Their glossy caramel tops look
/// towards the pig: that is where the birds bounce.
const COOKIE_WORLD_CARAMEL_CONES: [CaramelCone; 2] = [
    // Lying on its side, caramel towards the pig.
    CaramelCone {
        top: Vec3::new(5.30, -0.18, 0.41),
        facing: Vec3::new(-0.85, 0.52, 0.05),
        length: 1.60,
        radius: 0.42,
    },
    // Standing up, lower and a little behind.
    CaramelCone {
        top: Vec3::new(6.06, -2.00, -0.29),
        facing: Vec3::new(-0.30, 0.95, -0.05),
        length: 1.35,
        radius: 0.36,
    },
];
/// Warm key light shared by the caramel cones: the point it aims at and how
/// far from it the light sits.
const COOKIE_WORLD_CONES_KEY_TARGET: Vec3 = Vec3::new(5.68, -1.22, 0.06);
const COOKIE_WORLD_CONES_KEY_DISTANCE: f32 = 3.0;
/// Caramel scoop over the cone's mouth, in cone radii: its center sinks a
/// little into the cone and it is just wider than the mouth, so it covers
/// the waffle rim.
const CARAMEL_DOME_SINK: f32 = 0.15;
const CARAMEL_DOME_RADIUS: f32 = 1.03;
/// Caramel drips down the waffle from under the scoop: angle around the
/// cone and how far down they reach, as a fraction of the cone length.
const CARAMEL_DRIP_START: f32 = 0.10;
const CARAMEL_DRIPS: [(f32, f32); 5] = [
    (0.3, 0.26),
    (1.5, 0.18),
    (2.6, 0.32),
    (3.9, 0.21),
    (5.1, 0.29),
];
const WAFFLE_TEXTURE_SIZE: usize = 256;
const ROCK_TEXTURE_WIDTH: usize = 256;
const ROCK_TEXTURE_HEIGHT: usize = 128;
const ROCK_CRATER_COUNT: usize = 6;
const ROCK_LIGHT: Color = Color::new(0.68, 0.71, 0.73);
const ROCK_DARK: Color = Color::new(0.42, 0.45, 0.49);
/// Diamonds of the waffle pattern around the cone and along its height.
const WAFFLE_CELLS_AROUND: f32 = 12.0;
const WAFFLE_CELLS_ALONG: f32 = 7.0;
const WAFFLE_LIGHT: Color = Color::new(0.97, 0.72, 0.40);
const WAFFLE_GROOVE: Color = Color::new(0.66, 0.38, 0.14);
/// Popcorn floating around the bubble: cluster center, size and spin.
const COOKIE_WORLD_POPCORN: [(Vec3, f32, f32); 5] = [
    (Vec3::new(-1.41, 4.05, 0.55), 0.30, 0.0),
    (Vec3::new(4.07, 0.62, 0.99), 0.26, 1.3),
    (Vec3::new(-4.12, -0.92, 0.68), 0.28, 2.6),
    (Vec3::new(0.93, -4.08, 0.43), 0.27, 3.9),
    (Vec3::new(3.10, -3.10, -0.97), 0.22, 5.2),
];
/// Puffs of one popcorn cluster (offset and radius in cluster units) and
/// the golden kernel peeking out of it.
const POPCORN_PUFFS: [(Vec3, f32); 4] = [
    (Vec3::new(0.00, 0.00, 0.00), 0.50),
    (Vec3::new(0.48, 0.22, 0.10), 0.40),
    (Vec3::new(-0.36, 0.30, 0.12), 0.38),
    (Vec3::new(0.08, -0.38, 0.20), 0.34),
];
const POPCORN_KERNEL: (Vec3, f32) = (Vec3::new(0.16, 0.10, 0.44), 0.15);
/// Small asteroids around the bubble: center and radius. Each one gets a
/// smaller lump so it is not a perfect ball.
const COOKIE_WORLD_ROCKS: [(Vec3, f32); 10] = [
    (Vec3::new(1.54, 3.94, -0.37), 0.16),
    (Vec3::new(-2.89, 3.08, -0.68), 0.22),
    (Vec3::new(-3.81, 0.44, -1.13), 0.14),
    (Vec3::new(-1.91, -3.63, 1.11), 0.18),
    (Vec3::new(-0.68, -4.13, -0.74), 0.13),
    (Vec3::new(2.38, -3.66, 0.67), 0.17),
    (Vec3::new(4.09, -1.59, -0.49), 0.20),
    (Vec3::new(3.88, -2.93, 0.54), 0.15),
    (Vec3::new(2.57, 3.37, 0.92), 0.15),
    (Vec3::new(-4.13, 1.58, 0.97), 0.12),
];
/// Popcorn and asteroids behind the bubble (as seen from the initial camera),
/// so the level has depth when the camera turns around it. Same layout as
/// the ones around the bubble: center and size (and spin) or radius.
const COOKIE_WORLD_BACK_POPCORN: [(Vec3, f32, f32); 7] = [
    (Vec3::new(-2.80, 2.60, -4.20), 0.38, 0.7),
    (Vec3::new(3.20, 1.60, -4.80), 0.42, 2.1),
    (Vec3::new(-4.60, -1.20, -3.80), 0.36, 3.3),
    (Vec3::new(0.80, -2.60, -5.20), 0.40, 4.4),
    (Vec3::new(5.20, 3.40, -5.60), 0.36, 5.6),
    (Vec3::new(-6.20, 3.60, -5.00), 0.34, 1.0),
    (Vec3::new(1.60, 4.60, -4.40), 0.32, 2.8),
];
const COOKIE_WORLD_BACK_ROCKS: [(Vec3, f32); 12] = [
    (Vec3::new(0.60, 1.20, -4.60), 0.40),
    (Vec3::new(-1.80, -0.90, -5.00), 0.32),
    (Vec3::new(2.60, -0.60, -4.00), 0.26),
    (Vec3::new(-3.60, 1.20, -5.80), 0.45),
    (Vec3::new(4.40, -2.80, -4.60), 0.30),
    (Vec3::new(-2.20, -3.60, -4.40), 0.28),
    (Vec3::new(6.00, 0.90, -5.20), 0.34),
    (Vec3::new(-5.80, -2.80, -6.20), 0.40),
    (Vec3::new(-0.60, 3.60, -6.00), 0.30),
    (Vec3::new(3.40, 3.90, -3.60), 0.22),
    (Vec3::new(-4.80, 4.20, -4.20), 0.26),
    (Vec3::new(1.90, -4.30, -3.80), 0.24),
];
/// Two original cartoon songbirds: one waits next to the slingshot and the
/// other flies towards the pig.
pub const COOKIE_WORLD_BIRD_COUNT: usize = 2;
const COOKIE_WORLD_BIRD_RADIUS: f32 = 0.24;
/// Where the waiting bird stands on the slingshot cookie.
const COOKIE_WORLD_PERCHED_BIRD_DIRECTION: Vec3 = Vec3::new(-0.62, 0.72, 0.30);
const COOKIE_WORLD_FLYING_BIRD_CENTER: Vec3 = Vec3::new(-4.14, 2.34, 0.66);
/// Height of a standing bird's center over the ground, in body radii.
const SONGBIRD_STANDING_HEIGHT: f32 = 1.14;
/// Bird colors: body, belly and wing/tail feathers. The first bird is cool
/// teal, the second a warm robin with an orange breast.
const SONGBIRD_PALETTES: [(Color, Color, Color); COOKIE_WORLD_BIRD_COUNT] = [
    (
        Color::new(0.16, 0.66, 0.72),
        Color::new(0.93, 0.90, 0.74),
        Color::new(0.07, 0.38, 0.50),
    ),
    (
        Color::new(0.58, 0.34, 0.19),
        Color::new(0.99, 0.55, 0.20),
        Color::new(0.34, 0.18, 0.09),
    ),
];
/// Wing feather tips (from the shoulder) for a folded and an open wing, in
/// body radii for the bird's left side (+X).
const SONGBIRD_SHOULDER: Vec3 = Vec3::new(0.90, -0.02, -0.08);
const SONGBIRD_FOLDED_WING: [Vec3; 4] = [
    Vec3::new(1.08, -0.12, -0.90),
    Vec3::new(1.10, -0.34, -0.80),
    Vec3::new(1.07, -0.46, -0.70),
    Vec3::new(1.04, -0.55, -0.58),
];
/// Open wings spread out to the sides like a fan of long feathers, a little
/// raised.
const SONGBIRD_OPEN_WING: [Vec3; 4] = [
    Vec3::new(1.55, 0.55, -0.15),
    Vec3::new(1.72, 0.30, -0.30),
    Vec3::new(1.70, 0.02, -0.44),
    Vec3::new(1.50, -0.22, -0.54),
];
const SONGBIRD_TAIL: [Vec3; 3] = [
    Vec3::new(-0.26, 0.26, -1.46),
    Vec3::new(0.00, 0.34, -1.54),
    Vec3::new(0.26, 0.26, -1.46),
];
const SONGBIRD_CREST: [Vec3; 3] = [
    Vec3::new(-0.13, 1.36, 0.22),
    Vec3::new(0.00, 1.44, 0.08),
    Vec3::new(0.13, 1.36, 0.22),
];
/// The horns curl in a plane that faces the initial camera, so their spiral
/// is seen from the side, and their mouths turn a little towards it.
const COOKIE_WORLD_HORNS: [WaffleHorn; 2] = [
    // Under the upper right cookie, mouth to the left, tail curling down.
    WaffleHorn {
        cookie: 0,
        side: Vec3::new(-0.55, -0.80, 0.25),
        mouth_facing: Vec3::new(-0.80, 0.15, 0.55),
        curl_towards: Vec3::new(0.15, -1.0, 0.0),
        scale: 1.9,
    },
    // Right of the lower left cookie, mouth up and to the left, tail curling
    // down to the right.
    WaffleHorn {
        cookie: 1,
        side: Vec3::new(1.0, -0.20, 0.25),
        mouth_facing: Vec3::new(-0.35, 0.80, 0.50),
        curl_towards: Vec3::new(1.0, -0.15, 0.0),
        scale: 2.0,
    },
];
/// Sphere around the horn's mouth, used only to keep the mouth out of the
/// cookie when the horn is placed.
const WAFFLE_HORN_MOUTH_BOUND: (Vec3, f32) = (Vec3::new(0.0, -0.08, 0.0), 0.17);
/// How deep the horn's closest rib sinks into its cookie, in horn units.
const WAFFLE_HORN_CONTACT_DEPTH: f32 = 0.04;
/// Iterations used to slide a horn along its side until it touches the cookie.
const WAFFLE_HORN_CONTACT_STEPS: usize = 16;
const WAFFLE_HORN_MOUTH_RADIUS: f32 = 0.18;
const WAFFLE_HORN_MOUTH_HALF_DEPTH: f32 = 0.20;
/// The dark opening leaves a thin waffle rim around it.
const WAFFLE_HORN_OPENING_RADIUS: f32 = 0.15;
const WAFFLE_HORN_OPENING_HALF_DEPTH: f32 = 0.012;
/// The ribbed body starts straight under the mouth, then curls around.
const WAFFLE_HORN_STRAIGHT_RIBS: [(f32, f32); 3] = [(-0.16, 0.105), (-0.24, 0.098), (-0.32, 0.092)];
const WAFFLE_HORN_CURL_RADIUS: f32 = 0.20;
const WAFFLE_HORN_CURL_SHRINK: f32 = 0.45;
const WAFFLE_HORN_CURL_TURNS: f32 = 0.68;
const WAFFLE_HORN_TIP_RADIUS: f32 = 0.026;
/// Distance between neighbor ribs relative to their radius.
const WAFFLE_HORN_RIB_SPACING: f32 = 0.75;
const COOKIE_TEXTURE_WIDTH: usize = 512;
const COOKIE_TEXTURE_HEIGHT: usize = 256;
const COOKIE_CHIP_COUNT: usize = 34;
const COOKIE_DOUGH_LIGHT: Color = Color::new(0.95, 0.74, 0.50);
const COOKIE_DOUGH: Color = Color::new(0.89, 0.62, 0.37);
const COOKIE_DOUGH_BAKED: Color = Color::new(0.70, 0.44, 0.23);
const COOKIE_CHIP_DARK: Color = Color::new(0.26, 0.13, 0.07);
const COOKIE_CHIP_LIGHT: Color = Color::new(0.45, 0.27, 0.14);
pub const LEVEL_THREE_PLANET_CENTER: Vec3 = Vec3::new(-0.35, 0.05, 0.10);
pub const LEVEL_THREE_PLANET_RADIUS: f32 = 1.70;
pub const LEVEL_THREE_MAIN_ASTEROID_RADIUS: f32 = LEVEL_THREE_PLANET_RADIUS * 0.86;
pub const LEVEL_THREE_PIG_COUNT: usize = 4;
const LEVEL_THREE_SECONDARY_ASTEROID_RADIUS: f32 = 1.06;
/// Direction from the main asteroid to the secondary one, in the XY plane.
const LEVEL_THREE_BRIDGE_DIRECTION: Vec3 = Vec3::new(-0.9186, 0.3952, 0.0);
/// Side of the square cells of the level three structures.
const LEVEL_THREE_BLOCK: f32 = 0.458;
/// Thickness of the bars of frames and ladders.
const LEVEL_THREE_BAR: f32 = 0.065;
/// Half depth (along Z) of the level three blocks.
const LEVEL_THREE_DEPTH: f32 = 0.22;
const LEVEL_THREE_BRIDGE_LEVELS: usize = 9;
/// How deep the bridge starts inside the main asteroid and ends inside the
/// secondary one.
const LEVEL_THREE_BRIDGE_EMBED: f32 = 0.05;
const LEVEL_THREE_SECONDARY_EMBED: f32 = 0.02;
/// Face radius scale that makes the ice pyramid's faces almost flat.
const LEVEL_THREE_PYRAMID_FACE_RADIUS_SCALE: f32 = 6.0;
/// Floating rocks: offset from the main asteroid and radius.
/// Edge of the cubes that build the level three asteroids.
const LEVEL_THREE_CUBE_EDGE: f32 = 0.12;
/// Half width of the ground under the objects that stand on the main
/// asteroid, and how many points of it are checked.
const LEVEL_THREE_FOOTPRINT_HALF_WIDTH: f32 = 0.25;
const LEVEL_THREE_FOOTPRINT_SAMPLES: usize = 16;
/// Small asteroids and rocks use smaller cubes, so at least this many span
/// them and they still look round.
const VOXEL_ROCK_MIN_CUBES_ACROSS: f32 = 7.0;
/// Rocks are a ball with a smaller lump: lump offset in rock radii and lump
/// radius as a fraction of the rock radius.
pub(super) const ROCK_LUMP_OFFSET: Vec3 = Vec3::new(0.45, 0.30, 0.15);
pub(super) const ROCK_LUMP_RADIUS: f32 = 0.62;
const LEVEL_THREE_ROCKS: [(Vec3, f32); 3] = [
    (Vec3::new(-4.97, -1.42, 0.30), 0.30),
    (Vec3::new(-7.60, -2.80, -0.40), 0.36),
    (Vec3::new(-6.40, -3.60, 0.00), 0.17),
];
/// Red atmospheres (gravity fields) around both asteroids, as in the
/// reference level. They wrap every structure of their asteroid and overlap
/// around the middle of the bridge.
pub const LEVEL_THREE_MAIN_ATMOSPHERE_RADIUS: f32 = 4.3;
pub const LEVEL_THREE_SECONDARY_ATMOSPHERE_RADIUS: f32 = 3.4;
/// Small asteroid under the bridge, outside both atmospheres, that carries
/// the slingshot.
pub const LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER: Vec3 = Vec3::new(-4.30, -3.65, 0.60);
pub const LEVEL_THREE_SLINGSHOT_ASTEROID_RADIUS: f32 = 0.50;
/// The slingshot stands straight on top of its asteroid, so it stays out of
/// the main atmosphere.
const LEVEL_THREE_SLINGSHOT_DIRECTION: Vec3 = Vec3::new(-0.05, 1.0, 0.05);
const LEVEL_THREE_SLINGSHOT_SCALE: f32 = 1.3;
const LEVEL_THREE_SLINGSHOT_EMBED: f32 = 0.05;
/// The loaded bird looks up and right, towards the bridge.
const LEVEL_THREE_BIRD_AIM: Vec3 = Vec3::new(0.45, 1.0, 0.0);
/// The other two birds wait on the left of the slingshot asteroid, turned a
/// little towards the camera.
const LEVEL_THREE_WAITING_BIRDS: [Vec3; 2] =
    [Vec3::new(-0.88, 0.38, 0.30), Vec3::new(-0.80, -0.35, 0.55)];
/// Lights inside the level three atmospheres, as offsets from the asteroid
/// they light. The lens light sits where both atmospheres overlap, at
/// `LEVEL_THREE_LENS_DISTANCE` from the main asteroid along the bridge.
const LEVEL_THREE_MAIN_KEY_OFFSET: Vec3 = Vec3::new(-1.60, 2.20, 3.00);
const LEVEL_THREE_MAIN_FILL_OFFSET: Vec3 = Vec3::new(2.60, -1.60, 2.40);
const LEVEL_THREE_LENS_DISTANCE: f32 = 3.74;
const LEVEL_THREE_LENS_LIGHT_OFFSET: Vec3 = Vec3::new(0.0, 0.27, 1.45);
const LEVEL_THREE_SECONDARY_KEY_OFFSET: Vec3 = Vec3::new(-0.90, 1.60, 2.40);
const LEVEL_THREE_OUTSIDE_KEY_OFFSET: Vec3 = Vec3::new(-1.20, 1.40, 1.80);
const LEVEL_THREE_ROCKS_KEY_POSITION: Vec3 = Vec3::new(-6.50, -1.30, 2.40);
/// Rim lights just outside each atmosphere, in the plane through its center
/// that faces the default camera.
const LEVEL_THREE_MAIN_RIM_OFFSET: Vec3 = Vec3::new(4.85, 0.60, 0.0);
const LEVEL_THREE_SECONDARY_RIM_OFFSET: Vec3 = Vec3::new(-3.90, 0.40, 0.0);
const LEVEL_THREE_RIM_COLOR: Color = Color::new(1.0, 0.42, 0.38);
const LEVEL_THREE_RIM_INTENSITY: f32 = 9.0;
/// The level three background tile spans a quarter turn and repeats 4 times.
const LEVEL_THREE_SKYBOX_TILES: f32 = 4.0;
/// Point the level three camera orbits around: between both asteroids and a
/// little low, so the slingshot asteroid is in view.
pub const LEVEL_THREE_CAMERA_TARGET: Vec3 = Vec3::new(-2.45, 0.35, 0.10);
pub const SPACE_SUN_DIRECTION: Vec3 = Vec3::new(-0.76, 0.54, -0.36);
pub const COOKIE_LEVEL_PIG_COUNT: usize = 1;
pub const GALAXY_SELECTOR_BLUE_MOON_CENTER: Vec3 = Vec3::new(-4.65, -0.72, 0.0);
pub const GALAXY_SELECTOR_BLUE_MOON_RADIUS: f32 = 1.12;
pub const GALAXY_SELECTOR_COOKIE_CENTER: Vec3 = Vec3::new(-1.55, -0.88, 0.0);
pub const GALAXY_SELECTOR_COOKIE_RADIUS: f32 = 1.34;
pub const GALAXY_SELECTOR_LEVEL_THREE_CENTER: Vec3 = Vec3::new(1.55, -0.72, 0.0);
pub const GALAXY_SELECTOR_LEVEL_THREE_RADIUS: f32 = 1.14;
pub const GALAXY_SELECTOR_LEVEL_FOUR_CENTER: Vec3 = Vec3::new(4.65, -0.72, 0.0);
pub const GALAXY_SELECTOR_LEVEL_FOUR_RADIUS: f32 = 1.16;
const SELECTOR_CRYSTAL_TEXTURE_WIDTH: usize = 256;
const SELECTOR_CRYSTAL_TEXTURE_HEIGHT: usize = 128;
const SELECTOR_SUN_U: f32 = 0.25;
const SELECTOR_SUN_V: f32 = 0.49;
const SELECTOR_SUN_U_SCALE: f32 = 2.35;
const SPACE_SKYBOX_WIDTH: usize = 960;
const SPACE_SKYBOX_HEIGHT: usize = 480;
const SPACE_SUN_U: f32 = 0.125;
/// Low over the horizon, so Luna Azul shows it at the top left of its
/// initial view.
const SPACE_SUN_V: f32 = 0.606;
/// Size of the sun and its halos relative to the original skybox, so only a
/// corner of it shows in the initial view.
const SPACE_SUN_SIZE: f32 = 0.62;
/// Turns the Luna Azul skybox so its sun lies along
/// `blue_moon::BLUE_MOON_SUN_DIRECTION`.
const BLUE_MOON_SKYBOX_ROTATION: f32 = 0.9861;
const SKYBOX_ASTEROID_A_U: f32 = 0.675;
const SKYBOX_ASTEROID_A_V: f32 = 0.620;
const SKYBOX_ASTEROID_B_U: f32 = 0.835;
const SKYBOX_ASTEROID_B_V: f32 = 0.455;
const SKYBOX_ASTEROID_C_U: f32 = 0.425;
const SKYBOX_ASTEROID_C_V: f32 = 0.730;
// Slingshot geometry in the slingshot radial frame (see `add_blue_moon_slingshot`).
const BLUE_MOON_SLINGSHOT_TRUNK_BASE_Y: f32 = 0.44;
/// Turn of the slingshot around its trunk (about 20 degrees).
const BLUE_MOON_SLINGSHOT_FORK: Vec3 = Vec3::new(0.0, 0.74, 0.0);
const BLUE_MOON_SLINGSHOT_ELBOW: Vec3 = Vec3::new(0.13, 0.89, 0.0);
const BLUE_MOON_SLINGSHOT_TIP: Vec3 = Vec3::new(0.145, 1.03, 0.0);
const BLUE_MOON_SLINGSHOT_POUCH: Vec3 = Vec3::new(0.0, 0.90, 0.035);
const BLUE_MOON_SLINGSHOT_TRUNK_RADIUS: f32 = 0.050;
const BLUE_MOON_SLINGSHOT_FORK_RADIUS: f32 = 0.052;
const BLUE_MOON_SLINGSHOT_ARM_RADIUS: f32 = 0.039;
const BLUE_MOON_SLINGSHOT_PRONG_RADIUS: f32 = 0.035;
const BLUE_MOON_SLINGSHOT_WRAP_RADIUS: f32 = 0.041;
const BLUE_MOON_SLINGSHOT_ELASTIC_RADIUS: f32 = 0.011;
const BLUE_MOON_SLINGSHOT_POUCH_RADIUS: f32 = 0.026;
const BLUE_MOON_SLINGSHOT_POUCH_HALF_WIDTH: f32 = 0.048;
// Trail of the flying chuck (see `add_blue_moon_chuck_trail`).
const BLUE_MOON_PLANET_TEXTURE_PATH: &str = "assets/textures/blue_moon_planet.ppm";
const WOOD_BLOCK_TEXTURE_PATH: &str = "assets/textures/space_wood_block.ppm";
const TNT_CRATE_TEXTURE_PATH: &str = "assets/textures/tnt_crate.ppm";
const ICE_BLOCK_TEXTURE_PATH: &str = "assets/textures/space_ice_block.ppm";
const GRAY_STONE_BLOCK_TEXTURE_PATH: &str = "assets/textures/space_gray_stone_block.ppm";
const DANGER_ZONE_SKYBOX_TEXTURE_PATH: &str = "assets/textures/dangerzone_theme_parallax_1.ppm";
const ASTEROID_PLANET_TEXTURE_PATH: &str = "assets/textures/asteroid_planet.ppm";
const UTOPIA_SKYBOX_TEXTURE_PATH: &str = "assets/textures/utopia_theme_parallax.ppm";

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
    CosmicCrystals,
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
    CurvedTetrahedron(CurvedTetrahedronError),
    Crystal(CrystalError),
    OrientedBox(OrientedBoxError),
    RadialFrame(RadialFrameError),
    Basis(Basis3Error),
    Voxel(VoxelError),
}

#[derive(Debug, Clone, Copy)]
struct SpaceMaterials {
    gravity_field: usize,
    cookie_gravity_field: usize,
    level_three_asteroid: usize,
    level_three_stone: usize,
    ice: usize,
    level_three_gravity_field: usize,
    level_three_atmosphere: usize,
    level_four_gravity_field: usize,
    selector_locked_moon: usize,
    selector_locked_cookie: usize,
    selector_locked_level_three: usize,
    selector_locked_level_four: usize,
    cookie: usize,
    wood: usize,
    slingshot_wood: usize,
    slingshot_band: usize,
    tnt_crate: usize,
    pig: usize,
    snout: usize,
    eye: usize,
    pupil: usize,
}

/// Materials used only by level two. They are registered by the level two
/// builder, so the other scenes do not generate its procedural textures.
#[derive(Debug, Clone, Copy)]
struct CookieWorldMaterials {
    bubble: usize,
    pig_skin: usize,
    pig_ear: usize,
    pig_snout: usize,
    pig_nostril: usize,
    popcorn: usize,
    popcorn_kernel: usize,
    cookies: [usize; COOKIE_WORLD_COOKIE_COUNT],
    horn_waffle: usize,
    horn_opening: usize,
    waffle_cone: usize,
    caramel: usize,
    rock: usize,
    far_rock: usize,
    bird_body: [usize; COOKIE_WORLD_BIRD_COUNT],
    bird_belly: [usize; COOKIE_WORLD_BIRD_COUNT],
    bird_feather: [usize; COOKIE_WORLD_BIRD_COUNT],
    bird_beak: usize,
    bird_feet: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SpaceSceneMetadata {
    pub blue_moon_planet: Option<VoxelBodyParts>,
    pub cookie_planet_id: Option<usize>,
    pub cookie_gravity_field_id: Option<usize>,
    pub level_three_planet: Option<VoxelBodyParts>,
    pub level_three_secondary_asteroid: Option<VoxelBodyParts>,
    pub level_three_bridge_parts: usize,
    pub level_three_tower_parts: usize,
    pub level_three_lower_structure_parts: usize,
    pub level_three_rock_count: usize,
    pub level_three_rock_parts: usize,
    pub level_three_pig_count: usize,
    pub level_three_main_atmosphere_id: Option<usize>,
    pub level_three_secondary_atmosphere_id: Option<usize>,
    pub level_three_slingshot_asteroid: Option<VoxelBodyParts>,
    pub level_three_slingshot_parts: usize,
    pub level_three_bird_count: usize,
    pub level_three_bird_parts: usize,
    pub blue_moon_small_moon: Option<VoxelBodyParts>,
    pub blue_moon_soil_mound_count: usize,
    pub blue_moon_grass_tuft_count: usize,
    pub blue_moon_grass_blade_count: usize,
    pub blue_moon_garlic_parts: usize,
    pub blue_moon_rover_parts: usize,
    pub blue_moon_rover_wheel_count: usize,
    pub blue_moon_smoke_puff_count: usize,
    pub blue_moon_bubble_count: usize,
    pub blue_moon_bubble_content_parts: usize,
    pub blue_moon_floating_asteroid_count: usize,
    pub blue_moon_floating_asteroid_parts: usize,
    pub blue_moon_slingshot_wood_parts: usize,
    pub blue_moon_slingshot_joint_parts: usize,
    pub blue_moon_slingshot_band_parts: usize,
    pub blue_moon_atmosphere_id: Option<usize>,
    pub cookie_world_pig_id: Option<usize>,
    pub cookie_world_bubble_id: Option<usize>,
    pub cookie_world_pig_parts: usize,
    pub cookie_world_popcorn_parts: usize,
    pub cookie_world_cookie_count: usize,
    pub cookie_world_horn_parts: usize,
    pub cookie_world_slingshot_parts: usize,
    pub cookie_world_slingshot_cookie_id: Option<usize>,
    pub cookie_world_caramel_cone_parts: usize,
    pub cookie_world_popcorn_cluster_parts: usize,
    pub cookie_world_rock_parts: usize,
    pub cookie_world_back_decoration_parts: usize,
    pub cookie_world_bird_count: usize,
    pub cookie_world_bird_parts: usize,
    pub pig_count: usize,
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
            Self::CurvedTetrahedron(error) => {
                write!(formatter, "curved tetrahedron build error: {error:?}")
            }
            Self::Crystal(error) => write!(formatter, "crystal build error: {error:?}"),
            Self::OrientedBox(error) => write!(formatter, "oriented box build error: {error:?}"),
            Self::RadialFrame(error) => write!(formatter, "radial frame build error: {error:?}"),
            Self::Basis(error) => write!(formatter, "basis build error: {error:?}"),
            Self::Voxel(error) => write!(formatter, "voxel build error: {error}"),
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

impl From<CurvedTetrahedronError> for SpaceBuildError {
    fn from(error: CurvedTetrahedronError) -> Self {
        Self::CurvedTetrahedron(error)
    }
}

impl From<CrystalError> for SpaceBuildError {
    fn from(error: CrystalError) -> Self {
        Self::Crystal(error)
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

    scene.set_ambient_light(Color::new(0.200, 0.215, 0.270));
    // The sun key light goes first: it is the level's main outside light.
    blue_moon::add_outside_lighting(&mut scene);
    blue_moon::add_blue_moon_world(&mut scene, &mut metadata, materials)?;

    scene.build_bvh();

    Ok((scene, metadata))
}

pub fn build_cookie_world_scene() -> Result<Scene, SpaceBuildError> {
    build_cookie_world_scene_with_metadata().map(|built| built.0)
}

pub(crate) fn build_cookie_world_scene_with_metadata()
-> Result<(Scene, SpaceSceneMetadata), SpaceBuildError> {
    let (mut scene, materials) = base_space_scene_with_skybox(cookie_world_skybox()?)?;
    let mut metadata = SpaceSceneMetadata::default();

    scene.set_ambient_light(Color::new(0.300, 0.250, 0.320));
    let world = register_cookie_world_materials(&mut scene)?;
    add_cookie_world_bubble_pig(&mut scene, &mut metadata, materials, world)?;
    add_cookie_world_cookies(&mut scene, &mut metadata, materials, world)?;
    add_cookie_world_caramel_cones(&mut scene, &mut metadata, world)?;
    add_cookie_world_popcorn_and_rocks(&mut scene, &mut metadata, world)?;
    add_cookie_world_birds(&mut scene, &mut metadata, materials, world)?;
    add_cookie_world_lighting(&mut scene);

    scene.build_bvh();

    Ok((scene, metadata))
}

pub fn build_galaxy_selector_scene() -> Result<Scene, SpaceBuildError> {
    let (mut scene, materials) = base_space_scene_with_skybox(space_menu_skybox()?)?;

    scene.set_ambient_light(Color::new(0.360, 0.380, 0.440));
    add_selector_worlds(&mut scene, materials)?;
    add_selector_lighting(&mut scene);

    scene.build_bvh();

    Ok(scene)
}

pub fn build_level_three_scene() -> Result<Scene, SpaceBuildError> {
    build_level_three_scene_with_metadata().map(|built| built.0)
}

pub(crate) fn build_level_three_scene_with_metadata()
-> Result<(Scene, SpaceSceneMetadata), SpaceBuildError> {
    let (mut scene, materials) = base_space_scene_with_skybox(level_three_skybox()?)?;
    let mut metadata = SpaceSceneMetadata::default();

    add_level_three_asteroids(&mut scene, &mut metadata, materials)?;
    add_level_three_atmospheres(&mut scene, &mut metadata, materials)?;
    add_level_three_bridge(&mut scene, &mut metadata, materials)?;
    add_level_three_tower(&mut scene, &mut metadata, materials)?;
    add_level_three_lower_structure(&mut scene, &mut metadata, materials)?;
    add_level_three_rocks(&mut scene, &mut metadata, materials)?;
    add_level_three_slingshot_asteroid(&mut scene, &mut metadata, materials)?;
    add_level_three_lighting(&mut scene);

    scene.build_bvh();

    Ok((scene, metadata))
}

pub fn build_space_levels_scene() -> Result<Scene, SpaceBuildError> {
    build_space_levels_scene_with_metadata().map(|built| built.0)
}

pub(crate) fn build_space_levels_scene_with_metadata()
-> Result<(Scene, SpaceSceneMetadata), SpaceBuildError> {
    let (mut scene, materials) = base_space_scene()?;
    let mut metadata = SpaceSceneMetadata::default();

    blue_moon::add_blue_moon_world(&mut scene, &mut metadata, materials)?;
    add_cookie_level(&mut scene, &mut metadata, materials)?;
    add_space_lighting(&mut scene);
    add_cookie_level_lighting(&mut scene);

    scene.build_bvh();

    Ok((scene, metadata))
}

/// Level one is framed from the front, like the reference picture, and orbits
/// the center of the atmosphere that wraps the whole diorama.
pub fn blue_moon_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        blue_moon::BLUE_MOON_ATMOSPHERE_CENTER,
        0.0,
        0.10,
        7.8,
        50.0,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

pub fn cookie_world_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        COOKIE_WORLD_PIG_CENTER,
        0.0,
        0.06,
        8.9,
        55.0,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

pub fn level_three_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        LEVEL_THREE_CAMERA_TARGET,
        0.10,
        0.08,
        9.2,
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

pub fn galaxy_selector_worlds() -> [SelectorWorld; 4] {
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
        SelectorWorld {
            planet: PlanetType::CosmicCrystals,
            center: GALAXY_SELECTOR_LEVEL_FOUR_CENTER,
            radius: GALAXY_SELECTOR_LEVEL_FOUR_RADIUS,
            level_number: 4,
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
    Ok(Skybox::new(selector_skybox_texture()?)
        .with_intensity(1.18)
        .with_horizontal_rotation(0.0))
}

pub fn blue_moon_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(space_skybox_texture()?)
        .with_intensity(1.24)
        .with_horizontal_rotation(BLUE_MOON_SKYBOX_ROTATION))
}

/// Level two background: the Utopia/Steam parallax layers (purple bumpy
/// bands, clouds, pretzels and swirls) as a seamless quarter-turn tile.
pub fn cookie_world_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(utopia_skybox_texture()?)
        .with_intensity(1.04)
        .with_horizontal_rotation(0.05)
        .with_tiling(Vec2::new(COOKIE_WORLD_SKYBOX_TILES, 1.0)))
}

/// Level three background: spiked mines at three depths over a dark red
/// haze, from the Danger Zone parallax sprites, as a seamless quarter-turn
/// tile.
pub fn level_three_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(danger_zone_skybox_texture()?)
        .with_intensity(1.08)
        .with_horizontal_rotation(0.0)
        .with_tiling(Vec2::new(LEVEL_THREE_SKYBOX_TILES, 1.0)))
}

pub fn angry_birds_space_skybox() -> Result<Skybox, TextureError> {
    space_menu_skybox()
}

fn space_skybox_texture() -> Result<Texture, TextureError> {
    generate_space_skybox(space_skybox_color)
}

fn selector_skybox_texture() -> Result<Texture, TextureError> {
    generate_space_skybox(selector_skybox_color)
}

fn generate_space_skybox(color_at: fn(f32, f32) -> Color) -> Result<Texture, TextureError> {
    generate_skybox_texture(SPACE_SKYBOX_WIDTH, SPACE_SKYBOX_HEIGHT, color_at)
}

/// Equirectangular sky of `width` x `height` texels from a color per UV.
fn generate_skybox_texture(
    width: usize,
    height: usize,
    color_at: fn(f32, f32) -> Color,
) -> Result<Texture, TextureError> {
    let mut pixels = Vec::with_capacity(width * height);

    for y in 0..height {
        let v = 1.0 - y as f32 / (height - 1) as f32;

        for x in 0..width {
            let u = x as f32 / (width - 1) as f32;
            pixels.push(color_at(u, v));
        }
    }

    Texture::new(width, height, pixels)
}

fn selector_skybox_color(u: f32, v: f32) -> Color {
    let u = u.rem_euclid(1.0);
    let background = Color::new(0.010, 0.045, 0.130)
        .lerp(Color::new(0.020, 0.095, 0.230), smoothstep(0.0, 1.0, v));
    let mut color = add_cartoon_cloud_layers(background, u, v);
    let star = star_strength(u, v);
    color += Color::new(0.86, 0.96, 1.0) * star;
    color = add_background_asteroids(color, u, v);

    let distance = wrapped_uv_distance(u, v, SELECTOR_SUN_U, SELECTOR_SUN_V, SELECTOR_SUN_U_SCALE);
    let outer_glow = 1.0 - smoothstep(0.060, 0.17, distance);
    let inner_glow = 1.0 - smoothstep(0.045, 0.10, distance);
    color += Color::new(0.90, 0.48, 0.03) * (outer_glow * 0.28);
    color += Color::new(1.0, 0.78, 0.16) * (inner_glow * 0.50);

    let disk = 1.0 - smoothstep(0.067, 0.075, distance);
    let bands = smooth_noise(u * 28.0 + 3.0, v * 24.0 + 7.0) * 0.10;
    let sun = Color::new(1.0, 0.78 + bands, 0.20 + bands * 0.65);
    color.lerp(sun, disk).clamped()
}

fn selector_cookie_texture() -> Result<Texture, TextureError> {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 128;
    const CHIPS: [(f32, f32, f32); 8] = [
        (0.08, 0.32, 0.042),
        (0.24, 0.71, 0.052),
        (0.36, 0.42, 0.037),
        (0.49, 0.82, 0.044),
        (0.61, 0.23, 0.048),
        (0.72, 0.58, 0.041),
        (0.85, 0.76, 0.053),
        (0.94, 0.39, 0.036),
    ];
    let mut pixels = Vec::with_capacity(WIDTH * HEIGHT);

    for y in 0..HEIGHT {
        let v = y as f32 / (HEIGHT - 1) as f32;
        for x in 0..WIDTH {
            let u = x as f32 / (WIDTH - 1) as f32;
            let baked = smooth_noise(u * 9.0, v * 6.0) * 0.10;
            let mut color = Color::new(0.88 - baked, 0.59 - baked * 0.65, 0.30 - baked * 0.40);

            for (chip_u, chip_v, radius) in CHIPS {
                let du = ((u - chip_u + 0.5).rem_euclid(1.0) - 0.5) * 0.75;
                let dv = v - chip_v;
                let distance = (du * du + dv * dv).sqrt();
                let chip = 1.0 - smoothstep(radius * 0.75, radius, distance);
                color = color.lerp(Color::new(0.24, 0.11, 0.06), chip);
            }
            pixels.push(color);
        }
    }

    Texture::new(WIDTH, HEIGHT, pixels)
}

fn danger_zone_skybox_texture() -> Result<Texture, TextureError> {
    Texture::from_ppm_file(DANGER_ZONE_SKYBOX_TEXTURE_PATH)
}

fn utopia_skybox_texture() -> Result<Texture, TextureError> {
    Texture::from_ppm_file(UTOPIA_SKYBOX_TEXTURE_PATH)
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

    // Distance to the sun center in sun sizes.
    let sun_distance = wrapped_uv_distance(u, v, SPACE_SUN_U, SPACE_SUN_V, 1.18) / SPACE_SUN_SIZE;
    let broad_halo = 1.0 - smoothstep(0.13, 0.46, sun_distance);
    let warm_halo = 1.0 - smoothstep(0.065, 0.290, sun_distance);
    color += Color::new(0.070, 0.260, 0.310) * (broad_halo * 0.50);
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

    let spot = |offset_u: f32, offset_v: f32| {
        wrapped_uv_distance(
            u,
            v,
            SPACE_SUN_U + offset_u * SPACE_SUN_SIZE,
            SPACE_SUN_V + offset_v * SPACE_SUN_SIZE,
            1.35,
        ) / SPACE_SUN_SIZE
    };
    let spot_a = 1.0 - smoothstep(0.014, 0.034, spot(-0.025, 0.020));
    let spot_b = 1.0 - smoothstep(0.011, 0.028, spot(0.020, -0.035));
    let spot_c = 1.0 - smoothstep(0.009, 0.023, spot(0.045, 0.042));
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
    scene.add_sphere(Sphere::new(
        GALAXY_SELECTOR_LEVEL_FOUR_CENTER,
        GALAXY_SELECTOR_LEVEL_FOUR_RADIUS * 1.16,
        materials.level_four_gravity_field,
    )?)?;
    scene.add_sphere(Sphere::new(
        GALAXY_SELECTOR_LEVEL_FOUR_CENTER,
        GALAXY_SELECTOR_LEVEL_FOUR_RADIUS,
        materials.selector_locked_level_four,
    )?)?;

    Ok(())
}

fn register_space_materials(scene: &mut Scene) -> Result<SpaceMaterials, SpaceBuildError> {
    let moon_texture = scene.add_texture(Texture::from_ppm_file(BLUE_MOON_PLANET_TEXTURE_PATH)?);
    let selector_cookie_texture = scene.add_texture(selector_cookie_texture()?);
    let wood_block_texture = scene.add_texture(Texture::from_ppm_file(WOOD_BLOCK_TEXTURE_PATH)?);
    let tnt_crate_texture = scene.add_texture(Texture::from_ppm_file(TNT_CRATE_TEXTURE_PATH)?);
    let asteroid_texture = scene.add_texture(Texture::from_ppm_file(ASTEROID_PLANET_TEXTURE_PATH)?);
    let ice_block_texture = scene.add_texture(Texture::from_ppm_file(ICE_BLOCK_TEXTURE_PATH)?);
    let gray_stone_block_texture =
        scene.add_texture(Texture::from_ppm_file(GRAY_STONE_BLOCK_TEXTURE_PATH)?);
    let selector_crystal_texture = scene.add_texture(cosmic_crystals::crystal_planet_texture(
        SELECTOR_CRYSTAL_TEXTURE_WIDTH,
        SELECTOR_CRYSTAL_TEXTURE_HEIGHT,
    )?);

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
    let level_three_asteroid = scene.add_material(
        Material::new(
            Color::new(0.64, 0.60, 0.56),
            0.20,
            24.0,
            0.025,
            0.0,
            1.0,
            Color::new(0.018, 0.016, 0.014),
        )
        .with_texture(asteroid_texture, Vec2::new(1.0, 1.0), WrapMode::Clamp),
    )?;
    let level_three_stone = scene.add_material(
        Material::new(
            Color::new(0.80, 0.78, 0.78),
            0.25,
            28.0,
            0.02,
            0.0,
            1.0,
            Color::new(0.050, 0.046, 0.046),
        )
        .with_texture(
            gray_stone_block_texture,
            Vec2::new(1.2, 1.2),
            WrapMode::Repeat,
        ),
    )?;
    // Slightly transparent, refractive ice with a glossy highlight.
    let ice = scene.add_material(
        Material::new(
            Color::new(0.92, 1.0, 1.0),
            0.70,
            64.0,
            0.06,
            0.28,
            1.31,
            Color::new(0.040, 0.070, 0.080),
        )
        .with_texture(ice_block_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    let level_three_gravity_field = scene.add_material(Material::new(
        Color::new(0.78, 0.42, 1.0),
        0.10,
        48.0,
        0.012,
        0.982,
        1.005,
        Color::new(0.060, 0.018, 0.110),
    ))?;
    // Level three atmospheres: translucent red, without refraction so the
    // structures inside are not distorted.
    let level_three_atmosphere = scene.add_material(Material::new(
        Color::new(0.95, 0.30, 0.28),
        0.10,
        30.0,
        0.0,
        0.88,
        1.0,
        Color::new(0.110, 0.020, 0.018),
    ))?;
    let level_four_gravity_field = scene.add_material(Material::new(
        Color::new(1.0, 0.58, 0.92),
        0.10,
        48.0,
        0.012,
        0.982,
        1.005,
        Color::new(0.100, 0.030, 0.090),
    ))?;
    let selector_locked_moon = scene.add_material(
        Material::new(
            Color::new(0.82, 0.94, 1.0),
            0.18,
            24.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.10, 0.14, 0.18),
        )
        .with_texture(moon_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    let selector_locked_cookie = scene.add_material(
        Material::new(
            Color::new(1.0, 0.86, 0.68),
            0.22,
            24.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.10, 0.055, 0.02),
        )
        .with_texture(
            selector_cookie_texture,
            Vec2::new(1.0, 1.0),
            WrapMode::Repeat,
        ),
    )?;
    let selector_locked_level_three = scene.add_material(
        Material::new(
            Color::new(0.82, 0.75, 0.98),
            0.20,
            26.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.15, 0.11, 0.21),
        )
        .with_texture(moon_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    let selector_locked_level_four = scene.add_material(
        Material::new(
            Color::new(1.0, 0.84, 1.0),
            0.20,
            26.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.30, 0.15, 0.30),
        )
        .with_texture(
            selector_crystal_texture,
            Vec2::new(1.0, 1.0),
            WrapMode::Repeat,
        ),
    )?;
    let cookie = scene.add_material(Material::new(
        Color::new(0.86, 0.56, 0.26),
        0.18,
        22.0,
        0.03,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let wood = scene.add_material(
        Material::new(
            Color::new(0.96, 0.74, 0.54),
            0.25,
            24.0,
            0.02,
            0.0,
            1.0,
            Color::new(0.050, 0.026, 0.010),
        )
        .with_texture(wood_block_texture, Vec2::new(2.4, 1.1), WrapMode::Repeat),
    )?;
    let slingshot_wood = scene.add_material(
        Material::new(
            Color::new(1.0, 0.86, 0.70),
            0.30,
            30.0,
            0.02,
            0.0,
            1.0,
            Color::new(0.075, 0.038, 0.014),
        )
        .with_texture(wood_block_texture, Vec2::new(1.0, 0.30), WrapMode::Repeat),
    )?;
    let slingshot_band = scene.add_material(Material::new(
        Color::new(0.30, 0.16, 0.09),
        0.35,
        36.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.012, 0.006, 0.003),
    ))?;
    // Character materials keep a little emission so they stay readable on the
    // shadowed side of the planet, and a soft glossy highlight.
    let pig = scene.add_material(Material::new(
        Color::new(0.47, 0.90, 0.30),
        0.55,
        48.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.070, 0.150, 0.040),
    ))?;
    let snout = scene.add_material(Material::new(
        Color::new(0.74, 0.97, 0.50),
        0.45,
        36.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.120, 0.180, 0.060),
    ))?;
    let eye = scene.add_material(Material::new(
        Color::WHITE,
        0.60,
        64.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.450, 0.450, 0.450),
    ))?;
    let pupil = scene.add_material(Material::new(
        Color::new(0.02, 0.02, 0.02),
        0.90,
        96.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
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
        gravity_field,
        cookie_gravity_field,
        level_three_asteroid,
        level_three_stone,
        ice,
        level_three_gravity_field,
        level_three_atmosphere,
        level_four_gravity_field,
        selector_locked_moon,
        selector_locked_cookie,
        selector_locked_level_three,
        selector_locked_level_four,
        cookie,
        wood,
        slingshot_wood,
        slingshot_band,
        tnt_crate,
        pig,
        snout,
        eye,
        pupil,
    })
}

/// Number of primitives of each kind in one slingshot.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SlingshotParts {
    wood: usize,
    joints: usize,
    bands: usize,
}

impl SlingshotParts {
    fn total(self) -> usize {
        self.wood + self.joints + self.bands
    }
}

/// Y-shaped slingshot in `frame`, scaled by `scale` and turned by
/// `yaw_radians` around the frame's radial axis.
fn add_slingshot(
    scene: &mut Scene,
    frame: RadialFrame,
    yaw_radians: f32,
    scale: f32,
    materials: SpaceMaterials,
) -> Result<SlingshotParts, SpaceBuildError> {
    add_slingshot_parts(scene, frame, yaw_radians, scale, materials, true)
}

/// Slingshot without the elastic bands and the pouch, for a level that moves
/// them with the loaded bird.
fn add_slingshot_without_elastic(
    scene: &mut Scene,
    frame: RadialFrame,
    yaw_radians: f32,
    scale: f32,
    materials: SpaceMaterials,
) -> Result<SlingshotParts, SpaceBuildError> {
    add_slingshot_parts(scene, frame, yaw_radians, scale, materials, false)
}

/// A point of the slingshot model (the `BLUE_MOON_SLINGSHOT_*` points) in the
/// local coordinates of its radial frame, with the trunk planted at the frame
/// origin.
fn slingshot_local_point(point: Vec3, yaw_radians: f32, scale: f32) -> Vec3 {
    turned_about_radial_axis(
        (point - Vec3::new(0.0, BLUE_MOON_SLINGSHOT_TRUNK_BASE_Y, 0.0)) * scale,
        yaw_radians,
    )
}

fn add_slingshot_parts(
    scene: &mut Scene,
    frame: RadialFrame,
    yaw_radians: f32,
    scale: f32,
    materials: SpaceMaterials,
    elastic: bool,
) -> Result<SlingshotParts, SpaceBuildError> {
    let fork = BLUE_MOON_SLINGSHOT_FORK;
    // The trunk is planted at the frame origin, directly in the ground.
    let turn = |point: Vec3| slingshot_local_point(point, yaw_radians, scale);
    let mut parts = SlingshotParts::default();

    add_radial_segment(
        scene,
        frame,
        turn(Vec3::new(fork.x, BLUE_MOON_SLINGSHOT_TRUNK_BASE_Y, fork.z)),
        turn(fork),
        BLUE_MOON_SLINGSHOT_TRUNK_RADIUS * scale,
        materials.slingshot_wood,
    )?;
    parts.wood += 1;

    scene.add_sphere(Sphere::new(
        frame.local_to_world(turn(fork)),
        BLUE_MOON_SLINGSHOT_FORK_RADIUS * scale,
        materials.slingshot_wood,
    )?)?;
    parts.joints += 1;

    for side in [-1.0, 1.0] {
        let elbow = mirrored_x(BLUE_MOON_SLINGSHOT_ELBOW, side);
        let tip = mirrored_x(BLUE_MOON_SLINGSHOT_TIP, side);

        for (start, end, radius) in [
            (fork, elbow, BLUE_MOON_SLINGSHOT_ARM_RADIUS),
            (elbow, tip, BLUE_MOON_SLINGSHOT_PRONG_RADIUS),
        ] {
            add_radial_segment(
                scene,
                frame,
                turn(start),
                turn(end),
                radius * scale,
                materials.slingshot_wood,
            )?;
            parts.wood += 1;
        }

        for (center, radius) in [
            (elbow, BLUE_MOON_SLINGSHOT_ARM_RADIUS),
            (tip, BLUE_MOON_SLINGSHOT_PRONG_RADIUS),
        ] {
            scene.add_sphere(Sphere::new(
                frame.local_to_world(turn(center)),
                radius * scale,
                materials.slingshot_wood,
            )?)?;
            parts.joints += 1;
        }

        // Leather wrap around the prong, a little wider than the stick.
        let prong = tip - elbow;
        add_radial_segment(
            scene,
            frame,
            turn(elbow + prong * 0.48),
            turn(elbow + prong * 0.86),
            BLUE_MOON_SLINGSHOT_WRAP_RADIUS * scale,
            materials.slingshot_band,
        )?;
        parts.bands += 1;

        if !elastic {
            continue;
        }

        // Elastic band from the wrap to the side of the pouch.
        add_radial_segment(
            scene,
            frame,
            turn(elbow + prong * 0.67 + Vec3::new(0.0, 0.0, BLUE_MOON_SLINGSHOT_WRAP_RADIUS * 0.6)),
            turn(
                BLUE_MOON_SLINGSHOT_POUCH
                    + Vec3::new(side * BLUE_MOON_SLINGSHOT_POUCH_HALF_WIDTH, 0.0, 0.0),
            ),
            BLUE_MOON_SLINGSHOT_ELASTIC_RADIUS * scale,
            materials.slingshot_band,
        )?;
        parts.bands += 1;
    }

    if elastic {
        add_radial_segment(
            scene,
            frame,
            turn(
                BLUE_MOON_SLINGSHOT_POUCH
                    - Vec3::new(BLUE_MOON_SLINGSHOT_POUCH_HALF_WIDTH, 0.0, 0.0),
            ),
            turn(
                BLUE_MOON_SLINGSHOT_POUCH
                    + Vec3::new(BLUE_MOON_SLINGSHOT_POUCH_HALF_WIDTH, 0.0, 0.0),
            ),
            BLUE_MOON_SLINGSHOT_POUCH_RADIUS * scale,
            materials.slingshot_band,
        )?;
        parts.bands += 1;
    }

    Ok(parts)
}

/// Rotates a point given in a radial frame around the frame's radial (Y)
/// axis. A positive angle turns local +X towards local -Z.
fn turned_about_radial_axis(point: Vec3, angle_radians: f32) -> Vec3 {
    let (sin_angle, cos_angle) = angle_radians.sin_cos();

    Vec3::new(
        point.x * cos_angle + point.z * sin_angle,
        point.y,
        point.z * cos_angle - point.x * sin_angle,
    )
}

fn mirrored_x(point: Vec3, side: f32) -> Vec3 {
    Vec3::new(point.x * side, point.y, point.z)
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

    let gravity_id = scene.object_count();
    scene.add_sphere(Sphere::new(
        COOKIE_PLANET_CENTER,
        COOKIE_PLANET_RADIUS * 1.19,
        materials.cookie_gravity_field,
    )?)?;
    metadata.cookie_gravity_field_id = Some(gravity_id);

    Ok(())
}

/// Skin spots of the level two pig in its local frame (X right, Y up, Z
/// towards the viewer): direction and angular radius in radians.
const PIG_SKIN_SPOTS: [(Vec3, f32); 8] = [
    (Vec3::new(0.30, 0.62, 0.72), 0.20),
    (Vec3::new(0.54, 0.44, 0.72), 0.13),
    (Vec3::new(-0.80, 0.12, 0.58), 0.21),
    (Vec3::new(-0.68, 0.34, 0.65), 0.12),
    (Vec3::new(-0.30, 0.84, 0.46), 0.11),
    (Vec3::new(0.86, -0.30, 0.42), 0.15),
    (Vec3::new(0.20, 0.35, -0.92), 0.26),
    (Vec3::new(-0.62, -0.38, -0.69), 0.20),
];

/// Snout of the level two pig: two overlapping discs make a wide oval. Both
/// discs share the snout material and the same front plane, so the overlap
/// shades exactly like a single face (no visible z-fighting and no seam).
const PIG_SNOUT_CENTER: Vec3 = Vec3::new(0.0, -0.20, 0.0);
const PIG_SNOUT_LOBE_OFFSET: f32 = 0.08;
const PIG_SNOUT_RADIUS: f32 = 0.27;
const PIG_SNOUT_HALF_DEPTH: f32 = 0.16;
/// Distance from the pig center to the front face of the snout.
const PIG_SNOUT_FRONT: f32 = 1.06;
const PIG_NOSTRIL_OFFSET_X: f32 = 0.12;
const PIG_NOSTRIL_RADIUS: f32 = 0.065;
const PIG_NOSTRIL_HALF_DEPTH: f32 = 0.02;
/// How far the nostrils stick out of the snout face.
const PIG_NOSTRIL_LIFT: f32 = 0.008;
const PIG_EYE_DIRECTION: Vec3 = Vec3::new(0.55, 0.05, 0.83);
const PIG_EYE_DISTANCE: f32 = 0.87;
const PIG_EYE_RADIUS: f32 = 0.22;
const PIG_PUPIL_RADIUS: f32 = 0.09;
/// Both pupils look up and towards the viewer's right.
const PIG_GAZE_OFFSET: Vec3 = Vec3::new(0.34, 0.24, 0.0);
const PIG_PUPIL_DISTANCE: f32 = 0.155;
const PIG_EAR_DIRECTION: Vec3 = Vec3::new(0.40, 0.90, 0.10);
const PIG_EAR_DISTANCE: f32 = 0.94;
const PIG_EAR_RADIUS: f32 = 0.17;
/// Popcorn stuck on the right side of the pig's head: offset from the anchor
/// point (in pig radii), radius and whether it is the golden kernel.
const PIG_POPCORN_ANCHOR: Vec3 = Vec3::new(0.78, 0.52, 0.34);
const PIG_POPCORN_PIECES: [(Vec3, f32, bool); 5] = [
    (Vec3::new(0.00, 0.00, 0.00), 0.140, false),
    (Vec3::new(0.12, 0.09, 0.05), 0.115, false),
    (Vec3::new(-0.08, 0.13, 0.06), 0.110, false),
    (Vec3::new(0.05, -0.08, 0.12), 0.095, false),
    (Vec3::new(0.03, 0.06, 0.16), 0.055, true),
];

fn cookie_world_pig_basis() -> Result<Basis3, Basis3Error> {
    Basis3::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), COOKIE_WORLD_PIG_ROLL_RADIANS)
}

/// Level two centerpiece: a big spotted pig with popcorn stuck to its head,
/// floating inside a translucent gravity bubble.
fn add_cookie_world_bubble_pig(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
    world: CookieWorldMaterials,
) -> Result<(), SpaceBuildError> {
    let basis = cookie_world_pig_basis()?;
    let radius = COOKIE_WORLD_PIG_RADIUS;
    let at = |local: Vec3| COOKIE_WORLD_PIG_CENTER + basis.local_to_world_vector(local * radius);
    let first_part = scene.object_count();

    metadata.cookie_world_pig_id = Some(scene.object_count());
    scene.add_sphere(Sphere::new(
        COOKIE_WORLD_PIG_CENTER,
        radius,
        world.pig_skin,
    )?)?;

    let snout_basis = facing_basis(basis)?;
    for side in [-1.0, 1.0] {
        scene.add_cylinder(Cylinder::new(
            at(PIG_SNOUT_CENTER
                + Vec3::new(
                    side * PIG_SNOUT_LOBE_OFFSET,
                    0.0,
                    PIG_SNOUT_FRONT - PIG_SNOUT_HALF_DEPTH,
                )),
            PIG_SNOUT_RADIUS * radius,
            PIG_SNOUT_HALF_DEPTH * radius,
            snout_basis,
            world.pig_snout,
        )?)?;
    }

    let nostril_front = PIG_SNOUT_FRONT + PIG_NOSTRIL_LIFT;
    for side in [-1.0, 1.0] {
        scene.add_cylinder(Cylinder::new(
            at(PIG_SNOUT_CENTER
                + Vec3::new(
                    side * PIG_NOSTRIL_OFFSET_X,
                    0.0,
                    nostril_front - PIG_NOSTRIL_HALF_DEPTH,
                )),
            PIG_NOSTRIL_RADIUS * radius,
            PIG_NOSTRIL_HALF_DEPTH * radius,
            snout_basis,
            world.pig_nostril,
        )?)?;
    }

    for side in [-1.0, 1.0] {
        let eye_direction = mirrored_x(PIG_EYE_DIRECTION, side).normalized();
        let eye_center = eye_direction * PIG_EYE_DISTANCE;
        let gaze = (eye_direction + PIG_GAZE_OFFSET).normalized();

        scene.add_sphere(Sphere::new(
            at(eye_center),
            PIG_EYE_RADIUS * radius,
            materials.eye,
        )?)?;
        scene.add_sphere(Sphere::new(
            at(eye_center + gaze * PIG_PUPIL_DISTANCE),
            PIG_PUPIL_RADIUS * radius,
            materials.pupil,
        )?)?;
    }

    for side in [-1.0, 1.0] {
        let ear_direction = mirrored_x(PIG_EAR_DIRECTION, side).normalized();
        scene.add_sphere(Sphere::new(
            at(ear_direction * PIG_EAR_DISTANCE),
            PIG_EAR_RADIUS * radius,
            world.pig_ear,
        )?)?;
    }
    metadata.cookie_world_pig_parts = scene.object_count() - first_part;
    metadata.pig_count += 1;

    let anchor = PIG_POPCORN_ANCHOR.normalized();
    for (offset, piece_radius, kernel) in PIG_POPCORN_PIECES {
        let material = if kernel {
            world.popcorn_kernel
        } else {
            world.popcorn
        };
        scene.add_sphere(Sphere::new(
            at(anchor + offset),
            piece_radius * radius,
            material,
        )?)?;
        metadata.cookie_world_popcorn_parts += 1;
    }

    metadata.cookie_world_bubble_id = Some(scene.object_count());
    scene.add_sphere(Sphere::new(
        COOKIE_WORLD_PIG_CENTER,
        COOKIE_WORLD_BUBBLE_RADIUS,
        world.bubble,
    )?)?;

    Ok(())
}

fn register_cookie_world_materials(
    scene: &mut Scene,
) -> Result<CookieWorldMaterials, SpaceBuildError> {
    let pig_skin_texture = scene.add_texture(pig_skin_texture()?);
    // Level two bubble: almost clear, slightly bluish and brighter where the
    // outside lights graze its silhouette.
    let bubble = scene.add_material(Material::new(
        Color::new(0.80, 0.88, 1.0),
        0.15,
        60.0,
        0.0,
        0.89,
        1.0,
        Color::new(0.050, 0.040, 0.080),
    ))?;
    // The big pig: white albedo so the spotted skin texture gives its color.
    let pig_skin = scene.add_material(
        Material::new(
            Color::WHITE,
            0.45,
            40.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.045, 0.095, 0.030),
        )
        .with_texture(pig_skin_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    let pig_ear = scene.add_material(Material::new(
        PIG_SKIN_BASE_COLOR,
        0.40,
        36.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.045, 0.095, 0.030),
    ))?;
    let pig_snout = scene.add_material(Material::new(
        Color::new(0.70, 0.94, 0.20),
        0.40,
        32.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.080, 0.110, 0.020),
    ))?;
    let pig_nostril = scene.add_material(Material::new(
        Color::new(0.16, 0.34, 0.03),
        0.10,
        12.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ))?;
    let popcorn = scene.add_material(Material::new(
        Color::new(1.0, 0.96, 0.82),
        0.30,
        24.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.120, 0.110, 0.080),
    ))?;
    let popcorn_kernel = scene.add_material(Material::new(
        Color::new(0.98, 0.74, 0.26),
        0.35,
        28.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.090, 0.060, 0.015),
    ))?;

    let mut cookies = [0; COOKIE_WORLD_COOKIE_COUNT];
    for (slot, cookie) in cookies.iter_mut().zip(COOKIE_WORLD_COOKIES) {
        let texture = scene.add_texture(cookie_texture(cookie.seed)?);
        *slot = scene.add_material(
            Material::new(
                Color::WHITE,
                0.05,
                10.0,
                0.0,
                0.0,
                1.0,
                Color::new(0.100, 0.060, 0.025),
            )
            .with_texture(texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
        )?;
    }
    let horn = scene.add_material(Material::new(
        Color::new(0.98, 0.70, 0.36),
        0.30,
        22.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.080, 0.045, 0.015),
    ))?;
    let horn_opening = scene.add_material(Material::new(
        Color::new(0.30, 0.13, 0.05),
        0.05,
        8.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.030, 0.012, 0.004),
    ))?;

    let waffle_texture = scene.add_texture(waffle_texture()?);
    let waffle_cone = scene.add_material(
        Material::new(
            Color::WHITE,
            0.25,
            20.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.070, 0.040, 0.012),
        )
        .with_texture(waffle_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    // Glossy caramel: strong highlight and a little mirror so it looks wet.
    let caramel = scene.add_material(Material::new(
        Color::new(0.64, 0.27, 0.05),
        1.0,
        110.0,
        0.22,
        0.0,
        1.0,
        Color::new(0.075, 0.026, 0.004),
    ))?;
    let rock_texture = scene.add_texture(rock_texture()?);
    let rock = scene.add_material(
        Material::new(
            Color::WHITE,
            0.15,
            18.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.060, 0.064, 0.074),
        )
        .with_texture(rock_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    // Rocks far behind the bubble get almost no light (a light there would
    // also light the back of the bubble and cloud it), so they glow a bit.
    let far_rock = scene.add_material(
        Material::new(
            Color::new(0.85, 0.85, 0.90),
            0.10,
            14.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.200, 0.190, 0.230),
        )
        .with_texture(rock_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    // Birds keep a little emission, like the other characters, so they stay
    // readable away from the lights.
    let bird_material = |scene: &mut Scene, color: Color| {
        scene.add_material(Material::new(
            color,
            0.45,
            40.0,
            0.0,
            0.0,
            1.0,
            color * 0.12,
        ))
    };
    let mut bird_body = [0; COOKIE_WORLD_BIRD_COUNT];
    let mut bird_belly = [0; COOKIE_WORLD_BIRD_COUNT];
    let mut bird_feather = [0; COOKIE_WORLD_BIRD_COUNT];
    for (index, (body, belly, feather)) in SONGBIRD_PALETTES.into_iter().enumerate() {
        bird_body[index] = bird_material(scene, body)?;
        bird_belly[index] = bird_material(scene, belly)?;
        bird_feather[index] = bird_material(scene, feather)?;
    }
    let bird_beak = bird_material(scene, Color::new(1.0, 0.74, 0.18))?;
    let bird_feet = bird_material(scene, Color::new(0.95, 0.55, 0.12))?;

    Ok(CookieWorldMaterials {
        bubble,
        pig_skin,
        pig_ear,
        pig_snout,
        pig_nostril,
        popcorn,
        popcorn_kernel,
        cookies,
        horn_waffle: horn,
        horn_opening,
        waffle_cone,
        caramel,
        rock,
        far_rock,
        bird_body,
        bird_belly,
        bird_feather,
        bird_beak,
        bird_feet,
    })
}

/// Cookie planet of level two: a sphere with a procedural chocolate chip
/// texture.
#[derive(Debug, Clone, Copy)]
struct CookiePlanet {
    center: Vec3,
    radius: f32,
    seed: f32,
}

/// Ribbed waffle horn resting against a cookie: it sits on the cookie's
/// `side`, its open mouth looks along `mouth_facing` and its tail curls
/// towards `curl_towards`.
#[derive(Debug, Clone, Copy)]
struct WaffleHorn {
    cookie: usize,
    side: Vec3,
    mouth_facing: Vec3,
    curl_towards: Vec3,
    scale: f32,
}

/// Cookies around the bubble: two with a waffle horn and the one on the left
/// with the slingshot on top.
fn add_cookie_world_cookies(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
    world: CookieWorldMaterials,
) -> Result<(), SpaceBuildError> {
    for (index, (cookie, material)) in COOKIE_WORLD_COOKIES.iter().zip(world.cookies).enumerate() {
        if index == COOKIE_WORLD_SLINGSHOT_COOKIE {
            metadata.cookie_world_slingshot_cookie_id = Some(scene.object_count());
        }
        scene.add_sphere(Sphere::new(cookie.center, cookie.radius, material)?)?;
        metadata.cookie_world_cookie_count += 1;
    }

    for horn in COOKIE_WORLD_HORNS {
        metadata.cookie_world_horn_parts += add_waffle_horn(scene, horn, world)?;
    }

    let slingshot_cookie = COOKIE_WORLD_COOKIES[COOKIE_WORLD_SLINGSHOT_COOKIE];
    let frame = RadialFrame::from_normal(
        slingshot_cookie.center,
        slingshot_cookie.radius - COOKIE_WORLD_SLINGSHOT_EMBED,
        COOKIE_WORLD_SLINGSHOT_DIRECTION,
    )?;
    let parts = add_slingshot(
        scene,
        frame,
        COOKIE_WORLD_SLINGSHOT_YAW_RADIANS,
        COOKIE_WORLD_SLINGSHOT_SCALE,
        materials,
    )?;
    metadata.cookie_world_slingshot_parts += parts.total();

    Ok(())
}

/// Adds one waffle horn and returns how many primitives it used.
fn add_waffle_horn(
    scene: &mut Scene,
    horn: WaffleHorn,
    world: CookieWorldMaterials,
) -> Result<usize, SpaceBuildError> {
    let cookie = COOKIE_WORLD_COOKIES[horn.cookie];
    let up = horn.mouth_facing.normalized();
    let right = (horn.curl_towards - up * horn.curl_towards.dot(up)).normalized();
    let basis = Basis3::new(right, up, right.cross(up))?;
    let ribs = waffle_horn_ribs();
    let side = horn.side.normalized();
    // Start well away from the cookie and slide towards it until the closest
    // rib sinks a little into the dough.
    let mut mouth = cookie.center + side * (cookie.radius + horn.scale);
    for _ in 0..WAFFLE_HORN_CONTACT_STEPS {
        let closest_gap = ribs
            .iter()
            .chain(std::iter::once(&WAFFLE_HORN_MOUTH_BOUND))
            .map(|&(center, radius)| {
                let rib = mouth + basis.local_to_world_vector(center * horn.scale);
                (rib - cookie.center).length() - cookie.radius - radius * horn.scale
            })
            .fold(f32::INFINITY, f32::min);
        mouth -= side * (closest_gap + WAFFLE_HORN_CONTACT_DEPTH * horn.scale);
    }
    let at = |local: Vec3| mouth + basis.local_to_world_vector(local * horn.scale);
    let first_part = scene.object_count();

    // The cone's apex points down the horn, so its base is the open mouth.
    scene.add_cone(Cone::new(
        at(Vec3::new(0.0, -WAFFLE_HORN_MOUTH_HALF_DEPTH, 0.0)),
        WAFFLE_HORN_MOUTH_RADIUS * horn.scale,
        WAFFLE_HORN_MOUTH_HALF_DEPTH * horn.scale,
        Basis3::new(right, -up, -right.cross(up))?,
        world.horn_waffle,
    )?)?;
    // Dark inside of the horn, just over the mouth plane.
    scene.add_cylinder(Cylinder::new(
        at(Vec3::new(0.0, WAFFLE_HORN_OPENING_HALF_DEPTH * 0.5, 0.0)),
        WAFFLE_HORN_OPENING_RADIUS * horn.scale,
        WAFFLE_HORN_OPENING_HALF_DEPTH * horn.scale,
        basis,
        world.horn_opening,
    )?)?;
    for (center, radius) in ribs {
        scene.add_sphere(Sphere::new(
            at(center),
            radius * horn.scale,
            world.horn_waffle,
        )?)?;
    }

    Ok(scene.object_count() - first_part)
}

/// Centers and radii of the rib spheres of a waffle horn in its local frame:
/// the mouth is at the origin facing +Y, the body goes down and its tail
/// curls towards +X with a shrinking spiral. The last rib is the tail tip.
fn waffle_horn_ribs() -> Vec<(Vec3, f32)> {
    let mut ribs: Vec<(Vec3, f32)> = WAFFLE_HORN_STRAIGHT_RIBS
        .iter()
        .map(|&(y, radius)| (Vec3::new(0.0, y, 0.0), radius))
        .collect();
    let (start_y, start_radius) = WAFFLE_HORN_STRAIGHT_RIBS[WAFFLE_HORN_STRAIGHT_RIBS.len() - 1];
    let curl_center = Vec3::new(WAFFLE_HORN_CURL_RADIUS, start_y, 0.0);
    let total_angle = WAFFLE_HORN_CURL_TURNS * std::f32::consts::TAU;
    let mut angle = 0.0;
    let mut radius = start_radius;

    loop {
        let curl = WAFFLE_HORN_CURL_RADIUS * (1.0 - WAFFLE_HORN_CURL_SHRINK * angle / total_angle);
        let step = WAFFLE_HORN_RIB_SPACING * radius / curl.max(0.01);
        angle += step;
        if angle > total_angle {
            break;
        }

        let fraction = angle / total_angle;
        let curl = WAFFLE_HORN_CURL_RADIUS * (1.0 - WAFFLE_HORN_CURL_SHRINK * fraction);
        radius = start_radius + (WAFFLE_HORN_TIP_RADIUS - start_radius) * fraction;
        // Angle zero is the straight part's end, on the -X side of the curl.
        let direction = std::f32::consts::PI + angle;
        ribs.push((
            curl_center + Vec3::new(curl * direction.cos(), curl * direction.sin(), 0.0),
            radius,
        ));
    }

    ribs
}

/// Lone waffle cone with a caramel top. `top` is the center of its open
/// mouth, which looks along `facing`; the tip is `length` behind it.
#[derive(Debug, Clone, Copy)]
struct CaramelCone {
    top: Vec3,
    facing: Vec3,
    length: f32,
    radius: f32,
}

fn add_cookie_world_caramel_cones(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    world: CookieWorldMaterials,
) -> Result<(), SpaceBuildError> {
    let first_part = scene.object_count();

    for cone in COOKIE_WORLD_CARAMEL_CONES {
        let facing = cone.facing.normalized();
        let half_length = cone.length * 0.5;
        // The cone's apex points away from the caramel, so its base is the mouth.
        scene.add_cone(Cone::new(
            cone.top - facing * half_length,
            cone.radius,
            half_length,
            basis_with_up(-facing)?,
            world.waffle_cone,
        )?)?;
        scene.add_sphere(Sphere::new(
            cone.top - facing * (cone.radius * CARAMEL_DOME_SINK),
            cone.radius * CARAMEL_DOME_RADIUS,
            world.caramel,
        )?)?;

        let around = basis_with_up(facing)?;
        for (angle, reach) in CARAMEL_DRIPS {
            let outward = around.right() * angle.cos() + around.forward() * angle.sin();
            // Each drip is a fat drop under the rim and a thinner one lower.
            for (depth, drop_radius) in [(CARAMEL_DRIP_START, 0.17), (reach, 0.11)] {
                let distance = depth * cone.length;
                let cone_radius = cone.radius * (1.0 - distance / cone.length);
                scene.add_sphere(Sphere::new(
                    cone.top - facing * distance + outward * cone_radius,
                    cone.radius * drop_radius,
                    world.caramel,
                )?)?;
            }
        }
    }
    metadata.cookie_world_caramel_cone_parts = scene.object_count() - first_part;

    Ok(())
}

/// Popcorn clusters and small lumpy asteroids floating around the bubble.
fn add_cookie_world_popcorn_and_rocks(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    world: CookieWorldMaterials,
) -> Result<(), SpaceBuildError> {
    let first_popcorn = scene.object_count();
    add_popcorn_clusters(scene, &COOKIE_WORLD_POPCORN, world)?;
    metadata.cookie_world_popcorn_cluster_parts = scene.object_count() - first_popcorn;

    let first_rock = scene.object_count();
    add_lumpy_rocks(scene, &COOKIE_WORLD_ROCKS, 0.0, world.rock)?;
    metadata.cookie_world_rock_parts = scene.object_count() - first_rock;

    let first_back = scene.object_count();
    add_popcorn_clusters(scene, &COOKIE_WORLD_BACK_POPCORN, world)?;
    add_lumpy_rocks(
        scene,
        &COOKIE_WORLD_BACK_ROCKS,
        COOKIE_WORLD_ROCKS.len() as f32,
        world.far_rock,
    )?;
    metadata.cookie_world_back_decoration_parts = scene.object_count() - first_back;

    Ok(())
}

/// Clusters of popcorn puffs with a golden kernel: center, size and spin.
fn add_popcorn_clusters(
    scene: &mut Scene,
    clusters: &[(Vec3, f32, f32)],
    world: CookieWorldMaterials,
) -> Result<(), SpaceBuildError> {
    for &(center, size, spin) in clusters {
        let basis = Basis3::from_axis_angle(Vec3::new(0.3, 1.0, 0.5), spin)?;
        let at = |offset: Vec3| center + basis.local_to_world_vector(offset * size);

        for (offset, radius) in POPCORN_PUFFS {
            scene.add_sphere(Sphere::new(at(offset), radius * size, world.popcorn)?)?;
        }
        let (offset, radius) = POPCORN_KERNEL;
        scene.add_sphere(Sphere::new(
            at(offset),
            radius * size,
            world.popcorn_kernel,
        )?)?;
    }

    Ok(())
}

/// Rocks made of a ball and a smaller lump; `seed` varies the lumps.
fn add_lumpy_rocks(
    scene: &mut Scene,
    rocks: &[(Vec3, f32)],
    seed: f32,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    for (index, &(center, radius)) in rocks.iter().enumerate() {
        let i = index as f32 + seed;
        let lump = Vec3::new(
            hash3(i, 5.0, 1.0) - 0.5,
            hash3(i, 5.0, 2.0) - 0.5,
            hash3(i, 5.0, 3.0) - 0.5,
        )
        .normalized();
        scene.add_sphere(Sphere::new(center, radius, material_id)?)?;
        scene.add_sphere(Sphere::new(
            center + lump * (radius * 0.62),
            radius * 0.62,
            material_id,
        )?)?;
    }

    Ok(())
}

/// Original cartoon songbird. Its local frame has X to its left side, Y up
/// and Z forward (where the beak points); sizes are in body radii.
#[derive(Debug, Clone, Copy)]
struct Songbird {
    center: Vec3,
    radius: f32,
    forward: Vec3,
    up: Vec3,
    palette: usize,
    flying: bool,
    crest: bool,
    legs: bool,
}

/// Materials of one songbird.
#[derive(Debug, Clone, Copy)]
struct SongbirdMaterials {
    body: usize,
    belly: usize,
    feather: usize,
    beak: usize,
    feet: usize,
    eye: usize,
    pupil: usize,
}

impl CookieWorldMaterials {
    fn songbird(self, palette: usize, materials: SpaceMaterials) -> SongbirdMaterials {
        SongbirdMaterials {
            body: self.bird_body[palette],
            belly: self.bird_belly[palette],
            feather: self.bird_feather[palette],
            beak: self.bird_beak,
            feet: self.bird_feet,
            eye: materials.eye,
            pupil: materials.pupil,
        }
    }
}

fn cookie_world_birds() -> [Songbird; COOKIE_WORLD_BIRD_COUNT] {
    let cookie = COOKIE_WORLD_COOKIES[COOKIE_WORLD_SLINGSHOT_COOKIE];
    let ground = COOKIE_WORLD_PERCHED_BIRD_DIRECTION.normalized();

    [
        // Waits on the slingshot cookie, looking towards the pig and a little
        // towards the camera.
        Songbird {
            center: cookie.center
                + ground * (cookie.radius + COOKIE_WORLD_BIRD_RADIUS * SONGBIRD_STANDING_HEIGHT),
            radius: COOKIE_WORLD_BIRD_RADIUS,
            forward: Vec3::new(1.0, 0.0, 0.9),
            up: ground,
            palette: 0,
            flying: false,
            crest: true,
            legs: true,
        },
        // Flies from the slingshot towards the pig with open wings, turned a
        // little towards the camera so both wings show.
        Songbird {
            center: COOKIE_WORLD_FLYING_BIRD_CENTER,
            radius: COOKIE_WORLD_BIRD_RADIUS,
            forward: Vec3::new(0.75, -0.30, 0.65),
            up: Vec3::new(0.15, 1.0, 0.0),
            palette: 1,
            flying: true,
            crest: false,
            legs: false,
        },
    ]
}

fn add_cookie_world_birds(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
    world: CookieWorldMaterials,
) -> Result<(), SpaceBuildError> {
    let first_part = scene.object_count();

    for bird in cookie_world_birds() {
        add_songbird(scene, bird, world.songbird(bird.palette, materials))?;
        metadata.cookie_world_bird_count += 1;
    }
    metadata.cookie_world_bird_parts = scene.object_count() - first_part;

    Ok(())
}

/// Round body with a lighter belly, big friendly eyes with a highlight, an
/// open two-part beak, feathered wings and tail, and legs when standing.
fn add_songbird(
    scene: &mut Scene,
    bird: Songbird,
    colors: SongbirdMaterials,
) -> Result<(), SpaceBuildError> {
    let up = bird.up.normalized();
    let forward = (bird.forward - up * bird.forward.dot(up)).normalized();
    let left = up.cross(forward);
    let basis = Basis3::new(left, up, forward)?;
    let at = |local: Vec3| bird.center + basis.local_to_world_vector(local * bird.radius);
    let body = colors.body;
    let feather = colors.feather;

    scene.add_sphere(Sphere::new(bird.center, bird.radius, body)?)?;
    scene.add_sphere(Sphere::new(
        at(Vec3::new(0.0, -0.24, 0.38)),
        bird.radius * 0.66,
        colors.belly,
    )?)?;

    for side in [-1.0, 1.0] {
        let eye_direction = Vec3::new(side * 0.36, 0.30, 0.88).normalized();
        let eye = eye_direction * 0.76;
        let pupil = eye + (eye_direction + Vec3::new(0.0, 0.0, 0.3)).normalized() * 0.18;

        scene.add_sphere(Sphere::new(at(eye), bird.radius * 0.28, colors.eye)?)?;
        scene.add_sphere(Sphere::new(at(pupil), bird.radius * 0.13, colors.pupil)?)?;
        scene.add_sphere(Sphere::new(
            at(pupil + Vec3::new(side * 0.04, 0.07, 0.09)),
            bird.radius * 0.04,
            colors.eye,
        )?)?;
    }

    add_feather(
        scene,
        at(Vec3::new(0.0, 0.02, 0.86)),
        at(Vec3::new(0.0, -0.08, 1.55)),
        bird.radius * 0.24,
        colors.beak,
    )?;
    add_feather(
        scene,
        at(Vec3::new(0.0, -0.16, 0.82)),
        at(Vec3::new(0.0, -0.28, 1.28)),
        bird.radius * 0.16,
        colors.beak,
    )?;

    let wing = if bird.flying {
        SONGBIRD_OPEN_WING
    } else {
        SONGBIRD_FOLDED_WING
    };
    for side in [-1.0, 1.0] {
        let shoulder = mirrored_x(SONGBIRD_SHOULDER, side);
        scene.add_sphere(Sphere::new(at(shoulder), bird.radius * 0.26, feather)?)?;
        for tip in wing {
            add_feather(
                scene,
                at(shoulder),
                at(mirrored_x(tip, side)),
                bird.radius * 0.21,
                feather,
            )?;
        }
    }

    for tip in SONGBIRD_TAIL {
        add_feather(
            scene,
            at(Vec3::new(0.0, 0.0, -0.82)),
            at(tip),
            bird.radius * 0.14,
            feather,
        )?;
    }

    if bird.crest {
        for tip in SONGBIRD_CREST {
            add_feather(
                scene,
                at(Vec3::new(0.0, 0.90, 0.12)),
                at(tip),
                bird.radius * 0.08,
                feather,
            )?;
        }
    }

    if bird.legs {
        for side in [-1.0, 1.0] {
            let hip = Vec3::new(side * 0.26, -0.80, 0.08);
            let ankle = Vec3::new(side * 0.28, -1.10, 0.10);
            let start = at(hip);
            let end = at(ankle);
            scene.add_cylinder(Cylinder::new(
                (start + end) * 0.5,
                bird.radius * 0.05,
                (end - start).length() * 0.5,
                basis_with_up(end - start)?,
                colors.feet,
            )?)?;
            add_feather(
                scene,
                at(ankle + Vec3::new(0.0, -0.02, -0.06)),
                at(ankle + Vec3::new(0.0, -0.04, 0.34)),
                bird.radius * 0.07,
                colors.feet,
            )?;
        }
    }

    Ok(())
}

/// Cone with its round base at `base` and its tip at `tip`: feathers, beaks
/// and toes.
fn add_feather(
    scene: &mut Scene,
    base: Vec3,
    tip: Vec3,
    base_radius: f32,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    let axis = tip - base;

    scene.add_cone(Cone::new(
        (base + tip) * 0.5,
        base_radius,
        axis.length() * 0.5,
        basis_with_up(axis)?,
        material_id,
    )?)?;

    Ok(())
}

/// Light gray rock with darker blotches and a few small craters, in sphere
/// UVs.
fn rock_texture() -> Result<Texture, SpaceBuildError> {
    let craters: [(Vec3, f32); ROCK_CRATER_COUNT] = std::array::from_fn(|index| {
        let i = index as f32;
        let direction = Vec3::new(
            hash3(i, 9.0, 1.0) - 0.5,
            hash3(i, 9.0, 2.0) - 0.5,
            hash3(i, 9.0, 3.0) - 0.5,
        )
        .normalized();
        (direction, 0.16 + 0.14 * hash3(i, 9.0, 4.0))
    });
    let mut pixels = Vec::with_capacity(ROCK_TEXTURE_WIDTH * ROCK_TEXTURE_HEIGHT);

    for y in 0..ROCK_TEXTURE_HEIGHT {
        let v = 1.0 - y as f32 / (ROCK_TEXTURE_HEIGHT - 1) as f32;

        for x in 0..ROCK_TEXTURE_WIDTH {
            let u = x as f32 / (ROCK_TEXTURE_WIDTH - 1) as f32;
            let direction = sphere_direction_from_uv(u, v);
            let blotches = value_noise_3d(direction * 3.2 + Vec3::new(4.0, 1.0, 7.0));
            let mut color = ROCK_LIGHT.lerp(ROCK_DARK, smoothstep(0.45, 0.85, blotches));

            for &(center, angular_radius) in &craters {
                let angle = direction.dot(center).clamp(-1.0, 1.0).acos();
                // Dark floor with a lighter rim.
                let floor = 1.0 - smoothstep(angular_radius * 0.70, angular_radius, angle);
                let rim = smoothstep(angular_radius * 0.80, angular_radius, angle)
                    * (1.0 - smoothstep(angular_radius, angular_radius * 1.25, angle));
                color = color.lerp(ROCK_DARK * 0.85, floor * 0.8);
                color = color.lerp(ROCK_LIGHT * 1.08, rim * 0.5);
            }
            pixels.push(color.clamped());
        }
    }

    Ok(Texture::new(
        ROCK_TEXTURE_WIDTH,
        ROCK_TEXTURE_HEIGHT,
        pixels,
    )?)
}

/// Waffle pattern for the cones' side UVs: golden diamonds with darker
/// grooves, periodic around the cone.
fn waffle_texture() -> Result<Texture, SpaceBuildError> {
    let mut pixels = Vec::with_capacity(WAFFLE_TEXTURE_SIZE * WAFFLE_TEXTURE_SIZE);

    for y in 0..WAFFLE_TEXTURE_SIZE {
        let v = 1.0 - y as f32 / (WAFFLE_TEXTURE_SIZE - 1) as f32;

        for x in 0..WAFFLE_TEXTURE_SIZE {
            let u = x as f32 / (WAFFLE_TEXTURE_SIZE - 1) as f32;
            let rising = (u * WAFFLE_CELLS_AROUND + v * WAFFLE_CELLS_ALONG).rem_euclid(1.0);
            let falling = (u * WAFFLE_CELLS_AROUND - v * WAFFLE_CELLS_ALONG).rem_euclid(1.0);
            let edge = rising.min(1.0 - rising).min(falling.min(1.0 - falling));
            let groove = 1.0 - smoothstep(0.05, 0.11, edge);
            let cell_light = 0.88 + 0.12 * smoothstep(0.10, 0.50, edge);

            pixels.push((WAFFLE_LIGHT * cell_light).lerp(WAFFLE_GROOVE, groove));
        }
    }

    Ok(Texture::new(
        WAFFLE_TEXTURE_SIZE,
        WAFFLE_TEXTURE_SIZE,
        pixels,
    )?)
}

/// Chocolate chip cookie texture in sphere UVs: mottled dough with baked
/// patches and dark chips with a lighter side.
fn cookie_texture(seed: f32) -> Result<Texture, SpaceBuildError> {
    let chips = cookie_chips(seed);
    let mut pixels = Vec::with_capacity(COOKIE_TEXTURE_WIDTH * COOKIE_TEXTURE_HEIGHT);

    for y in 0..COOKIE_TEXTURE_HEIGHT {
        let v = 1.0 - y as f32 / (COOKIE_TEXTURE_HEIGHT - 1) as f32;

        for x in 0..COOKIE_TEXTURE_WIDTH {
            let u = x as f32 / (COOKIE_TEXTURE_WIDTH - 1) as f32;
            pixels.push(cookie_color(sphere_direction_from_uv(u, v), seed, &chips));
        }
    }

    Ok(Texture::new(
        COOKIE_TEXTURE_WIDTH,
        COOKIE_TEXTURE_HEIGHT,
        pixels,
    )?)
}

/// Chip directions spread evenly over the sphere (golden spiral) with a
/// seeded jitter, and their angular radius.
fn cookie_chips(seed: f32) -> [(Vec3, f32); COOKIE_CHIP_COUNT] {
    let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());

    std::array::from_fn(|index| {
        let i = index as f32;
        let y = 1.0 - 2.0 * (i + 0.5) / COOKIE_CHIP_COUNT as f32;
        let ring = (1.0 - y * y).max(0.0).sqrt();
        let theta = golden_angle * i + seed;
        let jitter = Vec3::new(
            hash3(i, seed, 1.0) - 0.5,
            hash3(i, seed, 2.0) - 0.5,
            hash3(i, seed, 3.0) - 0.5,
        ) * 0.35;
        let direction =
            (Vec3::new(ring * theta.cos(), y, ring * theta.sin()) + jitter).normalized();

        (direction, 0.075 + 0.065 * hash3(i, seed, 4.0))
    })
}

fn cookie_color(direction: Vec3, seed: f32, chips: &[(Vec3, f32)]) -> Color {
    let offset = Vec3::new(seed, seed * 1.7, seed * 0.3);
    let broad = value_noise_3d(direction * 2.6 + offset);
    let fine = value_noise_3d(direction * 9.0 + offset * 1.3);
    let mottling = broad * 0.65 + fine * 0.35;
    let mut color = COOKIE_DOUGH.lerp(COOKIE_DOUGH_LIGHT, smoothstep(0.52, 0.80, mottling));
    color = color.lerp(COOKIE_DOUGH_BAKED, 1.0 - smoothstep(0.18, 0.40, mottling));

    let wobble = value_noise_3d(direction * 22.0 + offset);
    for &(center, angular_radius) in chips {
        let cosine = direction.dot(center);
        if cosine < (angular_radius * 1.9).cos() {
            continue;
        }

        let angle = cosine.clamp(-1.0, 1.0).acos();
        let radius = angular_radius * (0.80 + 0.40 * wobble);
        // Toasted dough around the chip.
        let toasted = 1.0 - smoothstep(radius, radius * 1.7, angle);
        color = color.lerp(COOKIE_DOUGH_BAKED, toasted * 0.45);
        // The chip itself, lighter on its upper side.
        let chip = 1.0 - smoothstep(radius - 0.012, radius + 0.004, angle);
        let upper_side = (direction - center).dot(Vec3::new(0.0, 1.0, 0.0)) / radius.max(0.001);
        let chip_color = COOKIE_CHIP_DARK.lerp(COOKIE_CHIP_LIGHT, smoothstep(0.1, 0.9, upper_side));
        color = color.lerp(chip_color, chip);
    }

    color
}

fn hash3(x: f32, y: f32, z: f32) -> f32 {
    let value = (x * 12.9898 + y * 78.233 + z * 37.719).sin() * 43_758.547;
    value - value.floor()
}

/// Smooth 3D value noise in [0, 1].
fn value_noise_3d(point: Vec3) -> f32 {
    let base = Vec3::new(point.x.floor(), point.y.floor(), point.z.floor());
    let tx = smoothstep(0.0, 1.0, point.x - base.x);
    let ty = smoothstep(0.0, 1.0, point.y - base.y);
    let tz = smoothstep(0.0, 1.0, point.z - base.z);
    let corner = |dx: f32, dy: f32, dz: f32| hash3(base.x + dx, base.y + dy, base.z + dz);
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let bottom = lerp(
        lerp(corner(0.0, 0.0, 0.0), corner(1.0, 0.0, 0.0), tx),
        lerp(corner(0.0, 1.0, 0.0), corner(1.0, 1.0, 0.0), tx),
        ty,
    );
    let top = lerp(
        lerp(corner(0.0, 0.0, 1.0), corner(1.0, 0.0, 1.0), tx),
        lerp(corner(0.0, 1.0, 1.0), corner(1.0, 1.0, 1.0), tx),
        ty,
    );

    lerp(bottom, top, tz)
}

/// Spotted pig skin for the level two pig, generated in its sphere UVs.
fn pig_skin_texture() -> Result<Texture, SpaceBuildError> {
    let basis = cookie_world_pig_basis()?;
    let spots = PIG_SKIN_SPOTS.map(|(direction, angular_radius)| {
        (
            basis.local_to_world_vector(direction).normalized(),
            angular_radius,
        )
    });
    let mut pixels = Vec::with_capacity(PIG_SKIN_TEXTURE_WIDTH * PIG_SKIN_TEXTURE_HEIGHT);

    for y in 0..PIG_SKIN_TEXTURE_HEIGHT {
        let v = 1.0 - y as f32 / (PIG_SKIN_TEXTURE_HEIGHT - 1) as f32;

        for x in 0..PIG_SKIN_TEXTURE_WIDTH {
            let u = x as f32 / (PIG_SKIN_TEXTURE_WIDTH - 1) as f32;
            pixels.push(pig_skin_color(sphere_direction_from_uv(u, v), &spots));
        }
    }

    Ok(Texture::new(
        PIG_SKIN_TEXTURE_WIDTH,
        PIG_SKIN_TEXTURE_HEIGHT,
        pixels,
    )?)
}

/// Inverse of `Sphere::uv_from_normal`.
fn sphere_direction_from_uv(u: f32, v: f32) -> Vec3 {
    let longitude = (u - 0.5) * std::f32::consts::TAU;
    let latitude = (v - 0.5) * std::f32::consts::PI;

    Vec3::new(
        latitude.cos() * longitude.cos(),
        latitude.sin(),
        latitude.cos() * longitude.sin(),
    )
}

fn pig_skin_color(direction: Vec3, spots: &[(Vec3, f32)]) -> Color {
    let coverage = spots
        .iter()
        .map(|&(center, angular_radius)| {
            let angle = direction.dot(center).clamp(-1.0, 1.0).acos();
            1.0 - smoothstep(
                angular_radius - PIG_SKIN_SPOT_SOFTNESS,
                angular_radius + PIG_SKIN_SPOT_SOFTNESS,
                angle,
            )
        })
        .fold(0.0, f32::max);

    PIG_SKIN_BASE_COLOR.lerp(PIG_SKIN_SPOT_COLOR, coverage)
}

/// Main and secondary asteroids of level three. The secondary one sits at the
/// far end of the bridge, so its surface meets the last bridge level.
fn add_level_three_asteroids(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    metadata.level_three_planet = Some(voxel::add_voxel_body(
        scene,
        &level_three_main_asteroid_body(materials.level_three_asteroid)?,
    )?);
    metadata.level_three_secondary_asteroid = Some(voxel::add_voxel_body(
        scene,
        &VoxelBody::ball(
            VoxelBall::new(
                level_three_secondary_asteroid_center(),
                LEVEL_THREE_SECONDARY_ASTEROID_RADIUS,
                materials.level_three_asteroid,
            ),
            LEVEL_THREE_CUBE_EDGE,
            VOXEL_ROCK_MIN_CUBES_ACROSS,
        )?,
    )?);

    Ok(())
}

/// The main level three asteroid as a ball of cubes. The structures on it
/// stand on the top of its cubes.
fn level_three_main_asteroid_body(material_id: usize) -> Result<VoxelBody, SpaceBuildError> {
    VoxelBody::ball(
        VoxelBall::new(
            LEVEL_THREE_PLANET_CENTER,
            LEVEL_THREE_MAIN_ASTEROID_RADIUS,
            material_id,
        ),
        LEVEL_THREE_CUBE_EDGE,
        VOXEL_ROCK_MIN_CUBES_ACROSS,
    )
}

/// A rock made of cubes: a ball of `radius` with a smaller lump at
/// `lump_offset` (in rock radii), with cubes no larger than `cube_edge`.
pub(super) fn add_voxel_rock(
    scene: &mut Scene,
    center: Vec3,
    radius: f32,
    lump_offset: Vec3,
    material_id: usize,
    cube_edge: f32,
) -> Result<voxel::VoxelBodyParts, SpaceBuildError> {
    let body = VoxelBody::new(
        &[
            VoxelBall::new(center, radius, material_id),
            VoxelBall::new(
                center + lump_offset * radius,
                radius * ROCK_LUMP_RADIUS,
                material_id,
            ),
        ],
        voxel::fitted_cube_edge(radius, cube_edge, VOXEL_ROCK_MIN_CUBES_ACROSS),
    )?;

    voxel::add_voxel_body(scene, &body)
}

/// Atmospheres go right after the asteroids: shadow rays stop at the first
/// object they hit, and most rays from inside an atmosphere hit its sphere.
fn add_level_three_atmospheres(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    metadata.level_three_main_atmosphere_id = Some(scene.object_count());
    scene.add_sphere(Sphere::new(
        LEVEL_THREE_PLANET_CENTER,
        LEVEL_THREE_MAIN_ATMOSPHERE_RADIUS,
        materials.level_three_atmosphere,
    )?)?;
    metadata.level_three_secondary_atmosphere_id = Some(scene.object_count());
    scene.add_sphere(Sphere::new(
        level_three_secondary_asteroid_center(),
        LEVEL_THREE_SECONDARY_ATMOSPHERE_RADIUS,
        materials.level_three_atmosphere,
    )?)?;

    Ok(())
}

/// A small asteroid under the bridge with the slingshot alone on top.
fn add_level_three_slingshot_asteroid(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let body = VoxelBody::ball(
        VoxelBall::new(
            LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER,
            LEVEL_THREE_SLINGSHOT_ASTEROID_RADIUS,
            materials.level_three_asteroid,
        ),
        LEVEL_THREE_CUBE_EDGE,
        VOXEL_ROCK_MIN_CUBES_ACROSS,
    )?;
    metadata.level_three_slingshot_asteroid = Some(voxel::add_voxel_body(scene, &body)?);

    // The slingshot is planted in the top of the cubes under it.
    let ground = body.surface_distance(
        LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER,
        LEVEL_THREE_SLINGSHOT_DIRECTION,
    );
    let frame = RadialFrame::from_normal(
        LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER,
        ground - LEVEL_THREE_SLINGSHOT_EMBED,
        LEVEL_THREE_SLINGSHOT_DIRECTION,
    )?;
    let parts = add_slingshot(
        scene,
        frame,
        COOKIE_WORLD_SLINGSHOT_YAW_RADIANS,
        LEVEL_THREE_SLINGSHOT_SCALE,
        materials,
    )?;
    metadata.level_three_slingshot_parts = parts.total();

    // Three birds: one loaded in the pouch, aimed at the bridge, and two
    // waiting on the left of the asteroid.
    let bird_materials = birds::register_bird_materials(scene, materials)?;
    metadata.level_three_bird_parts = birds::add_slingshot_birds(
        scene,
        birds::SlingshotBirds {
            frame,
            yaw_radians: COOKIE_WORLD_SLINGSHOT_YAW_RADIANS,
            scale: LEVEL_THREE_SLINGSHOT_SCALE,
            aim: LEVEL_THREE_BIRD_AIM,
            ground: &body,
            waiting_directions: &LEVEL_THREE_WAITING_BIRDS,
        },
        bird_materials,
    )?;
    metadata.level_three_bird_count = birds::BIRD_COUNT;

    Ok(())
}

fn level_three_secondary_asteroid_center() -> Vec3 {
    let bridge_length = LEVEL_THREE_BLOCK * LEVEL_THREE_BRIDGE_LEVELS as f32;
    let distance = LEVEL_THREE_MAIN_ASTEROID_RADIUS + bridge_length
        - LEVEL_THREE_BRIDGE_EMBED
        - LEVEL_THREE_SECONDARY_EMBED
        + LEVEL_THREE_SECONDARY_ASTEROID_RADIUS;

    LEVEL_THREE_PLANET_CENTER + LEVEL_THREE_BRIDGE_DIRECTION.normalized() * distance
}

/// Frame for the level three layout, which is modeled in the XY plane like the
/// reference picture: `up` points away from the asteroid inside that plane,
/// `right` is `up` turned a quarter turn clockwise and forward is +Z, towards
/// the default camera.
#[derive(Debug, Clone, Copy)]
struct LayoutFrame {
    origin: Vec3,
    basis: Basis3,
}

impl LayoutFrame {
    /// Frame on the ground of the main asteroid, `angle_radians` clockwise
    /// from +Y as seen from the default camera. The ground is the lowest top
    /// of the asteroid cubes under the footprint of what stands there, so
    /// nothing floats over a lower step; higher cubes sink into it.
    fn on_main_asteroid(angle_radians: f32) -> Result<Self, SpaceBuildError> {
        let up = Vec3::new(angle_radians.sin(), angle_radians.cos(), 0.0);
        let shape = [(LEVEL_THREE_PLANET_CENTER, LEVEL_THREE_MAIN_ASTEROID_RADIUS)];
        let spread = LEVEL_THREE_FOOTPRINT_HALF_WIDTH / LEVEL_THREE_MAIN_ASTEROID_RADIUS;
        let ground = (0..=LEVEL_THREE_FOOTPRINT_SAMPLES)
            .map(|sample| {
                let fraction = sample as f32 / LEVEL_THREE_FOOTPRINT_SAMPLES as f32;
                let angle = angle_radians + spread * (fraction * 2.0 - 1.0);
                let direction = Vec3::new(angle.sin(), angle.cos(), 0.0);

                voxel::surface_distance(
                    &shape,
                    LEVEL_THREE_CUBE_EDGE,
                    LEVEL_THREE_PLANET_CENTER,
                    direction,
                ) * direction.dot(up)
            })
            .fold(f32::INFINITY, f32::min);

        Self::with_up(LEVEL_THREE_PLANET_CENTER + up * ground, up)
    }

    fn with_up(origin: Vec3, up: Vec3) -> Result<Self, SpaceBuildError> {
        let up = up.normalized();
        let forward = Vec3::new(0.0, 0.0, 1.0);

        Ok(Self {
            origin,
            basis: Basis3::new(up.cross(forward), up, forward)?,
        })
    }

    fn point(self, local: Vec3) -> Vec3 {
        self.origin + self.basis.local_to_world_vector(local)
    }

    /// Basis turned by `angle_radians` around the forward axis
    /// (counterclockwise as seen from the camera).
    fn turned(self, angle_radians: f32) -> Result<Basis3, SpaceBuildError> {
        let (sin_angle, cos_angle) = angle_radians.sin_cos();
        let right = self.basis.right();
        let up = self.basis.up();

        Ok(Basis3::new(
            right * cos_angle + up * sin_angle,
            up * cos_angle - right * sin_angle,
            self.basis.forward(),
        )?)
    }
}

fn add_layout_box(
    scene: &mut Scene,
    frame: LayoutFrame,
    local_center: Vec3,
    half_extents: Vec3,
    angle_radians: f32,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    scene.add_oriented_box(OrientedBox::new(
        frame.point(local_center),
        half_extents,
        frame.turned(angle_radians)?,
        material_id,
    )?)?;

    Ok(())
}

/// Square frame of four bars in the layout plane, optionally braced with a
/// diagonal bar. Returns the number of boxes added.
fn add_layout_square(
    scene: &mut Scene,
    frame: LayoutFrame,
    center: Vec3,
    size: f32,
    bar: f32,
    braced: bool,
    material_id: usize,
) -> Result<usize, SpaceBuildError> {
    let half = size * 0.5;
    let depth = LEVEL_THREE_DEPTH;
    let mut parts = 0;

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
            Vec3::new(bar * 0.5, half - bar, depth),
        ),
        (
            Vec3::new(-half + bar * 0.5, 0.0, 0.0),
            Vec3::new(bar * 0.5, half - bar, depth),
        ),
    ] {
        add_layout_box(
            scene,
            frame,
            center + offset,
            half_extents,
            0.0,
            material_id,
        )?;
        parts += 1;
    }

    if braced {
        // Slightly thinner in depth so its faces never coincide with the bars.
        let inner = size - bar * 2.0;
        add_layout_box(
            scene,
            frame,
            center,
            Vec3::new(
                inner * std::f32::consts::FRAC_1_SQRT_2,
                bar * 0.45,
                depth * 0.9,
            ),
            std::f32::consts::FRAC_PI_4,
            material_id,
        )?;
        parts += 1;
    }

    Ok(parts)
}

/// Ladder of two rails from `start` to `end` with `rungs` rungs between them.
/// Returns the number of boxes added.
fn add_layout_ladder(
    scene: &mut Scene,
    frame: LayoutFrame,
    start: Vec3,
    end: Vec3,
    width: f32,
    rungs: usize,
    material_id: usize,
) -> Result<usize, SpaceBuildError> {
    let axis = end - start;
    let length = axis.length();
    let angle = (-axis.x).atan2(axis.y);
    let across = Vec3::new(angle.cos(), angle.sin(), 0.0);
    let bar = LEVEL_THREE_BAR;
    let middle = (start + end) * 0.5;
    let mut parts = 0;

    for side in [-1.0, 1.0] {
        add_layout_box(
            scene,
            frame,
            middle + across * (side * (width * 0.5 - bar * 0.5)),
            Vec3::new(bar * 0.5, length * 0.5, LEVEL_THREE_DEPTH * 0.8),
            angle,
            material_id,
        )?;
        parts += 1;
    }

    for rung in 1..=rungs {
        let along = rung as f32 / (rungs + 1) as f32;

        add_layout_box(
            scene,
            frame,
            start + axis * along,
            Vec3::new(width * 0.5 - bar, bar * 0.4, LEVEL_THREE_DEPTH * 0.6),
            angle,
            material_id,
        )?;
        parts += 1;
    }

    Ok(parts)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BridgeCell {
    StoneHollow,
    StoneBraced,
    WoodBraced,
    PigFrame,
    /// First of the two cells covered by a ladder.
    Ladder,
    /// Second cell of a ladder, already built with the first one.
    LadderEnd,
    StoneSolid,
    Tnt,
}

/// Bridge cells from the main asteroid to the secondary one: the column on the
/// upper side of the bridge (local +X) and the one on the lower side.
const LEVEL_THREE_BRIDGE_UPPER: [BridgeCell; LEVEL_THREE_BRIDGE_LEVELS] = [
    BridgeCell::Tnt,
    BridgeCell::StoneBraced,
    BridgeCell::Ladder,
    BridgeCell::LadderEnd,
    BridgeCell::WoodBraced,
    BridgeCell::PigFrame,
    BridgeCell::StoneBraced,
    BridgeCell::StoneBraced,
    BridgeCell::StoneHollow,
];
const LEVEL_THREE_BRIDGE_LOWER: [BridgeCell; LEVEL_THREE_BRIDGE_LEVELS] = [
    BridgeCell::StoneHollow,
    BridgeCell::StoneSolid,
    BridgeCell::WoodBraced,
    BridgeCell::PigFrame,
    BridgeCell::Ladder,
    BridgeCell::LadderEnd,
    BridgeCell::StoneBraced,
    BridgeCell::StoneHollow,
    BridgeCell::WoodBraced,
];

/// Two-column bridge of frames, crates and ladders from the main asteroid to
/// the secondary one, with a pig inside a frame in each column and two stone
/// slabs propping its lower side against the main asteroid.
fn add_level_three_bridge(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let frame = LayoutFrame::with_up(
        LEVEL_THREE_PLANET_CENTER
            + LEVEL_THREE_BRIDGE_DIRECTION.normalized() * LEVEL_THREE_MAIN_ASTEROID_RADIUS,
        LEVEL_THREE_BRIDGE_DIRECTION,
    )?;
    let block = LEVEL_THREE_BLOCK;
    let bar = LEVEL_THREE_BAR;
    let pig_radius = 0.15;

    for (column_x, cells) in [
        (block * 0.5, LEVEL_THREE_BRIDGE_UPPER),
        (-block * 0.5, LEVEL_THREE_BRIDGE_LOWER),
    ] {
        for (level, cell) in cells.into_iter().enumerate() {
            let bottom = block * level as f32 - LEVEL_THREE_BRIDGE_EMBED;
            let center = Vec3::new(column_x, bottom + block * 0.5, 0.0);

            metadata.level_three_bridge_parts += match cell {
                BridgeCell::StoneHollow => add_layout_square(
                    scene,
                    frame,
                    center,
                    block,
                    bar,
                    false,
                    materials.level_three_stone,
                )?,
                BridgeCell::StoneBraced => add_layout_square(
                    scene,
                    frame,
                    center,
                    block,
                    bar,
                    true,
                    materials.level_three_stone,
                )?,
                BridgeCell::WoodBraced => {
                    add_layout_square(scene, frame, center, block, bar, true, materials.wood)?
                }
                BridgeCell::PigFrame => {
                    add_space_pig(
                        scene,
                        frame.point(Vec3::new(column_x, bottom + bar + pig_radius, 0.0)),
                        pig_radius,
                        frame.basis,
                        materials,
                    )?;
                    metadata.level_three_pig_count += 1;
                    metadata.pig_count += 1;
                    add_layout_square(
                        scene,
                        frame,
                        center,
                        block,
                        bar,
                        false,
                        materials.level_three_stone,
                    )?
                }
                BridgeCell::Ladder => add_layout_ladder(
                    scene,
                    frame,
                    Vec3::new(column_x, bottom, 0.0),
                    Vec3::new(column_x, bottom + block * 2.0, 0.0),
                    block,
                    4,
                    materials.wood,
                )?,
                BridgeCell::LadderEnd => 0,
                BridgeCell::StoneSolid => {
                    add_layout_box(
                        scene,
                        frame,
                        center,
                        Vec3::new(block * 0.5, block * 0.5, LEVEL_THREE_DEPTH),
                        0.0,
                        materials.level_three_stone,
                    )?;
                    1
                }
                BridgeCell::Tnt => {
                    let half = 0.16;
                    add_layout_box(
                        scene,
                        frame,
                        Vec3::new(column_x, half - 0.02, 0.0),
                        Vec3::new(half, half, half),
                        0.0,
                        materials.tnt_crate,
                    )?;
                    metadata.tnt_parts += 1;
                    1
                }
            };
        }
    }

    // Stone slabs propping the lower side of the bridge.
    for (x, top) in [(-block - 0.07, block * 2.2), (-block - 0.21, block * 1.4)] {
        let bottom = -0.20;
        add_layout_box(
            scene,
            frame,
            Vec3::new(x, (bottom + top) * 0.5, 0.0),
            Vec3::new(0.07, (top - bottom) * 0.5, LEVEL_THREE_DEPTH * 0.95),
            0.0,
            materials.level_three_stone,
        )?;
        metadata.level_three_bridge_parts += 1;
    }

    Ok(())
}

/// Tower on top of the main asteroid: a stone arch, an ice square, stacked
/// stone slabs with wood blocks, an ice slab and an ice pyramid on top, a
/// leaning ladder with a pig on it, an ice wedge and a TNT crate beside it.
fn add_level_three_tower(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let tower_angle = 30.0_f32.to_radians();
    let frame = LayoutFrame::on_main_asteroid(tower_angle)?;
    let stone = materials.level_three_stone;
    let mut parts = 0;

    for (center, half_extents, material_id) in [
        // Arch legs and top.
        (
            Vec3::new(-0.17, 0.23, 0.0),
            Vec3::new(0.07, 0.28, LEVEL_THREE_DEPTH),
            stone,
        ),
        (
            Vec3::new(0.17, 0.23, 0.0),
            Vec3::new(0.07, 0.28, LEVEL_THREE_DEPTH),
            stone,
        ),
        (
            Vec3::new(0.0, 0.56, 0.0),
            Vec3::new(0.26, 0.05, LEVEL_THREE_DEPTH),
            stone,
        ),
        // Slabs, the wood blocks between two of them and the ice slab.
        (
            Vec3::new(0.10, 0.645, 0.0),
            Vec3::new(0.44, 0.035, LEVEL_THREE_DEPTH),
            stone,
        ),
        (
            Vec3::new(0.22, 1.23, 0.0),
            Vec3::new(0.42, 0.035, LEVEL_THREE_DEPTH),
            stone,
        ),
        (
            Vec3::new(-0.08, 1.355, 0.0),
            Vec3::new(0.12, 0.09, LEVEL_THREE_DEPTH * 0.9),
            materials.wood,
        ),
        (
            Vec3::new(0.22, 1.355, 0.0),
            Vec3::new(0.12, 0.09, LEVEL_THREE_DEPTH * 0.9),
            materials.wood,
        ),
        (
            Vec3::new(0.12, 1.48, 0.0),
            Vec3::new(0.34, 0.035, LEVEL_THREE_DEPTH),
            stone,
        ),
        (
            Vec3::new(0.40, 1.575, 0.0),
            Vec3::new(0.14, 0.06, LEVEL_THREE_DEPTH * 0.9),
            materials.ice,
        ),
    ] {
        add_layout_box(scene, frame, center, half_extents, 0.0, material_id)?;
        parts += 1;
    }

    // Ice square between the two lowest slabs.
    parts += add_layout_square(
        scene,
        frame,
        Vec3::new(0.0, 0.93, 0.0),
        0.52,
        0.08,
        false,
        materials.ice,
    )?;

    // Ice pyramid: a tetrahedron with almost flat faces resting on the top slab.
    let pyramid_radius = 0.30;
    scene.add_curved_tetrahedron(CurvedTetrahedron::new(
        frame.point(Vec3::new(-0.02, 1.515 + pyramid_radius / 3.0 - 0.01, 0.0)),
        pyramid_radius,
        LEVEL_THREE_PYRAMID_FACE_RADIUS_SCALE,
        frame.basis,
        materials.ice,
    )?)?;
    parts += 1;

    // Leaning ladder with a pig on top.
    let ladder_top = Vec3::new(-0.40, 1.22, 0.0);
    parts += add_layout_ladder(
        scene,
        frame,
        Vec3::new(-0.66, -0.17, 0.0),
        ladder_top,
        0.26,
        4,
        materials.wood,
    )?;
    let pig_radius = 0.15;
    add_space_pig(
        scene,
        frame.point(ladder_top + Vec3::new(0.0, pig_radius, 0.0)),
        pig_radius,
        frame.basis,
        materials,
    )?;
    metadata.level_three_pig_count += 1;
    metadata.pig_count += 1;

    // Ice wedge on the ground to the left: a square turned 45 degrees and half
    // buried in the asteroid, so it shows as a triangle.
    let wedge_frame =
        LayoutFrame::on_main_asteroid(tower_angle - 1.0 / LEVEL_THREE_MAIN_ASTEROID_RADIUS)?;
    add_layout_box(
        scene,
        wedge_frame,
        Vec3::new(0.0, -0.02, 0.0),
        Vec3::new(0.17, 0.17, LEVEL_THREE_DEPTH * 0.9),
        std::f32::consts::FRAC_PI_4,
        materials.ice,
    )?;
    parts += 1;

    // TNT crate on the ground to the right of the tower.
    let tnt_frame =
        LayoutFrame::on_main_asteroid(tower_angle + 0.62 / LEVEL_THREE_MAIN_ASTEROID_RADIUS)?;
    let tnt_half = 0.14;
    add_layout_box(
        scene,
        tnt_frame,
        Vec3::new(0.0, tnt_half - 0.02, 0.0),
        Vec3::new(tnt_half, tnt_half, tnt_half),
        0.0,
        materials.tnt_crate,
    )?;
    metadata.tnt_parts += 1;
    parts += 1;

    metadata.level_three_tower_parts += parts;

    Ok(())
}

/// Structure hanging from the lower right side of the main asteroid: two stone
/// blocks under an ice square, a ladder beside them, a braced stone frame on
/// top with a pig at its end, and a TNT crate on the ground.
fn add_level_three_lower_structure(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let structure_angle = 135.0_f32.to_radians();
    let frame = LayoutFrame::on_main_asteroid(structure_angle)?;
    let mut parts = 0;

    for x in [0.30, 0.10] {
        add_layout_box(
            scene,
            frame,
            Vec3::new(x, 0.19, 0.0),
            Vec3::new(0.08, 0.26, LEVEL_THREE_DEPTH),
            0.0,
            materials.level_three_stone,
        )?;
        parts += 1;
    }

    parts += add_layout_square(
        scene,
        frame,
        Vec3::new(0.20, 0.67, 0.0),
        0.44,
        0.075,
        false,
        materials.ice,
    )?;
    parts += add_layout_ladder(
        scene,
        frame,
        Vec3::new(-0.26, -0.10, 0.0),
        Vec3::new(-0.26, 0.89, 0.0),
        0.26,
        3,
        materials.wood,
    )?;

    let braced_size = 0.56;
    let braced_bottom = 0.89;
    parts += add_layout_square(
        scene,
        frame,
        Vec3::new(-0.02, braced_bottom + braced_size * 0.5, 0.0),
        braced_size,
        0.07,
        true,
        materials.level_three_stone,
    )?;

    let pig_radius = 0.15;
    add_space_pig(
        scene,
        frame.point(Vec3::new(
            -0.02,
            braced_bottom + braced_size + pig_radius,
            0.0,
        )),
        pig_radius,
        frame.basis,
        materials,
    )?;
    metadata.level_three_pig_count += 1;
    metadata.pig_count += 1;

    let tnt_frame =
        LayoutFrame::on_main_asteroid(structure_angle - 0.60 / LEVEL_THREE_MAIN_ASTEROID_RADIUS)?;
    let tnt_half = 0.14;
    add_layout_box(
        scene,
        tnt_frame,
        Vec3::new(0.0, tnt_half - 0.02, 0.0),
        Vec3::new(tnt_half, tnt_half, tnt_half),
        0.0,
        materials.tnt_crate,
    )?;
    metadata.tnt_parts += 1;
    parts += 1;

    metadata.level_three_lower_structure_parts += parts;

    Ok(())
}

/// Small lumpy rocks floating below the bridge, each made of a main sphere and
/// a smaller bump, with the asteroid material.
fn add_level_three_rocks(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    for (offset, radius) in LEVEL_THREE_ROCKS {
        let parts = add_voxel_rock(
            scene,
            LEVEL_THREE_PLANET_CENTER + offset,
            radius,
            ROCK_LUMP_OFFSET,
            materials.level_three_asteroid,
            LEVEL_THREE_CUBE_EDGE,
        )?;
        metadata.level_three_rock_count += 1;
        metadata.level_three_rock_parts += parts.total();
    }

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
    for (world, color) in galaxy_selector_worlds().into_iter().zip([
        Color::new(0.82, 0.94, 1.0),
        Color::new(1.0, 0.84, 0.62),
        Color::new(0.92, 0.82, 1.0),
        Color::new(1.0, 0.80, 0.96),
    ]) {
        scene.add_light(PointLight::new(
            world.center + Vec3::new(-world.radius * 0.30, world.radius * 0.38, world.radius),
            color,
            1.6,
        ));
    }
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

/// Level two. Shadow rays stop at any object, including the transparent
/// bubble, so the pig is lit by lights inside the bubble: a warm key from the
/// upper left, a soft frontal light for the face, a lilac fill from the lower
/// right and a top rim from behind.
/// Lights outside the bubble cannot reach the pig; they graze the bubble's
/// silhouette and give it a light rim.
fn add_cookie_world_lighting(scene: &mut Scene) {
    let pig_area = COOKIE_WORLD_PIG_RADIUS * COOKIE_WORLD_PIG_RADIUS;
    for (offset, color, intensity) in COOKIE_WORLD_INNER_LIGHTS {
        scene.add_light(PointLight::new(
            COOKIE_WORLD_PIG_CENTER + offset * COOKIE_WORLD_PIG_RADIUS,
            color,
            intensity * pig_area,
        ));
    }
    for cookie in COOKIE_WORLD_COOKIES {
        scene.add_light(PointLight::new(
            cookie.center
                + COOKIE_WORLD_COOKIE_KEY_DIRECTION.normalized()
                    * (cookie.radius + COOKIE_WORLD_COOKIE_KEY_DISTANCE),
            Color::new(1.0, 0.90, 0.76),
            COOKIE_WORLD_COOKIE_KEY_INTENSITY,
        ));
    }
    scene.add_light(PointLight::new(
        COOKIE_WORLD_CONES_KEY_TARGET
            + COOKIE_WORLD_COOKIE_KEY_DIRECTION.normalized() * COOKIE_WORLD_CONES_KEY_DISTANCE,
        Color::new(1.0, 0.90, 0.76),
        COOKIE_WORLD_COOKIE_KEY_INTENSITY,
    ));
    for direction in COOKIE_WORLD_RIM_LIGHT_DIRECTIONS {
        scene.add_light(PointLight::new(
            COOKIE_WORLD_PIG_CENTER
                + direction * (COOKIE_WORLD_BUBBLE_RADIUS + COOKIE_WORLD_RIM_LIGHT_GAP),
            Color::new(0.80, 0.90, 1.0),
            COOKIE_WORLD_RIM_LIGHT_INTENSITY,
        ));
    }
}

/// Level three: a warm key light from the front left, a soft neutral fill
/// from the right and a red rim light from behind that matches the red
/// background.
fn add_level_three_lighting(scene: &mut Scene) {
    // Shadow rays stop at the atmospheres too, so the lights outside only
    // reach the rocks, the slingshot asteroid and the atmospheres themselves.
    // Each part of the level inside an atmosphere gets its own lights: the
    // main atmosphere alone, the lens where both overlap (middle of the
    // bridge) and the secondary atmosphere alone.
    let main = LEVEL_THREE_PLANET_CENTER;
    let secondary = level_three_secondary_asteroid_center();
    let axis = (secondary - main).normalized();
    let lens = main + axis * LEVEL_THREE_LENS_DISTANCE;
    for (position, color, intensity) in [
        (
            main + LEVEL_THREE_MAIN_KEY_OFFSET,
            Color::new(1.0, 0.94, 0.86),
            20.0,
        ),
        (
            main + LEVEL_THREE_MAIN_FILL_OFFSET,
            Color::new(0.85, 0.88, 0.95),
            9.0,
        ),
        (
            lens + LEVEL_THREE_LENS_LIGHT_OFFSET,
            Color::new(1.0, 0.94, 0.86),
            5.0,
        ),
        (
            secondary + LEVEL_THREE_SECONDARY_KEY_OFFSET,
            Color::new(1.0, 0.94, 0.86),
            12.0,
        ),
    ] {
        scene.add_light(PointLight::new(position, color, intensity));
    }

    // Outside: a close warm key for the slingshot asteroid and one for the
    // floating rocks, so the atmospheres get little of their light, and two
    // red lights around the atmospheres, in the plane of their silhouettes
    // seen from the camera, that give them a brighter red rim.
    for (position, color, intensity) in [
        (
            LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER + LEVEL_THREE_OUTSIDE_KEY_OFFSET,
            Color::new(1.0, 0.94, 0.86),
            5.0,
        ),
        (
            LEVEL_THREE_ROCKS_KEY_POSITION,
            Color::new(1.0, 0.94, 0.86),
            5.0,
        ),
        (
            main + LEVEL_THREE_MAIN_RIM_OFFSET,
            LEVEL_THREE_RIM_COLOR,
            LEVEL_THREE_RIM_INTENSITY,
        ),
        (
            secondary + LEVEL_THREE_SECONDARY_RIM_OFFSET,
            LEVEL_THREE_RIM_COLOR,
            LEVEL_THREE_RIM_INTENSITY,
        ),
    ] {
        scene.add_light(PointLight::new(position, color, intensity));
    }
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

/// Pig centered at `center`. Its local axes follow `basis`: X is right, Y is
/// up (away from the ground) and Z is the direction the pig faces.
fn add_space_pig(
    scene: &mut Scene,
    center: Vec3,
    radius: f32,
    basis: Basis3,
    materials: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let at = |local: Vec3| center + basis.local_to_world_vector(local);

    scene.add_sphere(Sphere::new(center, radius, materials.pig)?)?;
    scene.add_cylinder(Cylinder::new(
        // Front cap slightly outside the body so the snout face is visible.
        at(Vec3::new(0.0, -radius * 0.03, radius * 0.86)),
        radius * 0.34,
        radius * 0.18,
        facing_basis(basis)?,
        materials.snout,
    )?)?;

    for eye_x in [-radius * 0.34, radius * 0.34] {
        scene.add_sphere(Sphere::new(
            at(Vec3::new(eye_x, radius * 0.42, radius * 0.90)),
            radius * 0.16,
            materials.eye,
        )?)?;
        scene.add_sphere(Sphere::new(
            at(Vec3::new(eye_x, radius * 0.42, radius * 1.04)),
            radius * 0.07,
            materials.pupil,
        )?)?;
    }

    for ear_x in [-radius * 0.44, radius * 0.44] {
        scene.add_cone(Cone::new(
            at(Vec3::new(ear_x, radius * 0.88, 0.0)),
            radius * 0.14,
            radius * 0.18,
            basis,
            materials.pig,
        )?)?;
    }

    Ok(())
}

/// Basis whose local Y axis points along the frame tangent Z, so cylinders and
/// cones built with it lie parallel to the ground and point to the front.
fn facing_basis(basis: Basis3) -> Result<Basis3, SpaceBuildError> {
    Ok(Basis3::new(basis.right(), basis.forward(), -basis.up())?)
}

fn basis_with_up(up_axis: Vec3) -> Result<Basis3, SpaceBuildError> {
    let up = up_axis.normalized();
    let auxiliary = if up.y.abs() < 0.85 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let right = up.cross(auxiliary).normalized();
    let forward = right.cross(up).normalized();

    Ok(Basis3::new(right, up, forward)?)
}

/// Cylinder between two points given in the local coordinates of `frame`.
fn add_radial_segment(
    scene: &mut Scene,
    frame: RadialFrame,
    local_start: Vec3,
    local_end: Vec3,
    radius: f32,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    let start = frame.local_to_world(local_start);
    let end = frame.local_to_world(local_end);
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

#[cfg(test)]
mod tests {
    use super::{
        BLUE_MOON_PLANET_CENTER, BLUE_MOON_PLANET_RADIUS, CARAMEL_DRIPS, COOKIE_LEVEL_PIG_COUNT,
        COOKIE_PLANET_CENTER, COOKIE_PLANET_RADIUS, COOKIE_WORLD_BACK_POPCORN,
        COOKIE_WORLD_BACK_ROCKS, COOKIE_WORLD_BIRD_COUNT, COOKIE_WORLD_BIRD_RADIUS,
        COOKIE_WORLD_BUBBLE_RADIUS, COOKIE_WORLD_CARAMEL_CONES, COOKIE_WORLD_COOKIE_COUNT,
        COOKIE_WORLD_COOKIES, COOKIE_WORLD_HORNS, COOKIE_WORLD_PIG_CENTER, COOKIE_WORLD_PIG_RADIUS,
        COOKIE_WORLD_POPCORN, COOKIE_WORLD_ROCKS, COOKIE_WORLD_SKYBOX_TILES,
        COOKIE_WORLD_SLINGSHOT_COOKIE, COOKIE_WORLD_SLINGSHOT_DIRECTION, CookieWorldMaterials,
        GALAXY_SELECTOR_BLUE_MOON_CENTER, GALAXY_SELECTOR_COOKIE_CENTER,
        GALAXY_SELECTOR_LEVEL_FOUR_CENTER, GALAXY_SELECTOR_LEVEL_THREE_CENTER,
        GRAY_STONE_BLOCK_TEXTURE_PATH, ICE_BLOCK_TEXTURE_PATH, LEVEL_THREE_BLOCK,
        LEVEL_THREE_BRIDGE_DIRECTION, LEVEL_THREE_BRIDGE_LEVELS, LEVEL_THREE_CAMERA_TARGET,
        LEVEL_THREE_CUBE_EDGE, LEVEL_THREE_FOOTPRINT_HALF_WIDTH, LEVEL_THREE_LENS_DISTANCE,
        LEVEL_THREE_MAIN_ASTEROID_RADIUS, LEVEL_THREE_MAIN_ATMOSPHERE_RADIUS,
        LEVEL_THREE_PIG_COUNT, LEVEL_THREE_PLANET_CENTER, LEVEL_THREE_ROCKS,
        LEVEL_THREE_SECONDARY_ASTEROID_RADIUS, LEVEL_THREE_SECONDARY_ATMOSPHERE_RADIUS,
        LEVEL_THREE_SKYBOX_TILES, LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER,
        LEVEL_THREE_SLINGSHOT_ASTEROID_RADIUS, LEVEL_THREE_SLINGSHOT_DIRECTION, LayoutFrame,
        PIG_SKIN_BASE_COLOR, PIG_SKIN_TEXTURE_HEIGHT, PIG_SKIN_TEXTURE_WIDTH, POPCORN_PUFFS,
        PlanetType, SKYBOX_ASTEROID_A_U, SKYBOX_ASTEROID_A_V, SPACE_SUN_DIRECTION, SPACE_SUN_U,
        SPACE_SUN_V, SceneState, Songbird, SpaceMaterials, TNT_CRATE_TEXTURE_PATH,
        WOOD_BLOCK_TEXTURE_PATH, blue_moon_orbit_camera, blue_moon_skybox,
        build_blue_moon_scene_with_metadata, build_cookie_world_scene_with_metadata,
        build_galaxy_selector_scene, build_level_three_scene_with_metadata,
        build_space_levels_scene_with_metadata, cookie_texture, cookie_world_birds,
        cookie_world_orbit_camera, cookie_world_skybox, danger_zone_skybox_texture,
        galaxy_selector_orbit_camera, galaxy_selector_worlds, level_three_orbit_camera,
        level_three_secondary_asteroid_center, level_three_skybox, pig_skin_texture,
        register_cookie_world_materials, register_space_materials, space_levels_orbit_camera,
        space_menu_skybox, space_skybox_color, space_skybox_texture, sun_light_position,
        utopia_skybox_texture,
        voxel::{self, assert_voxel_ball},
        wrapped_uv_distance,
    };
    use crate::{
        color::Color,
        math::{Vec2, Vec3},
        primitive::Primitive,
        ray::Ray,
        scene::Scene,
        sphere::Sphere,
        texture::{Texture, WrapMode},
    };

    /// Level two materials with the ids they get in the level two scene,
    /// where they are registered after the shared space materials.
    fn cookie_world_material_ids() -> CookieWorldMaterials {
        let mut scene = Scene::new();
        register_space_materials(&mut scene).unwrap();
        register_cookie_world_materials(&mut scene).unwrap()
    }

    fn color_delta(left: Color, right: Color) -> f32 {
        (left.r - right.r).abs() + (left.g - right.g).abs() + (left.b - right.b).abs()
    }

    fn luminance(color: Color) -> f32 {
        color.r * 0.2126 + color.g * 0.7152 + color.b * 0.0722
    }

    #[test]
    fn blue_moon_scene_builds_valid_scene() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();

        let moon = metadata.blue_moon_planet.unwrap();
        let small_moon = metadata.blue_moon_small_moon.unwrap();
        let voxel_parts =
            moon.total() + small_moon.total() + metadata.blue_moon_floating_asteroid_parts;

        assert!(scene.object_count() > voxel_parts);
        // Budget: the diorama parts plus a few thousand cubes.
        assert!(scene.object_count() - voxel_parts < 260);
        assert!(scene.cube_count() < 5_000);
        assert!(moon.cube_count > small_moon.cube_count);
        assert!(metadata.blue_moon_atmosphere_id.is_some());
        assert!(metadata.cookie_planet_id.is_none());
        assert_eq!(metadata.pig_count, 0);
        assert_eq!(scene.curved_tetrahedron_count(), 0);
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
        assert_ne!(
            SceneState::Planet(PlanetType::AsteroidBelt),
            SceneState::Planet(PlanetType::CosmicCrystals)
        );
    }

    #[test]
    fn galaxy_selector_declares_four_clickable_numbered_locked_worlds() {
        let worlds = galaxy_selector_worlds();

        assert_eq!(worlds.len(), 4);
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
        assert_eq!(worlds[3].planet, PlanetType::CosmicCrystals);
        assert_eq!(worlds[3].center, GALAXY_SELECTOR_LEVEL_FOUR_CENTER);
        assert_eq!(worlds[3].level_number, 4);
        assert!(worlds[3].locked);
        assert!(worlds[3].radius > 0.0);
        assert!(worlds[0].center.x < worlds[1].center.x);
        assert!(worlds[1].center.x < worlds[2].center.x);
        assert!(worlds[2].center.x < worlds[3].center.x);
        assert!(worlds.iter().all(|world| world.center.y < 0.0));
        assert!(worlds[1].radius > worlds[0].radius);
        assert!(worlds[1].radius > worlds[2].radius);
        assert!(worlds[1].radius > worlds[3].radius);
    }

    #[test]
    fn galaxy_selector_scene_contains_only_selector_world_geometry() {
        let scene = build_galaxy_selector_scene().unwrap();

        assert_eq!(scene.sphere_count(), 8);
        assert_eq!(scene.object_count(), 8);
        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 3);
        assert!(scene.ambient_light().b > 0.39);
    }

    #[test]
    fn galaxy_selector_planets_are_colored_and_lit_within_gravity_fields() {
        let scene = build_galaxy_selector_scene().unwrap();

        for world in galaxy_selector_worlds() {
            let body = scene
                .objects()
                .iter()
                .filter_map(|object| object.as_sphere())
                .find(|sphere| sphere.center() == world.center && sphere.radius() == world.radius)
                .unwrap();
            let material = scene.material(body.material_id()).unwrap();

            assert!(material.albedo.r + material.albedo.g + material.albedo.b > 1.8);
            assert!(material.texture_id.is_some());
            assert_eq!(material.transparency, 0.0);
            assert!(scene.lights().iter().any(|light| {
                let distance = (light.position - world.center).length();
                distance > world.radius && distance < world.radius * 1.16
            }));
        }
    }

    #[test]
    fn selector_skybox_has_a_central_sun_over_dark_blue_space() {
        let sun = super::selector_skybox_color(super::SELECTOR_SUN_U, super::SELECTOR_SUN_V);
        let background = super::selector_skybox_color(0.50, super::SELECTOR_SUN_V);

        assert!(sun.r > 0.95 && sun.g > 0.70 && sun.b < 0.45);
        assert!(background.b > background.r * 2.0);
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
    fn level_three_skybox_uses_danger_zone_background_asset() {
        let texture = danger_zone_skybox_texture().unwrap();
        let last_column = texture.width() - 1;
        let horizon = texture
            .pixel(texture.width() / 3, texture.height() / 2)
            .unwrap();
        let mut dark_mine_texels = 0;

        assert_eq!(texture.width(), 1025);
        assert_eq!(texture.height(), 1025);
        for y in 0..texture.height() {
            assert!(
                color_delta(
                    texture.pixel(0, y).unwrap(),
                    texture.pixel(last_column, y).unwrap()
                ) < 0.001,
                "tile seam at row {y}"
            );
            for x in 0..texture.width() {
                let texel = texture.pixel(x, y).unwrap();
                // Mines are dark gray, the haze behind them is red.
                if luminance(texel) < 0.13 && texel.r - texel.g < 0.04 {
                    dark_mine_texels += 1;
                }
            }
        }
        // Dark red haze with plenty of mines in front of it.
        assert!(horizon.r < 0.35);
        assert!(dark_mine_texels > texture.width() * texture.height() / 20);
        let corner = texture.pixel(0, texture.height() / 2).unwrap();
        assert!(corner.r >= corner.g && corner.r >= corner.b);
    }

    #[test]
    fn cookie_world_skybox_uses_seamless_utopia_parallax_background() {
        let texture = utopia_skybox_texture().unwrap();
        let last_column = texture.width() - 1;
        let top = texture.pixel(texture.width() / 2, 0).unwrap();
        let bottom = texture
            .pixel(texture.width() / 2, texture.height() - 1)
            .unwrap();
        let horizon = texture
            .pixel(texture.width() / 2, texture.height() / 2)
            .unwrap();

        assert_eq!(texture.width(), 1025);
        assert_eq!(texture.height(), 1025);
        for y in 0..texture.height() {
            assert!(
                color_delta(
                    texture.pixel(0, y).unwrap(),
                    texture.pixel(last_column, y).unwrap()
                ) < 0.001,
                "tile seam at row {y}"
            );
        }
        // Dark plum sky on top, lighter magenta bands below.
        assert!(luminance(bottom) > luminance(top) * 2.0);
        for color in [top, horizon, bottom] {
            assert!(color.r > color.g * 2.0);
            assert!(color.b > color.g);
        }
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
        let danger_zone_texture = danger_zone_skybox_texture().unwrap();
        let utopia_texture = utopia_skybox_texture().unwrap();

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
        assert_ne!(cookie_preset.texture(), level_three_preset.texture());

        for scene in [&selector, &blue, &combined] {
            let skybox = scene.skybox().unwrap();

            assert_eq!(skybox.texture().width(), super::SPACE_SKYBOX_WIDTH);
            assert_eq!(skybox.texture().height(), super::SPACE_SKYBOX_HEIGHT);
            assert!(skybox.intensity() > 1.0);
            assert_eq!(skybox.uv_scale(), Vec2::new(1.0, 1.0));
            assert_eq!(skybox.wrap_mode(), WrapMode::Clamp);
        }

        let cookie_skybox = cookie.skybox().unwrap();
        assert_eq!(cookie_skybox.texture().width(), utopia_texture.width());
        assert_eq!(cookie_skybox.texture().height(), utopia_texture.height());
        assert!(cookie_skybox.intensity() > 1.0);
        assert_eq!(
            cookie_skybox.uv_scale(),
            Vec2::new(COOKIE_WORLD_SKYBOX_TILES, 1.0)
        );
        assert_eq!(cookie_skybox.wrap_mode(), WrapMode::Repeat);

        assert_eq!(
            level_three.skybox().unwrap().texture().width(),
            danger_zone_texture.width()
        );
        assert_eq!(
            level_three.skybox().unwrap().texture().height(),
            danger_zone_texture.height()
        );
        assert!(level_three.skybox().unwrap().intensity() > 1.0);
        assert_eq!(
            level_three.skybox().unwrap().uv_scale(),
            Vec2::new(LEVEL_THREE_SKYBOX_TILES, 1.0)
        );
        assert_eq!(level_three.skybox().unwrap().wrap_mode(), WrapMode::Repeat);
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
        let expected_direction = SPACE_SUN_DIRECTION.normalized();
        let key_light = selector.lights()[0];
        let actual_direction = key_light.position.normalized();

        assert!(actual_direction.dot(expected_direction) > 0.999);
        assert_eq!(key_light.position, sun_light_position(10.0));
        assert!(key_light.color.r > key_light.color.b);
        assert!(key_light.intensity >= 60.0);
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
    fn tnt_crate_texture_loads_from_assets() {
        let texture = Texture::from_ppm_file(TNT_CRATE_TEXTURE_PATH).unwrap();

        assert!(texture.width() > 0);
        assert!(texture.height() > 0);
    }

    #[test]
    fn block_textures_load_from_assets() {
        let texture = Texture::from_ppm_file(WOOD_BLOCK_TEXTURE_PATH).unwrap();
        let first = texture.pixel(0, 0).unwrap();
        let middle = texture
            .pixel(texture.width() / 2, texture.height() / 2)
            .unwrap();

        assert_eq!(texture.width(), 32);
        assert_eq!(texture.height(), 32);
        assert_ne!(first, middle);
    }

    #[test]
    fn wood_material_uses_block_texture() {
        let mut scene = Scene::new();
        let SpaceMaterials { wood, .. } = register_space_materials(&mut scene).unwrap();
        let wood_material = scene.material(wood).unwrap();

        assert!(scene.texture(wood_material.texture_id.unwrap()).is_some());
        assert_eq!(wood_material.wrap_mode, WrapMode::Repeat);
        assert!(wood_material.uv_scale.u > 1.0);
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
        // Level one frames the whole diorama inside its atmosphere.
        assert_eq!(
            blue_moon_orbit_camera(4.0 / 3.0).target,
            super::blue_moon::BLUE_MOON_ATMOSPHERE_CENTER
        );
        assert_eq!(
            cookie_world_orbit_camera(4.0 / 3.0).target,
            COOKIE_WORLD_PIG_CENTER
        );
        // Level three frames both asteroids, so it orbits a point between them.
        let level_three_target = level_three_orbit_camera(4.0 / 3.0).target;
        assert_eq!(level_three_target, LEVEL_THREE_CAMERA_TARGET);
        assert!(level_three_target.x < LEVEL_THREE_PLANET_CENTER.x);
        assert!(level_three_target.x > level_three_secondary_asteroid_center().x);
    }

    #[test]
    fn space_levels_scene_contains_two_main_planets() {
        let (scene, metadata) = build_space_levels_scene_with_metadata().unwrap();
        let moon = metadata.blue_moon_planet.unwrap();
        let cookie = scene.objects()[metadata.cookie_planet_id.unwrap()]
            .as_sphere()
            .unwrap();
        let cubes: Vec<_> = (moon.first_id..moon.first_id + moon.cube_count)
            .map(|id| scene.objects()[id].as_cube().unwrap())
            .collect();
        let lowest = cubes
            .iter()
            .map(|cube| cube.min.y)
            .fold(f32::INFINITY, f32::min);
        let (left, right) = cubes
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |bounds, cube| {
                (bounds.0.min(cube.min.x), bounds.1.max(cube.max.x))
            });

        // The moon is a ball of cubes around its center.
        assert!(moon.cube_count > 0);
        assert!((lowest - (BLUE_MOON_PLANET_CENTER.y - BLUE_MOON_PLANET_RADIUS)).abs() < 0.15);
        assert!(((left + right) * 0.5 - BLUE_MOON_PLANET_CENTER.x).abs() < 0.15);
        assert_eq!(cookie.center(), COOKIE_PLANET_CENTER);
        assert_eq!(cookie.radius(), COOKIE_PLANET_RADIUS);
        assert!(metadata.cookie_gravity_field_id.is_some());
    }

    #[test]
    fn cookie_world_scene_has_a_pig_floating_in_a_bubble() {
        let (scene, metadata) = build_cookie_world_scene_with_metadata().unwrap();
        let CookieWorldMaterials {
            bubble, pig_skin, ..
        } = cookie_world_material_ids();
        let pig = scene.objects()[metadata.cookie_world_pig_id.unwrap()]
            .as_sphere()
            .unwrap();
        let bubble_sphere = scene.objects()[metadata.cookie_world_bubble_id.unwrap()]
            .as_sphere()
            .unwrap();
        let bubble_material = scene.material(bubble_sphere.material_id()).unwrap();

        assert_eq!(pig.center(), COOKIE_WORLD_PIG_CENTER);
        assert_eq!(pig.radius(), COOKIE_WORLD_PIG_RADIUS);
        assert_eq!(pig.material_id(), pig_skin);
        assert_eq!(bubble_sphere.center(), COOKIE_WORLD_PIG_CENTER);
        assert_eq!(bubble_sphere.radius(), COOKIE_WORLD_BUBBLE_RADIUS);
        assert_eq!(bubble_sphere.material_id(), bubble);
        assert!(bubble_material.transparency > 0.8);
        assert!(bubble_material.transparency < 1.0);
        assert_eq!(bubble_material.refractive_index, 1.0);
        assert_eq!(metadata.pig_count, COOKIE_LEVEL_PIG_COUNT);
        assert_eq!(metadata.tnt_parts, 0);
        assert!(metadata.blue_moon_planet.is_none());
        assert!(metadata.cookie_planet_id.is_none());
        assert!(scene.skybox().is_some());
    }

    #[test]
    fn cookie_world_scene_counts_match_metadata() {
        let (scene, metadata) = build_cookie_world_scene_with_metadata().unwrap();

        // Body, two snout discs, two nostrils, two eyes, two pupils, two ears.
        assert_eq!(metadata.cookie_world_pig_parts, 11);
        assert_eq!(metadata.cookie_world_popcorn_parts, 5);
        assert_eq!(
            metadata.cookie_world_cookie_count,
            COOKIE_WORLD_COOKIE_COUNT
        );
        assert!(metadata.cookie_world_horn_parts > COOKIE_WORLD_HORNS.len() * 10);
        assert_eq!(
            scene.object_count(),
            metadata.cookie_world_pig_parts
                + metadata.cookie_world_popcorn_parts
                + 1
                + metadata.cookie_world_cookie_count
                + metadata.cookie_world_horn_parts
                + metadata.cookie_world_slingshot_parts
                + metadata.cookie_world_caramel_cone_parts
                + metadata.cookie_world_popcorn_cluster_parts
                + metadata.cookie_world_rock_parts
                + metadata.cookie_world_back_decoration_parts
                + metadata.cookie_world_bird_parts
        );
        assert_eq!(
            metadata.cookie_world_back_decoration_parts,
            COOKIE_WORLD_BACK_POPCORN.len() * (POPCORN_PUFFS.len() + 1)
                + COOKIE_WORLD_BACK_ROCKS.len() * 2
        );
        // Each caramel cone: waffle cone, caramel scoop and two drops per drip.
        assert_eq!(
            metadata.cookie_world_caramel_cone_parts,
            COOKIE_WORLD_CARAMEL_CONES.len() * (2 + CARAMEL_DRIPS.len() * 2)
        );
        // Each popcorn cluster: its puffs and a kernel. Each rock: two lumps.
        assert_eq!(
            metadata.cookie_world_popcorn_cluster_parts,
            COOKIE_WORLD_POPCORN.len() * (POPCORN_PUFFS.len() + 1)
        );
        assert_eq!(
            metadata.cookie_world_rock_parts,
            COOKIE_WORLD_ROCKS.len() * 2
        );
        assert_eq!(metadata.cookie_world_bird_count, COOKIE_WORLD_BIRD_COUNT);
        assert_eq!(scene.oriented_box_count(), 0);
        assert_eq!(scene.cube_count(), 0);
    }

    /// Center and radius of a sphere that wraps a level two primitive.
    fn bounding_sphere(object: &Primitive) -> (Vec3, f32) {
        if let Some(sphere) = object.as_sphere() {
            (sphere.center(), sphere.radius())
        } else if let Some(cylinder) = object.as_cylinder() {
            (
                cylinder.center(),
                cylinder.radius().hypot(cylinder.half_height()),
            )
        } else {
            let cone = object.as_cone().unwrap();
            (cone.center(), cone.base_radius().hypot(cone.half_height()))
        }
    }

    /// Bounding spheres of all objects of the level two scene whose material
    /// is not in `own_materials`.
    fn other_bounds(scene: &Scene, own_materials: &[usize]) -> Vec<(Vec3, f32)> {
        scene
            .objects()
            .iter()
            .filter(|object| !own_materials.contains(&object.material_id()))
            .map(bounding_sphere)
            .collect()
    }

    #[test]
    fn cookie_world_caramel_cones_float_on_the_right_with_caramel_towards_the_pig() {
        let (scene, _) = build_cookie_world_scene_with_metadata().unwrap();
        let CookieWorldMaterials {
            waffle_cone,
            caramel,
            ..
        } = cookie_world_material_ids();
        let caramel_material = scene.material(caramel).unwrap();
        let cones: Vec<_> = scene
            .cones()
            .filter(|cone| cone.material_id() == waffle_cone)
            .collect();
        let caramel_parts = scene
            .objects()
            .iter()
            .filter(|object| object.material_id() == caramel)
            .count();
        let others = other_bounds(&scene, &[waffle_cone, caramel]);

        assert_eq!(cones.len(), COOKIE_WORLD_CARAMEL_CONES.len());
        assert_eq!(
            caramel_parts,
            COOKIE_WORLD_CARAMEL_CONES.len() * (1 + CARAMEL_DRIPS.len() * 2)
        );
        assert!(caramel_material.specular_strength > 0.8);
        assert!(caramel_material.reflectivity > 0.1);
        for (cone, spec) in cones.iter().zip(COOKIE_WORLD_CARAMEL_CONES) {
            let apex = cone.center() + cone.orientation().up() * cone.half_height();
            let (center, radius) = bounding_sphere(&Primitive::from(**cone));

            assert!(spec.top.x > COOKIE_WORLD_BUBBLE_RADIUS + 1.0);
            assert!(
                (spec.top - COOKIE_WORLD_PIG_CENTER).length()
                    < (apex - COOKIE_WORLD_PIG_CENTER).length()
            );
            for (other_center, other_radius) in &others {
                assert!((center - *other_center).length() > radius + other_radius);
            }
        }
    }

    #[test]
    fn cookie_world_popcorn_and_rocks_float_around_the_bubble_without_touching_anything() {
        let (scene, _) = build_cookie_world_scene_with_metadata().unwrap();
        let CookieWorldMaterials {
            popcorn,
            popcorn_kernel,
            rock,
            ..
        } = cookie_world_material_ids();
        let others = other_bounds(&scene, &[popcorn, popcorn_kernel, rock]);
        let groups = COOKIE_WORLD_POPCORN
            .iter()
            .map(|&(center, size, _)| (center, size * 1.0))
            .chain(
                COOKIE_WORLD_ROCKS
                    .iter()
                    .map(|&(center, radius)| (center, radius * 2.24)),
            );

        for (center, radius) in groups {
            let distance = (center - COOKIE_WORLD_PIG_CENTER).length();

            assert!(
                distance - radius > COOKIE_WORLD_BUBBLE_RADIUS + 0.2,
                "{center:?}"
            );
            assert!(distance < COOKIE_WORLD_BUBBLE_RADIUS + 2.0, "{center:?}");
            for (other_center, other_radius) in &others {
                if (*other_center - COOKIE_WORLD_PIG_CENTER).length() < COOKIE_WORLD_BUBBLE_RADIUS {
                    continue;
                }
                assert!(
                    (center - *other_center).length() > radius + other_radius,
                    "{center:?} touches an object at {other_center:?}"
                );
            }
        }
        // Every piece of popcorn and rock outside the bubble belongs to a group.
        let loose_pieces = scene
            .objects()
            .iter()
            .filter(|object| [popcorn, rock].contains(&object.material_id()))
            .map(bounding_sphere)
            .filter(|(center, _)| (*center - COOKIE_WORLD_PIG_CENTER).length() > 2.0)
            .count();
        assert_eq!(
            loose_pieces,
            (COOKIE_WORLD_POPCORN.len() + COOKIE_WORLD_BACK_POPCORN.len()) * POPCORN_PUFFS.len()
                + COOKIE_WORLD_ROCKS.len() * 2
        );
    }

    #[test]
    fn cookie_world_back_popcorn_and_rocks_give_depth_behind_the_bubble() {
        let (scene, _) = build_cookie_world_scene_with_metadata().unwrap();
        let CookieWorldMaterials {
            popcorn,
            popcorn_kernel,
            far_rock,
            ..
        } = cookie_world_material_ids();
        let camera = cookie_world_orbit_camera(16.0 / 9.0).to_camera();
        let basis = camera.basis();
        let half_height = (camera.vertical_fov_degrees.to_radians() * 0.5).tan();
        let half_width = half_height * camera.aspect_ratio;
        let others = other_bounds(&scene, &[popcorn, popcorn_kernel, far_rock]);
        let groups = COOKIE_WORLD_BACK_POPCORN
            .iter()
            .map(|&(center, size, _)| (center, size * 1.0))
            .chain(
                COOKIE_WORLD_BACK_ROCKS
                    .iter()
                    .map(|&(center, radius)| (center, radius * 2.24)),
            );

        for (center, radius) in groups {
            let offset = center - camera.position;
            let depth = offset.dot(basis.forward);

            // Behind the bubble from the initial camera, outside it and in view.
            assert!(center.z < COOKIE_WORLD_PIG_CENTER.z - COOKIE_WORLD_BUBBLE_RADIUS);
            assert!(
                (center - COOKIE_WORLD_PIG_CENTER).length() - radius > COOKIE_WORLD_BUBBLE_RADIUS
            );
            assert!((offset.dot(basis.right) / depth).abs() < half_width);
            assert!((offset.dot(basis.up) / depth).abs() < half_height);
            for (other_center, other_radius) in &others {
                if (*other_center - COOKIE_WORLD_PIG_CENTER).length() < COOKIE_WORLD_BUBBLE_RADIUS {
                    continue;
                }
                assert!(
                    (center - *other_center).length() > radius + other_radius,
                    "{center:?} touches an object at {other_center:?}"
                );
            }
        }
        assert!(
            scene
                .objects()
                .iter()
                .any(|object| object.material_id() == far_rock)
        );
    }

    #[test]
    fn cookie_world_has_two_original_birds_by_the_slingshot() {
        let (scene, _) = build_cookie_world_scene_with_metadata().unwrap();
        let SpaceMaterials { eye, pupil, .. } =
            register_space_materials(&mut Scene::new()).unwrap();
        let world = cookie_world_material_ids();
        let slingshot_cookie = COOKIE_WORLD_COOKIES[COOKIE_WORLD_SLINGSHOT_COOKIE];
        let near = |bird: Songbird, object: &Primitive, reach: f32| {
            (bounding_sphere(object).0 - bird.center).length() < bird.radius * reach
        };

        for (index, bird) in cookie_world_birds().into_iter().enumerate() {
            let parts_with = |material: usize, reach: f32| {
                scene
                    .objects()
                    .iter()
                    .filter(|object| object.material_id() == material && near(bird, object, reach))
                    .count()
            };
            let body = scene
                .objects()
                .iter()
                .filter_map(|object| object.as_sphere())
                .find(|sphere| sphere.material_id() == world.bird_body[bird.palette])
                .unwrap();

            assert_eq!(body.center(), bird.center, "bird {index}");
            assert_eq!(body.radius(), COOKIE_WORLD_BIRD_RADIUS);
            // Eyes with a highlight each, pupils, a two part beak and a belly.
            assert_eq!(parts_with(eye, 1.2), 4, "bird {index} eyes");
            assert_eq!(parts_with(pupil, 1.2), 2, "bird {index} pupils");
            assert_eq!(parts_with(world.bird_beak, 1.6), 2, "bird {index} beak");
            assert_eq!(parts_with(world.bird_belly[bird.palette], 1.0), 1);
            // Two wings of four feathers plus their shoulders and a tail.
            assert!(parts_with(world.bird_feather[bird.palette], 2.2) >= 2 * 5 + 3);
            assert!(
                (bird.center - slingshot_cookie.center).length() < 3.0,
                "bird {index} is far from the slingshot"
            );
            assert!((COOKIE_WORLD_PIG_CENTER - bird.center).dot(bird.forward) > 0.0);
        }

        // The standing bird's feet touch the cookie.
        let feet_depth = scene
            .objects()
            .iter()
            .filter(|object| object.material_id() == world.bird_feet)
            .map(|object| {
                let (center, radius) = bounding_sphere(object);
                (center - slingshot_cookie.center).length() - radius - slingshot_cookie.radius
            })
            .fold(f32::INFINITY, f32::min);
        assert!(feet_depth < 0.0);
        assert!(feet_depth > -0.15);
    }

    #[test]
    fn cookie_world_slingshot_reuses_the_luna_azul_slingshot_shape() {
        let (_, level_two) = build_cookie_world_scene_with_metadata().unwrap();
        let (_, blue_moon) = build_blue_moon_scene_with_metadata().unwrap();

        assert_eq!(
            level_two.cookie_world_slingshot_parts,
            blue_moon.blue_moon_slingshot_wood_parts
                + blue_moon.blue_moon_slingshot_joint_parts
                + blue_moon.blue_moon_slingshot_band_parts
        );
    }

    #[test]
    fn cookie_world_cookies_are_well_apart_from_the_bubble_and_in_view() {
        let (scene, metadata) = build_cookie_world_scene_with_metadata().unwrap();
        let camera = cookie_world_orbit_camera(16.0 / 9.0).to_camera();
        let basis = camera.basis();
        let half_height = (camera.vertical_fov_degrees.to_radians() * 0.5).tan();
        let half_width = half_height * camera.aspect_ratio;
        let first_cookie =
            metadata.cookie_world_slingshot_cookie_id.unwrap() - COOKIE_WORLD_SLINGSHOT_COOKIE;

        for (index, cookie) in COOKIE_WORLD_COOKIES.iter().enumerate() {
            let sphere = scene.objects()[first_cookie + index].as_sphere().unwrap();
            let gap = (cookie.center - COOKIE_WORLD_PIG_CENTER).length()
                - COOKIE_WORLD_BUBBLE_RADIUS
                - cookie.radius;
            let offset = cookie.center - camera.position;
            let depth = offset.dot(basis.forward);
            let hit = scene
                .intersect(&Ray::new(camera.position, offset), 0.001, 100.0)
                .unwrap();

            assert_eq!(sphere.center(), cookie.center);
            assert_eq!(sphere.radius(), cookie.radius);
            assert!(gap > 1.0, "cookie {index} is too close to the bubble");
            assert!(gap < 3.0, "cookie {index} is too far from the scene");
            assert!((offset.dot(basis.right) / depth).abs() < half_width * 0.9);
            assert!((offset.dot(basis.up) / depth).abs() < half_height * 0.9);
            assert_eq!(
                hit.material_id,
                sphere.material_id(),
                "cookie {index} is hidden"
            );
        }
    }

    #[test]
    fn cookie_world_horns_rest_against_their_cookies() {
        let (scene, metadata) = build_cookie_world_scene_with_metadata().unwrap();
        let first_cookie =
            metadata.cookie_world_slingshot_cookie_id.unwrap() - COOKIE_WORLD_SLINGSHOT_COOKIE;
        let horn_material = cookie_world_material_ids().horn_waffle;
        let mouths: Vec<_> = scene
            .cones()
            .filter(|cone| cone.material_id() == horn_material)
            .collect();
        let horn_ribs: Vec<_> = scene
            .objects()
            .iter()
            .filter_map(|object| object.as_sphere())
            .filter(|sphere| sphere.material_id() == horn_material)
            .collect();

        assert_eq!(mouths.len(), COOKIE_WORLD_HORNS.len());
        for (mouth, horn) in mouths.into_iter().zip(COOKIE_WORLD_HORNS) {
            let cookie = scene.objects()[first_cookie + horn.cookie]
                .as_sphere()
                .unwrap();
            let gap = |center: Vec3, radius: f32| {
                (center - cookie.center()).length() - cookie.radius() - radius
            };
            // The widest part of the open mouth, around the cone's base.
            let mouth_rim = mouth.center() - mouth.orientation().up() * mouth.half_height();
            let closest = horn_ribs
                .iter()
                .map(|rib| gap(rib.center(), rib.radius()))
                .fold(gap(mouth_rim, mouth.base_radius() * 0.9), f32::min);

            // The horn touches the dough without sinking into it.
            assert!(closest < 0.05, "horn of cookie {} floats", horn.cookie);
            assert!(
                closest > -0.15 * horn.scale,
                "horn of cookie {} sinks",
                horn.cookie
            );
            assert!(gap(mouth.center(), 0.0) > 0.0);
        }
    }

    #[test]
    fn cookie_world_slingshot_stands_on_top_of_the_left_cookie() {
        let (scene, metadata) = build_cookie_world_scene_with_metadata().unwrap();
        let SpaceMaterials {
            slingshot_wood,
            slingshot_band,
            ..
        } = register_space_materials(&mut Scene::new()).unwrap();
        let cookie = scene.objects()[metadata.cookie_world_slingshot_cookie_id.unwrap()]
            .as_sphere()
            .unwrap();
        let up = COOKIE_WORLD_SLINGSHOT_DIRECTION.normalized();
        let height = |point: Vec3| (point - cookie.center()).dot(up);
        let wood: Vec<_> = scene
            .cylinders()
            .filter(|part| part.material_id() == slingshot_wood)
            .collect();
        let tips: Vec<_> = scene
            .objects()
            .iter()
            .filter_map(|object| object.as_sphere())
            .filter(|part| part.material_id() == slingshot_wood)
            .collect();
        let bands = scene
            .cylinders()
            .filter(|part| part.material_id() == slingshot_band)
            .count();
        let trunk = wood
            .iter()
            .min_by(|left, right| height(left.center()).total_cmp(&height(right.center())))
            .unwrap();
        let mut tips = tips;
        tips.sort_by(|left, right| height(right.center()).total_cmp(&height(left.center())));
        let spread = tips[0].center() - tips[1].center();

        assert!(cookie.center().x < COOKIE_WORLD_PIG_CENTER.x - COOKIE_WORLD_BUBBLE_RADIUS);
        assert_eq!(wood.len(), 5);
        assert_eq!(bands, 5);
        // The trunk starts inside the dough and the rest stands above it.
        assert!(height(trunk.center()) - trunk.half_height() < cookie.radius());
        assert!(height(trunk.center()) + trunk.half_height() > cookie.radius());
        for part in &wood {
            assert!(height(part.center()) > cookie.radius() * 0.9);
        }
        // The fork opens across the view, so its Y shape is seen from the camera.
        assert!(spread.x.abs() > spread.z.abs());
    }

    #[test]
    fn cookie_world_cookies_get_their_own_warm_key_lights() {
        let (scene, metadata) = build_cookie_world_scene_with_metadata().unwrap();
        let camera = cookie_world_orbit_camera(16.0 / 9.0).to_camera();
        let first_cookie =
            metadata.cookie_world_slingshot_cookie_id.unwrap() - COOKIE_WORLD_SLINGSHOT_COOKIE;

        for (index, cookie) in COOKIE_WORLD_COOKIES.iter().enumerate() {
            let hit = scene
                .intersect(
                    &Ray::new(camera.position, cookie.center - camera.position),
                    0.001,
                    100.0,
                )
                .unwrap();
            let lit_by_warm_key = scene.lights().iter().any(|light| {
                (light.position - COOKIE_WORLD_PIG_CENTER).length()
                    > COOKIE_WORLD_BUBBLE_RADIUS * 1.3
                    && light.color.r > light.color.b
                    && crate::renderer::is_light_visible(&scene, &hit, light)
            });

            assert_eq!(
                scene.objects()[first_cookie + index].material_id(),
                hit.material_id
            );
            assert!(lit_by_warm_key, "cookie {index} has no visible warm key");
        }
    }

    #[test]
    fn cookie_textures_have_dark_chips_on_warm_dough_and_differ_by_seed() {
        let first = cookie_texture(COOKIE_WORLD_COOKIES[0].seed).unwrap();
        let second = cookie_texture(COOKIE_WORLD_COOKIES[1].seed).unwrap();
        let mut chip_texels = 0;
        let mut different_texels = 0;

        for y in 0..first.height() {
            let left = first.pixel(0, y).unwrap();
            let right = first.pixel(first.width() - 1, y).unwrap();
            assert!(color_delta(left, right) < 0.001);

            for x in 0..first.width() {
                let texel = first.pixel(x, y).unwrap();
                assert!(texel.r >= texel.g && texel.g >= texel.b);
                if luminance(texel) < 0.25 {
                    chip_texels += 1;
                }
                if color_delta(texel, second.pixel(x, y).unwrap()) > 0.05 {
                    different_texels += 1;
                }
            }
        }

        let texel_count = first.width() * first.height();
        assert!(chip_texels > texel_count / 50);
        assert!(chip_texels < texel_count / 4);
        assert!(different_texels > texel_count / 4);
    }

    #[test]
    fn cookie_world_pig_parts_float_inside_the_bubble() {
        let (scene, metadata) = build_cookie_world_scene_with_metadata().unwrap();
        let first_part = metadata.cookie_world_pig_id.unwrap();
        let last_part =
            first_part + metadata.cookie_world_pig_parts + metadata.cookie_world_popcorn_parts;

        for (index, object) in scene.objects()[first_part..last_part].iter().enumerate() {
            let farthest = if let Some(sphere) = object.as_sphere() {
                (sphere.center() - COOKIE_WORLD_PIG_CENTER).length() + sphere.radius()
            } else {
                let cylinder = object.as_cylinder().unwrap();
                (cylinder.center() - COOKIE_WORLD_PIG_CENTER).length()
                    + (cylinder.radius().powi(2) + cylinder.half_height().powi(2)).sqrt()
            };

            assert!(farthest < COOKIE_WORLD_BUBBLE_RADIUS * 0.75, "part {index}");
            assert!(farthest > COOKIE_WORLD_PIG_RADIUS * 0.99, "part {index}");
        }
    }

    #[test]
    fn cookie_world_pig_face_looks_towards_the_initial_camera() {
        let (scene, _) = build_cookie_world_scene_with_metadata().unwrap();
        let SpaceMaterials { eye, pupil, .. } =
            register_space_materials(&mut Scene::new()).unwrap();
        let CookieWorldMaterials {
            pig_snout,
            pig_nostril,
            ..
        } = cookie_world_material_ids();
        let camera = cookie_world_orbit_camera(16.0 / 9.0).to_camera();
        let towards_camera = (camera.position - COOKIE_WORLD_PIG_CENTER).normalized();
        let facing = |center: Vec3| {
            (center - COOKIE_WORLD_PIG_CENTER)
                .normalized()
                .dot(towards_camera)
        };
        let spheres_with = |material_id: usize| -> Vec<_> {
            scene
                .objects()
                .iter()
                .filter_map(|object| object.as_sphere())
                .filter(|sphere| {
                    sphere.material_id() == material_id
                        && (sphere.center() - COOKIE_WORLD_PIG_CENTER).length()
                            < COOKIE_WORLD_BUBBLE_RADIUS
                })
                .collect()
        };
        let eyes = spheres_with(eye);
        let pupils = spheres_with(pupil);
        let snout_parts: Vec<_> = scene
            .cylinders()
            .filter(|cylinder| {
                cylinder.material_id() == pig_snout || cylinder.material_id() == pig_nostril
            })
            .collect();

        assert_eq!(eyes.len(), 2);
        assert_eq!(pupils.len(), 2);
        assert_eq!(snout_parts.len(), 4);
        for (eye, pupil) in eyes.iter().zip(&pupils) {
            assert!(facing(eye.center()) > 0.5);
            assert!((pupil.center() - eye.center()).dot(towards_camera) > 0.0);
            assert!(pupil.radius() < eye.radius());
        }
        for part in snout_parts {
            assert!(facing(part.center()) > 0.9);
            assert!(part.orientation().up().dot(towards_camera) > 0.95);
        }
    }

    #[test]
    fn cookie_world_pig_is_lit_from_inside_the_bubble() {
        let (scene, metadata) = build_cookie_world_scene_with_metadata().unwrap();
        let pig_id = metadata.cookie_world_pig_id.unwrap();
        let camera = cookie_world_orbit_camera(16.0 / 9.0).to_camera();
        let ray = Ray::new(
            camera.position,
            COOKIE_WORLD_PIG_CENTER + Vec3::new(-0.45, 0.45, 0.0) - camera.position,
        );
        let first_hit = scene.intersect(&ray, 0.001, 100.0).unwrap();
        let inner_ray = Ray::new(ray.at(first_hit.distance + 0.01), ray.direction);
        let pig_hit = scene.intersect(&inner_ray, 0.001, 100.0).unwrap();
        let inner_lights: Vec<_> = scene
            .lights()
            .iter()
            .filter(|light| {
                let distance = (light.position - COOKIE_WORLD_PIG_CENTER).length();
                distance > COOKIE_WORLD_PIG_RADIUS * 1.5 && distance < COOKIE_WORLD_BUBBLE_RADIUS
            })
            .collect();
        let rim_lights = scene
            .lights()
            .iter()
            .filter(|light| {
                (light.position - COOKIE_WORLD_PIG_CENTER).length() > COOKIE_WORLD_BUBBLE_RADIUS
            })
            .count();

        assert_eq!(
            first_hit.material_id,
            scene.objects()[metadata.cookie_world_bubble_id.unwrap()].material_id()
        );
        assert_eq!(pig_hit.material_id, scene.objects()[pig_id].material_id());
        assert!(inner_lights.len() >= 3);
        assert!(rim_lights >= 2);
        assert!(
            inner_lights
                .iter()
                .any(|light| crate::renderer::is_light_visible(&scene, &pig_hit, light))
        );
    }

    #[test]
    fn cookie_world_bubble_is_mostly_clear_with_a_brighter_rim() {
        let (scene, _) = build_cookie_world_scene_with_metadata().unwrap();
        let camera = cookie_world_orbit_camera(16.0 / 9.0).to_camera();
        let forward = (COOKIE_WORLD_PIG_CENTER - camera.position).normalized();
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let up = (world_up - forward * world_up.dot(forward)).normalized();
        let sample = |height: f32| {
            let target = COOKIE_WORLD_PIG_CENTER + up * height;
            crate::renderer::trace_primary_ray(
                &Ray::new(camera.position, target - camera.position),
                &scene,
            )
        };
        // Heights on the plane through the bubble center facing the camera,
        // where the bubble's silhouette sits a bit above its radius.
        let distance = (camera.position - COOKIE_WORLD_PIG_CENTER).length();
        let silhouette = COOKIE_WORLD_BUBBLE_RADIUS * distance
            / (distance * distance - COOKIE_WORLD_BUBBLE_RADIUS * COOKIE_WORLD_BUBBLE_RADIUS)
                .sqrt();
        let inside = sample(COOKIE_WORLD_PIG_RADIUS * 1.45);
        let rim = sample(silhouette * 0.985);
        let outside = sample(silhouette * 1.04);

        assert!(luminance(rim) > luminance(inside) + 0.05);
        assert!(color_delta(inside, outside) < 0.20);
    }

    #[test]
    fn pig_skin_texture_has_darker_spots_and_wraps_seamlessly() {
        let texture = pig_skin_texture().unwrap();
        let mut spot_texels = 0;
        let mut base_texels = 0;

        assert_eq!(texture.width(), PIG_SKIN_TEXTURE_WIDTH);
        assert_eq!(texture.height(), PIG_SKIN_TEXTURE_HEIGHT);
        for y in 0..texture.height() {
            let left = texture.pixel(0, y).unwrap();
            let right = texture.pixel(texture.width() - 1, y).unwrap();
            assert!(color_delta(left, right) < 0.001);

            for x in 0..texture.width() {
                let texel = texture.pixel(x, y).unwrap();
                if color_delta(texel, PIG_SKIN_BASE_COLOR) < 0.001 {
                    base_texels += 1;
                } else if luminance(texel) < luminance(PIG_SKIN_BASE_COLOR) * 0.85 {
                    spot_texels += 1;
                }
                assert!(texel.g > texel.r && texel.g > texel.b);
            }
        }

        assert!(spot_texels > 0);
        assert!(base_texels > spot_texels);
    }

    #[test]
    fn level_three_scene_recreates_the_asteroid_bridge_level() {
        let (scene, metadata) = build_level_three_scene_with_metadata().unwrap();
        let main = metadata.level_three_planet.unwrap();
        let secondary = metadata.level_three_secondary_asteroid.unwrap();
        let slingshot_asteroid = metadata.level_three_slingshot_asteroid.unwrap();

        // Both asteroids are balls of cubes of the same size.
        let main_edge = assert_voxel_ball(
            &scene,
            main,
            LEVEL_THREE_PLANET_CENTER,
            LEVEL_THREE_MAIN_ASTEROID_RADIUS,
        );
        let secondary_edge = assert_voxel_ball(
            &scene,
            secondary,
            level_three_secondary_asteroid_center(),
            LEVEL_THREE_SECONDARY_ASTEROID_RADIUS,
        );
        assert!((main_edge - LEVEL_THREE_CUBE_EDGE).abs() < 1.0e-5);
        assert!((secondary_edge - LEVEL_THREE_CUBE_EDGE).abs() < 1.0e-5);
        assert!(secondary.cube_count < main.cube_count);
        assert_eq!(main.core_count, 1);
        assert_eq!(metadata.level_three_pig_count, LEVEL_THREE_PIG_COUNT);
        assert_eq!(metadata.pig_count, LEVEL_THREE_PIG_COUNT);
        assert_eq!(metadata.tnt_parts, 3);
        assert!(metadata.level_three_bridge_parts >= 60);
        assert!(metadata.level_three_tower_parts >= 15);
        assert!(metadata.level_three_lower_structure_parts >= 10);
        assert_eq!(metadata.level_three_rock_count, LEVEL_THREE_ROCKS.len());
        assert!(metadata.level_three_rock_parts > LEVEL_THREE_ROCKS.len() * 20);
        assert_eq!(scene.curved_tetrahedron_count(), 1);
        // Two asteroids, two atmospheres, the structures, the rocks and the
        // slingshot asteroid with its slingshot.
        assert_eq!(
            scene.object_count(),
            main.total()
                + secondary.total()
                + 2
                + metadata.level_three_bridge_parts
                + metadata.level_three_tower_parts
                + metadata.level_three_lower_structure_parts
                + metadata.level_three_rock_parts
                + metadata.level_three_pig_count * 8
                + slingshot_asteroid.total()
                + metadata.level_three_slingshot_parts
                + metadata.level_three_bird_parts
        );
        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 3);
    }

    /// Bounding sphere of any level three primitive.
    fn level_three_bounds(object: &Primitive) -> (Vec3, f32) {
        if let Some(cube) = object.as_cube() {
            (
                (cube.min + cube.max) * 0.5,
                (cube.max - cube.min).length() * 0.5,
            )
        } else if let Some(sphere) = object.as_sphere() {
            (sphere.center(), sphere.radius())
        } else if let Some(cylinder) = object.as_cylinder() {
            (
                cylinder.center(),
                cylinder.radius().hypot(cylinder.half_height()),
            )
        } else if let Some(cone) = object.as_cone() {
            (cone.center(), cone.base_radius().hypot(cone.half_height()))
        } else if let Some(part) = object.as_oriented_box() {
            (part.center(), part.half_extents().length())
        } else {
            let tetrahedron = object.as_curved_tetrahedron().unwrap();
            (tetrahedron.center(), tetrahedron.circumradius())
        }
    }

    #[test]
    fn level_three_atmospheres_wrap_each_asteroid_and_its_structures() {
        let (scene, metadata) = build_level_three_scene_with_metadata().unwrap();
        let SpaceMaterials {
            level_three_atmosphere,
            ..
        } = register_space_materials(&mut Scene::new()).unwrap();
        let main_id = metadata.level_three_main_atmosphere_id.unwrap();
        let secondary_id = metadata.level_three_secondary_atmosphere_id.unwrap();
        let main = scene.objects()[main_id].as_sphere().unwrap();
        let secondary = scene.objects()[secondary_id].as_sphere().unwrap();
        let material = scene.material(level_three_atmosphere).unwrap();
        let slingshot_asteroid = metadata.level_three_slingshot_asteroid.unwrap();
        let asteroid_parts =
            slingshot_asteroid.first_id..slingshot_asteroid.first_id + slingshot_asteroid.total();
        // The slingshot and its birds float outside both atmospheres.
        let slingshot_parts = asteroid_parts.end
            ..asteroid_parts.end
                + metadata.level_three_slingshot_parts
                + metadata.level_three_bird_parts;
        let rock_parts = slingshot_asteroid.first_id - metadata.level_three_rock_parts
            ..slingshot_asteroid.first_id;
        let inside = |atmosphere: &Sphere, (center, radius): (Vec3, f32)| {
            (center - atmosphere.center()).length() + radius < atmosphere.radius()
        };
        let outside = |atmosphere: &Sphere, (center, radius): (Vec3, f32)| {
            (center - atmosphere.center()).length() - radius > atmosphere.radius()
        };

        assert_eq!(main.center(), LEVEL_THREE_PLANET_CENTER);
        assert_eq!(main.radius(), LEVEL_THREE_MAIN_ATMOSPHERE_RADIUS);
        assert_eq!(main.material_id(), level_three_atmosphere);
        assert_eq!(secondary.center(), level_three_secondary_asteroid_center());
        assert_eq!(secondary.radius(), LEVEL_THREE_SECONDARY_ATMOSPHERE_RADIUS);
        assert_eq!(secondary.material_id(), level_three_atmosphere);
        assert!(material.transparency > 0.8 && material.transparency < 1.0);
        assert_eq!(material.refractive_index, 1.0);
        assert!(material.albedo.r > material.albedo.g * 2.0);
        // Both fields overlap around the bridge, like in the reference level.
        assert!((main.center() - secondary.center()).length() < main.radius() + secondary.radius());

        for (index, object) in scene.objects().iter().enumerate() {
            if index == main_id || index == secondary_id {
                continue;
            }
            let bounds = level_three_bounds(object);

            if rock_parts.contains(&index)
                || asteroid_parts.contains(&index)
                || slingshot_parts.contains(&index)
            {
                assert!(
                    outside(main, bounds) && outside(secondary, bounds),
                    "part {index} at {:?} (radius {}) is inside an atmosphere",
                    bounds.0,
                    bounds.1
                );
            } else {
                assert!(
                    inside(main, bounds) || inside(secondary, bounds),
                    "part {index} at {:?} (radius {}) is outside both atmospheres",
                    bounds.0,
                    bounds.1
                );
            }
        }
    }

    #[test]
    fn level_three_is_lit_inside_each_atmosphere_and_outside() {
        let (scene, metadata) = build_level_three_scene_with_metadata().unwrap();
        let camera = level_three_orbit_camera(16.0 / 9.0).to_camera();
        let atmospheres = [
            metadata.level_three_main_atmosphere_id.unwrap(),
            metadata.level_three_secondary_atmosphere_id.unwrap(),
        ];
        let bridge_middle = LEVEL_THREE_PLANET_CENTER
            + LEVEL_THREE_BRIDGE_DIRECTION.normalized() * LEVEL_THREE_LENS_DISTANCE;
        let targets = [
            LEVEL_THREE_PLANET_CENTER,
            level_three_secondary_asteroid_center(),
            bridge_middle,
            LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER,
        ];
        // First surface that is not an atmosphere along the view ray.
        let first_surface = |target: Vec3| {
            let mut ray = Ray::new(camera.position, target - camera.position);
            loop {
                let hit = scene.intersect(&ray, 0.001, 100.0)?;
                let is_atmosphere = atmospheres.iter().any(|&id| {
                    scene.objects()[id].material_id() == hit.material_id
                        && (hit.position - level_three_bounds(&scene.objects()[id]).0).length()
                            > level_three_bounds(&scene.objects()[id]).1 - 0.01
                });
                if !is_atmosphere {
                    break Some(hit);
                }
                ray = Ray::new(
                    hit.position + ray.direction.normalized() * 0.01,
                    ray.direction,
                );
            }
        };
        // The asteroids are made of cubes: next to each target there are
        // dark seams and side faces turned away from the lights, so the probe
        // is the first cube face near the target that faces the camera.
        let nudges = (0..5).flat_map(|row| {
            (0..5).map(move |column| {
                Vec3::new(column as f32 - 2.0, row as f32 - 2.0, 0.0)
                    * (LEVEL_THREE_CUBE_EDGE * 0.4)
            })
        });

        for target in targets {
            let hit = nudges
                .clone()
                .filter_map(|nudge| first_surface(target + nudge))
                .find(|hit| hit.normal.z > 0.9)
                .or_else(|| first_surface(target))
                .unwrap();

            assert!(
                scene
                    .lights()
                    .iter()
                    .any(|light| crate::renderer::is_light_visible(&scene, &hit, light)),
                "nothing lights the surface seen towards {target:?}"
            );
        }
    }

    #[test]
    fn level_three_structures_stand_on_the_asteroid_cubes() {
        let shape = [(LEVEL_THREE_PLANET_CENTER, LEVEL_THREE_MAIN_ASTEROID_RADIUS)];
        let top = |direction: Vec3| {
            voxel::surface_distance(
                &shape,
                LEVEL_THREE_CUBE_EDGE,
                LEVEL_THREE_PLANET_CENTER,
                direction,
            )
        };

        for degrees in [30.0_f32, 12.0, 54.0, 135.0, 210.0, 300.0] {
            let angle = degrees.to_radians();
            let up = Vec3::new(angle.sin(), angle.cos(), 0.0);
            let frame = LayoutFrame::on_main_asteroid(angle).unwrap();
            let ground = (frame.origin - LEVEL_THREE_PLANET_CENTER).dot(up);

            assert!((frame.basis.up() - up).length() < 1.0e-5);
            // Within a cube of the old sphere, and never over a lower step of
            // the cubes under the footprint.
            assert!(ground > LEVEL_THREE_MAIN_ASTEROID_RADIUS - LEVEL_THREE_CUBE_EDGE);
            assert!(ground <= top(up) + 1.0e-4);
            for sample in -4..=4 {
                let spread = LEVEL_THREE_FOOTPRINT_HALF_WIDTH / LEVEL_THREE_MAIN_ASTEROID_RADIUS;
                let side = angle + spread * sample as f32 / 4.0;
                let direction = Vec3::new(side.sin(), side.cos(), 0.0);

                assert!(top(direction) * direction.dot(up) >= ground - 1.0e-4);
            }
        }
    }

    #[test]
    fn level_three_slingshot_asteroid_floats_under_the_bridge() {
        let (scene, metadata) = build_level_three_scene_with_metadata().unwrap();
        let (_, blue_moon) = build_blue_moon_scene_with_metadata().unwrap();
        let SpaceMaterials { slingshot_wood, .. } =
            register_space_materials(&mut Scene::new()).unwrap();
        let asteroid = metadata.level_three_slingshot_asteroid.unwrap();
        let center = LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER;
        let edge = assert_voxel_ball(
            &scene,
            asteroid,
            center,
            LEVEL_THREE_SLINGSHOT_ASTEROID_RADIUS,
        );
        let up = LEVEL_THREE_SLINGSHOT_DIRECTION.normalized();
        let ground = voxel::surface_distance(
            &[(center, LEVEL_THREE_SLINGSHOT_ASTEROID_RADIUS)],
            edge,
            center,
            up,
        );
        let height = |point: Vec3| (point - center).dot(up);
        let trunk = scene
            .cylinders()
            .filter(|part| {
                part.material_id() == slingshot_wood && (part.center() - center).length() < 2.0
            })
            .min_by(|left, right| height(left.center()).total_cmp(&height(right.center())))
            .unwrap();

        // Small asteroids still have enough cubes to look round.
        assert!(LEVEL_THREE_SLINGSHOT_ASTEROID_RADIUS * 2.0 / edge >= 7.0 - 1.0e-3);
        assert!(center.y < LEVEL_THREE_PLANET_CENTER.y);
        assert!(center.y < level_three_secondary_asteroid_center().y);
        assert_eq!(
            metadata.level_three_slingshot_parts,
            blue_moon.blue_moon_slingshot_wood_parts
                + blue_moon.blue_moon_slingshot_joint_parts
                + blue_moon.blue_moon_slingshot_band_parts
        );
        // The trunk starts inside the top cubes and the slingshot stands on
        // top.
        assert!(height(trunk.center()) - trunk.half_height() < ground);
        assert!(height(trunk.center()) + trunk.half_height() > ground);
    }

    #[test]
    fn level_three_has_three_birds_at_the_slingshot() {
        let (scene, metadata) = build_level_three_scene_with_metadata().unwrap();
        let asteroid = metadata.level_three_slingshot_asteroid.unwrap();
        let first_bird =
            asteroid.first_id + asteroid.total() + metadata.level_three_slingshot_parts;
        let birds = &scene.objects()[first_bird..first_bird + metadata.level_three_bird_parts];
        // Each bird starts with its round body.
        let bodies: Vec<_> = birds
            .iter()
            .filter_map(Primitive::as_sphere)
            .filter(|sphere| sphere.radius() > 0.12)
            .collect();

        assert_eq!(metadata.level_three_bird_count, 3);
        assert_eq!(bodies.len(), 3);
        // The loaded bird sits over the slingshot, the others next to the
        // asteroid, all of them close to it.
        assert!(bodies[0].center().y > LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER.y + 0.5);
        for body in &bodies {
            let distance = (body.center() - LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER).length();
            assert!(distance > LEVEL_THREE_SLINGSHOT_ASTEROID_RADIUS);
            assert!(distance < LEVEL_THREE_SLINGSHOT_ASTEROID_RADIUS + 1.2);
        }
    }

    #[test]
    fn level_three_bridge_reaches_both_asteroids() {
        let (scene, _) = build_level_three_scene_with_metadata().unwrap();
        let main = (LEVEL_THREE_PLANET_CENTER, LEVEL_THREE_MAIN_ASTEROID_RADIUS);
        let secondary = (
            level_three_secondary_asteroid_center(),
            LEVEL_THREE_SECONDARY_ASTEROID_RADIUS,
        );
        let axis = (secondary.0 - main.0).normalized();
        let gap = (secondary.0 - main.0).length() - main.1 - secondary.1;
        let bridge_length = LEVEL_THREE_BLOCK * LEVEL_THREE_BRIDGE_LEVELS as f32;

        // The bridge runs along the line between both centers and is a little
        // longer than the gap, so each end rests inside one asteroid.
        assert!(bridge_length > gap);
        assert!(bridge_length - gap < 0.1);
        assert!((axis - LEVEL_THREE_BRIDGE_DIRECTION.normalized()).length() < 0.001);

        // Some bridge block touches each asteroid.
        let boxes: Vec<_> = scene.oriented_boxes().collect();
        for (center, radius) in [main, secondary] {
            let touching = boxes.iter().any(|part| {
                let distance = (part.center() - center).length();

                distance < radius + LEVEL_THREE_BLOCK * 0.6 && part.center().z.abs() < 1.0
            });

            assert!(touching);
        }
    }

    #[test]
    fn level_three_pigs_do_not_overlap_blocks() {
        const TOLERANCE: f32 = 0.002;
        let (scene, _) = build_level_three_scene_with_metadata().unwrap();
        let SpaceMaterials { pig, .. } = register_space_materials(&mut Scene::new()).unwrap();
        let pigs: Vec<_> = scene
            .objects()
            .iter()
            .filter_map(|primitive| primitive.as_sphere())
            .filter(|sphere| sphere.material_id() == pig)
            .collect();

        assert_eq!(pigs.len(), LEVEL_THREE_PIG_COUNT);

        for body in pigs {
            for part in scene.oriented_boxes() {
                let local = part
                    .orientation()
                    .world_to_local_vector(body.center() - part.center());
                let half = part.half_extents();
                let closest = Vec3::new(
                    local.x.clamp(-half.x, half.x),
                    local.y.clamp(-half.y, half.y),
                    local.z.clamp(-half.z, half.z),
                );

                assert!(
                    (local - closest).length() >= body.radius() - TOLERANCE,
                    "pig at {:?} overlaps the block at {:?}",
                    body.center(),
                    part.center()
                );
            }
        }
    }

    #[test]
    fn level_three_ice_is_refractive_and_block_textures_load() {
        let (scene, _) = build_level_three_scene_with_metadata().unwrap();
        let SpaceMaterials {
            ice,
            level_three_stone,
            ..
        } = register_space_materials(&mut Scene::new()).unwrap();
        let ice_material = scene.material(ice).unwrap();
        let stone_material = scene.material(level_three_stone).unwrap();

        assert!(ice_material.transparency > 0.1 && ice_material.transparency < 0.5);
        assert!(ice_material.refractive_index > 1.2);
        assert!(scene.texture(ice_material.texture_id.unwrap()).is_some());
        assert!(scene.texture(stone_material.texture_id.unwrap()).is_some());
        assert!(scene.oriented_boxes().any(|part| part.material_id() == ice));

        for path in [ICE_BLOCK_TEXTURE_PATH, GRAY_STONE_BLOCK_TEXTURE_PATH] {
            let texture = Texture::from_ppm_file(path).unwrap();

            assert_eq!(texture.width(), 32);
            assert_eq!(texture.height(), 32);
        }
    }

    #[test]
    fn level_three_camera_frames_both_asteroids() {
        let orbit = level_three_orbit_camera(16.0 / 9.0);
        let camera = orbit.to_camera();
        let basis = camera.basis();
        let half_height = (camera.vertical_fov_degrees.to_radians() * 0.5).tan();
        let half_width = half_height * camera.aspect_ratio;

        for (center, radius) in [
            (LEVEL_THREE_PLANET_CENTER, LEVEL_THREE_MAIN_ASTEROID_RADIUS),
            (
                level_three_secondary_asteroid_center(),
                LEVEL_THREE_SECONDARY_ASTEROID_RADIUS,
            ),
            (
                LEVEL_THREE_SLINGSHOT_ASTEROID_CENTER,
                LEVEL_THREE_SLINGSHOT_ASTEROID_RADIUS,
            ),
        ] {
            let offset = center - camera.position;
            let depth = offset.dot(basis.forward);

            assert!(offset.length() > radius * 1.3);
            assert!(depth > 0.0);
            assert!((offset.dot(basis.right) / depth).abs() < half_width);
            assert!((offset.dot(basis.up) / depth).abs() < half_height);
        }
    }

    #[test]
    fn space_levels_scene_has_no_pigs() {
        let (_, metadata) = build_space_levels_scene_with_metadata().unwrap();

        assert_eq!(metadata.pig_count, 0);
    }

    #[test]
    fn space_levels_scene_has_skybox_lights_and_scene_budget() {
        let (scene, metadata) = build_space_levels_scene_with_metadata().unwrap();
        let voxel_parts = metadata.blue_moon_planet.unwrap().total()
            + metadata.blue_moon_small_moon.unwrap().total()
            + metadata.blue_moon_floating_asteroid_parts;

        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 7);
        assert!(scene.object_count() - voxel_parts >= 40);
        // Budget: the diorama parts plus a few thousand cubes.
        assert!(scene.object_count() - voxel_parts < 260);
        assert!(scene.cube_count() < 5_000);

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

        assert!(
            (camera.position - COOKIE_WORLD_PIG_CENTER).length()
                > COOKIE_WORLD_BUBBLE_RADIUS * 1.30
        );
        assert!(hit.distance > 1.0);
    }
}
