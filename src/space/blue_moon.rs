//! Level one, Luna Azul: a gray cratered moon whose top is covered by dark
//! soil mounds with tufts of grass and a garlic bulb, a moon rover parked on
//! the soil puffing smoke, four floating bubbles with a radish, a carrot and
//! two moon rocks, and a small moon on the upper left with the slingshot. A
//! single atmosphere wraps the whole diorama and a few purple asteroids float
//! outside it. Both moons and the asteroids are balls of small cubes (see
//! `voxel`), and everything on the big moon stands on the top of its cubes.
//!
//! The level is laid out in the XY plane like the reference picture: Y up,
//! +X to the right and +Z towards the default camera. Directions from a moon
//! center are given by an `angle` in degrees, clockwise from +Y as seen from
//! the camera, and a `depth` in degrees that tilts them towards the camera.
//!
//! Shadow rays stop at every object, including transparent ones, so the
//! lights of the diorama are inside the atmosphere, each bubble has a small
//! light next to its content and the rover's glass dome has its own light.
//! Lights outside the atmosphere only reach its shell and the asteroids.

use super::{
    BLUE_MOON_PLANET_CENTER, BLUE_MOON_PLANET_RADIUS, SpaceBuildError, SpaceMaterials,
    SpaceSceneMetadata, add_slingshot, add_voxel_rock, basis_with_up, hash3, rock_texture,
    smoothstep, sphere_direction_from_uv, value_noise_3d,
    voxel::{self, VoxelBall, VoxelBody},
};
use crate::{
    basis::Basis3,
    color::Color,
    cone::Cone,
    cylinder::Cylinder,
    light::PointLight,
    material::Material,
    math::{Vec2, Vec3},
    oriented_box::OrientedBox,
    radial::RadialFrame,
    scene::Scene,
    sphere::Sphere,
    texture::{Texture, WrapMode},
};

/// The atmosphere is centered between the big moon, the small moon and the
/// bubbles, so it wraps the whole diorama with a small margin. The level
/// camera orbits around this point.
pub(super) const BLUE_MOON_ATMOSPHERE_CENTER: Vec3 = Vec3::new(-0.30, 1.05, 0.0);
pub(super) const BLUE_MOON_ATMOSPHERE_RADIUS: f32 = 3.0;
pub(super) const BLUE_MOON_SMALL_MOON_CENTER: Vec3 = Vec3::new(-2.15, 2.05, -0.10);
pub(super) const BLUE_MOON_SMALL_MOON_RADIUS: f32 = 0.42;
/// Direction of the sun drawn in the Luna Azul skybox, seen at the top left
/// of the initial view. The outside key light comes from there.
pub(super) const BLUE_MOON_SUN_DIRECTION: Vec3 = Vec3::new(-0.608, 0.326, -0.724);
pub(super) const BLUE_MOON_SUN_DISTANCE: f32 = 10.0;

/// Edge of the cubes that build the moons and asteroids of level one.
pub(super) const BLUE_MOON_CUBE_EDGE: f32 = 0.105;
/// The small moon and the asteroids are at least this many cubes across.
const BLUE_MOON_MIN_CUBES_ACROSS: f32 = 7.0;

const MOON_TEXTURE_WIDTH: usize = 512;
const MOON_TEXTURE_HEIGHT: usize = 256;
const SOIL_TEXTURE_WIDTH: usize = 512;
const SOIL_TEXTURE_HEIGHT: usize = 256;
const GARLIC_TEXTURE_WIDTH: usize = 256;
const GARLIC_TEXTURE_HEIGHT: usize = 128;

const MOON_LIGHT: Color = Color::new(0.60, 0.61, 0.66);
const MOON_BASE: Color = Color::new(0.48, 0.49, 0.55);
const MOON_CRATER_FLOOR: Color = Color::new(0.35, 0.34, 0.45);
const MOON_CRATER_SHADOW: Color = Color::new(0.25, 0.23, 0.36);
const MOON_CRATER_LIT: Color = Color::new(0.55, 0.53, 0.66);
const MOON_CRATER_RIM: Color = Color::new(0.70, 0.71, 0.77);
/// Craters are painted as if lit from the upper left, like the key light.
const MOON_CRATER_LIGHT_DIRECTION: Vec3 = Vec3::new(-0.55, 0.65, 0.52);
/// Big craters on the face of the moon that looks at the camera, below the
/// soil: direction and angular radius in radians.
const MOON_FRONT_CRATERS: [(Vec3, f32); 7] = [
    (Vec3::new(0.18, -0.30, 0.94), 0.34),
    (Vec3::new(-0.58, -0.08, 0.81), 0.24),
    (Vec3::new(0.64, -0.50, 0.58), 0.22),
    (Vec3::new(-0.24, -0.80, 0.55), 0.20),
    (Vec3::new(0.78, 0.05, 0.62), 0.15),
    (Vec3::new(-0.78, -0.52, 0.34), 0.17),
    (Vec3::new(0.30, 0.22, 0.93), 0.10),
];
/// Smaller craters spread over the rest of the moon.
const MOON_SCATTERED_CRATER_COUNT: usize = 22;

const SOIL_DARK: Color = Color::new(0.13, 0.145, 0.19);
const SOIL_LIGHT: Color = Color::new(0.24, 0.26, 0.32);
const SOIL_PEBBLE: Color = Color::new(0.38, 0.40, 0.47);

const GARLIC_SKIN: Color = Color::new(0.97, 0.94, 0.86);
const GARLIC_LINE: Color = Color::new(0.66, 0.56, 0.60);
const GARLIC_ROOT: Color = Color::new(0.80, 0.70, 0.56);
const GARLIC_CLOVES: f32 = 7.0;

/// A soil mound: a sphere whose center lies inside the moon, so only its cap
/// shows above the surface.
#[derive(Debug, Clone, Copy)]
pub(super) struct SoilMound {
    pub angle: f32,
    pub depth: f32,
    /// Distance from the moon center to the mound's center.
    pub distance: f32,
    pub radius: f32,
}

impl SoilMound {
    pub(super) fn center(self) -> Vec3 {
        BLUE_MOON_PLANET_CENTER + surface_direction(self.angle, self.depth) * self.distance
    }
}

const fn mound(angle: f32, depth: f32, distance: f32, radius: f32) -> SoilMound {
    SoilMound {
        angle,
        depth,
        distance,
        radius,
    }
}

/// The first mound is the wide soil layer over the top of the moon, where the
/// rover stands; the others are the hills around it.
pub(super) const SOIL_MOUNDS: [SoilMound; 7] = [
    mound(6.0, 0.0, 0.35, 1.30),
    // Big hill on the left, with the garlic.
    mound(-40.0, 12.0, 1.10, 0.60),
    mound(-64.0, 20.0, 1.12, 0.40),
    mound(44.0, 10.0, 1.16, 0.46),
    mound(-12.0, -42.0, 1.14, 0.56),
    mound(30.0, 44.0, 1.13, 0.38),
    mound(-28.0, 46.0, 1.12, 0.42),
];
/// Axis of the soil layer: tufts of grass grow along its edge.
const SOIL_LAYER_ANGLE: f32 = 6.0;
/// Soil thinner than this counts as bare moon when looking for the edge.
const SOIL_EDGE_THICKNESS: f32 = 0.035;
/// Tufts sit this many degrees inside the soil edge so their base is on soil.
const SOIL_EDGE_INSET_DEGREES: f32 = 2.5;

/// A tuft of grass: where it grows, how many blades it has and how long they
/// are relative to `GRASS_BLADE_LENGTH`.
#[derive(Debug, Clone, Copy)]
pub(super) struct GrassTuft {
    pub placement: TuftPlacement,
    pub blades: usize,
    pub scale: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum TuftPlacement {
    /// On the edge of the soil layer, `azimuth` degrees around its axis:
    /// 0 is the right side, 90 the side facing the camera, 180 the left side.
    SoilEdge { azimuth: f32 },
    /// On top of the ground in a given direction from the moon center.
    Direction { angle: f32, depth: f32 },
}

const fn edge_tuft(azimuth: f32, blades: usize, scale: f32) -> GrassTuft {
    GrassTuft {
        placement: TuftPlacement::SoilEdge { azimuth },
        blades,
        scale,
    }
}

const fn ground_tuft(angle: f32, depth: f32, blades: usize, scale: f32) -> GrassTuft {
    GrassTuft {
        placement: TuftPlacement::Direction { angle, depth },
        blades,
        scale,
    }
}

/// Denser and taller on the left and right sides, like the reference.
pub(super) const GRASS_TUFTS: [GrassTuft; 20] = [
    edge_tuft(-12.0, 7, 1.00),
    edge_tuft(8.0, 6, 0.85),
    edge_tuft(26.0, 5, 0.75),
    edge_tuft(56.0, 5, 0.70),
    edge_tuft(73.0, 5, 0.65),
    edge_tuft(90.0, 6, 0.80),
    edge_tuft(107.0, 5, 0.65),
    edge_tuft(124.0, 5, 0.70),
    edge_tuft(150.0, 6, 0.95),
    edge_tuft(168.0, 7, 1.15),
    edge_tuft(186.0, 7, 1.05),
    edge_tuft(204.0, 5, 0.85),
    edge_tuft(236.0, 5, 0.80),
    edge_tuft(270.0, 6, 0.90),
    edge_tuft(304.0, 5, 0.80),
    edge_tuft(334.0, 6, 0.95),
    // On top of the hills.
    ground_tuft(-52.0, 2.0, 6, 0.90),
    ground_tuft(-34.0, 30.0, 5, 0.70),
    ground_tuft(47.0, -6.0, 5, 0.80),
    ground_tuft(-14.0, -48.0, 6, 0.85),
];
const GRASS_BLADE_LENGTH: f32 = 0.28;
const GRASS_BLADE_RADIUS: f32 = 0.042;
/// Blades start this far below the ground so their base never shows.
const GRASS_BLADE_SINK: f32 = 0.02;

const GARLIC_ANGLE: f32 = -44.0;
const GARLIC_DEPTH: f32 = 24.0;
const GARLIC_BULB_RADIUS: f32 = 0.13;
/// Fraction of the bulb buried in the soil.
const GARLIC_BURIED: f32 = 0.45;
/// One leaf on each side of the neck.
const GARLIC_LEAVES: usize = 2;

/// The rover is parked on the soil layer, a little right of the top.
pub(super) const ROVER_ANGLE: f32 = 8.0;
/// The rover is modeled in local units and drawn this much bigger.
pub(super) const ROVER_SCALE: f32 = 1.2;
pub(super) const ROVER_WHEEL_RADIUS: f32 = 0.13;
const ROVER_WHEEL_HALF_WIDTH: f32 = 0.045;
/// Wheel positions along the rover (X) and across it (Z).
pub(super) const ROVER_WHEELS: [(f32, f32); 4] =
    [(-0.30, -0.22), (-0.30, 0.22), (0.30, -0.22), (0.30, 0.22)];
/// Wheels sink this much into the soil so they rest on it.
const ROVER_WHEEL_SINK: f32 = 0.015;
pub(super) const ROVER_DOME_CENTER: Vec3 = Vec3::new(-0.04, 0.45, 0.0);
pub(super) const ROVER_DOME_RADIUS: f32 = 0.17;
const ROVER_EXHAUST_BASE: Vec3 = Vec3::new(-0.31, 0.34, -0.08);
const ROVER_EXHAUST_TOP: Vec3 = Vec3::new(-0.35, 0.64, -0.08);
const ROVER_EXHAUST_RADIUS: f32 = 0.034;
/// Smoke puffs rising from the exhaust and drifting back: local center and
/// radius.
pub(super) const ROVER_SMOKE_PUFFS: [(Vec3, f32); 4] = [
    (Vec3::new(-0.37, 0.74, -0.08), 0.050),
    (Vec3::new(-0.42, 0.85, -0.07), 0.062),
    (Vec3::new(-0.50, 0.97, -0.06), 0.074),
    (Vec3::new(-0.60, 1.08, -0.05), 0.060),
];

pub(super) const BUBBLE_RADIUS: f32 = 0.25;
pub(super) const BUBBLES: [Bubble; 4] = [
    Bubble {
        center: Vec3::new(-0.85, 2.55, 0.30),
        content: BubbleContent::Rock { seed: 2.0 },
    },
    Bubble {
        center: Vec3::new(0.25, 3.05, 0.10),
        content: BubbleContent::Radish,
    },
    Bubble {
        center: Vec3::new(0.95, 2.85, -0.05),
        content: BubbleContent::Rock { seed: 7.0 },
    },
    Bubble {
        center: Vec3::new(1.45, 2.25, 0.25),
        content: BubbleContent::Carrot,
    },
];
/// Light inside each bubble, towards the camera and the upper left of its
/// content.
const BUBBLE_LIGHT_OFFSET: Vec3 = Vec3::new(-0.09, 0.06, 0.19);
const BUBBLE_LIGHT_INTENSITY: f32 = 0.42;

#[derive(Debug, Clone, Copy)]
pub(super) struct Bubble {
    pub center: Vec3,
    pub content: BubbleContent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum BubbleContent {
    Radish,
    Carrot,
    Rock { seed: f32 },
}

/// The slingshot stands on top of the small moon, leaning a little towards
/// the rover.
const SMALL_MOON_SLINGSHOT_DIRECTION: Vec3 = Vec3::new(0.14, 1.0, 0.0);
const SMALL_MOON_SLINGSHOT_SCALE: f32 = 1.0;
const SMALL_MOON_SLINGSHOT_EMBED: f32 = 0.03;
const SMALL_MOON_SLINGSHOT_YAW_RADIANS: f32 = -std::f32::consts::FRAC_PI_2 + 0.30;

/// Purple asteroids floating outside the atmosphere: center and radius.
pub(super) const FLOATING_ASTEROIDS: [(Vec3, f32); 4] = [
    (Vec3::new(-3.95, 3.35, -1.30), 0.20),
    (Vec3::new(2.75, 4.35, -1.70), 0.15),
    (Vec3::new(-4.40, -0.55, -1.10), 0.16),
    (Vec3::new(3.75, 1.05, -0.90), 0.18),
];

/// Lights inside the atmosphere, relative to its center: a warm key from the
/// upper left front, a cool fill from the right, a lilac fill under the moon
/// and a rim from behind the top.
const INSIDE_LIGHTS: [(Vec3, Color, f32); 4] = [
    (
        Vec3::new(-0.90, 1.20, 2.50),
        Color::new(1.0, 0.94, 0.84),
        7.5,
    ),
    (
        Vec3::new(2.20, 0.30, 1.80),
        Color::new(0.74, 0.84, 1.0),
        2.8,
    ),
    (
        Vec3::new(-0.30, -2.40, 1.30),
        Color::new(0.72, 0.64, 1.0),
        1.4,
    ),
    (
        Vec3::new(0.80, 1.20, -2.30),
        Color::new(0.80, 0.90, 1.0),
        3.2,
    ),
];
/// Lights outside, around the atmosphere's silhouette as seen from the
/// camera, that brighten its rim.
const ATMOSPHERE_RIM_LIGHTS: [(Vec3, Color, f32); 2] = [
    (
        Vec3::new(-3.70, 1.80, 0.0),
        Color::new(0.82, 0.86, 1.0),
        7.0,
    ),
    (
        Vec3::new(3.60, -1.50, 0.0),
        Color::new(0.76, 0.72, 1.0),
        6.0,
    ),
];
const DOME_LIGHT_OFFSET: Vec3 = Vec3::new(0.05, 0.10, 0.09);
const DOME_LIGHT_INTENSITY: f32 = 0.25;
const SUN_KEY_INTENSITY: f32 = 45.0;

/// Materials used only by level one. They are registered by the level one
/// builder, so the other scenes do not generate its procedural textures.
#[derive(Debug, Clone, Copy)]
pub(super) struct BlueMoonMaterials {
    pub moon: usize,
    pub soil: usize,
    pub grass_light: usize,
    pub grass_dark: usize,
    pub garlic: usize,
    pub rover_paint: usize,
    pub rover_dark: usize,
    pub rover_metal: usize,
    pub tire: usize,
    pub glass: usize,
    pub seat: usize,
    pub headlight: usize,
    pub beacon: usize,
    pub smoke: usize,
    pub bubble: usize,
    pub radish: usize,
    pub radish_root: usize,
    pub carrot: usize,
    pub rock: usize,
    pub asteroid: usize,
    pub atmosphere: usize,
}

/// Direction from a moon center, `angle` degrees clockwise from +Y as seen
/// from the camera and tilted `depth` degrees towards the camera.
pub(super) fn surface_direction(angle: f32, depth: f32) -> Vec3 {
    let (sin_angle, cos_angle) = angle.to_radians().sin_cos();
    let (sin_depth, cos_depth) = depth.to_radians().sin_cos();

    Vec3::new(sin_angle * cos_depth, cos_angle * cos_depth, sin_depth)
}

/// Center and radius of the balls that build the big moon's ground: the moon
/// and the soil mounds over it, in the order they are painted.
fn ground_balls() -> [(Vec3, f32); SOIL_MOUNDS.len() + 1] {
    let mut balls = [(BLUE_MOON_PLANET_CENTER, BLUE_MOON_PLANET_RADIUS); SOIL_MOUNDS.len() + 1];

    for (ball, soil_mound) in balls.iter_mut().skip(1).zip(SOIL_MOUNDS) {
        *ball = (soil_mound.center(), soil_mound.radius);
    }

    balls
}

/// Distance from the moon center to the top of the ground cubes (soil or
/// bare moon) along `direction`. Everything on the moon stands here.
pub(super) fn ground_distance(direction: Vec3) -> f32 {
    voxel::surface_distance(
        &ground_balls(),
        BLUE_MOON_CUBE_EDGE,
        BLUE_MOON_PLANET_CENTER,
        direction,
    )
}

/// Distance from the moon center to the top of the smooth ground the cubes
/// are built from (soil mounds or bare moon) along `direction`.
pub(super) fn smooth_ground_distance(direction: Vec3) -> f32 {
    let direction = direction.normalized();

    SOIL_MOUNDS
        .iter()
        .fold(BLUE_MOON_PLANET_RADIUS, |top, soil_mound| {
            let center = soil_mound.center() - BLUE_MOON_PLANET_CENTER;
            let along = direction.dot(center);
            let discriminant =
                soil_mound.radius * soil_mound.radius - center.length_squared() + along * along;

            if discriminant < 0.0 {
                top
            } else {
                top.max(along + discriminant.sqrt())
            }
        })
}

/// Point on the ground along `direction`, sunk `sink` below it.
fn ground_point(direction: Vec3, sink: f32) -> Vec3 {
    let direction = direction.normalized();

    BLUE_MOON_PLANET_CENTER + direction * (ground_distance(direction) - sink)
}

/// Direction on the edge of the soil layer, `azimuth` degrees around its
/// axis (see `TuftPlacement::SoilEdge`), moved a little inside the edge.
pub(super) fn soil_edge_direction(azimuth: f32) -> Vec3 {
    let axis = surface_direction(SOIL_LAYER_ANGLE, 0.0);
    let front = Vec3::new(0.0, 0.0, 1.0);
    let side = axis.cross(front).normalized();
    let (sin_azimuth, cos_azimuth) = azimuth.to_radians().sin_cos();
    let around = side * cos_azimuth + front * sin_azimuth;
    let along_meridian = |polar: f32| {
        let (sin_polar, cos_polar) = polar.to_radians().sin_cos();
        (axis * cos_polar + around * sin_polar).normalized()
    };

    // Walk down the meridian until the soil gets too thin.
    let mut edge = 30.0;
    let mut polar = 30.0;
    while polar < 110.0 {
        let thickness = smooth_ground_distance(along_meridian(polar)) - BLUE_MOON_PLANET_RADIUS;
        if thickness > SOIL_EDGE_THICKNESS {
            edge = polar;
        }
        polar += 0.5;
    }

    along_meridian(edge - SOIL_EDGE_INSET_DEGREES)
}

fn tuft_direction(placement: TuftPlacement) -> Vec3 {
    match placement {
        TuftPlacement::SoilEdge { azimuth } => soil_edge_direction(azimuth),
        TuftPlacement::Direction { angle, depth } => surface_direction(angle, depth),
    }
}

/// Frame on the ground: `up` points away from the moon inside the level plane,
/// `right` is `up` turned a quarter turn clockwise and forward is +Z.
#[derive(Debug, Clone, Copy)]
pub(super) struct GroundFrame {
    pub origin: Vec3,
    pub basis: Basis3,
    /// Local coordinates and sizes are multiplied by this.
    pub scale: f32,
}

impl GroundFrame {
    fn at_angle(angle: f32, scale: f32) -> Result<Self, SpaceBuildError> {
        let up = surface_direction(angle, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0);

        Ok(Self {
            origin: ground_point(up, 0.0),
            basis: Basis3::new(up.cross(forward), up, forward)?,
            scale,
        })
    }

    pub(super) fn point(self, local: Vec3) -> Vec3 {
        self.origin + self.basis.local_to_world_vector(local * self.scale)
    }

    pub(super) fn size(self, local_size: f32) -> f32 {
        local_size * self.scale
    }

    /// Height of the ground under the local point `(x, _, z)`, in local
    /// units.
    fn ground_height(self, x: f32, z: f32) -> f32 {
        let above = self.point(Vec3::new(x, 0.0, z)) - BLUE_MOON_PLANET_CENTER;
        let ground = ground_point(above, 0.0);

        (ground - self.origin).dot(self.basis.up()) / self.scale
    }

    /// Basis whose Y axis runs along the rover (local X): cylinders built with
    /// it lie flat and point to the side.
    fn across_basis(self) -> Result<Basis3, SpaceBuildError> {
        Ok(Basis3::new(
            -self.basis.up(),
            self.basis.right(),
            self.basis.forward(),
        )?)
    }

    /// Basis whose Y axis points to the front (+Z), for wheels and lights.
    fn facing_basis(self) -> Result<Basis3, SpaceBuildError> {
        Ok(Basis3::new(
            self.basis.right(),
            self.basis.forward(),
            -self.basis.up(),
        )?)
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

pub(super) fn rover_frame() -> Result<GroundFrame, SpaceBuildError> {
    GroundFrame::at_angle(ROVER_ANGLE, ROVER_SCALE)
}

/// The slingshot is planted in the top of the small moon's cubes.
pub(super) fn small_moon_slingshot_frame() -> Result<RadialFrame, SpaceBuildError> {
    let ground = voxel::surface_distance(
        &[(BLUE_MOON_SMALL_MOON_CENTER, BLUE_MOON_SMALL_MOON_RADIUS)],
        small_moon_cube_edge(),
        BLUE_MOON_SMALL_MOON_CENTER,
        SMALL_MOON_SLINGSHOT_DIRECTION,
    );

    Ok(RadialFrame::from_normal(
        BLUE_MOON_SMALL_MOON_CENTER,
        ground - SMALL_MOON_SLINGSHOT_EMBED,
        SMALL_MOON_SLINGSHOT_DIRECTION,
    )?)
}

fn small_moon_cube_edge() -> f32 {
    voxel::fitted_cube_edge(
        BLUE_MOON_SMALL_MOON_RADIUS,
        BLUE_MOON_CUBE_EDGE,
        BLUE_MOON_MIN_CUBES_ACROSS,
    )
}

/// The whole Luna Azul diorama with the lights inside its atmosphere. The
/// lights outside the atmosphere belong to the level scene.
pub(super) fn add_blue_moon_world(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    space: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let materials = register_blue_moon_materials(scene)?;

    add_moons(scene, metadata, materials)?;
    add_grass(scene, metadata, materials)?;
    add_garlic(scene, metadata, materials)?;
    add_rover(scene, metadata, materials)?;
    add_bubbles(scene, metadata, materials)?;
    add_small_moon_slingshot(scene, metadata, space)?;
    add_floating_asteroids(scene, metadata, materials)?;
    add_atmosphere(scene, metadata, materials)?;
    add_inside_lighting(scene)?;

    Ok(())
}

/// Lights outside the atmosphere: the sun of the skybox, which gives the
/// shell and the asteroids their key light, and two rim lights.
pub(super) fn add_outside_lighting(scene: &mut Scene) {
    scene.add_light(PointLight::new(
        blue_moon_sun_light_position(),
        Color::new(1.0, 0.88, 0.58),
        SUN_KEY_INTENSITY,
    ));
    for (offset, color, intensity) in ATMOSPHERE_RIM_LIGHTS {
        scene.add_light(PointLight::new(
            BLUE_MOON_ATMOSPHERE_CENTER + offset,
            color,
            intensity,
        ));
    }
}

pub(super) fn blue_moon_sun_light_position() -> Vec3 {
    BLUE_MOON_ATMOSPHERE_CENTER + BLUE_MOON_SUN_DIRECTION.normalized() * BLUE_MOON_SUN_DISTANCE
}

fn add_inside_lighting(scene: &mut Scene) -> Result<(), SpaceBuildError> {
    for (offset, color, intensity) in INSIDE_LIGHTS {
        scene.add_light(PointLight::new(
            BLUE_MOON_ATMOSPHERE_CENTER + offset,
            color,
            intensity,
        ));
    }
    for bubble in BUBBLES {
        scene.add_light(PointLight::new(
            bubble.center + BUBBLE_LIGHT_OFFSET,
            Color::new(1.0, 0.96, 0.90),
            BUBBLE_LIGHT_INTENSITY,
        ));
    }
    scene.add_light(PointLight::new(
        rover_frame()?.point(ROVER_DOME_CENTER + DOME_LIGHT_OFFSET),
        Color::new(1.0, 0.96, 0.90),
        DOME_LIGHT_INTENSITY,
    ));

    Ok(())
}

pub(super) fn register_blue_moon_materials(
    scene: &mut Scene,
) -> Result<BlueMoonMaterials, SpaceBuildError> {
    let moon_texture = scene.add_texture(moon_texture()?);
    let soil_texture = scene.add_texture(soil_texture()?);
    let garlic_texture = scene.add_texture(garlic_texture(garlic_axis())?);
    let rock_texture = scene.add_texture(rock_texture()?);

    let moon = scene.add_material(
        Material::new(
            Color::WHITE,
            0.12,
            18.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.040, 0.040, 0.052),
        )
        .with_texture(moon_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    let soil = scene.add_material(
        Material::new(
            Color::WHITE,
            0.10,
            14.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.022, 0.024, 0.034),
        )
        .with_texture(soil_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    let grass_light = scene.add_material(Material::new(
        Color::new(0.52, 0.88, 0.24),
        0.30,
        28.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.060, 0.110, 0.030),
    ))?;
    let grass_dark = scene.add_material(Material::new(
        Color::new(0.27, 0.64, 0.17),
        0.25,
        24.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.030, 0.075, 0.020),
    ))?;
    let garlic = scene.add_material(
        Material::new(
            Color::WHITE,
            0.35,
            30.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.080, 0.076, 0.070),
        )
        .with_texture(garlic_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    // Olive gray paint with a soft metallic highlight.
    let rover_paint = scene.add_material(Material::new(
        Color::new(0.55, 0.60, 0.46),
        0.50,
        42.0,
        0.03,
        0.0,
        1.0,
        Color::new(0.050, 0.055, 0.040),
    ))?;
    let rover_dark = scene.add_material(Material::new(
        Color::new(0.25, 0.27, 0.27),
        0.45,
        40.0,
        0.02,
        0.0,
        1.0,
        Color::new(0.020, 0.022, 0.024),
    ))?;
    let rover_metal = scene.add_material(Material::new(
        Color::new(0.80, 0.82, 0.84),
        0.70,
        70.0,
        0.08,
        0.0,
        1.0,
        Color::new(0.060, 0.062, 0.068),
    ))?;
    let tire = scene.add_material(Material::new(
        Color::new(0.09, 0.09, 0.10),
        0.25,
        20.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.012, 0.012, 0.016),
    ))?;
    // Glass dome: clear, with a strong highlight and a little refraction.
    let glass = scene.add_material(Material::new(
        Color::new(0.86, 0.95, 1.0),
        0.90,
        120.0,
        0.06,
        0.84,
        1.08,
        Color::new(0.020, 0.030, 0.040),
    ))?;
    let seat = scene.add_material(Material::new(
        Color::new(0.62, 0.18, 0.14),
        0.35,
        30.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.040, 0.012, 0.010),
    ))?;
    let headlight = scene.add_material(Material::new(
        Color::new(1.0, 0.95, 0.62),
        0.80,
        90.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.80, 0.70, 0.32),
    ))?;
    let beacon = scene.add_material(Material::new(
        Color::new(1.0, 0.30, 0.22),
        0.80,
        90.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.70, 0.12, 0.08),
    ))?;
    // Smoke: light gray, slightly see-through and glowing a little so it
    // reads against the dark sky.
    let smoke = scene.add_material(Material::new(
        Color::new(0.84, 0.86, 0.90),
        0.05,
        8.0,
        0.0,
        0.30,
        1.0,
        Color::new(0.120, 0.122, 0.135),
    ))?;
    // Bubbles: almost clear with a bright highlight, a faint reflection and a
    // slight refraction.
    let bubble = scene.add_material(Material::new(
        Color::new(0.88, 0.94, 1.0),
        0.90,
        110.0,
        0.05,
        0.90,
        1.03,
        Color::new(0.030, 0.040, 0.060),
    ))?;
    let radish = scene.add_material(Material::new(
        Color::new(0.96, 0.24, 0.36),
        0.60,
        56.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.090, 0.020, 0.040),
    ))?;
    let radish_root = scene.add_material(Material::new(
        Color::new(0.98, 0.93, 0.92),
        0.40,
        40.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.090, 0.085, 0.085),
    ))?;
    let carrot = scene.add_material(Material::new(
        Color::new(1.0, 0.52, 0.10),
        0.50,
        44.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.100, 0.045, 0.008),
    ))?;
    let rock = scene.add_material(
        Material::new(
            Color::WHITE,
            0.15,
            18.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.050, 0.052, 0.062),
        )
        .with_texture(rock_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    // Purple tint over the gray rock texture.
    let asteroid = scene.add_material(
        Material::new(
            Color::new(0.62, 0.46, 0.98),
            0.12,
            16.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.070, 0.045, 0.120),
        )
        .with_texture(rock_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    // Atmosphere: mostly clear lilac blue, without refraction so the diorama
    // inside is not distorted.
    let atmosphere = scene.add_material(Material::new(
        Color::new(0.72, 0.80, 1.0),
        0.45,
        60.0,
        0.02,
        0.92,
        1.0,
        Color::new(0.040, 0.055, 0.110),
    ))?;

    Ok(BlueMoonMaterials {
        moon,
        soil,
        grass_light,
        grass_dark,
        garlic,
        rover_paint,
        rover_dark,
        rover_metal,
        tire,
        glass,
        seat,
        headlight,
        beacon,
        smoke,
        bubble,
        radish,
        radish_root,
        carrot,
        rock,
        asteroid,
        atmosphere,
    })
}

/// Both moons are balls of cubes. The big one is one body with the soil
/// mounds, so the soil cubes replace the moon cubes under them.
fn add_moons(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: BlueMoonMaterials,
) -> Result<(), SpaceBuildError> {
    let mut balls = Vec::with_capacity(SOIL_MOUNDS.len() + 1);
    for (index, (center, radius)) in ground_balls().into_iter().enumerate() {
        let material_id = if index == 0 {
            materials.moon
        } else {
            materials.soil
        };
        balls.push(VoxelBall::new(center, radius, material_id));
    }

    metadata.blue_moon_planet = Some(voxel::add_voxel_body(
        scene,
        &VoxelBody::new(&balls, BLUE_MOON_CUBE_EDGE)?,
    )?);
    metadata.blue_moon_soil_mound_count += SOIL_MOUNDS.len();

    metadata.blue_moon_small_moon = Some(voxel::add_voxel_body(
        scene,
        &VoxelBody::new(
            &[VoxelBall::new(
                BLUE_MOON_SMALL_MOON_CENTER,
                BLUE_MOON_SMALL_MOON_RADIUS,
                materials.moon,
            )],
            small_moon_cube_edge(),
        )?,
    )?);

    Ok(())
}

/// Each tuft is a fan of pointed blades: one standing up in the middle and the
/// rest leaning out around it, alternating two greens.
fn add_grass(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: BlueMoonMaterials,
) -> Result<(), SpaceBuildError> {
    for (tuft_index, tuft) in GRASS_TUFTS.iter().enumerate() {
        let direction = tuft_direction(tuft.placement);
        let base = ground_point(direction, GRASS_BLADE_SINK);
        let frame = basis_with_up(direction)?;
        let seed = tuft_index as f32;

        for blade in 0..tuft.blades {
            let index = blade as f32;
            let (lean, turn) = if blade == 0 {
                (0.10, 0.0)
            } else {
                let around = (index - 1.0) / (tuft.blades - 1) as f32;
                (
                    0.45 + 0.45 * hash3(seed, index, 1.0),
                    around * std::f32::consts::TAU + 0.6 * hash3(seed, index, 2.0),
                )
            };
            let (sin_lean, cos_lean) = f32::sin_cos(lean);
            let (sin_turn, cos_turn) = f32::sin_cos(turn);
            let blade_direction = (frame.up() * cos_lean
                + (frame.right() * cos_turn + frame.forward() * sin_turn) * sin_lean)
                .normalized();
            let length = GRASS_BLADE_LENGTH
                * tuft.scale
                * (0.70 + 0.45 * hash3(seed, index, 3.0))
                * if blade == 0 { 1.15 } else { 1.0 };
            let material = if blade % 2 == 0 {
                materials.grass_light
            } else {
                materials.grass_dark
            };

            scene.add_cone(Cone::new(
                base + blade_direction * (length * 0.5),
                GRASS_BLADE_RADIUS * tuft.scale.sqrt(),
                length * 0.5,
                basis_with_up(blade_direction)?,
                material,
            )?)?;
            metadata.blue_moon_grass_blade_count += 1;
        }
        metadata.blue_moon_grass_tuft_count += 1;
    }

    Ok(())
}

fn garlic_axis() -> Vec3 {
    surface_direction(GARLIC_ANGLE, GARLIC_DEPTH)
}

/// Garlic bulb half buried in the left hill: a striped bulb, a pointed neck and
/// two leaves.
fn add_garlic(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: BlueMoonMaterials,
) -> Result<(), SpaceBuildError> {
    let axis = garlic_axis();
    let bulb_center =
        ground_point(axis, 0.0) + axis * (GARLIC_BULB_RADIUS * (1.0 - 2.0 * GARLIC_BURIED));
    let neck_half_height = GARLIC_BULB_RADIUS * 0.75;
    let neck_base = bulb_center + axis * (GARLIC_BULB_RADIUS * 0.62);

    scene.add_sphere(Sphere::new(
        bulb_center,
        GARLIC_BULB_RADIUS,
        materials.garlic,
    )?)?;
    scene.add_cone(Cone::new(
        neck_base + axis * neck_half_height,
        GARLIC_BULB_RADIUS * 0.50,
        neck_half_height,
        basis_with_up(axis)?,
        materials.garlic,
    )?)?;
    metadata.blue_moon_garlic_parts += 2;

    let frame = basis_with_up(axis)?;
    for leaf_index in 0..GARLIC_LEAVES {
        let side = if leaf_index % 2 == 0 { -1.0 } else { 1.0 };
        let leaf = (axis + frame.right() * (0.45 * side) + frame.forward() * 0.15).normalized();
        let length = GARLIC_BULB_RADIUS * 1.6;
        scene.add_cone(Cone::new(
            neck_base + axis * (neck_half_height * 1.2) + leaf * (length * 0.5),
            GARLIC_BULB_RADIUS * 0.14,
            length * 0.5,
            basis_with_up(leaf)?,
            materials.grass_light,
        )?)?;
        metadata.blue_moon_garlic_parts += 1;
    }

    Ok(())
}

/// Moon rover parked on the soil: a dark chassis on four wheels, an olive
/// body with a sloped hood and fenders, a glass dome with a seat and a control
/// stick, headlights, an antenna with a red beacon and an exhaust pipe that
/// puffs smoke.
fn add_rover(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: BlueMoonMaterials,
) -> Result<(), SpaceBuildError> {
    let frame = rover_frame()?;
    let mut parts = 0;
    let mut add_box = |scene: &mut Scene,
                       center: Vec3,
                       half_extents: Vec3,
                       angle_radians: f32,
                       material_id: usize|
     -> Result<(), SpaceBuildError> {
        scene.add_oriented_box(OrientedBox::new(
            frame.point(center),
            half_extents * frame.scale,
            frame.turned(angle_radians)?,
            material_id,
        )?)?;
        parts += 1;
        Ok(())
    };

    // Chassis, body, hood and the flat deck around the dome.
    add_box(
        scene,
        Vec3::new(0.0, 0.20, 0.0),
        Vec3::new(0.40, 0.045, 0.17),
        0.0,
        materials.rover_dark,
    )?;
    add_box(
        scene,
        Vec3::new(-0.03, 0.31, 0.0),
        Vec3::new(0.32, 0.08, 0.16),
        0.0,
        materials.rover_paint,
    )?;
    add_box(
        scene,
        Vec3::new(0.30, 0.30, 0.0),
        Vec3::new(0.09, 0.055, 0.15),
        -0.42,
        materials.rover_paint,
    )?;
    add_box(
        scene,
        Vec3::new(-0.05, 0.405, 0.0),
        Vec3::new(0.25, 0.018, 0.14),
        0.0,
        materials.rover_metal,
    )?;
    // Rear bumper and a side stripe on each flank.
    add_box(
        scene,
        Vec3::new(-0.41, 0.22, 0.0),
        Vec3::new(0.025, 0.035, 0.15),
        0.0,
        materials.rover_metal,
    )?;
    for side in [-1.0, 1.0] {
        add_box(
            scene,
            Vec3::new(-0.03, 0.30, side * 0.163),
            Vec3::new(0.28, 0.018, 0.006),
            0.0,
            materials.rover_dark,
        )?;
    }
    // Fenders over the wheels.
    for (x, z) in ROVER_WHEELS {
        add_box(
            scene,
            Vec3::new(x, 0.285, z),
            Vec3::new(0.15, 0.018, 0.058),
            0.0,
            materials.rover_paint,
        )?;
    }
    // Seat inside the dome.
    add_box(
        scene,
        ROVER_DOME_CENTER + Vec3::new(-0.07, 0.035, 0.0),
        Vec3::new(0.022, 0.055, 0.07),
        -0.20,
        materials.seat,
    )?;

    // Wheels: black tires with a metal hub, each resting on the ground under
    // it.
    let wheel_basis = frame.facing_basis()?;
    for (x, z) in ROVER_WHEELS {
        let ground = frame.ground_height(x, z);
        let center = frame.point(Vec3::new(
            x,
            ground + ROVER_WHEEL_RADIUS - ROVER_WHEEL_SINK,
            z,
        ));
        scene.add_cylinder(Cylinder::new(
            center,
            frame.size(ROVER_WHEEL_RADIUS),
            frame.size(ROVER_WHEEL_HALF_WIDTH),
            wheel_basis,
            materials.tire,
        )?)?;
        scene.add_cylinder(Cylinder::new(
            center,
            frame.size(ROVER_WHEEL_RADIUS * 0.45),
            frame.size(ROVER_WHEEL_HALF_WIDTH * 1.18),
            wheel_basis,
            materials.rover_metal,
        )?)?;
        parts += 2;
        metadata.blue_moon_rover_wheel_count += 1;
    }
    // Axles joining each pair of wheels under the chassis.
    for x in [-0.30, 0.30] {
        scene.add_cylinder(Cylinder::new(
            frame.point(Vec3::new(x, ROVER_WHEEL_RADIUS - ROVER_WHEEL_SINK, 0.0)),
            frame.size(0.022),
            frame.size(0.20),
            wheel_basis,
            materials.rover_dark,
        )?)?;
        parts += 1;
    }

    // Glass dome on a metal ring, with a control stick inside.
    scene.add_cylinder(Cylinder::new(
        frame.point(Vec3::new(ROVER_DOME_CENTER.x, 0.43, 0.0)),
        frame.size(ROVER_DOME_RADIUS + 0.012),
        frame.size(0.022),
        frame.basis,
        materials.rover_metal,
    )?)?;
    scene.add_sphere(Sphere::new(
        frame.point(ROVER_DOME_CENTER),
        frame.size(ROVER_DOME_RADIUS),
        materials.glass,
    )?)?;
    let stick_base = ROVER_DOME_CENTER + Vec3::new(0.06, -0.02, 0.0);
    let stick_top = ROVER_DOME_CENTER + Vec3::new(0.08, 0.08, 0.0);
    add_segment(
        scene,
        frame.point(stick_base),
        frame.point(stick_top),
        frame.size(0.010),
        materials.rover_dark,
    )?;
    scene.add_sphere(Sphere::new(
        frame.point(stick_top),
        frame.size(0.022),
        materials.beacon,
    )?)?;
    parts += 4;

    // Headlights on the hood.
    for z in [-0.09, 0.09] {
        let center = Vec3::new(0.395, 0.29, z);
        scene.add_cylinder(Cylinder::new(
            frame.point(center),
            frame.size(0.038),
            frame.size(0.018),
            frame.across_basis()?,
            materials.rover_metal,
        )?)?;
        scene.add_sphere(Sphere::new(
            frame.point(center + Vec3::new(0.012, 0.0, 0.0)),
            frame.size(0.028),
            materials.headlight,
        )?)?;
        parts += 2;
    }

    // Antenna with a red beacon.
    let antenna_base = Vec3::new(0.18, 0.41, -0.10);
    let antenna_top = Vec3::new(0.24, 0.70, -0.10);
    add_segment(
        scene,
        frame.point(antenna_base),
        frame.point(antenna_top),
        frame.size(0.011),
        materials.rover_metal,
    )?;
    scene.add_sphere(Sphere::new(
        frame.point(antenna_top),
        frame.size(0.030),
        materials.beacon,
    )?)?;
    parts += 2;

    // Exhaust pipe with a rim, and the smoke it puffs.
    add_segment(
        scene,
        frame.point(ROVER_EXHAUST_BASE),
        frame.point(ROVER_EXHAUST_TOP),
        frame.size(ROVER_EXHAUST_RADIUS),
        materials.rover_dark,
    )?;
    let pipe_axis = (frame.point(ROVER_EXHAUST_TOP) - frame.point(ROVER_EXHAUST_BASE)).normalized();
    scene.add_cylinder(Cylinder::new(
        frame.point(ROVER_EXHAUST_TOP) - pipe_axis * frame.size(0.02),
        frame.size(ROVER_EXHAUST_RADIUS * 1.35),
        frame.size(0.02),
        basis_with_up(pipe_axis)?,
        materials.rover_metal,
    )?)?;
    parts += 2;
    for (center, radius) in ROVER_SMOKE_PUFFS {
        scene.add_sphere(Sphere::new(
            frame.point(center),
            frame.size(radius),
            materials.smoke,
        )?)?;
        metadata.blue_moon_smoke_puff_count += 1;
    }

    metadata.blue_moon_rover_parts += parts;

    Ok(())
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

fn add_bubbles(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: BlueMoonMaterials,
) -> Result<(), SpaceBuildError> {
    for bubble in BUBBLES {
        metadata.blue_moon_bubble_content_parts += match bubble.content {
            BubbleContent::Radish => add_radish(scene, bubble.center, materials)?,
            BubbleContent::Carrot => add_carrot(scene, bubble.center, materials)?,
            BubbleContent::Rock { seed } => add_moon_rock(scene, bubble.center, seed, materials)?,
        };
        scene.add_sphere(Sphere::new(bubble.center, BUBBLE_RADIUS, materials.bubble)?)?;
        metadata.blue_moon_bubble_count += 1;
    }

    Ok(())
}

/// Leaves fanning out of `base` around `axis`: `count` pointed cones leaning
/// `lean` radians away from the axis.
fn add_leaves(
    scene: &mut Scene,
    base: Vec3,
    axis: Vec3,
    count: usize,
    lean: f32,
    length: f32,
    materials: BlueMoonMaterials,
) -> Result<usize, SpaceBuildError> {
    let frame = basis_with_up(axis)?;
    let (sin_lean, cos_lean) = lean.sin_cos();

    for leaf in 0..count {
        let turn = leaf as f32 / count as f32 * std::f32::consts::TAU + 0.4;
        let (sin_turn, cos_turn) = turn.sin_cos();
        let direction = (frame.up() * cos_lean
            + (frame.right() * cos_turn + frame.forward() * sin_turn) * sin_lean)
            .normalized();
        let material = if leaf % 2 == 0 {
            materials.grass_light
        } else {
            materials.grass_dark
        };

        scene.add_cone(Cone::new(
            base + direction * (length * 0.5),
            length * 0.20,
            length * 0.5,
            basis_with_up(direction)?,
            material,
        )?)?;
    }

    Ok(count)
}

/// Radish tilted to the right: a round pink root with a white tail and three
/// leaves on top.
fn add_radish(
    scene: &mut Scene,
    center: Vec3,
    materials: BlueMoonMaterials,
) -> Result<usize, SpaceBuildError> {
    let axis = Vec3::new(0.30, 1.0, 0.12).normalized();
    let body_radius = 0.10;
    let body = center - axis * 0.03;
    let tail_half_height = 0.045;

    scene.add_sphere(Sphere::new(body, body_radius, materials.radish)?)?;
    scene.add_cone(Cone::new(
        body - axis * (body_radius * 0.85 + tail_half_height),
        0.042,
        tail_half_height,
        basis_with_up(-axis)?,
        materials.radish_root,
    )?)?;
    let leaves = add_leaves(
        scene,
        body + axis * (body_radius * 0.80),
        axis,
        3,
        0.42,
        0.12,
        materials,
    )?;

    Ok(2 + leaves)
}

/// Carrot lying diagonally with its tip down to the left and three leaves at
/// its rounded top.
fn add_carrot(
    scene: &mut Scene,
    center: Vec3,
    materials: BlueMoonMaterials,
) -> Result<usize, SpaceBuildError> {
    let axis = Vec3::new(0.62, 0.78, 0.12).normalized();
    let half_height = 0.13;
    let top_radius = 0.062;
    let body = center - axis * 0.03;
    let top = body + axis * half_height;

    scene.add_cone(Cone::new(
        body,
        top_radius,
        half_height,
        basis_with_up(-axis)?,
        materials.carrot,
    )?)?;
    scene.add_sphere(Sphere::new(
        top - axis * (top_radius * 0.35),
        top_radius,
        materials.carrot,
    )?)?;
    let leaves = add_leaves(
        scene,
        top + axis * (top_radius * 0.35),
        axis,
        3,
        0.35,
        0.085,
        materials,
    )?;

    Ok(2 + leaves)
}

/// Lumpy gray moon rock: a main ball and two smaller lumps.
fn add_moon_rock(
    scene: &mut Scene,
    center: Vec3,
    seed: f32,
    materials: BlueMoonMaterials,
) -> Result<usize, SpaceBuildError> {
    let radius = 0.115;
    scene.add_sphere(Sphere::new(center, radius, materials.rock)?)?;

    for lump in 0..2 {
        let index = lump as f32;
        let direction = Vec3::new(
            hash3(seed, index, 1.0) - 0.5,
            hash3(seed, index, 2.0) - 0.5,
            hash3(seed, index, 3.0) - 0.5,
        )
        .normalized();
        let lump_radius = radius * (0.55 + 0.15 * hash3(seed, index, 4.0));
        scene.add_sphere(Sphere::new(
            center + direction * (radius * 0.62),
            lump_radius,
            materials.rock,
        )?)?;
    }

    Ok(3)
}

fn add_small_moon_slingshot(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    space: SpaceMaterials,
) -> Result<(), SpaceBuildError> {
    let parts = add_slingshot(
        scene,
        small_moon_slingshot_frame()?,
        SMALL_MOON_SLINGSHOT_YAW_RADIANS,
        SMALL_MOON_SLINGSHOT_SCALE,
        space,
    )?;
    metadata.blue_moon_slingshot_wood_parts += parts.wood;
    metadata.blue_moon_slingshot_joint_parts += parts.joints;
    metadata.blue_moon_slingshot_band_parts += parts.bands;

    Ok(())
}

fn add_floating_asteroids(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: BlueMoonMaterials,
) -> Result<(), SpaceBuildError> {
    for (index, (center, radius)) in FLOATING_ASTEROIDS.into_iter().enumerate() {
        let seed = index as f32 + 11.0;
        let lump = Vec3::new(
            hash3(seed, 3.0, 1.0) - 0.5,
            hash3(seed, 3.0, 2.0) - 0.5,
            hash3(seed, 3.0, 3.0) - 0.5,
        )
        .normalized();

        let parts = add_voxel_rock(
            scene,
            center,
            radius,
            lump * 0.60,
            materials.asteroid,
            BLUE_MOON_CUBE_EDGE,
        )?;
        metadata.blue_moon_floating_asteroid_count += 1;
        metadata.blue_moon_floating_asteroid_parts += parts.total();
    }

    Ok(())
}

fn add_atmosphere(
    scene: &mut Scene,
    metadata: &mut SpaceSceneMetadata,
    materials: BlueMoonMaterials,
) -> Result<(), SpaceBuildError> {
    metadata.blue_moon_atmosphere_id = Some(scene.object_count());
    scene.add_sphere(Sphere::new(
        BLUE_MOON_ATMOSPHERE_CENTER,
        BLUE_MOON_ATMOSPHERE_RADIUS,
        materials.atmosphere,
    )?)?;

    Ok(())
}

/// Gray moon with lilac craters in sphere UVs. Craters are dark bowls with a
/// shadowed wall towards the light, a lit wall on the other side and a light
/// rim, like the cartoon moon of the reference.
fn moon_texture() -> Result<Texture, SpaceBuildError> {
    let craters = moon_craters();
    let mut pixels = Vec::with_capacity(MOON_TEXTURE_WIDTH * MOON_TEXTURE_HEIGHT);

    for y in 0..MOON_TEXTURE_HEIGHT {
        let v = 1.0 - y as f32 / (MOON_TEXTURE_HEIGHT - 1) as f32;

        for x in 0..MOON_TEXTURE_WIDTH {
            let u = x as f32 / (MOON_TEXTURE_WIDTH - 1) as f32;
            pixels.push(moon_color(sphere_direction_from_uv(u, v), &craters));
        }
    }

    Ok(Texture::new(
        MOON_TEXTURE_WIDTH,
        MOON_TEXTURE_HEIGHT,
        pixels,
    )?)
}

/// The front craters plus smaller ones spread evenly over the sphere (golden
/// spiral) with a jitter, skipping those that would overlap a front crater.
pub(super) fn moon_craters() -> Vec<(Vec3, f32)> {
    let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let mut craters: Vec<(Vec3, f32)> = MOON_FRONT_CRATERS
        .iter()
        .map(|&(direction, radius)| (direction.normalized(), radius))
        .collect();

    for index in 0..MOON_SCATTERED_CRATER_COUNT {
        let i = index as f32;
        let y = 1.0 - 2.0 * (i + 0.5) / MOON_SCATTERED_CRATER_COUNT as f32;
        let ring = (1.0 - y * y).max(0.0).sqrt();
        let theta = golden_angle * i + 0.7;
        let jitter = Vec3::new(
            hash3(i, 4.0, 1.0) - 0.5,
            hash3(i, 4.0, 2.0) - 0.5,
            hash3(i, 4.0, 3.0) - 0.5,
        ) * 0.30;
        let direction =
            (Vec3::new(ring * theta.cos(), y, ring * theta.sin()) + jitter).normalized();
        let radius = 0.08 + 0.10 * hash3(i, 4.0, 4.0);
        let overlaps = craters.iter().any(|&(other, other_radius)| {
            direction.dot(other).clamp(-1.0, 1.0).acos() < (radius + other_radius) * 0.95
        });

        if !overlaps {
            craters.push((direction, radius));
        }
    }

    craters
}

fn moon_color(direction: Vec3, craters: &[(Vec3, f32)]) -> Color {
    let broad = value_noise_3d(direction * 2.4 + Vec3::new(3.0, 1.0, 5.0));
    let fine = value_noise_3d(direction * 11.0 + Vec3::new(7.0, 2.0, 9.0));
    let mut color = MOON_BASE.lerp(MOON_LIGHT, smoothstep(0.30, 0.78, broad * 0.7 + fine * 0.3));
    let light = MOON_CRATER_LIGHT_DIRECTION.normalized();

    for &(center, radius) in craters {
        let cosine = direction.dot(center);
        if cosine < (radius * 1.3).cos() {
            continue;
        }

        let fraction = cosine.clamp(-1.0, 1.0).acos() / radius;
        let offset = direction - center * cosine;
        let light_along = light - center * light.dot(center);
        // +1 on the side of the crater towards the light, -1 on the far side.
        let side = if offset.length_squared() > 1.0e-8 && light_along.length_squared() > 1.0e-8 {
            offset.normalized().dot(light_along.normalized())
        } else {
            0.0
        };

        let rim = smoothstep(0.90, 1.0, fraction) * (1.0 - smoothstep(1.0, 1.25, fraction));
        color = color.lerp(MOON_CRATER_RIM, rim * (0.20 + 0.50 * side.max(0.0)));
        color = color.lerp(MOON_BASE * 0.80, rim * 0.35 * (-side).max(0.0));

        let inside = 1.0 - smoothstep(0.93, 1.0, fraction);
        let wall = smoothstep(0.40, 0.95, fraction) * side.abs();
        let wall_color = if side > 0.0 {
            MOON_CRATER_SHADOW
        } else {
            MOON_CRATER_LIT
        };
        color = color.lerp(MOON_CRATER_FLOOR.lerp(wall_color, wall), inside);
    }

    color.clamped()
}

/// Dark slate soil with lighter patches and small pebbles, in sphere UVs.
fn soil_texture() -> Result<Texture, SpaceBuildError> {
    let mut pixels = Vec::with_capacity(SOIL_TEXTURE_WIDTH * SOIL_TEXTURE_HEIGHT);

    for y in 0..SOIL_TEXTURE_HEIGHT {
        let v = 1.0 - y as f32 / (SOIL_TEXTURE_HEIGHT - 1) as f32;

        for x in 0..SOIL_TEXTURE_WIDTH {
            let u = x as f32 / (SOIL_TEXTURE_WIDTH - 1) as f32;
            let direction = sphere_direction_from_uv(u, v);
            let broad = value_noise_3d(direction * 2.6 + Vec3::new(1.0, 6.0, 2.0));
            let fine = value_noise_3d(direction * 7.0 + Vec3::new(4.0, 3.0, 8.0));
            let pebbles = value_noise_3d(direction * 11.0 + Vec3::new(9.0, 5.0, 1.0));
            let mut color = SOIL_DARK.lerp(
                SOIL_LIGHT,
                smoothstep(0.25, 0.80, broad * 0.65 + fine * 0.35),
            );
            color = color.lerp(SOIL_PEBBLE, smoothstep(0.72, 0.90, pebbles) * 0.45);
            pixels.push(color.clamped());
        }
    }

    Ok(Texture::new(
        SOIL_TEXTURE_WIDTH,
        SOIL_TEXTURE_HEIGHT,
        pixels,
    )?)
}

/// Garlic skin in sphere UVs: cream with faint lines between the cloves,
/// running from the root to the neck around `axis`.
fn garlic_texture(axis: Vec3) -> Result<Texture, SpaceBuildError> {
    let frame = basis_with_up(axis)?;
    let mut pixels = Vec::with_capacity(GARLIC_TEXTURE_WIDTH * GARLIC_TEXTURE_HEIGHT);

    for y in 0..GARLIC_TEXTURE_HEIGHT {
        let v = 1.0 - y as f32 / (GARLIC_TEXTURE_HEIGHT - 1) as f32;

        for x in 0..GARLIC_TEXTURE_WIDTH {
            let u = x as f32 / (GARLIC_TEXTURE_WIDTH - 1) as f32;
            let direction = sphere_direction_from_uv(u, v);
            let along = direction.dot(frame.up());
            let around = direction
                .dot(frame.forward())
                .atan2(direction.dot(frame.right()));
            let seam = (around * GARLIC_CLOVES * 0.5).sin().abs();
            let fade = smoothstep(-0.85, -0.35, along) * (1.0 - smoothstep(0.55, 0.92, along));
            let line = (1.0 - smoothstep(0.04, 0.22, seam)) * fade;
            let root = 1.0 - smoothstep(-0.98, -0.80, along);
            let color = GARLIC_SKIN
                .lerp(GARLIC_LINE, line * 0.75)
                .lerp(GARLIC_ROOT, root);
            pixels.push(color.clamped());
        }
    }

    Ok(Texture::new(
        GARLIC_TEXTURE_WIDTH,
        GARLIC_TEXTURE_HEIGHT,
        pixels,
    )?)
}

#[cfg(test)]
mod tests {
    use super::{
        ATMOSPHERE_RIM_LIGHTS, BLUE_MOON_ATMOSPHERE_CENTER, BLUE_MOON_ATMOSPHERE_RADIUS,
        BLUE_MOON_CUBE_EDGE, BLUE_MOON_SMALL_MOON_CENTER, BLUE_MOON_SMALL_MOON_RADIUS,
        BLUE_MOON_SUN_DIRECTION, BUBBLE_RADIUS, BUBBLES, BlueMoonMaterials, BubbleContent,
        FLOATING_ASTEROIDS, GARLIC_LEAVES, GRASS_TUFTS, INSIDE_LIGHTS, MOON_BASE, ROVER_ANGLE,
        ROVER_DOME_CENTER, ROVER_DOME_RADIUS, ROVER_SMOKE_PUFFS, ROVER_WHEEL_RADIUS,
        SOIL_EDGE_THICKNESS, SOIL_MOUNDS, blue_moon_sun_light_position, ground_balls,
        ground_distance, moon_color, moon_craters, register_blue_moon_materials, rover_frame,
        small_moon_slingshot_frame, smooth_ground_distance, soil_edge_direction, surface_direction,
    };
    use crate::{
        cone::Cone,
        cube::Cube,
        math::Vec3,
        primitive::Primitive,
        scene::Scene,
        space::{
            BLUE_MOON_PLANET_CENTER, BLUE_MOON_PLANET_RADIUS, SPACE_SUN_U, SPACE_SUN_V,
            SpaceMaterials, blue_moon_orbit_camera, build_blue_moon_scene_with_metadata,
            register_space_materials,
            voxel::{VOXEL_CUBE_FILL, assert_voxel_ball},
        },
    };

    /// Level one materials with the ids they get in the level one scene,
    /// where they are registered after the shared space materials.
    fn material_ids() -> (SpaceMaterials, BlueMoonMaterials) {
        let mut scene = Scene::new();
        let space = register_space_materials(&mut scene).unwrap();
        let level = register_blue_moon_materials(&mut scene).unwrap();

        (space, level)
    }

    /// Center and radius of a sphere that contains the primitive.
    fn bounding_sphere(object: &Primitive) -> (Vec3, f32) {
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
                (cylinder.radius().powi(2) + cylinder.half_height().powi(2)).sqrt(),
            )
        } else if let Some(cone) = object.as_cone() {
            (
                cone.center(),
                (cone.base_radius().powi(2) + cone.half_height().powi(2)).sqrt(),
            )
        } else if let Some(part) = object.as_oriented_box() {
            (part.center(), part.half_extents().length())
        } else {
            panic!("unexpected primitive in level one: {object:?}");
        }
    }

    fn is_inside_atmosphere(point: Vec3) -> bool {
        (point - BLUE_MOON_ATMOSPHERE_CENTER).length() < BLUE_MOON_ATMOSPHERE_RADIUS
    }

    #[test]
    fn scene_holds_only_the_diorama_parts() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();

        assert_eq!(
            scene.object_count(),
            metadata.blue_moon_planet.unwrap().total()
                + metadata.blue_moon_small_moon.unwrap().total()
                + metadata.blue_moon_grass_blade_count
                + metadata.blue_moon_garlic_parts
                + metadata.blue_moon_rover_parts
                + metadata.blue_moon_smoke_puff_count
                + metadata.blue_moon_bubble_count
                + metadata.blue_moon_bubble_content_parts
                + metadata.blue_moon_slingshot_wood_parts
                + metadata.blue_moon_slingshot_joint_parts
                + metadata.blue_moon_slingshot_band_parts
                + metadata.blue_moon_floating_asteroid_parts
                + 1
        );
        assert_eq!(metadata.blue_moon_soil_mound_count, SOIL_MOUNDS.len());
        assert_eq!(metadata.blue_moon_grass_tuft_count, GRASS_TUFTS.len());
        assert_eq!(
            metadata.blue_moon_grass_blade_count,
            GRASS_TUFTS.iter().map(|tuft| tuft.blades).sum::<usize>()
        );
        assert_eq!(metadata.blue_moon_rover_wheel_count, 4);
        assert_eq!(metadata.blue_moon_smoke_puff_count, ROVER_SMOKE_PUFFS.len());
        assert_eq!(metadata.blue_moon_bubble_count, BUBBLES.len());
        assert_eq!(
            metadata.blue_moon_floating_asteroid_count,
            FLOATING_ASTEROIDS.len()
        );
        assert_eq!(metadata.pig_count, 0);
        assert_eq!(metadata.tnt_parts, 0);
    }

    #[test]
    fn moons_are_balls_of_cubes_colored_from_the_cratered_texture() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let (_, materials) = material_ids();
        let moon = metadata.blue_moon_planet.unwrap();
        let small_moon = metadata.blue_moon_small_moon.unwrap();
        let moon_material = scene.material(materials.moon).unwrap();
        let cubes = |ids: std::ops::Range<usize>| -> Vec<Cube> {
            ids.map(|id| *scene.objects()[id].as_cube().unwrap())
                .collect()
        };

        // The texture is still there: the cube colors are sampled from it.
        assert!(scene.texture(moon_material.texture_id.unwrap()).is_some());
        assert_eq!(moon.core_count, 1 + SOIL_MOUNDS.len());
        assert!(moon.palette_materials > 2);

        // Every cube of the big moon lies in the moon or in a soil mound, on
        // a grid of level one cubes with a cube corner at the moon center.
        let ground = ground_balls();
        let moon_cubes = cubes(moon.ids());
        for cube in &moon_cubes {
            let cell = (cube.min + cube.max) * 0.5;
            let size = cube.max.x - cube.min.x;

            assert!((size - BLUE_MOON_CUBE_EDGE * VOXEL_CUBE_FILL).abs() < 1.0e-4);
            assert!(
                ground
                    .iter()
                    .any(|&(center, radius)| (cell - center).length() <= radius + 1.0e-4)
            );
            let steps = (cell - BLUE_MOON_PLANET_CENTER) / BLUE_MOON_CUBE_EDGE;
            for step in [steps.x, steps.y, steps.z] {
                assert!((step - 0.5 - (step - 0.5).round()).abs() < 1.0e-2);
            }
            let material = scene.material(cube.material_id).unwrap();
            assert!(material.texture_id.is_none());
        }
        // The bottom half is bare gray moon, the top is darker soil.
        let bottom = moon_cubes
            .iter()
            .min_by(|left, right| left.min.y.total_cmp(&right.min.y))
            .unwrap();
        let top = moon_cubes
            .iter()
            .max_by(|left, right| left.max.y.total_cmp(&right.max.y))
            .unwrap();
        let bottom_color = scene.material(bottom.material_id).unwrap().albedo;
        let top_color = scene.material(top.material_id).unwrap().albedo;
        assert!(bottom.min.y < BLUE_MOON_PLANET_CENTER.y - BLUE_MOON_PLANET_RADIUS + 0.05);
        assert!(bottom_color.b >= bottom_color.r);
        assert!(
            top_color.r + top_color.g + top_color.b
                < bottom_color.r + bottom_color.g + bottom_color.b
        );

        let edge = assert_voxel_ball(
            &scene,
            small_moon,
            BLUE_MOON_SMALL_MOON_CENTER,
            BLUE_MOON_SMALL_MOON_RADIUS,
        );
        assert!(edge <= BLUE_MOON_CUBE_EDGE + 1.0e-5);
        assert!(BLUE_MOON_SMALL_MOON_RADIUS * 2.0 / edge >= 7.0 - 1.0e-3);
        // The small moon floats apart from the big one, at its upper left.
        assert!(
            (BLUE_MOON_SMALL_MOON_CENTER - BLUE_MOON_PLANET_CENTER).length()
                > BLUE_MOON_PLANET_RADIUS + BLUE_MOON_SMALL_MOON_RADIUS + 1.0
        );
        const {
            assert!(
                BLUE_MOON_SMALL_MOON_CENTER.x < BLUE_MOON_PLANET_CENTER.x - BLUE_MOON_PLANET_RADIUS
            );
            assert!(
                BLUE_MOON_SMALL_MOON_CENTER.y > BLUE_MOON_PLANET_CENTER.y + BLUE_MOON_PLANET_RADIUS
            );
        }
    }

    #[test]
    fn moon_texture_has_dark_lilac_craters_over_gray() {
        let craters = moon_craters();
        let (center, _) = craters[0];
        let crater = moon_color(center, &craters);
        let far_from_craters = (0..64)
            .map(|index| surface_direction(index as f32 * 5.625, -60.0 + index as f32 * 1.9))
            .find(|&direction| {
                craters.iter().all(|&(crater_center, radius)| {
                    direction.dot(crater_center).clamp(-1.0, 1.0).acos() > radius * 1.4
                })
            })
            .unwrap();
        let ground = moon_color(far_from_craters, &craters);

        assert!(craters.len() >= 15);
        assert!(crater.r + crater.g + crater.b < ground.r + ground.g + ground.b - 0.15);
        assert!(crater.b > crater.r + 0.05);
        assert!((ground.r - MOON_BASE.r).abs() < 0.2);
        assert!(ground.b >= ground.r);
    }

    #[test]
    fn soil_mounds_rise_over_the_moon_from_inside_it() {
        for soil_mound in SOIL_MOUNDS {
            let distance = (soil_mound.center() - BLUE_MOON_PLANET_CENTER).length();
            let height = distance + soil_mound.radius - BLUE_MOON_PLANET_RADIUS;

            assert!(distance < BLUE_MOON_PLANET_RADIUS);
            assert!(height > 0.05 && height < 0.40);
        }
        // The soil covers the top of the moon but not its bottom half.
        assert!(smooth_ground_distance(Vec3::new(0.0, 1.0, 0.0)) > BLUE_MOON_PLANET_RADIUS + 0.15);
        assert_eq!(
            smooth_ground_distance(Vec3::new(0.0, -1.0, 0.0)),
            BLUE_MOON_PLANET_RADIUS
        );
        assert_eq!(
            smooth_ground_distance(Vec3::new(0.0, 0.0, 1.0)),
            BLUE_MOON_PLANET_RADIUS
        );
        // The ground of cubes follows it within a cube.
        for direction in [
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-0.6, 0.5, 0.3),
        ] {
            let difference = ground_distance(direction) - smooth_ground_distance(direction);
            assert!(difference.abs() < BLUE_MOON_CUBE_EDGE, "{difference}");
        }
    }

    #[test]
    fn grass_grows_out_of_the_soil_near_its_edge() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let (_, materials) = material_ids();
        // Green cones on the big moon: grass blades and the garlic's leaves.
        let green_cones: Vec<_> = scene
            .cones()
            .filter(|cone| {
                (cone.material_id() == materials.grass_light
                    || cone.material_id() == materials.grass_dark)
                    && (cone.center() - BLUE_MOON_PLANET_CENTER).length() < 2.2
            })
            .collect();
        let base_height = |cone: &Cone| {
            let base = cone.center() - cone.orientation().up() * cone.half_height();
            let outward = base - BLUE_MOON_PLANET_CENTER;
            outward.length() - ground_distance(outward)
        };
        let blades: Vec<_> = green_cones
            .iter()
            .filter(|cone| base_height(cone) < 0.0)
            .collect();

        assert_eq!(blades.len(), metadata.blue_moon_grass_blade_count);
        assert_eq!(green_cones.len() - blades.len(), GARLIC_LEAVES);
        for blade in blades {
            let axis = blade.orientation().up();
            let base = blade.center() - axis * blade.half_height();
            let outward = (base - BLUE_MOON_PLANET_CENTER).normalized();

            // The base is just under the ground, on soil, and the blade grows
            // outwards.
            assert!(base_height(blade) > -0.05);
            assert!(axis.dot(outward) > 0.2);
            assert!(
                smooth_ground_distance(outward)
                    > BLUE_MOON_PLANET_RADIUS + SOIL_EDGE_THICKNESS * 0.5
            );
        }
        for azimuth in [0.0, 90.0, 180.0, 270.0] {
            let edge = soil_edge_direction(azimuth);
            assert!(smooth_ground_distance(edge) > BLUE_MOON_PLANET_RADIUS + SOIL_EDGE_THICKNESS);
        }
    }

    #[test]
    fn garlic_is_half_buried_in_the_left_hill() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let (_, materials) = material_ids();
        let bulb = scene
            .objects()
            .iter()
            .filter_map(|object| object.as_sphere())
            .find(|sphere| sphere.material_id() == materials.garlic)
            .unwrap();
        let outward = bulb.center() - BLUE_MOON_PLANET_CENTER;
        let ground = ground_distance(outward);

        assert_eq!(metadata.blue_moon_garlic_parts, 2 + GARLIC_LEAVES);
        assert!(bulb.center().x < -0.5);
        assert!(outward.length() - bulb.radius() < ground);
        assert!(outward.length() + bulb.radius() > ground + bulb.radius() * 0.5);
    }

    #[test]
    fn rover_is_parked_on_the_soil_with_its_wheels_on_the_ground() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let (_, materials) = material_ids();
        let frame = rover_frame().unwrap();
        let wheels: Vec<_> = scene
            .cylinders()
            .filter(|part| part.material_id() == materials.tire)
            .collect();

        assert_eq!(wheels.len(), metadata.blue_moon_rover_wheel_count);
        assert!(frame.basis.up().dot(surface_direction(ROVER_ANGLE, 0.0)) > 0.999);
        assert!((frame.origin - BLUE_MOON_PLANET_CENTER).length() > BLUE_MOON_PLANET_RADIUS + 0.1);
        for wheel in wheels {
            let outward = (wheel.center() - BLUE_MOON_PLANET_CENTER).normalized();
            let lowest = (wheel.center() - BLUE_MOON_PLANET_CENTER).length() - wheel.radius();
            let gap = lowest - ground_distance(outward);

            assert!(
                (wheel.radius() - frame.size(ROVER_WHEEL_RADIUS)).abs() < 1.0e-5,
                "wheels keep the rover scale"
            );
            // Wheels face the camera, so their axis runs along Z.
            assert!(wheel.orientation().up().z.abs() > 0.99);
            assert!(gap < 0.01 && gap > -0.05, "wheel floats or sinks: {gap}");
        }
    }

    #[test]
    fn rover_has_a_glass_dome_with_its_own_light() {
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();
        let (_, materials) = material_ids();
        let frame = rover_frame().unwrap();
        let dome_center = frame.point(ROVER_DOME_CENTER);
        let dome = scene
            .objects()
            .iter()
            .filter_map(|object| object.as_sphere())
            .find(|sphere| sphere.material_id() == materials.glass)
            .unwrap();
        let glass = scene.material(materials.glass).unwrap();

        assert!((dome.center() - dome_center).length() < 1.0e-5);
        assert!((dome.radius() - frame.size(ROVER_DOME_RADIUS)).abs() < 1.0e-5);
        assert!(glass.transparency > 0.7);
        assert!(glass.refractive_index > 1.0);
        assert!(
            scene
                .lights()
                .iter()
                .any(|light| (light.position - dome.center()).length() < dome.radius())
        );
    }

    #[test]
    fn smoke_rises_from_the_exhaust_and_grows() {
        let frame = rover_frame().unwrap();
        let heights: Vec<f32> = ROVER_SMOKE_PUFFS
            .iter()
            .map(|&(center, _)| (frame.point(center) - frame.origin).dot(frame.basis.up()))
            .collect();

        assert!(heights.windows(2).all(|pair| pair[1] > pair[0]));
        assert!(ROVER_SMOKE_PUFFS[0].1 < ROVER_SMOKE_PUFFS[2].1);
    }

    #[test]
    fn bubbles_hold_a_radish_a_carrot_and_two_rocks() {
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();
        let (_, materials) = material_ids();
        let bubble_material = scene.material(materials.bubble).unwrap();
        let contents = |bubble_center: Vec3| -> Vec<usize> {
            scene
                .objects()
                .iter()
                .filter(|object| object.material_id() != materials.bubble)
                .filter(|object| {
                    let (center, radius) = bounding_sphere(object);
                    (center - bubble_center).length() + radius < BUBBLE_RADIUS * 1.2
                })
                .map(Primitive::material_id)
                .collect()
        };

        assert!(bubble_material.transparency > 0.85);
        assert_eq!(
            BUBBLES
                .iter()
                .filter(|bubble| bubble.content == BubbleContent::Radish)
                .count(),
            1
        );
        assert_eq!(
            BUBBLES
                .iter()
                .filter(|bubble| bubble.content == BubbleContent::Carrot)
                .count(),
            1
        );
        assert_eq!(
            BUBBLES
                .iter()
                .filter(|bubble| matches!(bubble.content, BubbleContent::Rock { .. }))
                .count(),
            2
        );
        for bubble in BUBBLES {
            let inside = contents(bubble.center);
            let expected = match bubble.content {
                BubbleContent::Radish => materials.radish,
                BubbleContent::Carrot => materials.carrot,
                BubbleContent::Rock { .. } => materials.rock,
            };

            assert!(
                scene.objects().iter().any(|object| {
                    object.material_id() == materials.bubble
                        && object.as_sphere().is_some_and(|sphere| {
                            sphere.center() == bubble.center && sphere.radius() == BUBBLE_RADIUS
                        })
                }),
                "missing bubble at {:?}",
                bubble.center
            );
            assert!(inside.contains(&expected));
            // Every part of the content stays inside its bubble.
            for object in scene.objects() {
                let (center, radius) = bounding_sphere(object);
                if object.material_id() != materials.bubble
                    && (center - bubble.center).length() < BUBBLE_RADIUS
                {
                    let reach = match object.as_sphere() {
                        Some(sphere) => {
                            (sphere.center() - bubble.center).length() + sphere.radius()
                        }
                        None => (center - bubble.center).length() + radius * 0.8,
                    };
                    assert!(reach < BUBBLE_RADIUS, "content pokes out of its bubble");
                }
            }
        }
    }

    #[test]
    fn bubbles_float_clear_of_the_rover_the_smoke_and_each_other() {
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();
        let (_, materials) = material_ids();

        for (index, bubble) in BUBBLES.iter().enumerate() {
            for other in &BUBBLES[index + 1..] {
                assert!((bubble.center - other.center).length() > BUBBLE_RADIUS * 2.2);
            }
            for object in scene.objects() {
                if object.material_id() == materials.bubble
                    || object.material_id() == materials.atmosphere
                {
                    continue;
                }
                let (center, radius) = bounding_sphere(object);
                let distance = (center - bubble.center).length();
                // Either inside the bubble (its content) or clearly outside.
                assert!(
                    distance + radius * 0.8 < BUBBLE_RADIUS || distance > BUBBLE_RADIUS + radius,
                    "object touches the bubble at {:?}",
                    bubble.center
                );
            }
            let over_ground = (bubble.center - BLUE_MOON_PLANET_CENTER).length()
                - ground_distance(bubble.center - BLUE_MOON_PLANET_CENTER);
            assert!(over_ground > 0.6);
        }
    }

    #[test]
    fn each_bubble_has_a_light_inside_next_to_its_content() {
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();

        for bubble in BUBBLES {
            let light = scene
                .lights()
                .iter()
                .find(|light| (light.position - bubble.center).length() < BUBBLE_RADIUS)
                .unwrap();
            assert!((light.position - bubble.center).length() > BUBBLE_RADIUS * 0.6);
            assert!(light.position.z > bubble.center.z);
        }
    }

    #[test]
    fn slingshot_stands_on_top_of_the_small_moon() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let (space, _) = material_ids();
        let frame = small_moon_slingshot_frame().unwrap();
        let up = frame.outward();
        let height = |point: Vec3| (point - BLUE_MOON_SMALL_MOON_CENTER).dot(up);
        let parts: Vec<_> = scene
            .cylinders()
            .filter(|part| part.material_id() == space.slingshot_wood)
            .collect();
        let trunk = parts
            .iter()
            .min_by(|left, right| height(left.center()).total_cmp(&height(right.center())))
            .unwrap();

        assert!(metadata.blue_moon_slingshot_wood_parts > 0);
        assert!(metadata.blue_moon_slingshot_band_parts > 0);
        assert!(up.y > 0.95);
        assert!(
            parts
                .iter()
                .all(|part| (part.center() - BLUE_MOON_SMALL_MOON_CENTER).length() < 1.2)
        );
        assert!(height(trunk.center()) - trunk.half_height() < BLUE_MOON_SMALL_MOON_RADIUS);
        assert!(height(trunk.center()) + trunk.half_height() > BLUE_MOON_SMALL_MOON_RADIUS);
    }

    #[test]
    fn one_atmosphere_wraps_the_whole_diorama() {
        let (scene, metadata) = build_blue_moon_scene_with_metadata().unwrap();
        let (_, materials) = material_ids();
        let atmosphere = scene.objects()[metadata.blue_moon_atmosphere_id.unwrap()]
            .as_sphere()
            .unwrap();
        let material = scene.material(atmosphere.material_id()).unwrap();
        let atmospheres = scene
            .objects()
            .iter()
            .filter(|object| object.material_id() == materials.atmosphere)
            .count();

        assert_eq!(atmospheres, 1);
        assert_eq!(atmosphere.center(), BLUE_MOON_ATMOSPHERE_CENTER);
        assert_eq!(atmosphere.radius(), BLUE_MOON_ATMOSPHERE_RADIUS);
        assert!(material.transparency > 0.85);
        assert_eq!(material.refractive_index, 1.0);
        // The asteroids are the last parts before the atmosphere.
        let atmosphere_id = metadata.blue_moon_atmosphere_id.unwrap();
        let asteroids = atmosphere_id - metadata.blue_moon_floating_asteroid_parts..atmosphere_id;
        for (index, object) in scene.objects().iter().enumerate() {
            if object.material_id() == materials.atmosphere {
                continue;
            }
            let (center, radius) = bounding_sphere(object);
            let reach = (center - BLUE_MOON_ATMOSPHERE_CENTER).length();

            if asteroids.contains(&index) {
                assert!(
                    reach - radius > BLUE_MOON_ATMOSPHERE_RADIUS,
                    "asteroid inside"
                );
            } else {
                assert!(
                    reach + radius < BLUE_MOON_ATMOSPHERE_RADIUS,
                    "object outside the atmosphere at {center:?}"
                );
            }
        }
    }

    #[test]
    fn diorama_is_lit_from_inside_the_atmosphere_and_the_shell_from_outside() {
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();
        let inside = scene
            .lights()
            .iter()
            .filter(|light| is_inside_atmosphere(light.position))
            .count();
        let outside = scene.lights().len() - inside;

        assert_eq!(inside, INSIDE_LIGHTS.len() + BUBBLES.len() + 1);
        assert_eq!(outside, 1 + ATMOSPHERE_RIM_LIGHTS.len());
        // The main inside lights are out in the open, not buried in the moons.
        for (offset, _, _) in INSIDE_LIGHTS {
            let position = BLUE_MOON_ATMOSPHERE_CENTER + offset;
            let from_moon = position - BLUE_MOON_PLANET_CENTER;

            assert!(from_moon.length() > ground_distance(from_moon) + 0.3);
            assert!(
                (position - BLUE_MOON_SMALL_MOON_CENTER).length()
                    > BLUE_MOON_SMALL_MOON_RADIUS + 0.3
            );
        }
    }

    #[test]
    fn outside_key_light_comes_from_the_sun_shown_at_the_top_left() {
        let (scene, _) = build_blue_moon_scene_with_metadata().unwrap();
        let skybox = scene.skybox().unwrap();
        let key_light = scene.lights()[0];
        let uv = skybox.direction_to_uv(BLUE_MOON_SUN_DIRECTION).unwrap();
        let camera = blue_moon_orbit_camera(16.0 / 9.0).to_camera();
        let corner = camera.ray_for_pixel(0, 0, 160, 90);
        let corner_color = skybox.sample_direction(corner.direction);

        assert_eq!(key_light.position, blue_moon_sun_light_position());
        assert!(key_light.color.r > key_light.color.b);
        assert!((uv.u - SPACE_SUN_U).abs() < 0.005);
        assert!((uv.v - SPACE_SUN_V).abs() < 0.005);
        assert!(corner_color.r > 0.9 && corner_color.g > 0.6);
        assert!(scene.intersect(&corner, 0.001, 100.0).is_none());
    }

    #[test]
    fn initial_camera_sees_the_whole_atmosphere_from_outside() {
        let orbit = blue_moon_orbit_camera(16.0 / 9.0);
        let camera = orbit.to_camera();
        let distance = (camera.position - BLUE_MOON_ATMOSPHERE_CENTER).length();
        let angular_radius = (BLUE_MOON_ATMOSPHERE_RADIUS / distance).asin();

        assert_eq!(orbit.target, BLUE_MOON_ATMOSPHERE_CENTER);
        assert!(distance > BLUE_MOON_ATMOSPHERE_RADIUS * 1.5);
        assert!(angular_radius < (orbit.vertical_fov_degrees * 0.5).to_radians());
        assert!(camera.position.z > BLUE_MOON_ATMOSPHERE_CENTER.z);
    }
}
