//! World selector, styled like the world map of the original game: each world
//! is a planet of small cubes dressed like its level, with a wooden sign on
//! top where the app writes the world's name. Pigs in bubbles, a satellite,
//! floating crystals, rocks and a space mine decorate the space between the
//! planets, over a blue-to-purple sky with soft bands and faint star
//! crystals.
//!
//! The planets are in a row in the XY plane (Y up, +Z towards the default
//! camera), like before, so the app keeps picking them by their center and
//! radius (`galaxy_selector_worlds`).

use super::{
    GALAXY_SELECTOR_BLUE_MOON_CENTER, GALAXY_SELECTOR_BLUE_MOON_RADIUS,
    GALAXY_SELECTOR_COOKIE_CENTER, GALAXY_SELECTOR_COOKIE_RADIUS,
    GALAXY_SELECTOR_LEVEL_FOUR_CENTER, GALAXY_SELECTOR_LEVEL_FOUR_RADIUS,
    GALAXY_SELECTOR_LEVEL_THREE_CENTER, GALAXY_SELECTOR_LEVEL_THREE_RADIUS, PlanetType,
    ROCK_LUMP_OFFSET, SpaceBuildError, SpaceMaterials, add_space_pig, add_voxel_rock,
    base_space_scene_with_skybox, basis_with_up, galaxy_selector_worlds, generate_skybox_texture,
    hash3, level_one, smooth_hash, smoothstep, value_noise_3d,
    voxel::{self, VoxelBall, VoxelBody, VoxelBodyParts},
};
use crate::{
    basis::Basis3,
    color::Color,
    cone::Cone,
    crystal::{Crystal, CrystalShape},
    light::PointLight,
    material::Material,
    math::{Vec2, Vec3},
    oriented_box::OrientedBox,
    scene::Scene,
    skybox::Skybox,
    sphere::Sphere,
    texture::{TextureError, WrapMode},
};

/// Cubes of the selector planets.
const SELECTOR_CUBE_EDGE: f32 = 0.12;
const SELECTOR_MIN_CUBES_ACROSS: f32 = 9.0;
const PLANET_TEXTURE_WIDTH: usize = 256;
const PLANET_TEXTURE_HEIGHT: usize = 128;

/// Wooden sign over each planet: half size of the board, how far its bottom
/// is over the planet top, the posts and the tilt of each sign in degrees.
pub const SELECTOR_SIGN_HALF_WIDTH: f32 = 1.10;
pub const SELECTOR_SIGN_HALF_HEIGHT: f32 = 0.31;
/// Sign boards are dark brown planks with the wood texture.
const SIGN_WOOD_TINT: Color = Color::new(0.60, 0.38, 0.22);
const SIGN_HALF_DEPTH: f32 = 0.06;
const SIGN_LIFT: f32 = 0.30;
const SIGN_POST_OFFSET: f32 = 0.45;
const SIGN_POST_RADIUS: f32 = 0.045;
const SIGN_POST_SINK: f32 = 0.25;
/// The signs sit a little in front of the planet centers.
const SIGN_DEPTH: f32 = 0.15;
const SIGN_TILTS_DEGREES: [f32; 4] = [-4.0, 3.0, -3.0, 4.0];

/// Leafy lumps on the first planet: direction and radius (in planet radii).
const LEAF_LUMPS: [(Vec3, f32); 8] = [
    (Vec3::new(0.0, 1.0, 0.3), 0.34),
    (Vec3::new(-0.55, 0.80, 0.25), 0.30),
    (Vec3::new(0.60, 0.75, 0.20), 0.30),
    (Vec3::new(-0.85, 0.35, 0.40), 0.26),
    (Vec3::new(0.85, 0.30, 0.45), 0.26),
    (Vec3::new(0.20, 0.55, 0.80), 0.32),
    (Vec3::new(-0.30, 0.45, 0.85), 0.28),
    (Vec3::new(0.10, -0.95, 0.30), 0.22),
];

/// Crystals growing out of the fourth planet: angle clockwise from +Y and
/// depth towards the camera, in degrees, length and radius.
const PLANET_FOUR_CRYSTALS: [(f32, f32, f32, f32); 8] = [
    (58.0, 15.0, 0.55, 0.12),
    (75.0, -10.0, 0.70, 0.15),
    (95.0, 25.0, 0.45, 0.10),
    (150.0, 20.0, 0.40, 0.10),
    (205.0, 10.0, 0.50, 0.12),
    (240.0, 25.0, 0.65, 0.14),
    (262.0, -15.0, 0.50, 0.12),
    (120.0, 55.0, 0.35, 0.09),
];

/// Pigs standing on the planets: which planet, angle and depth in degrees,
/// and radius.
const PLANET_PIGS: [(usize, f32, f32, f32); 5] = [
    (0, 55.0, 18.0, 0.17),
    (0, -52.0, 22.0, 0.14),
    (2, 58.0, 20.0, 0.16),
    (3, -58.0, 18.0, 0.17),
    (1, 150.0, 30.0, 0.15),
];

/// Pigs floating in bubbles between the planets: center, bubble radius.
const PIG_BUBBLES: [(Vec3, f32); 2] = [
    (Vec3::new(0.02, 0.62, 0.30), 0.30),
    (Vec3::new(0.30, -0.02, 0.45), 0.19),
];

/// The satellite between the first two planets.
const SATELLITE_CENTER: Vec3 = Vec3::new(-3.27, 2.05, -0.60);
const SATELLITE_TILT_RADIANS: f32 = 0.35;
/// Pairs of long crystals crossed like an X: center, size and tilt.
const CROSSED_CRYSTALS: [(Vec3, f32, f32); 2] = [
    (Vec3::new(0.05, -2.62, -0.30), 1.0, 0.25),
    (Vec3::new(3.30, 2.05, -1.00), 0.65, -0.35),
];
/// Small rocks of cubes floating between the planets.
const FLOATING_ROCKS: [(Vec3, f32, FloatingRock); 4] = [
    (Vec3::new(3.12, -2.35, 0.10), 0.22, FloatingRock::Moon),
    (Vec3::new(3.58, -1.95, -0.40), 0.14, FloatingRock::Violet),
    (Vec3::new(-3.40, -2.40, -0.20), 0.18, FloatingRock::Violet),
    (Vec3::new(-7.30, 1.85, -0.80), 0.24, FloatingRock::Moon),
];

/// Floating rocks are blue-gray moon rock or violet moon rock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FloatingRock {
    Moon,
    Violet,
}
const MINE_CENTER: Vec3 = Vec3::new(7.35, -2.55, -0.60);
const MINE_RADIUS: f32 = 0.48;
const MINE_SPIKES: usize = 12;

/// The sky: sharper than 960 wide so the bands stay smooth.
const SKY_WIDTH: usize = 2048;
const SKY_HEIGHT: usize = 1024;

/// Materials used only by the selector.
#[derive(Debug, Clone, Copy)]
struct SelectorMaterials {
    space: SpaceMaterials,
    rocky_planet: usize,
    leaves: usize,
    crystal: usize,
    ice_crystal: usize,
    bubble: usize,
    metal: usize,
    panel: usize,
    mine: usize,
    mine_spike: usize,
    sign: usize,
    crystal_planet: usize,
}

/// Where the parts of the selector went in the scene, for tests.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SelectorParts {
    pub planets: [VoxelBodyParts; 4],
    pub sign_parts: usize,
    pub pig_count: usize,
    pub decoration_parts: usize,
}

pub(super) fn build_selector_scene() -> Result<(Scene, SelectorParts), SpaceBuildError> {
    let (mut scene, space) = base_space_scene_with_skybox(selector_skybox()?)?;
    scene.set_ambient_light(Color::new(0.40, 0.42, 0.50));
    let materials = register_selector_materials(&mut scene, space)?;
    let mut parts = SelectorParts::default();

    let bodies = selector_planet_bodies(materials)?;
    for (slot, body) in parts.planets.iter_mut().zip(&bodies) {
        *slot = voxel::add_voxel_body(&mut scene, body)?;
    }

    let first = scene.object_count();
    for (index, world) in galaxy_selector_worlds().into_iter().enumerate() {
        add_sign(&mut scene, &bodies[index], index, world.center, materials)?;
    }
    parts.sign_parts = scene.object_count() - first;

    parts.pig_count = add_planet_pigs(&mut scene, &bodies, materials)?;
    let first = scene.object_count();
    parts.pig_count += add_planet_dressing(&mut scene, &bodies, materials)?;
    parts.pig_count += add_pig_bubbles(&mut scene, materials)?;
    add_satellite(&mut scene, materials)?;
    add_crossed_crystals(&mut scene, materials)?;
    for (center, radius, rock) in FLOATING_ROCKS {
        let material_id = match rock {
            FloatingRock::Moon => space.selector_locked_moon,
            FloatingRock::Violet => space.selector_locked_level_three,
        };
        add_voxel_rock(
            &mut scene,
            center,
            radius,
            ROCK_LUMP_OFFSET,
            material_id,
            SELECTOR_CUBE_EDGE,
        )?;
    }
    add_mine(&mut scene, materials)?;
    parts.decoration_parts = scene.object_count() - first;

    add_selector_lighting(&mut scene);
    scene.build_bvh();

    Ok((scene, parts))
}

/// Center of the wooden sign of `planet`, where the app writes its name.
pub fn galaxy_selector_sign_center(planet: PlanetType) -> Vec3 {
    let worlds = galaxy_selector_worlds();
    let index = worlds
        .iter()
        .position(|world| world.planet == planet)
        .unwrap_or(0);
    let world = worlds[index];

    sign_center(world.center, world.radius)
}

/// Tilt of the sign of `planet` in degrees, counterclockwise as seen from the
/// default camera, so the app can turn the name with the board.
pub fn galaxy_selector_sign_tilt_degrees(planet: PlanetType) -> f32 {
    galaxy_selector_worlds()
        .iter()
        .position(|world| world.planet == planet)
        .map_or(0.0, |index| {
            SIGN_TILTS_DEGREES[index % SIGN_TILTS_DEGREES.len()]
        })
}

fn sign_center(planet_center: Vec3, planet_radius: f32) -> Vec3 {
    planet_center
        + Vec3::new(
            0.0,
            planet_radius + SIGN_LIFT + SELECTOR_SIGN_HALF_HEIGHT,
            SIGN_DEPTH,
        )
}

pub fn selector_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(generate_skybox_texture(
        SKY_WIDTH,
        SKY_HEIGHT,
        selector_sky_color,
    )?)
    .with_intensity(1.0)
    .with_horizontal_rotation(0.0))
}

/// Teal blue at the top and purple at the bottom, crossed by soft wavy bands,
/// with faint four-pointed star crystals and a few small stars.
pub(super) fn selector_sky_color(u: f32, v: f32) -> Color {
    let u = u.rem_euclid(1.0);
    let top = Color::new(0.090, 0.270, 0.380);
    let middle = Color::new(0.150, 0.250, 0.450);
    let bottom = Color::new(0.330, 0.250, 0.520);
    let mut color = if v > 0.5 {
        middle.lerp(top, smoothstep(0.50, 0.66, v))
    } else {
        middle.lerp(bottom, smoothstep(0.50, 0.36, v))
    };

    // Wavy bands, a little lighter or darker than the sky around them.
    for (height, amplitude, frequency, phase, width, strength) in [
        (0.585, 0.012, 9.0, 0.3, 0.010, 0.10),
        (0.545, 0.010, 12.0, 1.7, 0.008, -0.06),
        (0.470, 0.014, 8.0, 2.9, 0.012, 0.08),
        (0.415, 0.010, 11.0, 4.1, 0.009, -0.05),
        (0.625, 0.014, 7.0, 5.2, 0.012, -0.05),
    ] {
        let center = height
            + (u * frequency * std::f32::consts::TAU + phase).sin() * amplitude
            + (u * frequency * 2.3 * std::f32::consts::TAU + phase * 1.7).sin() * amplitude * 0.35;
        let band = 1.0 - smoothstep(width * 0.3, width, (v - center).abs());
        color += Color::new(0.55, 0.62, 0.85) * (band * strength);
    }

    color += Color::new(0.52, 0.58, 0.78) * (star_crystal(u, v) * 0.35);
    color += Color::new(0.88, 0.94, 1.0) * small_star(u, v);

    color.clamped()
}

/// Faint four-pointed crystal shapes on a sparse grid.
fn star_crystal(u: f32, v: f32) -> f32 {
    let grid_x = u * 120.0;
    let grid_y = v * 60.0;
    let cell_x = grid_x.floor();
    let cell_y = grid_y.floor();
    if smooth_hash(cell_x + 3.0, cell_y + 11.0) < 0.90 {
        return 0.0;
    }

    let jitter_x = smooth_hash(cell_x + 7.0, cell_y) * 0.4 - 0.2;
    let jitter_y = smooth_hash(cell_x, cell_y + 5.0) * 0.4 - 0.2;
    let x = (grid_x.fract() - 0.5 - jitter_x).abs();
    let y = (grid_y.fract() - 0.5 - jitter_y).abs();
    let size = 0.22 + smooth_hash(cell_x + 1.0, cell_y + 2.0) * 0.12;
    // Two thin diamonds crossed: a sparkle with four long points.
    let arms = (1.0 - smoothstep(0.0, 1.0, (x / size + y * 6.0).min(y / size + x * 6.0))).max(0.0);

    arms * (0.6 + smooth_hash(cell_x + 9.0, cell_y + 4.0) * 0.4)
}

fn small_star(u: f32, v: f32) -> f32 {
    let grid_x = u * 260.0;
    let grid_y = v * 130.0;
    let cell_x = grid_x.floor();
    let cell_y = grid_y.floor();
    let noise = smooth_hash(cell_x + 31.0, cell_y + 17.0);
    if noise < 0.975 {
        return 0.0;
    }

    let x = grid_x.fract() - 0.5;
    let y = grid_y.fract() - 0.5;
    let distance = (x * x + y * y).sqrt();
    (1.0 - smoothstep(0.05, 0.18, distance)) * 0.5
}

fn register_selector_materials(
    scene: &mut Scene,
    space: SpaceMaterials,
) -> Result<SelectorMaterials, SpaceBuildError> {
    let rocky_texture = scene.add_texture(level_one::sphere_texture(
        PLANET_TEXTURE_WIDTH,
        PLANET_TEXTURE_HEIGHT,
        rocky_planet_color,
    )?);
    let leaf_texture = scene.add_texture(level_one::sphere_texture(
        PLANET_TEXTURE_WIDTH,
        PLANET_TEXTURE_HEIGHT,
        level_one::leaf_color,
    )?);
    let textured = |texture: usize, emission: Color| {
        Material::new(Color::WHITE, 0.15, 18.0, 0.0, 0.0, 1.0, emission).with_texture(
            texture,
            Vec2::new(1.0, 1.0),
            WrapMode::Repeat,
        )
    };

    let rocky_planet =
        scene.add_material(textured(rocky_texture, Color::new(0.030, 0.035, 0.045)))?;
    let leaves = scene.add_material(textured(leaf_texture, Color::new(0.020, 0.050, 0.015)))?;
    let crystal = scene.add_material(Material::new(
        Color::new(0.96, 0.58, 0.92),
        0.85,
        90.0,
        0.07,
        0.0,
        1.0,
        Color::new(0.110, 0.045, 0.110),
    ))?;
    let ice_crystal = scene.add_material(Material::new(
        Color::new(0.74, 0.80, 0.98),
        0.80,
        80.0,
        0.05,
        0.45,
        1.15,
        Color::new(0.080, 0.090, 0.140),
    ))?;
    let bubble = scene.add_material(Material::new(
        Color::new(0.80, 0.92, 1.0),
        0.60,
        70.0,
        0.0,
        0.86,
        1.0,
        Color::new(0.060, 0.080, 0.110),
    ))?;
    let metal = scene.add_material(Material::new(
        Color::new(0.82, 0.84, 0.88),
        0.90,
        80.0,
        0.15,
        0.0,
        1.0,
        Color::new(0.050, 0.050, 0.060),
    ))?;
    let panel = scene.add_material(Material::new(
        Color::new(0.20, 0.36, 0.78),
        0.90,
        90.0,
        0.10,
        0.0,
        1.0,
        Color::new(0.030, 0.060, 0.160),
    ))?;
    let mine = scene.add_material(Material::new(
        Color::new(0.52, 0.12, 0.12),
        0.50,
        40.0,
        0.05,
        0.0,
        1.0,
        Color::new(0.060, 0.010, 0.010),
    ))?;
    let mine_spike = scene.add_material(Material::new(
        Color::new(0.30, 0.28, 0.30),
        0.60,
        50.0,
        0.10,
        0.0,
        1.0,
        Color::new(0.020, 0.018, 0.020),
    ))?;

    // The crystal world is darker than its old selector planet, like level
    // four itself.
    let crystal_planet = scene.add_material(Material {
        albedo: Color::new(0.78, 0.66, 0.90),
        emission: Color::new(0.060, 0.030, 0.070),
        ..scene
            .material(space.selector_locked_level_four)
            .copied()
            .unwrap_or_default()
    })?;
    let sign = scene.add_material(Material {
        albedo: SIGN_WOOD_TINT,
        emission: Color::new(0.030, 0.016, 0.006),
        ..scene.material(space.wood).copied().unwrap_or_default()
    })?;

    Ok(SelectorMaterials {
        space,
        sign,
        crystal_planet,
        rocky_planet,
        leaves,
        crystal,
        ice_crystal,
        bubble,
        metal,
        panel,
        mine,
        mine_spike,
    })
}

/// Gray rock with round craters, like the first world of the original map.
fn rocky_planet_color(direction: Vec3) -> Color {
    let grain = value_noise_3d(direction * 5.0 + Vec3::new(2.0, 7.0, 1.0));
    let mut color = Color::new(0.30, 0.31, 0.34).lerp(Color::new(0.48, 0.48, 0.50), grain);

    for index in 0..10 {
        let i = index as f32;
        let crater = Vec3::new(
            hash3(i, 2.0, 5.0) - 0.5,
            hash3(i, 3.0, 5.0) - 0.5,
            hash3(i, 4.0, 5.0) * 0.8,
        )
        .normalized();
        let radius = 0.18 + hash3(i, 5.0, 5.0) * 0.18;
        let distance = direction.dot(crater).clamp(-1.0, 1.0).acos() / radius;
        let floor = 1.0 - smoothstep(0.75, 0.95, distance);
        color = color.lerp(Color::new(0.20, 0.21, 0.24), floor * 0.7);
    }

    color
}

/// The four planets, in the order of `galaxy_selector_worlds`.
fn selector_planet_bodies(materials: SelectorMaterials) -> Result<[VoxelBody; 4], SpaceBuildError> {
    let ball = |center: Vec3, radius: f32, material_id: usize| {
        VoxelBody::ball(
            VoxelBall::new(center, radius, material_id),
            SELECTOR_CUBE_EDGE,
            SELECTOR_MIN_CUBES_ACROSS,
        )
    };

    // A cratered rock covered by leafy lumps on top.
    let center = GALAXY_SELECTOR_BLUE_MOON_CENTER;
    let radius = GALAXY_SELECTOR_BLUE_MOON_RADIUS;
    let mut balls = vec![VoxelBall::new(center, radius, materials.rocky_planet)];
    for (direction, lump) in LEAF_LUMPS {
        balls.push(VoxelBall::new(
            center + direction.normalized() * (radius * 0.80),
            radius * lump,
            materials.leaves,
        ));
    }
    let first = VoxelBody::new(&balls, SELECTOR_CUBE_EDGE)?;

    Ok([
        first,
        ball(
            GALAXY_SELECTOR_COOKIE_CENTER,
            GALAXY_SELECTOR_COOKIE_RADIUS,
            materials.space.selector_locked_cookie,
        )?,
        ball(
            GALAXY_SELECTOR_LEVEL_THREE_CENTER,
            GALAXY_SELECTOR_LEVEL_THREE_RADIUS,
            materials.space.level_three_asteroid,
        )?,
        ball(
            GALAXY_SELECTOR_LEVEL_FOUR_CENTER,
            GALAXY_SELECTOR_LEVEL_FOUR_RADIUS,
            materials.crystal_planet,
        )?,
    ])
}

/// Unit vector `angle` degrees clockwise from +Y, tilted `depth` degrees
/// towards the camera.
fn surface_direction(angle_degrees: f32, depth_degrees: f32) -> Vec3 {
    let (sin_angle, cos_angle) = angle_degrees.to_radians().sin_cos();
    let (sin_depth, cos_depth) = depth_degrees.to_radians().sin_cos();

    Vec3::new(sin_angle * cos_depth, cos_angle * cos_depth, sin_depth)
}

/// Board on two posts planted in the top of the planet's cubes.
fn add_sign(
    scene: &mut Scene,
    body: &VoxelBody,
    index: usize,
    planet_center: Vec3,
    materials: SelectorMaterials,
) -> Result<(), SpaceBuildError> {
    let planet_radius = galaxy_selector_worlds()[index].radius;
    let center = sign_center(planet_center, planet_radius);
    let tilt = SIGN_TILTS_DEGREES[index % SIGN_TILTS_DEGREES.len()].to_radians();
    let basis = Basis3::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), tilt)?;

    scene.add_oriented_box(OrientedBox::new(
        center,
        Vec3::new(
            SELECTOR_SIGN_HALF_WIDTH,
            SELECTOR_SIGN_HALF_HEIGHT,
            SIGN_HALF_DEPTH,
        ),
        basis,
        materials.sign,
    )?)?;

    for side in [-1.0, 1.0] {
        let top = center
            + basis.local_to_world_vector(Vec3::new(
                side * SIGN_POST_OFFSET,
                -SELECTOR_SIGN_HALF_HEIGHT * 0.5,
                -SIGN_HALF_DEPTH - SIGN_POST_RADIUS,
            ));
        let down = (Vec3::new(top.x, top.y, 0.0) - planet_center).normalized();
        let ground = body.surface_distance(planet_center, down);
        let bottom = planet_center + down * (ground - SIGN_POST_SINK);
        let bottom = Vec3::new(bottom.x, bottom.y, top.z);
        level_one::add_segment(
            scene,
            bottom,
            top,
            SIGN_POST_RADIUS,
            materials.space.slingshot_wood,
        )?;
    }

    Ok(())
}

/// Pigs standing on the planets, facing the camera.
fn add_planet_pigs(
    scene: &mut Scene,
    bodies: &[VoxelBody; 4],
    materials: SelectorMaterials,
) -> Result<usize, SpaceBuildError> {
    for (planet, angle, depth, radius) in PLANET_PIGS {
        let body = &bodies[planet];
        let up = surface_direction(angle, depth);
        let ground = body.surface_distance(body.origin(), up);
        let center = body.origin() + up * (ground + radius * 0.95);
        add_pig(scene, center, radius, up, materials)?;
    }

    Ok(PLANET_PIGS.len())
}

fn add_pig(
    scene: &mut Scene,
    center: Vec3,
    radius: f32,
    up: Vec3,
    materials: SelectorMaterials,
) -> Result<(), SpaceBuildError> {
    let up = up.normalized();
    let look = Vec3::new(0.0, -0.1, 1.0);
    let forward = (look - up * look.dot(up)).normalized();
    let basis = Basis3::new(up.cross(forward), up, forward)?;

    add_space_pig(scene, center, radius, basis, materials.space)
}

/// What stands on each planet besides its pigs: a wooden frame with a small
/// pig on the first, a stone block and a TNT crate on the third and crystals
/// on the fourth. Returns how many pigs it added.
fn add_planet_dressing(
    scene: &mut Scene,
    bodies: &[VoxelBody; 4],
    materials: SelectorMaterials,
) -> Result<usize, SpaceBuildError> {
    let on_ground = |body: &VoxelBody, angle: f32, depth: f32, lift: f32| {
        let up = surface_direction(angle, depth);
        let ground = body.surface_distance(body.origin(), up);
        (body.origin() + up * (ground + lift), up)
    };

    // First planet: a square wooden frame on the left, with a small pig in it.
    let frame_half = 0.20;
    let (frame_center, up) = on_ground(&bodies[0], -72.0, 32.0, frame_half * 0.9);
    let frame_basis = basis_with_up(up)?;
    let bar = 0.025;
    for (offset, half_extents) in [
        (
            Vec3::new(0.0, frame_half - bar, 0.0),
            Vec3::new(frame_half, bar, 0.08),
        ),
        (
            Vec3::new(0.0, -frame_half + bar, 0.0),
            Vec3::new(frame_half, bar, 0.08),
        ),
        (
            Vec3::new(frame_half - bar, 0.0, 0.0),
            Vec3::new(bar, frame_half - bar * 2.0, 0.08),
        ),
        (
            Vec3::new(-frame_half + bar, 0.0, 0.0),
            Vec3::new(bar, frame_half - bar * 2.0, 0.08),
        ),
    ] {
        scene.add_oriented_box(OrientedBox::new(
            frame_center + frame_basis.local_to_world_vector(offset),
            half_extents,
            frame_basis,
            materials.space.wood,
        )?)?;
    }
    add_pig(scene, frame_center, 0.12, up, materials)?;

    // Third planet: a stone block and a TNT crate standing on the rock.
    for (angle, half, material_id) in [
        (-48.0, 0.17, materials.space.level_three_stone),
        (122.0, 0.14, materials.space.tnt_crate),
    ] {
        let (center, up) = on_ground(&bodies[2], angle, 18.0, half * 0.9);
        scene.add_oriented_box(OrientedBox::new(
            center,
            Vec3::new(half, half, half),
            basis_with_up(up)?,
            material_id,
        )?)?;
    }

    // Fourth planet: pink crystals growing out of it.
    for (index, (angle, depth, length, radius)) in PLANET_FOUR_CRYSTALS.into_iter().enumerate() {
        let body = &bodies[3];
        let up = surface_direction(angle, depth);
        let ground = body.surface_distance(body.origin(), up);
        let embed = radius * 1.3;
        let total = length + embed;
        let twist = index as f32 * 2.399;
        let basis = basis_with_up(up)?;
        let (sin_twist, cos_twist) = twist.sin_cos();
        let basis = Basis3::new(
            basis.right() * cos_twist + basis.forward() * sin_twist,
            basis.up(),
            basis.forward() * cos_twist - basis.right() * sin_twist,
        )?;
        scene.add_crystal(Crystal::new(
            body.origin() + up * (ground - embed),
            CrystalShape {
                radius,
                body_height: total * 0.7,
                tip_height: total * 0.3,
                tip_cut: 0.0,
                base_tip_height: 0.0,
                sides: 6,
            },
            basis,
            materials.crystal,
        )?)?;
    }

    Ok(1)
}

/// Pigs floating in bubbles, like the ones around the frozen world of the
/// original map. Returns how many pigs were added.
fn add_pig_bubbles(
    scene: &mut Scene,
    materials: SelectorMaterials,
) -> Result<usize, SpaceBuildError> {
    for (center, radius) in PIG_BUBBLES {
        add_pig(
            scene,
            center,
            radius * 0.56,
            Vec3::new(0.0, 1.0, 0.0),
            materials,
        )?;
        scene.add_sphere(Sphere::new(center, radius, materials.bubble)?)?;
    }

    Ok(PIG_BUBBLES.len())
}

/// A small satellite: a metal body, a dish and two blue solar panels.
fn add_satellite(scene: &mut Scene, materials: SelectorMaterials) -> Result<(), SpaceBuildError> {
    let basis = Basis3::from_axis_angle(Vec3::new(0.2, 0.3, 1.0), SATELLITE_TILT_RADIANS)?;
    let at = |local: Vec3| SATELLITE_CENTER + basis.local_to_world_vector(local);

    scene.add_oriented_box(OrientedBox::new(
        SATELLITE_CENTER,
        Vec3::new(0.16, 0.12, 0.12),
        basis,
        materials.metal,
    )?)?;
    level_one::add_segment(
        scene,
        at(Vec3::new(-0.62, 0.0, 0.0)),
        at(Vec3::new(0.62, 0.0, 0.0)),
        0.018,
        materials.metal,
    )?;
    for side in [-1.0, 1.0] {
        for panel in 0..2 {
            let x = side * (0.30 + 0.17 + panel as f32 * 0.32);
            scene.add_oriented_box(OrientedBox::new(
                at(Vec3::new(x, 0.0, 0.0)),
                Vec3::new(0.15, 0.14, 0.012),
                basis,
                materials.panel,
            )?)?;
        }
    }
    let dish_axis = basis.local_to_world_vector(Vec3::new(0.0, 1.0, 0.4));
    scene.add_cone(Cone::new(
        at(Vec3::new(0.0, 0.20, 0.05)),
        0.11,
        0.05,
        basis_with_up(-dish_axis)?,
        materials.metal,
    )?)?;
    level_one::add_segment(
        scene,
        at(Vec3::new(0.0, 0.12, 0.0)),
        at(Vec3::new(0.0, 0.32, 0.12)),
        0.012,
        materials.metal,
    )?;

    Ok(())
}

/// Two long pale crystals crossed like an X.
fn add_crossed_crystals(
    scene: &mut Scene,
    materials: SelectorMaterials,
) -> Result<(), SpaceBuildError> {
    for (center, size, tilt) in CROSSED_CRYSTALS {
        for angle in [tilt + 0.62, tilt - 0.62] {
            let axis = Vec3::new(angle.sin(), angle.cos(), 0.15).normalized();
            let length = size * 1.2;
            scene.add_crystal(Crystal::new(
                center - axis * (length * 0.5),
                CrystalShape {
                    radius: size * 0.09,
                    body_height: length * 0.8,
                    tip_height: length * 0.2,
                    tip_cut: 0.0,
                    base_tip_height: length * 0.15,
                    sides: 4,
                },
                basis_with_up(axis)?,
                materials.ice_crystal,
            )?)?;
        }
    }

    Ok(())
}

/// A spiky space mine at the right edge of the map.
fn add_mine(scene: &mut Scene, materials: SelectorMaterials) -> Result<(), SpaceBuildError> {
    scene.add_sphere(Sphere::new(MINE_CENTER, MINE_RADIUS, materials.mine)?)?;

    for index in 0..MINE_SPIKES {
        // Spikes spread over the sphere with the golden angle.
        let i = index as f32 + 0.5;
        let y = 1.0 - 2.0 * i / MINE_SPIKES as f32;
        let ring = (1.0 - y * y).max(0.0).sqrt();
        let angle = i * 2.399;
        let direction = Vec3::new(ring * angle.cos(), y, ring * angle.sin());
        scene.add_cone(Cone::new(
            MINE_CENTER + direction * (MINE_RADIUS + 0.10),
            0.07,
            0.14,
            basis_with_up(direction)?,
            materials.mine_spike,
        )?)?;
    }

    Ok(())
}

/// Flat cartoon lighting: a key light from the upper left and front, a cool
/// fill from the right and a soft light from below.
fn add_selector_lighting(scene: &mut Scene) {
    for (position, color, intensity) in [
        (
            Vec3::new(-5.0, 6.0, 10.0),
            Color::new(1.0, 0.96, 0.90),
            120.0,
        ),
        (Vec3::new(7.0, 1.5, 9.0), Color::new(0.78, 0.86, 1.0), 55.0),
        (Vec3::new(0.0, -6.0, 6.0), Color::new(0.80, 0.70, 1.0), 25.0),
    ] {
        scene.add_light(PointLight::new(position, color, intensity));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PIG_BUBBLES, PLANET_PIGS, build_selector_scene, galaxy_selector_sign_center,
        selector_sky_color,
    };
    use crate::{
        ray::Ray,
        space::{galaxy_selector_orbit_camera, galaxy_selector_worlds},
    };

    #[test]
    fn selector_planets_are_balls_of_cubes_with_signs_pigs_and_decorations() {
        let (scene, parts) = build_selector_scene().unwrap();

        for (planet, world) in parts.planets.iter().zip(galaxy_selector_worlds()) {
            assert!(planet.cube_count > 150, "{:?}", world.planet);
            // Every cube of a planet is close to its center.
            for cube in scene.objects()[planet.first_id..planet.first_id + planet.cube_count]
                .iter()
                .filter_map(|object| object.as_cube())
            {
                let center = (cube.min + cube.max) * 0.5;
                assert!((center - world.center).length() < world.radius * 1.4);
            }
        }
        // A board and two posts per sign.
        assert_eq!(parts.sign_parts, 4 * 3);
        assert_eq!(parts.pig_count, PLANET_PIGS.len() + 1 + PIG_BUBBLES.len());
        assert!(parts.decoration_parts > 30);
        assert!(scene.lights().len() >= 3);
        assert!(scene.skybox().is_some());
    }

    #[test]
    fn signs_stand_over_their_planets_and_the_camera_sees_every_planet() {
        let (scene, _) = build_selector_scene().unwrap();
        let camera = galaxy_selector_orbit_camera(16.0 / 9.0).to_camera();

        for world in galaxy_selector_worlds() {
            let sign = galaxy_selector_sign_center(world.planet);
            assert!(sign.y > world.center.y + world.radius);
            assert!((sign.x - world.center.x).abs() < 1.0e-4);

            let hit = scene
                .intersect(
                    &Ray::new(camera.position, world.center - camera.position),
                    0.001,
                    100.0,
                )
                .unwrap();
            assert!((hit.position - world.center).length() < world.radius * 1.3);
            assert_eq!(
                scene.material(hit.material_id).unwrap().transparency,
                0.0,
                "{:?} is hidden",
                world.planet
            );
        }
    }

    #[test]
    fn selector_sky_goes_from_teal_blue_to_purple() {
        let top = selector_sky_color(0.25, 0.64);
        let bottom = selector_sky_color(0.25, 0.37);

        assert!(top.b > top.r * 2.0);
        assert!(top.g > top.r);
        assert!(bottom.r > top.r * 1.5);
        assert!(bottom.b > bottom.g);
    }
}
