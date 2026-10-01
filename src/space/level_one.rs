//! Level one, playable: a slingshot on a small rock, two leafy asteroids with
//! overlapping atmospheres and a pig held by a wood frame on the far side of
//! the second asteroid, under a big yellow moon.
//!
//! The level is laid out in the XY plane (Y up, +X right, +Z towards the
//! default camera), which is also the plane of the game (see `game`). The
//! asteroids and the rock are balls of small cubes (see `voxel`). Everything
//! that moves (birds, pig, wood blocks, the slingshot elastic and puffs of
//! smoke) is rebuilt as dynamic scene objects whenever the game changes.
//!
//! Shadow rays stop at every object, including the transparent atmospheres,
//! so each atmosphere and the lens where both overlap have their own lights.

use super::{
    BLUE_MOON_SLINGSHOT_ELASTIC_RADIUS, BLUE_MOON_SLINGSHOT_ELBOW, BLUE_MOON_SLINGSHOT_POUCH,
    BLUE_MOON_SLINGSHOT_POUCH_HALF_WIDTH, BLUE_MOON_SLINGSHOT_POUCH_RADIUS,
    BLUE_MOON_SLINGSHOT_TIP, BLUE_MOON_SLINGSHOT_WRAP_RADIUS, Songbird, SongbirdMaterials,
    SpaceBuildError, SpaceMaterials, add_slingshot_without_elastic, add_songbird, add_space_pig,
    base_space_scene_with_skybox, basis_with_up,
    birds::{
        BIRD_COUNT, BIRD_POUCH_LIFT, BIRD_RADIUS_PER_SLINGSHOT_SCALE, STANDING_HEIGHT,
        register_bird_materials,
    },
    generate_skybox_texture, hash3, mirrored_x, slingshot_local_point, smoothstep,
    sphere_direction_from_uv, star_strength, value_noise_3d,
    voxel::{self, VoxelBall, VoxelBody, VoxelBodyParts},
};
use crate::{
    basis::Basis3,
    camera::OrbitCamera,
    color::Color,
    cylinder::Cylinder,
    game::{
        BirdSpot, BlockKind, BlockLayout, BlockState, Game, GravityPlanet, LevelLayout, PuffKind,
        SolidRock,
    },
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

/// Left asteroid: the first gravity field the birds cross.
pub const LEVEL_ONE_LEFT_CENTER: Vec3 = Vec3::new(-0.46, -0.55, 0.0);
pub const LEVEL_ONE_LEFT_RADIUS: f32 = 0.42;
pub const LEVEL_ONE_LEFT_ATMOSPHERE_RADIUS: f32 = 1.80;
/// Right asteroid, with the pig on its far side.
pub const LEVEL_ONE_RIGHT_CENTER: Vec3 = Vec3::new(3.04, -0.55, 0.0);
pub const LEVEL_ONE_RIGHT_RADIUS: f32 = 0.48;
pub const LEVEL_ONE_RIGHT_ATMOSPHERE_RADIUS: f32 = 2.12;
/// Pull of both atmospheres, in units per second squared.
pub const LEVEL_ONE_GRAVITY: f32 = 4.0;
/// Small rock under the slingshot. It has no atmosphere.
pub const LEVEL_ONE_ROCK_CENTER: Vec3 = Vec3::new(-3.56, -0.80, 0.0);
pub const LEVEL_ONE_ROCK_RADIUS: f32 = 0.26;
/// Asteroids are this much wider than their main ball where birds collide,
/// because the leafy lumps stick out a little.
const COLLISION_RADIUS_SCALE: f32 = 1.04;

pub const LEVEL_ONE_BIRD_COUNT: usize = BIRD_COUNT;
pub const LEVEL_ONE_BIRD_RADIUS: f32 = BIRD_RADIUS_PER_SLINGSHOT_SCALE * SLINGSHOT_SCALE;
pub const LEVEL_ONE_PIG_RADIUS: f32 = 0.15;
/// Half size of the square wood frame planted in the right asteroid and of
/// the plank that holds the pig.
const FRAME_HALF_SIZE: f32 = 0.12;
const FRAME_EMBED: f32 = 0.04;
const FRAME_BAR: f32 = 0.045;
const PLANK_HALF_LENGTH: f32 = 0.15;
const PLANK_HALF_HEIGHT: f32 = 0.035;
const BLOCK_HALF_DEPTH: f32 = 0.10;
const PLANK_HALF_DEPTH: f32 = 0.07;

/// The level box: anything leaving it is gone.
const BOUNDS_MIN: Vec3 = Vec3::new(-8.5, -5.5, 0.0);
const BOUNDS_MAX: Vec3 = Vec3::new(10.0, 5.0, 0.0);

/// Cube edges of the asteroids and the rock.
const ASTEROID_CUBE_EDGE: f32 = 0.07;
const ROCK_CUBE_EDGE: f32 = 0.052;
const MIN_CUBES_ACROSS: f32 = 9.0;
/// Leafy lumps on the asteroids: direction (in the asteroid frame), distance
/// and radius in asteroid radii. None points to +X, where the right asteroid
/// holds the wood frame.
const ASTEROID_LUMPS: [(Vec3, f32, f32); 9] = [
    (Vec3::new(0.10, 1.0, 0.25), 0.72, 0.36),
    (Vec3::new(-0.80, 0.55, 0.20), 0.72, 0.34),
    (Vec3::new(-0.95, -0.35, 0.10), 0.70, 0.36),
    (Vec3::new(-0.20, -1.0, 0.30), 0.72, 0.34),
    (Vec3::new(0.55, -0.80, 0.25), 0.70, 0.32),
    (Vec3::new(0.60, 0.75, -0.10), 0.70, 0.33),
    (Vec3::new(0.05, 0.15, 1.0), 0.70, 0.36),
    (Vec3::new(-0.40, -0.10, -1.0), 0.70, 0.34),
    (Vec3::new(0.35, -0.30, 0.90), 0.68, 0.30),
];

const SLINGSHOT_DIRECTION: Vec3 = Vec3::new(0.0, 1.0, 0.0);
const SLINGSHOT_SCALE: f32 = 1.0;
const SLINGSHOT_EMBED: f32 = 0.03;
/// Turns the fork so it opens left and right, seen from the camera, with a
/// little depth.
const SLINGSHOT_YAW_RADIANS: f32 = -std::f32::consts::FRAC_PI_2 + 0.25;
/// Birds wait on the left of the rock, in these directions from its center.
const WAITING_DIRECTIONS: [Vec3; 2] = [Vec3::new(-0.95, 0.28, 0.0), Vec3::new(-0.62, -0.78, 0.0)];

/// The big cartoon moon painted in the sky, up and to the left of the
/// default view: its direction and angular radius in radians.
const MOON_DIRECTION: Vec3 = Vec3::new(-0.60, 0.31, -1.0);
const MOON_ANGULAR_RADIUS: f32 = 0.27;
/// Craters on the moon disc: center and radius in moon radii, seen from the
/// front with +X right and +Y up.
const MOON_CRATERS: [(f32, f32, f32); 8] = [
    (-0.38, 0.38, 0.24),
    (0.22, 0.52, 0.14),
    (0.48, -0.08, 0.20),
    (-0.08, -0.38, 0.27),
    (-0.62, -0.18, 0.12),
    (0.14, 0.08, 0.10),
    (0.62, 0.36, 0.09),
    (-0.30, -0.02, 0.08),
];
const LEAF_TEXTURE_WIDTH: usize = 256;
const LEAF_TEXTURE_HEIGHT: usize = 128;

pub const LEVEL_ONE_CAMERA_TARGET: Vec3 = Vec3::new(0.80, -0.20, 0.0);
const CAMERA_DISTANCE: f32 = 8.3;
const CAMERA_FOV_DEGREES: f32 = 45.0;
/// The sky is sharper than the other levels' so the small cloud bumps keep
/// round edges.
const SKYBOX_WIDTH: usize = 2048;
const SKYBOX_HEIGHT: usize = 1024;

/// Materials of the level; the dynamic objects use them every frame.
#[derive(Debug, Clone, Copy)]
struct LevelOneMaterials {
    space: SpaceMaterials,
    leaves: usize,
    rock: usize,
    atmosphere: usize,
    birds: [SongbirdMaterials; LEVEL_ONE_BIRD_COUNT],
    smoke: usize,
    wood_smoke: usize,
}

/// Slingshot points the elastic needs: where it is tied to the prongs and the
/// pouch at rest, with the half length of the pouch.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Elastic {
    anchors: [Vec3; 2],
    pouch_rest: Vec3,
    pouch_half: Vec3,
}

/// Where the parts of level one went in the scene.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelOneParts {
    pub left_asteroid: VoxelBodyParts,
    pub right_asteroid: VoxelBodyParts,
    pub rock: VoxelBodyParts,
    pub atmosphere_ids: [usize; 2],
    pub slingshot_parts: usize,
}

/// The game of level one together with what it needs to draw it.
#[derive(Debug, Clone)]
pub struct LevelOneGame {
    game: Game,
    materials: LevelOneMaterials,
    elastic: Elastic,
    synced_revision: Option<u64>,
}

impl LevelOneGame {
    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }

    /// Rebuilds the dynamic objects of `scene` when the game changed since the
    /// last call. Returns whether it did.
    pub fn sync_scene(&mut self, scene: &mut Scene) -> Result<bool, SpaceBuildError> {
        if self.synced_revision == Some(self.game.revision()) {
            return Ok(false);
        }

        write_dynamic_objects(scene, &self.game, self.materials, self.elastic)?;
        self.synced_revision = Some(self.game.revision());
        Ok(true)
    }
}

pub fn build_level_one() -> Result<(Scene, LevelOneGame), SpaceBuildError> {
    build_level_one_with_parts().map(|(scene, level, _)| (scene, level))
}

pub(crate) fn build_level_one_with_parts()
-> Result<(Scene, LevelOneGame, LevelOneParts), SpaceBuildError> {
    let (mut scene, space) = base_space_scene_with_skybox(level_one_skybox()?)?;
    scene.set_ambient_light(Color::new(0.22, 0.25, 0.32));
    let materials = register_level_one_materials(&mut scene, space)?;
    let mut parts = LevelOneParts {
        left_asteroid: voxel::add_voxel_body(
            &mut scene,
            &asteroid_body(
                LEVEL_ONE_LEFT_CENTER,
                LEVEL_ONE_LEFT_RADIUS,
                materials.leaves,
                1.0,
            )?,
        )?,
        right_asteroid: voxel::add_voxel_body(
            &mut scene,
            &asteroid_body(
                LEVEL_ONE_RIGHT_CENTER,
                LEVEL_ONE_RIGHT_RADIUS,
                materials.leaves,
                -1.0,
            )?,
        )?,
        rock: voxel::add_voxel_body(&mut scene, &rock_body(materials.rock)?)?,
        ..LevelOneParts::default()
    };

    for (index, (center, radius)) in [
        (LEVEL_ONE_LEFT_CENTER, LEVEL_ONE_LEFT_ATMOSPHERE_RADIUS),
        (LEVEL_ONE_RIGHT_CENTER, LEVEL_ONE_RIGHT_ATMOSPHERE_RADIUS),
    ]
    .into_iter()
    .enumerate()
    {
        parts.atmosphere_ids[index] = scene.object_count();
        scene.add_sphere(Sphere::new(center, radius, materials.atmosphere)?)?;
    }

    let first_slingshot_part = scene.object_count();
    add_slingshot_without_elastic(
        &mut scene,
        slingshot_frame()?,
        SLINGSHOT_YAW_RADIANS,
        SLINGSHOT_SCALE,
        space,
    )?;
    parts.slingshot_parts = scene.object_count() - first_slingshot_part;

    add_level_one_lighting(&mut scene);
    scene.freeze_static_objects();

    let mut level = LevelOneGame {
        game: Game::new(level_one_layout()?),
        materials,
        elastic: slingshot_elastic()?,
        synced_revision: None,
    };
    level.sync_scene(&mut scene)?;

    Ok((scene, level, parts))
}

/// Level one scene with the game at its start.
pub fn build_level_one_scene() -> Result<Scene, SpaceBuildError> {
    build_level_one().map(|(scene, _)| scene)
}

/// Front view of the whole level, like the reference picture.
pub fn level_one_orbit_camera(aspect_ratio: f32) -> OrbitCamera {
    OrbitCamera::new(
        LEVEL_ONE_CAMERA_TARGET,
        0.0,
        0.0,
        CAMERA_DISTANCE,
        CAMERA_FOV_DEGREES,
        aspect_ratio,
        Vec3::new(0.0, 1.0, 0.0),
    )
}

/// Where everything of the game is, taken from the geometry of the scene.
pub fn level_one_layout() -> Result<LevelLayout, SpaceBuildError> {
    let rock = rock_body(0)?;
    let waiting_spots = WAITING_DIRECTIONS
        .iter()
        .map(|&direction| {
            let up = direction.normalized();
            let ground = rock.surface_distance(LEVEL_ONE_ROCK_CENTER, up);

            BirdSpot {
                position: LEVEL_ONE_ROCK_CENTER
                    + up * (ground + LEVEL_ONE_BIRD_RADIUS * STANDING_HEIGHT),
                up,
            }
        })
        .collect();
    let right_surface = LEVEL_ONE_RIGHT_CENTER.x + LEVEL_ONE_RIGHT_RADIUS;
    let frame_center = Vec3::new(
        right_surface - FRAME_EMBED + FRAME_HALF_SIZE,
        LEVEL_ONE_RIGHT_CENTER.y,
        0.0,
    );
    let plank_center = frame_center + Vec3::new(FRAME_HALF_SIZE + PLANK_HALF_LENGTH, 0.0, 0.0);
    let pig_center =
        plank_center + Vec3::new(PLANK_HALF_LENGTH + LEVEL_ONE_PIG_RADIUS * 0.92, 0.0, 0.0);

    Ok(LevelLayout {
        planets: vec![
            GravityPlanet {
                center: LEVEL_ONE_LEFT_CENTER,
                body_radius: LEVEL_ONE_LEFT_RADIUS * COLLISION_RADIUS_SCALE,
                atmosphere_radius: LEVEL_ONE_LEFT_ATMOSPHERE_RADIUS,
                gravity: LEVEL_ONE_GRAVITY,
            },
            GravityPlanet {
                center: LEVEL_ONE_RIGHT_CENTER,
                body_radius: LEVEL_ONE_RIGHT_RADIUS * COLLISION_RADIUS_SCALE,
                atmosphere_radius: LEVEL_ONE_RIGHT_ATMOSPHERE_RADIUS,
                gravity: LEVEL_ONE_GRAVITY,
            },
        ],
        rocks: vec![SolidRock {
            center: LEVEL_ONE_ROCK_CENTER,
            radius: LEVEL_ONE_ROCK_RADIUS,
        }],
        slingshot_rest: slingshot_elastic()?.pouch_rest
            + SLINGSHOT_DIRECTION.normalized() * (BIRD_POUCH_LIFT * SLINGSHOT_SCALE),
        waiting_spots,
        bird_count: LEVEL_ONE_BIRD_COUNT,
        bird_radius: LEVEL_ONE_BIRD_RADIUS,
        pig_center,
        pig_radius: LEVEL_ONE_PIG_RADIUS,
        pig_support: Some(1),
        blocks: vec![
            BlockLayout {
                kind: BlockKind::Frame,
                center: frame_center,
                half_width: FRAME_HALF_SIZE,
                half_height: FRAME_HALF_SIZE,
                angle: 0.0,
                support: None,
            },
            BlockLayout {
                kind: BlockKind::Plank,
                center: plank_center,
                half_width: PLANK_HALF_LENGTH,
                half_height: PLANK_HALF_HEIGHT,
                angle: 0.0,
                support: Some(0),
            },
        ],
        bounds_min: BOUNDS_MIN,
        bounds_max: BOUNDS_MAX,
    })
}

/// Night sky with stars and rows of round blue clouds at the bottom.
pub fn level_one_skybox() -> Result<Skybox, TextureError> {
    Ok(Skybox::new(generate_skybox_texture(
        SKYBOX_WIDTH,
        SKYBOX_HEIGHT,
        level_one_sky_color,
    )?)
    .with_intensity(1.0)
    .with_horizontal_rotation(0.0))
}

fn level_one_sky_color(u: f32, v: f32) -> Color {
    let u = u.rem_euclid(1.0);
    let low = Color::new(0.035, 0.090, 0.215);
    let high = Color::new(0.012, 0.040, 0.120);
    let mut color = low.lerp(high, smoothstep(0.40, 0.80, v));

    color += Color::new(0.86, 0.94, 1.0) * star_strength(u, v);
    color = add_moon(color, sphere_direction_from_uv(u, v));

    // Rows of small round clouds at the bottom of the view, from the back
    // (higher and lighter) to the front.
    for (base, height, bumps, phase, fill, rim) in [
        (
            0.421,
            0.013,
            84.0,
            0.30,
            Color::new(0.120, 0.225, 0.430),
            Color::new(0.300, 0.440, 0.690),
        ),
        (
            0.409,
            0.012,
            97.0,
            0.70,
            Color::new(0.090, 0.180, 0.380),
            Color::new(0.240, 0.370, 0.620),
        ),
        (
            0.396,
            0.012,
            111.0,
            0.15,
            Color::new(0.065, 0.140, 0.320),
            Color::new(0.190, 0.310, 0.550),
        ),
        (
            0.383,
            0.011,
            126.0,
            0.55,
            Color::new(0.045, 0.105, 0.260),
            Color::new(0.150, 0.260, 0.480),
        ),
    ] {
        color = add_cloud_row(color, u, v, base, height, bumps, phase, fill, rim);
    }

    color.clamped()
}

/// Flat yellow cartoon moon with a warm glow around it.
fn add_moon(color: Color, direction: Vec3) -> Color {
    let axis = MOON_DIRECTION.normalized();
    let angle = direction.dot(axis).clamp(-1.0, 1.0).acos();
    let glow = 1.0 - smoothstep(MOON_ANGULAR_RADIUS, MOON_ANGULAR_RADIUS * 2.2, angle);
    let color = color.lerp(Color::new(0.24, 0.30, 0.40), glow.powf(1.5) * 0.45);
    let disc = 1.0 - smoothstep(MOON_ANGULAR_RADIUS - 0.003, MOON_ANGULAR_RADIUS, angle);
    if disc <= 0.0 {
        return color;
    }

    // Position on the disc in moon radii, +X right and +Y up.
    let right = axis.cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
    let up = right.cross(axis).normalized();
    let x = direction.dot(right) / MOON_ANGULAR_RADIUS.sin();
    let y = direction.dot(up) / MOON_ANGULAR_RADIUS.sin();
    let light = smoothstep(-0.9, 0.9, y - x);
    let mut moon = Color::new(0.98, 0.76, 0.24).lerp(Color::new(1.0, 0.90, 0.46), light);

    for (crater_x, crater_y, radius) in MOON_CRATERS {
        let distance = ((x - crater_x).powi(2) + (y - crater_y).powi(2)).sqrt() / radius;
        if distance > 1.3 {
            continue;
        }
        // Lit rim on the lower right, like a bowl lit from the upper left.
        let toward_light = ((crater_x - x) - (crater_y - y)) / (radius * 1.414);
        let floor = 1.0 - smoothstep(0.82, 0.96, distance);
        let rim = smoothstep(0.86, 1.0, distance)
            * (1.0 - smoothstep(1.0, 1.25, distance))
            * smoothstep(-0.2, 0.6, -toward_light);
        moon = moon
            .lerp(Color::new(0.94, 0.62, 0.18), floor * 0.75)
            .lerp(Color::new(1.0, 0.95, 0.66), rim * 0.8);
    }

    color.lerp(moon, disc)
}

/// A row of round cloud bumps whose top edge is `base` plus bumps of up to
/// `height`, with a lighter rim along the edge.
#[allow(clippy::too_many_arguments)]
fn add_cloud_row(
    color: Color,
    u: f32,
    v: f32,
    base: f32,
    height: f32,
    bumps: f32,
    phase: f32,
    fill: Color,
    rim: Color,
) -> Color {
    let position = u * bumps + phase;
    let cell = position.floor();
    let across = (position - cell) * 2.0 - 1.0;
    let size = 0.55 + 0.45 * hash3(cell, bumps, 3.0);
    let bump = (1.0 - across * across).max(0.0).sqrt();
    let top = base + height * size * bump;

    if v > top {
        return color;
    }

    let edge = 1.0 - smoothstep(0.0012, 0.0030, top - v);
    let shade = smoothstep(0.0, 0.02, top - v);
    fill.lerp(fill * 0.8, shade).lerp(rim, edge)
}

fn register_level_one_materials(
    scene: &mut Scene,
    space: SpaceMaterials,
) -> Result<LevelOneMaterials, SpaceBuildError> {
    let leaf_texture = scene.add_texture(sphere_texture(
        LEAF_TEXTURE_WIDTH,
        LEAF_TEXTURE_HEIGHT,
        leaf_color,
    )?);
    let rock_texture = scene.add_texture(sphere_texture(
        LEAF_TEXTURE_WIDTH,
        LEAF_TEXTURE_HEIGHT,
        rock_color,
    )?);

    let leaves = scene.add_material(
        Material::new(
            Color::WHITE,
            0.18,
            20.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.020, 0.050, 0.015),
        )
        .with_texture(leaf_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    let rock = scene.add_material(
        Material::new(
            Color::WHITE,
            0.15,
            18.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.030, 0.035, 0.040),
        )
        .with_texture(rock_texture, Vec2::new(1.0, 1.0), WrapMode::Repeat),
    )?;
    // Pale cyan and almost clear, without refraction so the asteroids and the
    // birds inside are not distorted.
    let atmosphere = scene.add_material(Material::new(
        Color::new(0.55, 0.85, 1.0),
        0.35,
        40.0,
        0.0,
        0.86,
        1.0,
        Color::new(0.070, 0.150, 0.220),
    ))?;
    let birds = register_bird_materials(scene, space)?;
    let smoke = scene.add_material(Material::new(
        Color::new(0.95, 0.96, 1.0),
        0.10,
        10.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.450, 0.460, 0.500),
    ))?;
    let wood_smoke = scene.add_material(Material::new(
        Color::new(0.95, 0.72, 0.45),
        0.10,
        10.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.400, 0.280, 0.150),
    ))?;

    Ok(LevelOneMaterials {
        space,
        leaves,
        rock,
        atmosphere,
        birds,
        smoke,
        wood_smoke,
    })
}

pub(super) fn sphere_texture(
    width: usize,
    height: usize,
    color_at: fn(Vec3) -> Color,
) -> Result<Texture, SpaceBuildError> {
    let mut pixels = Vec::with_capacity(width * height);

    for y in 0..height {
        let v = 1.0 - y as f32 / (height - 1) as f32;
        for x in 0..width {
            let u = x as f32 / (width - 1) as f32;
            pixels.push(color_at(sphere_direction_from_uv(u, v)));
        }
    }

    Ok(Texture::new(width, height, pixels)?)
}

/// Bushy leaves in four greens, with dark gaps between them.
pub(super) fn leaf_color(direction: Vec3) -> Color {
    let leaf = value_noise_3d(direction * 5.0 + Vec3::new(3.1, 1.7, 5.3)) * 0.6
        + value_noise_3d(direction * 12.0 + Vec3::new(7.2, 2.9, 0.4)) * 0.4;
    let gap = Color::new(0.07, 0.24, 0.07);
    let dark = Color::new(0.20, 0.46, 0.10);
    let mid = Color::new(0.38, 0.70, 0.16);
    let light = Color::new(0.62, 0.90, 0.28);

    gap.lerp(dark, smoothstep(0.25, 0.40, leaf))
        .lerp(mid, smoothstep(0.45, 0.58, leaf))
        .lerp(light, smoothstep(0.64, 0.76, leaf))
}

/// Gray rock with leaves over its top.
fn rock_color(direction: Vec3) -> Color {
    let grain = value_noise_3d(direction * 6.0 + Vec3::new(1.3, 4.1, 2.2));
    let stone = Color::new(0.34, 0.36, 0.42).lerp(Color::new(0.56, 0.58, 0.62), grain);
    let cover = smoothstep(-0.75, -0.40, direction.y + (grain - 0.5) * 0.6);

    stone.lerp(leaf_color(direction), cover)
}

/// A leafy asteroid: a ball of cubes with rounder lumps around it. `side`
/// mirrors the lumps so both asteroids do not look the same.
fn asteroid_body(
    center: Vec3,
    radius: f32,
    material_id: usize,
    side: f32,
) -> Result<VoxelBody, SpaceBuildError> {
    let mut balls = vec![VoxelBall::new(center, radius, material_id)];

    for (direction, distance, lump_radius) in ASTEROID_LUMPS {
        let direction = Vec3::new(direction.x * side, direction.y, direction.z).normalized();
        // The right asteroid holds the wood frame on +X: no lump there.
        if side < 0.0 && direction.x > 0.5 {
            continue;
        }
        balls.push(VoxelBall::new(
            center + direction * (radius * distance),
            radius * lump_radius,
            material_id,
        ));
    }

    VoxelBody::new(
        &balls,
        voxel::fitted_cube_edge(radius, ASTEROID_CUBE_EDGE, MIN_CUBES_ACROSS),
    )
}

fn rock_body(material_id: usize) -> Result<VoxelBody, SpaceBuildError> {
    VoxelBody::ball(
        VoxelBall::new(LEVEL_ONE_ROCK_CENTER, LEVEL_ONE_ROCK_RADIUS, material_id),
        ROCK_CUBE_EDGE,
        MIN_CUBES_ACROSS,
    )
}

/// The slingshot stands on the top of the rock's cubes.
fn slingshot_frame() -> Result<RadialFrame, SpaceBuildError> {
    let ground = rock_body(0)?.surface_distance(LEVEL_ONE_ROCK_CENTER, SLINGSHOT_DIRECTION);

    Ok(RadialFrame::from_normal(
        LEVEL_ONE_ROCK_CENTER,
        ground - SLINGSHOT_EMBED,
        SLINGSHOT_DIRECTION,
    )?)
}

fn slingshot_elastic() -> Result<Elastic, SpaceBuildError> {
    let frame = slingshot_frame()?;
    let at = |local: Vec3| {
        frame.local_to_world(slingshot_local_point(
            local,
            SLINGSHOT_YAW_RADIANS,
            SLINGSHOT_SCALE,
        ))
    };
    let anchor = |side: f32| {
        let elbow = mirrored_x(BLUE_MOON_SLINGSHOT_ELBOW, side);
        let prong = mirrored_x(BLUE_MOON_SLINGSHOT_TIP, side) - elbow;
        at(elbow + prong * 0.67 + Vec3::new(0.0, 0.0, BLUE_MOON_SLINGSHOT_WRAP_RADIUS * 0.6))
    };
    let pouch_rest = at(BLUE_MOON_SLINGSHOT_POUCH);
    let pouch_end =
        at(BLUE_MOON_SLINGSHOT_POUCH + Vec3::new(BLUE_MOON_SLINGSHOT_POUCH_HALF_WIDTH, 0.0, 0.0));

    Ok(Elastic {
        anchors: [anchor(-1.0), anchor(1.0)],
        pouch_rest,
        pouch_half: pouch_end - pouch_rest,
    })
}

fn add_level_one_lighting(scene: &mut Scene) {
    let lens_center = Vec3::new(
        (LEVEL_ONE_LEFT_CENTER.x + LEVEL_ONE_LEFT_ATMOSPHERE_RADIUS + LEVEL_ONE_RIGHT_CENTER.x
            - LEVEL_ONE_RIGHT_ATMOSPHERE_RADIUS)
            * 0.5,
        LEVEL_ONE_LEFT_CENTER.y,
        0.0,
    );
    let warm = Color::new(1.0, 0.95, 0.84);
    let cool = Color::new(0.80, 0.90, 1.0);

    for (position, color, intensity) in [
        // Outside: key light for the slingshot rock and the atmospheres, and a
        // soft fill near the camera for birds flying between the fields.
        (Vec3::new(-4.8, 2.6, 4.2), warm, 26.0),
        (Vec3::new(1.2, 3.8, 6.5), cool, 18.0),
        // Inside each atmosphere, in front of its asteroid.
        (
            LEVEL_ONE_LEFT_CENTER + Vec3::new(-0.45, 0.70, 1.30),
            warm,
            5.5,
        ),
        (
            LEVEL_ONE_RIGHT_CENTER + Vec3::new(-0.35, 0.80, 1.45),
            warm,
            6.5,
        ),
        (
            LEVEL_ONE_RIGHT_CENTER + Vec3::new(0.95, -0.35, 1.20),
            cool,
            2.5,
        ),
        // The lens where both atmospheres overlap.
        (lens_center + Vec3::new(0.0, 0.30, 0.65), warm, 1.6),
    ] {
        scene.add_light(PointLight::new(position, color, intensity));
    }
}

/// Replaces the dynamic objects of `scene` with the current state of the
/// game.
fn write_dynamic_objects(
    scene: &mut Scene,
    game: &Game,
    materials: LevelOneMaterials,
    elastic: Elastic,
) -> Result<(), SpaceBuildError> {
    scene.clear_dynamic_objects();

    add_elastic(scene, game, materials, elastic)?;

    for view in game.bird_views() {
        // Birds look a little towards the camera so their faces show.
        let forward = (view.forward + Vec3::new(0.0, 0.0, 0.55)).normalized();
        add_songbird(
            scene,
            Songbird {
                center: view.center,
                radius: game.layout().bird_radius,
                forward,
                up: view.up,
                palette: view.index,
            },
            materials.birds[view.index % LEVEL_ONE_BIRD_COUNT],
        )?;
    }

    let pig = game.pig();
    if pig.alive {
        // The pig faces the camera, turned a little towards the slingshot.
        let forward = Vec3::new(-0.35, 0.0, 0.94).normalized();
        let up = Vec3::new(0.0, 1.0, 0.0);
        add_space_pig(
            scene,
            pig.position,
            game.layout().pig_radius,
            Basis3::new(up.cross(forward), up, forward)?,
            materials.space,
        )?;
    }

    for block in game.blocks() {
        if block.state != BlockState::Destroyed {
            add_block(scene, *block, materials.space.wood)?;
        }
    }

    for puff in game.puffs() {
        let material = match puff.kind {
            PuffKind::Wood => materials.wood_smoke,
            PuffKind::Bird | PuffKind::Pig => materials.smoke,
        };
        add_puff(scene, puff.center, puff.size, puff.progress(), material)?;
    }

    scene.build_dynamic_bvh();
    Ok(())
}

/// Elastic from both prongs to the pouch, which follows the loaded bird.
fn add_elastic(
    scene: &mut Scene,
    game: &Game,
    materials: LevelOneMaterials,
    elastic: Elastic,
) -> Result<(), SpaceBuildError> {
    let pouch = elastic.pouch_rest + (game.pouch_position() - game.layout().slingshot_rest);
    let band = materials.space.slingshot_band;
    let ends = [pouch - elastic.pouch_half, pouch + elastic.pouch_half];

    for (anchor, end) in elastic.anchors.into_iter().zip(ends) {
        add_segment(
            scene,
            anchor,
            end,
            BLUE_MOON_SLINGSHOT_ELASTIC_RADIUS * SLINGSHOT_SCALE,
            band,
        )?;
    }
    add_segment(
        scene,
        ends[0],
        ends[1],
        BLUE_MOON_SLINGSHOT_POUCH_RADIUS * SLINGSHOT_SCALE,
        band,
    )
}

pub(super) fn add_segment(
    scene: &mut Scene,
    start: Vec3,
    end: Vec3,
    radius: f32,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    let axis = end - start;
    if axis.length() <= 1.0e-4 {
        return Ok(());
    }

    scene.add_cylinder(Cylinder::new(
        (start + end) * 0.5,
        radius,
        axis.length() * 0.5,
        basis_with_up(axis)?,
        material_id,
    )?)?;
    Ok(())
}

/// Wood block: the frame is a square of four bars, the plank a single box.
fn add_block(
    scene: &mut Scene,
    block: crate::game::Block,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    let (sin, cos) = block.angle.sin_cos();
    let right = Vec3::new(cos, sin, 0.0);
    let up = Vec3::new(-sin, cos, 0.0);
    let basis = Basis3::new(right, up, Vec3::new(0.0, 0.0, 1.0))?;
    let (half_width, half_height) = (block.layout.half_width, block.layout.half_height);

    match block.layout.kind {
        BlockKind::Plank => {
            scene.add_oriented_box(OrientedBox::new(
                block.center,
                Vec3::new(half_width, half_height, PLANK_HALF_DEPTH),
                basis,
                material_id,
            )?)?;
        }
        BlockKind::Frame => {
            let bar = FRAME_BAR * 0.5;
            for (offset, half_extents) in [
                (
                    Vec3::new(0.0, half_height - bar, 0.0),
                    Vec3::new(half_width, bar, BLOCK_HALF_DEPTH),
                ),
                (
                    Vec3::new(0.0, -half_height + bar, 0.0),
                    Vec3::new(half_width, bar, BLOCK_HALF_DEPTH),
                ),
                (
                    Vec3::new(half_width - bar, 0.0, 0.0),
                    Vec3::new(bar, half_height - FRAME_BAR, BLOCK_HALF_DEPTH),
                ),
                (
                    Vec3::new(-half_width + bar, 0.0, 0.0),
                    Vec3::new(bar, half_height - FRAME_BAR, BLOCK_HALF_DEPTH),
                ),
            ] {
                scene.add_oriented_box(OrientedBox::new(
                    block.center + basis.local_to_world_vector(offset),
                    half_extents,
                    basis,
                    material_id,
                )?)?;
            }
        }
    }

    Ok(())
}

/// Puff of smoke: a few balls that grow and then shrink away.
fn add_puff(
    scene: &mut Scene,
    center: Vec3,
    size: f32,
    progress: f32,
    material_id: usize,
) -> Result<(), SpaceBuildError> {
    let swell = (progress * std::f32::consts::PI).sin().max(0.0);
    let radius = size * 0.45 * swell;
    if radius <= 1.0e-3 {
        return Ok(());
    }

    let spread = size * (0.35 + 0.45 * progress);
    scene.add_sphere(Sphere::new(center, radius * 1.1, material_id)?)?;
    for index in 0..5 {
        let angle = index as f32 * std::f32::consts::TAU / 5.0 + 0.4;
        let offset = Vec3::new(angle.cos(), angle.sin(), 0.35 * (index as f32 - 2.0) / 2.0);
        scene.add_sphere(Sphere::new(
            center + offset * spread,
            radius * (0.75 + 0.1 * (index % 2) as f32),
            material_id,
        )?)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        LEVEL_ONE_BIRD_COUNT, LEVEL_ONE_LEFT_ATMOSPHERE_RADIUS, LEVEL_ONE_LEFT_CENTER,
        LEVEL_ONE_RIGHT_ATMOSPHERE_RADIUS, LEVEL_ONE_RIGHT_CENTER, LEVEL_ONE_ROCK_CENTER,
        build_level_one_with_parts, level_one_layout, level_one_orbit_camera,
    };
    use crate::{
        game::{BirdPhase, Game, Outcome, launch_velocity},
        math::Vec3,
        primitive::Primitive,
        ray::Ray,
    };

    #[test]
    fn level_one_builds_voxel_asteroids_and_two_atmospheres() {
        let (scene, level, parts) = build_level_one_with_parts().unwrap();

        assert!(parts.left_asteroid.cube_count > 200);
        assert!(parts.right_asteroid.cube_count > 200);
        assert!(parts.rock.cube_count > 50);
        assert!(scene.cube_count() >= parts.left_asteroid.cube_count * 2);
        assert!(scene.skybox().is_some());
        assert!(scene.lights().len() >= 5);
        assert!(parts.slingshot_parts > 5);

        for (id, center, radius) in [
            (
                parts.atmosphere_ids[0],
                LEVEL_ONE_LEFT_CENTER,
                LEVEL_ONE_LEFT_ATMOSPHERE_RADIUS,
            ),
            (
                parts.atmosphere_ids[1],
                LEVEL_ONE_RIGHT_CENTER,
                LEVEL_ONE_RIGHT_ATMOSPHERE_RADIUS,
            ),
        ] {
            let Primitive::Sphere(sphere) = scene.objects()[id] else {
                panic!("atmosphere {id} is not a sphere");
            };
            assert!(sphere.center().approx_eq(center));
            assert!((sphere.radius() - radius).abs() < 1.0e-5);
            let material = scene.material(sphere.material_id()).unwrap();
            assert!(material.transparency > 0.5);
        }

        // Both atmospheres overlap, like in the reference.
        let gap = (LEVEL_ONE_RIGHT_CENTER - LEVEL_ONE_LEFT_CENTER).length();
        assert!(gap < LEVEL_ONE_LEFT_ATMOSPHERE_RADIUS + LEVEL_ONE_RIGHT_ATMOSPHERE_RADIUS);
        assert!(level.game().layout().planets.len() == 2);
    }

    #[test]
    fn level_one_starts_with_three_birds_a_pig_and_the_wood_frame() {
        let (scene, level, _) = build_level_one_with_parts().unwrap();
        let game = level.game();

        assert_eq!(game.birds_left(), LEVEL_ONE_BIRD_COUNT);
        assert_eq!(game.bird_views().len(), LEVEL_ONE_BIRD_COUNT);
        assert!(game.pig().alive);
        assert_eq!(game.blocks().len(), 2);
        assert!(!scene.dynamic_objects().is_empty());
    }

    #[test]
    fn waiting_birds_stand_on_the_rock_left_of_the_slingshot() {
        let layout = level_one_layout().unwrap();

        for spot in &layout.waiting_spots {
            assert!(spot.position.x < LEVEL_ONE_ROCK_CENTER.x);
            let distance = (spot.position - LEVEL_ONE_ROCK_CENTER).length();
            assert!(distance > layout.rocks[0].radius);
            assert!(distance < layout.rocks[0].radius + layout.bird_radius * 2.0);
        }
        assert!(layout.slingshot_rest.y > LEVEL_ONE_ROCK_CENTER.y + layout.rocks[0].radius);
    }

    #[test]
    fn slingshot_is_outside_both_atmospheres() {
        let layout = level_one_layout().unwrap();

        for planet in &layout.planets {
            assert!(!planet.contains(layout.slingshot_rest));
        }
    }

    #[test]
    fn pig_hangs_from_the_plank_inside_the_right_atmosphere() {
        let layout = level_one_layout().unwrap();

        assert!(layout.planets[1].contains(layout.pig_center));
        assert!(!layout.planets[0].contains(layout.pig_center));
        assert!(layout.pig_center.x > LEVEL_ONE_RIGHT_CENTER.x + layout.planets[1].body_radius);
        assert_eq!(layout.pig_support, Some(1));
        assert_eq!(layout.blocks[1].support, Some(0));
    }

    #[test]
    fn dynamic_objects_follow_the_game() {
        let (mut scene, mut level, _) = build_level_one_with_parts().unwrap();
        let static_count = scene.static_object_count();
        let rest = level.game().layout().slingshot_rest;

        assert!(!level.sync_scene(&mut scene).unwrap());
        assert!(level.game_mut().try_grab(rest));
        level.game_mut().aim_at(rest + Vec3::new(-0.5, -0.1, 0.0));
        assert!(level.sync_scene(&mut scene).unwrap());
        assert_eq!(scene.static_object_count(), static_count);

        // The loaded bird is drawn where it is pulled.
        let bird = level.game().loaded_bird().unwrap();
        let ray = Ray::new(
            bird.position + Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, -1.0),
        );
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();
        assert!(hit.position.z > 0.0 && hit.position.z < 0.2);
    }

    #[test]
    fn a_popped_pig_leaves_the_scene() {
        let (mut scene, mut level, _) = build_level_one_with_parts().unwrap();
        let pig = level.game().pig().position;
        let objects_with_pig = scene.dynamic_objects().len();
        let mut layout = level.game().layout().clone();
        layout.pig_support = None;
        layout.pig_center = pig;
        *level.game_mut() = Game::new(layout);
        // The loose pig falls into the right asteroid and pops.
        for _ in 0..240 {
            level.game_mut().update(1.0 / 60.0);
        }
        level.sync_scene(&mut scene).unwrap();

        assert!(!level.game().pig().alive);
        assert_eq!(level.game().outcome(), Outcome::Won);
        assert!(scene.dynamic_objects().len() < objects_with_pig);
    }

    /// Some launch with the first bird defeats the pig: the level can be won.
    #[test]
    fn level_one_can_be_won_with_one_bird() {
        let layout = level_one_layout().unwrap();
        let mut winning_launches = 0;

        for angle_step in 0..36 {
            let angle = (-50.0 + angle_step as f32 * 2.8_f32).to_radians();
            for pull_step in 0..6 {
                let pull_length = 0.30 + pull_step as f32 * 0.09;
                let pull = Vec3::new(-angle.cos(), -angle.sin(), 0.0) * pull_length;
                let mut game = Game::new(layout.clone());
                let rest = layout.slingshot_rest;
                assert!(game.try_grab(rest));
                game.aim_at(rest + pull);
                assert!(game.release());
                assert!(launch_velocity(pull).x > 0.0);

                for _ in 0..(14 * 60) {
                    game.update(1.0 / 60.0);
                    if game.birds()[0].phase == BirdPhase::Spent || !game.pig().alive {
                        break;
                    }
                }
                for _ in 0..(4 * 60) {
                    game.update(1.0 / 60.0);
                }
                if game.outcome() == Outcome::Won {
                    winning_launches += 1;
                }
            }
        }

        assert!(winning_launches >= 2, "{winning_launches} winning launches");
    }

    #[test]
    fn camera_looks_at_the_level_from_the_front() {
        let camera = level_one_orbit_camera(16.0 / 9.0).to_camera();

        assert!(camera.position.z > 5.0);
        assert!((camera.position.x - camera.target.x).abs() < 1.0e-4);
    }
}
