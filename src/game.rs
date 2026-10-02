//! Playable level one: birds are launched from a slingshot and fly through
//! the gravity fields of the asteroids, like in Angry Birds Space.
//!
//! The game is 2D and lives in the XY plane of the level (Z = 0): Y up and +X
//! to the right, like the default level camera. It does not know how the
//! level is drawn; `space::level_one` builds the scene from the same layout
//! and turns the state of the game into dynamic scene objects.
//!
//! Rules of the gravity fields:
//! - Outside every atmosphere there is no gravity and birds fly straight.
//! - Inside an atmosphere, gravity pulls towards the center of its asteroid.
//!   Where two atmospheres overlap, both pulls add up.
//! - Asteroids are solid: birds, loose blocks and the pig bounce and roll on
//!   them.

use crate::math::Vec3;

/// Fixed physics step. Frames advance the game in steps of this size, so the
/// flight does not depend on how long a render takes.
pub const GAME_STEP_SECONDS: f32 = 1.0 / 240.0;
/// Longest frame the game catches up with; slower frames run in slow motion
/// instead of skipping through collisions.
const MAX_FRAME_SECONDS: f32 = 0.1;

/// How far (in world units) the bird can be pulled back from the pouch.
pub const MAX_PULL: f32 = 0.75;
/// Shorter pulls are cancelled instead of launched.
pub const MIN_PULL: f32 = 0.08;
/// Launch speed for each unit of pull.
pub const LAUNCH_SPEED_PER_PULL: f32 = 6.5;
/// A click this close to the loaded bird grabs it.
pub const GRAB_RADIUS: f32 = 0.45;

/// Time between trajectory preview dots, and how much flight they cover.
const PREVIEW_DOT_SECONDS: f32 = 0.05;
const PREVIEW_SECONDS: f32 = 1.2;

const BIRD_RESTITUTION: f32 = 0.35;
/// Tangential speed lost per second while rolling on an asteroid.
const GROUND_FRICTION: f32 = 2.4;
/// A bird slower than this on the ground counts as resting.
const REST_SPEED: f32 = 0.25;
/// Resting birds disappear after this long.
const REST_SECONDS: f32 = 0.8;
/// Birds stuck in an orbit disappear after this long.
const MAX_FLIGHT_SECONDS: f32 = 12.0;
/// Pause before the next bird jumps into the slingshot.
const RELOAD_SECONDS: f32 = 0.6;
/// Pause before a level without birds counts as lost.
const LOSE_DELAY_SECONDS: f32 = 1.0;

/// Impact speed that breaks a wood block.
pub const BLOCK_BREAK_SPEED: f32 = 2.2;
/// Softer impacts over this speed pull an attached block loose.
pub const BLOCK_PUSH_SPEED: f32 = 0.6;
/// Fraction of the bird speed kept after breaking through a block.
const BIRD_SPEED_AFTER_BREAK: f32 = 0.7;
/// Share of the impact passed to a loose block.
const BLOCK_PUSH_SHARE: f32 = 0.55;
const BLOCK_RESTITUTION: f32 = 0.2;
const BLOCK_SPIN_DAMPING: f32 = 2.5;
/// Speed under which loose blocks and a fallen pig stop and sleep.
const SLEEP_SPEED: f32 = 0.08;

/// A hit this fast pops the pig.
pub const PIG_POP_SPEED: f32 = 0.6;
const PIG_RESTITUTION: f32 = 0.25;

pub const PIG_SCORE: u32 = 5000;
pub const BLOCK_SCORE: u32 = 500;
pub const UNUSED_BIRD_BONUS: u32 = 10000;

/// Lifetime of a puff of smoke.
pub const PUFF_SECONDS: f32 = 0.55;

/// An asteroid with a gravity field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GravityPlanet {
    pub center: Vec3,
    /// Radius of the solid asteroid.
    pub body_radius: f32,
    pub atmosphere_radius: f32,
    /// Pull inside the atmosphere, in units per second squared.
    pub gravity: f32,
}

impl GravityPlanet {
    pub fn contains(self, point: Vec3) -> bool {
        (point - self.center).length_squared() < self.atmosphere_radius * self.atmosphere_radius
    }
}

/// A solid rock without atmosphere, like the one under the slingshot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidRock {
    pub center: Vec3,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    /// Square wood frame planted in an asteroid.
    Frame,
    /// Wood plank that holds the pig.
    Plank,
}

/// A wood block: a rectangle in the level plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockLayout {
    pub kind: BlockKind,
    pub center: Vec3,
    /// Half width and half height of the rectangle.
    pub half_width: f32,
    pub half_height: f32,
    /// Rotation around +Z in radians.
    pub angle: f32,
    /// Block this one hangs from; `None` means planted in an asteroid.
    pub support: Option<usize>,
}

/// Where a waiting bird stands: position and the up direction of the ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BirdSpot {
    pub position: Vec3,
    pub up: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelLayout {
    pub planets: Vec<GravityPlanet>,
    pub rocks: Vec<SolidRock>,
    /// Center of the bird sitting in the slingshot pouch.
    pub slingshot_rest: Vec3,
    /// Places where the birds wait their turn, in order.
    pub waiting_spots: Vec<BirdSpot>,
    pub bird_count: usize,
    pub bird_radius: f32,
    pub pig_center: Vec3,
    pub pig_radius: f32,
    /// Block that holds the pig.
    pub pig_support: Option<usize>,
    pub blocks: Vec<BlockLayout>,
    /// Birds, blocks and pigs outside this box are gone.
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BirdPhase {
    Waiting,
    Loaded,
    Flying,
    Spent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bird {
    pub phase: BirdPhase,
    pub position: Vec3,
    pub velocity: Vec3,
    /// Direction the bird looks at while flying.
    pub heading: Vec3,
    flight_time: f32,
    rest_time: f32,
}

/// How a bird is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BirdPose {
    Standing,
    Loaded,
    Flying,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BirdView {
    /// Index of the bird, which also picks its colors.
    pub index: usize,
    pub center: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub pose: BirdPose,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pig {
    pub position: Vec3,
    pub velocity: Vec3,
    pub alive: bool,
    /// Still hanging from its block.
    pub attached: bool,
    asleep: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockState {
    Attached,
    Loose,
    Destroyed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    pub layout: BlockLayout,
    pub center: Vec3,
    pub angle: f32,
    pub velocity: Vec3,
    pub spin: f32,
    pub state: BlockState,
    asleep: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuffKind {
    Bird,
    Wood,
    Pig,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Puff {
    pub center: Vec3,
    pub age: f32,
    pub size: f32,
    pub kind: PuffKind,
}

impl Puff {
    /// 0 when it appears, 1 when it is gone.
    pub fn progress(self) -> f32 {
        (self.age / PUFF_SECONDS).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Playing,
    Won,
    Lost,
}

#[derive(Debug, Clone)]
pub struct Game {
    layout: LevelLayout,
    birds: Vec<Bird>,
    /// Bird in the slingshot or in the air.
    current: Option<usize>,
    /// Pull from the pouch while the player drags the loaded bird.
    aim_pull: Option<Vec3>,
    pig: Pig,
    blocks: Vec<Block>,
    puffs: Vec<Puff>,
    score: u32,
    reload_timer: f32,
    lose_timer: f32,
    outcome: Outcome,
    accumulator: f32,
    /// Grows every time something visible changes.
    revision: u64,
}

impl Game {
    pub fn new(layout: LevelLayout) -> Self {
        let bird_count = layout.bird_count;
        let blocks = layout
            .blocks
            .iter()
            .map(|&block| Block {
                layout: block,
                center: block.center,
                angle: block.angle,
                velocity: Vec3::ZERO,
                spin: 0.0,
                state: BlockState::Attached,
                asleep: false,
            })
            .collect();
        let pig = Pig {
            position: layout.pig_center,
            velocity: Vec3::ZERO,
            alive: true,
            attached: layout.pig_support.is_some(),
            asleep: false,
        };
        let mut game = Self {
            birds: vec![
                Bird {
                    phase: BirdPhase::Waiting,
                    position: layout.slingshot_rest,
                    velocity: Vec3::ZERO,
                    heading: Vec3::new(1.0, 0.0, 0.0),
                    flight_time: 0.0,
                    rest_time: 0.0,
                };
                bird_count
            ],
            layout,
            current: None,
            aim_pull: None,
            pig,
            blocks,
            puffs: Vec::new(),
            score: 0,
            reload_timer: 0.0,
            lose_timer: 0.0,
            outcome: Outcome::Playing,
            accumulator: 0.0,
            revision: 0,
        };
        game.load_next_bird();

        game
    }

    /// Back to the start of the level.
    pub fn restart(&mut self) {
        let revision = self.revision;
        *self = Self::new(self.layout.clone());
        self.revision = revision + 1;
    }

    pub fn layout(&self) -> &LevelLayout {
        &self.layout
    }

    pub fn birds(&self) -> &[Bird] {
        &self.birds
    }

    pub fn pig(&self) -> Pig {
        self.pig
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn puffs(&self) -> &[Puff] {
        &self.puffs
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn outcome(&self) -> Outcome {
        self.outcome
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Birds that were not launched yet, including the loaded one.
    pub fn birds_left(&self) -> usize {
        self.birds
            .iter()
            .filter(|bird| matches!(bird.phase, BirdPhase::Waiting | BirdPhase::Loaded))
            .count()
    }

    pub fn loaded_bird(&self) -> Option<Bird> {
        self.current
            .map(|index| self.birds[index])
            .filter(|bird| bird.phase == BirdPhase::Loaded)
    }

    pub fn is_aiming(&self) -> bool {
        self.aim_pull.is_some()
    }

    pub fn aim_pull(&self) -> Option<Vec3> {
        self.aim_pull
    }

    /// Where the pouch of the slingshot is: behind the loaded bird, or at
    /// rest when the slingshot is empty.
    pub fn pouch_position(&self) -> Vec3 {
        self.layout.slingshot_rest + self.aim_pull.unwrap_or(Vec3::ZERO)
    }

    /// Something is still moving, so the game keeps needing new frames.
    pub fn is_animating(&self) -> bool {
        let flying = self
            .birds
            .iter()
            .any(|bird| bird.phase == BirdPhase::Flying);
        let loose_blocks = self
            .blocks
            .iter()
            .any(|block| block.state == BlockState::Loose && !block.asleep);
        let falling_pig = self.pig.alive && !self.pig.attached && !self.pig.asleep;
        let reloading = self.outcome == Outcome::Playing
            && self.loaded_bird().is_none()
            && self
                .birds
                .iter()
                .any(|bird| bird.phase == BirdPhase::Waiting);
        let deciding = self.outcome == Outcome::Playing && self.birds_left() == 0;

        flying || loose_blocks || falling_pig || reloading || deciding || !self.puffs.is_empty()
    }

    /// Starts aiming when `point` (in the level plane) is on the loaded bird.
    pub fn try_grab(&mut self, point: Vec3) -> bool {
        if self.outcome != Outcome::Playing || self.loaded_bird().is_none() {
            return false;
        }
        if planar(point - self.layout.slingshot_rest).length() > GRAB_RADIUS {
            return false;
        }

        self.aim_pull = Some(Vec3::ZERO);
        self.aim_at(point);
        true
    }

    /// Pulls the loaded bird towards `point`, up to `MAX_PULL`.
    pub fn aim_at(&mut self, point: Vec3) {
        let Some(index) = self.current else {
            return;
        };
        if self.aim_pull.is_none() || !is_finite(point) {
            return;
        }

        let rest = self.layout.slingshot_rest;
        let mut pull = clamp_length(planar(point - rest), MAX_PULL);
        // The bird cannot be pulled into the rock under the slingshot.
        if let Some((normal, depth)) = self.solid_contact(rest + pull, self.layout.bird_radius) {
            pull = clamp_length(pull + normal * depth, MAX_PULL);
        }
        self.aim_pull = Some(pull);
        self.birds[index].position = self.layout.slingshot_rest + pull;
        self.revision += 1;
    }

    /// Launches the bird being aimed. Too short pulls put it back.
    pub fn release(&mut self) -> bool {
        let Some(pull) = self.aim_pull.take() else {
            return false;
        };
        let Some(index) = self.current else {
            return false;
        };
        self.revision += 1;

        if pull.length() < MIN_PULL {
            self.birds[index].position = self.layout.slingshot_rest;
            return false;
        }

        let velocity = launch_velocity(pull);
        let bird = &mut self.birds[index];
        bird.phase = BirdPhase::Flying;
        bird.position = self.layout.slingshot_rest + pull;
        bird.velocity = velocity;
        bird.heading = velocity.normalized();
        bird.flight_time = 0.0;
        bird.rest_time = 0.0;
        true
    }

    pub fn cancel_aim(&mut self) {
        if self.aim_pull.take().is_some()
            && let Some(index) = self.current
        {
            self.birds[index].position = self.layout.slingshot_rest;
            self.revision += 1;
        }
    }

    /// Where a bird launched with the current pull would go, as dots spaced
    /// in time. It stops at the first asteroid it would touch.
    pub fn trajectory_preview(&self) -> Vec<Vec3> {
        match self.aim_pull {
            Some(pull) if pull.length() >= MIN_PULL => self.predict_path(pull),
            _ => Vec::new(),
        }
    }

    fn predict_path(&self, pull: Vec3) -> Vec<Vec3> {
        let mut position = self.layout.slingshot_rest + pull;
        let mut velocity = launch_velocity(pull);
        let steps = (PREVIEW_SECONDS / GAME_STEP_SECONDS).round() as usize;
        let dot_every = (PREVIEW_DOT_SECONDS / GAME_STEP_SECONDS).round().max(1.0) as usize;
        let mut dots = Vec::with_capacity(steps / dot_every + 1);

        for step in 1..=steps {
            velocity += self.gravity_at(position) * GAME_STEP_SECONDS;
            position += velocity * GAME_STEP_SECONDS;

            if self
                .solid_contact(position, self.layout.bird_radius)
                .is_some()
                || !self.in_bounds(position)
            {
                break;
            }
            if step % dot_every == 0 {
                dots.push(position);
            }
        }

        dots
    }

    /// Sum of the pulls of every atmosphere that contains `point`.
    pub fn gravity_at(&self, point: Vec3) -> Vec3 {
        self.layout
            .planets
            .iter()
            .filter(|planet| planet.contains(point))
            .fold(Vec3::ZERO, |pull, planet| {
                let offset = planar(planet.center - point);
                let distance = offset.length();

                if distance > 1.0e-4 {
                    pull + offset * (planet.gravity / distance)
                } else {
                    pull
                }
            })
    }

    /// Advances the game by a frame of `delta_seconds`. Returns whether
    /// anything visible changed.
    pub fn update(&mut self, delta_seconds: f32) -> bool {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return false;
        }

        let revision = self.revision;
        self.accumulator += delta_seconds.min(MAX_FRAME_SECONDS);

        while self.accumulator >= GAME_STEP_SECONDS {
            self.accumulator -= GAME_STEP_SECONDS;
            self.step(GAME_STEP_SECONDS);
        }

        self.revision != revision
    }

    /// Where every visible bird is and how it is drawn.
    pub fn bird_views(&self) -> Vec<BirdView> {
        let mut views = Vec::with_capacity(self.birds.len());
        let mut spots = self.layout.waiting_spots.iter();

        for (index, bird) in self.birds.iter().enumerate() {
            match bird.phase {
                BirdPhase::Waiting => {
                    if let Some(spot) = spots.next() {
                        views.push(BirdView {
                            index,
                            center: spot.position,
                            forward: Vec3::new(1.0, 0.0, 0.0),
                            up: spot.up,
                            pose: BirdPose::Standing,
                        });
                    }
                }
                BirdPhase::Loaded => {
                    let forward = match self.aim_pull {
                        Some(pull) if pull.length() >= MIN_PULL => -pull.normalized(),
                        _ => Vec3::new(1.0, 0.0, 0.0),
                    };
                    views.push(BirdView {
                        index,
                        center: bird.position,
                        forward,
                        up: perpendicular(forward),
                        pose: BirdPose::Loaded,
                    });
                }
                BirdPhase::Flying => views.push(BirdView {
                    index,
                    center: bird.position,
                    forward: bird.heading,
                    up: perpendicular(bird.heading),
                    pose: BirdPose::Flying,
                }),
                BirdPhase::Spent => {}
            }
        }

        views
    }

    fn step(&mut self, dt: f32) {
        let mut changed = false;

        if let Some(index) = self.current
            && self.birds[index].phase == BirdPhase::Flying
        {
            self.step_bird(index, dt);
            changed = true;
        }

        changed |= self.step_blocks(dt);
        changed |= self.release_unsupported();
        changed |= self.step_pig(dt);
        changed |= self.step_puffs(dt);
        changed |= self.step_turns(dt);

        if changed {
            self.revision += 1;
        }
    }

    fn step_bird(&mut self, index: usize, dt: f32) {
        let radius = self.layout.bird_radius;
        let mut bird = self.birds[index];

        bird.velocity += self.gravity_at(bird.position) * dt;
        bird.position += bird.velocity * dt;
        bird.flight_time += dt;

        let mut touching = false;
        if let Some((normal, depth)) = self.solid_contact(bird.position, radius) {
            touching = true;
            bird.position += normal * depth;
            bird.velocity = bounce(bird.velocity, normal, BIRD_RESTITUTION, dt);
        }

        for block_index in 0..self.blocks.len() {
            let block = self.blocks[block_index];
            if block.state == BlockState::Destroyed {
                continue;
            }
            let Some((normal, depth)) = circle_block_contact(bird.position, radius, block) else {
                continue;
            };
            let impact = -(bird.velocity - block.velocity).dot(normal);
            if impact <= 0.0 {
                continue;
            }

            if impact >= BLOCK_BREAK_SPEED {
                self.destroy_block(block_index);
                bird.velocity *= BIRD_SPEED_AFTER_BREAK;
            } else {
                bird.position += normal * depth;
                bird.velocity += normal * ((1.0 + BIRD_RESTITUTION) * impact);
                touching = true;
                if impact >= BLOCK_PUSH_SPEED || block.state == BlockState::Loose {
                    self.push_block(block_index, -normal * (impact * BLOCK_PUSH_SHARE));
                }
            }
        }

        if self.pig.alive {
            let offset = planar(bird.position - self.pig.position);
            let reach = radius + self.layout.pig_radius;
            if offset.length_squared() < reach * reach {
                let normal = offset.normalized();
                let impact = (bird.velocity - self.pig.velocity).length();
                if impact >= PIG_POP_SPEED {
                    self.pop_pig();
                }
                if bird.velocity.dot(normal) < 0.0 {
                    bird.velocity = bounce(bird.velocity, normal, BIRD_RESTITUTION, 0.0);
                }
                bird.position = self.pig.position + normal * reach;
            }
        }

        let speed = bird.velocity.length();
        if speed > 0.2 {
            bird.heading = bird.velocity / speed;
        }
        if touching && speed < REST_SPEED {
            bird.rest_time += dt;
        } else {
            bird.rest_time = 0.0;
        }

        let gone = !self.in_bounds(bird.position);
        if gone || bird.rest_time > REST_SECONDS || bird.flight_time > MAX_FLIGHT_SECONDS {
            bird.phase = BirdPhase::Spent;
            if !gone {
                self.add_puff(bird.position, radius * 1.3, PuffKind::Bird);
            }
            self.reload_timer = RELOAD_SECONDS;
        }

        self.birds[index] = bird;
    }

    fn step_blocks(&mut self, dt: f32) -> bool {
        let mut changed = false;

        for index in 0..self.blocks.len() {
            let mut block = self.blocks[index];
            if block.state != BlockState::Loose || block.asleep {
                continue;
            }
            changed = true;

            let gravity = self.gravity_at(block.center);
            block.velocity += gravity * dt;
            block.center += block.velocity * dt;
            block.angle += block.spin * dt;
            if gravity == Vec3::ZERO && block.velocity.length() < SLEEP_SPEED {
                block.velocity = Vec3::ZERO;
                block.spin = 0.0;
                block.asleep = true;
            }

            let radius = block.layout.half_width.min(block.layout.half_height);
            if let Some((normal, depth)) = self.solid_contact(block.center, radius) {
                block.center += normal * depth;
                block.velocity = bounce(block.velocity, normal, BLOCK_RESTITUTION, dt);
                block.spin *= (-BLOCK_SPIN_DAMPING * dt).exp();
                if block.velocity.length() < SLEEP_SPEED {
                    block.velocity = Vec3::ZERO;
                    block.spin = 0.0;
                    block.asleep = true;
                }
            }

            if self.pig.alive && !block.asleep {
                let reach = bounding_radius(block.layout) + self.layout.pig_radius;
                let offset = planar(self.pig.position - block.center);
                let impact = (block.velocity - self.pig.velocity).length();
                if offset.length_squared() < reach * reach && impact >= PIG_POP_SPEED {
                    self.pop_pig();
                }
            }

            if !self.in_bounds(block.center) {
                block.state = BlockState::Destroyed;
            }
            self.blocks[index] = block;
        }

        changed
    }

    /// Blocks and the pig fall when what held them is gone.
    fn release_unsupported(&mut self) -> bool {
        let mut changed = false;

        for index in 0..self.blocks.len() {
            let block = self.blocks[index];
            if block.state == BlockState::Attached
                && let Some(support) = block.layout.support
                && self
                    .blocks
                    .get(support)
                    .is_none_or(|support| support.state != BlockState::Attached)
            {
                self.blocks[index].state = BlockState::Loose;
                changed = true;
            }
        }

        if self.pig.alive
            && self.pig.attached
            && self.layout.pig_support.is_none_or(|support| {
                self.blocks
                    .get(support)
                    .is_none_or(|block| block.state != BlockState::Attached)
            })
        {
            self.pig.attached = false;
            changed = true;
        }

        changed
    }

    fn step_pig(&mut self, dt: f32) -> bool {
        if !self.pig.alive || self.pig.attached || self.pig.asleep {
            return false;
        }

        let mut pig = self.pig;
        let gravity = self.gravity_at(pig.position);
        pig.velocity += gravity * dt;
        pig.position += pig.velocity * dt;
        if gravity == Vec3::ZERO && pig.velocity.length() < SLEEP_SPEED {
            pig.velocity = Vec3::ZERO;
            pig.asleep = true;
        }

        if let Some((normal, depth)) = self.solid_contact(pig.position, self.layout.pig_radius) {
            let impact = -pig.velocity.dot(normal);
            pig.position += normal * depth;
            if impact >= PIG_POP_SPEED {
                self.pig = pig;
                self.pop_pig();
                return true;
            }
            pig.velocity = bounce(pig.velocity, normal, PIG_RESTITUTION, dt);
            if pig.velocity.length() < SLEEP_SPEED {
                pig.velocity = Vec3::ZERO;
                pig.asleep = true;
            }
        }

        self.pig = pig;
        if !self.in_bounds(pig.position) {
            // A pig lost in space counts as defeated.
            self.pig.alive = false;
            self.score += PIG_SCORE;
        }

        true
    }

    fn step_puffs(&mut self, dt: f32) -> bool {
        if self.puffs.is_empty() {
            return false;
        }

        for puff in &mut self.puffs {
            puff.age += dt;
        }
        self.puffs.retain(|puff| puff.age < PUFF_SECONDS);
        true
    }

    /// Reloads the slingshot and decides when the level is over.
    fn step_turns(&mut self, dt: f32) -> bool {
        if self.outcome != Outcome::Playing {
            return false;
        }

        if !self.pig.alive {
            self.outcome = Outcome::Won;
            self.aim_pull = None;
            self.score += UNUSED_BIRD_BONUS * self.birds_left() as u32;
            return true;
        }

        let bird_in_play = self.current.is_some_and(|index| {
            matches!(
                self.birds[index].phase,
                BirdPhase::Loaded | BirdPhase::Flying
            )
        });
        if bird_in_play {
            return false;
        }

        if self
            .birds
            .iter()
            .any(|bird| bird.phase == BirdPhase::Waiting)
        {
            self.reload_timer -= dt;
            if self.reload_timer <= 0.0 {
                self.load_next_bird();
                return true;
            }
            return false;
        }

        let settled = self
            .blocks
            .iter()
            .all(|block| block.state != BlockState::Loose || block.asleep)
            && (self.pig.attached || self.pig.asleep);
        if settled {
            self.lose_timer += dt;
            if self.lose_timer >= LOSE_DELAY_SECONDS {
                self.outcome = Outcome::Lost;
                return true;
            }
        }

        false
    }

    fn load_next_bird(&mut self) {
        self.current = self
            .birds
            .iter()
            .position(|bird| bird.phase == BirdPhase::Waiting);

        if let Some(index) = self.current {
            let bird = &mut self.birds[index];
            bird.phase = BirdPhase::Loaded;
            bird.position = self.layout.slingshot_rest;
            bird.velocity = Vec3::ZERO;
            bird.heading = Vec3::new(1.0, 0.0, 0.0);
        }
        self.revision += 1;
    }

    fn destroy_block(&mut self, index: usize) {
        let block = self.blocks[index];
        self.blocks[index].state = BlockState::Destroyed;
        self.score += BLOCK_SCORE;
        self.add_puff(block.center, bounding_radius(block.layout), PuffKind::Wood);
    }

    fn push_block(&mut self, index: usize, impulse: Vec3) {
        let block = &mut self.blocks[index];
        if block.state == BlockState::Attached {
            block.state = BlockState::Loose;
        }
        block.velocity += impulse;
        // Blocks pushed on one side start to spin; the sign only depends on
        // the push, so the game stays deterministic.
        block.spin += if impulse.y >= 0.0 { 2.4 } else { -2.4 };
        block.asleep = false;
    }

    fn pop_pig(&mut self) {
        if !self.pig.alive {
            return;
        }
        self.pig.alive = false;
        self.score += PIG_SCORE;
        self.add_puff(
            self.pig.position,
            self.layout.pig_radius * 1.6,
            PuffKind::Pig,
        );
    }

    fn add_puff(&mut self, center: Vec3, size: f32, kind: PuffKind) {
        self.puffs.push(Puff {
            center,
            age: 0.0,
            size,
            kind,
        });
    }

    /// Deepest overlap of a circle with the asteroids and rocks, as the
    /// direction that pushes it out and how far.
    fn solid_contact(&self, center: Vec3, radius: f32) -> Option<(Vec3, f32)> {
        let planets = self
            .layout
            .planets
            .iter()
            .map(|planet| (planet.center, planet.body_radius));
        let rocks = self
            .layout
            .rocks
            .iter()
            .map(|rock| (rock.center, rock.radius));

        planets
            .chain(rocks)
            .filter_map(|(solid_center, solid_radius)| {
                let offset = planar(center - solid_center);
                let distance = offset.length();
                let depth = solid_radius + radius - distance;

                (depth > 0.0).then(|| {
                    let normal = if distance > 1.0e-5 {
                        offset / distance
                    } else {
                        Vec3::new(0.0, 1.0, 0.0)
                    };
                    (normal, depth)
                })
            })
            .max_by(|left, right| left.1.total_cmp(&right.1))
    }

    fn in_bounds(&self, point: Vec3) -> bool {
        let (min, max) = (self.layout.bounds_min, self.layout.bounds_max);

        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }
}

pub fn launch_velocity(pull: Vec3) -> Vec3 {
    -planar(clamp_length(pull, MAX_PULL)) * LAUNCH_SPEED_PER_PULL
}

/// Velocity after touching a surface with `normal`: the part into the surface
/// bounces back with `restitution` and the part along it slows down with
/// ground friction over `dt`.
fn bounce(velocity: Vec3, normal: Vec3, restitution: f32, dt: f32) -> Vec3 {
    let normal_speed = velocity.dot(normal);
    let tangent = velocity - normal * normal_speed;
    let normal_speed = if normal_speed < 0.0 {
        -normal_speed * restitution
    } else {
        normal_speed
    };

    normal * normal_speed + tangent * (-GROUND_FRICTION * dt).exp()
}

/// Overlap of a circle with a block rectangle: the direction that pushes the
/// circle out of the block and how far.
pub fn circle_block_contact(center: Vec3, radius: f32, block: Block) -> Option<(Vec3, f32)> {
    let (sin, cos) = block.angle.sin_cos();
    let offset = center - block.center;
    // Circle center in the block's frame.
    let local_x = offset.x * cos + offset.y * sin;
    let local_y = -offset.x * sin + offset.y * cos;
    let (half_width, half_height) = (block.layout.half_width, block.layout.half_height);
    let closest_x = local_x.clamp(-half_width, half_width);
    let closest_y = local_y.clamp(-half_height, half_height);
    let (dx, dy) = (local_x - closest_x, local_y - closest_y);
    let distance_squared = dx * dx + dy * dy;

    let (normal_x, normal_y, depth) = if distance_squared > 1.0e-10 {
        let distance = distance_squared.sqrt();
        if distance >= radius {
            return None;
        }
        (dx / distance, dy / distance, radius - distance)
    } else {
        // The center is inside the block: leave through the nearest side.
        let inside_x = half_width - local_x.abs();
        let inside_y = half_height - local_y.abs();
        if inside_x < inside_y {
            (local_x.signum(), 0.0, inside_x + radius)
        } else {
            (0.0, local_y.signum(), inside_y + radius)
        }
    };

    let normal = Vec3::new(
        normal_x * cos - normal_y * sin,
        normal_x * sin + normal_y * cos,
        0.0,
    );
    Some((normal, depth))
}

fn bounding_radius(block: BlockLayout) -> f32 {
    (block.half_width * block.half_width + block.half_height * block.half_height).sqrt()
}

fn planar(vector: Vec3) -> Vec3 {
    Vec3::new(vector.x, vector.y, 0.0)
}

fn clamp_length(vector: Vec3, max_length: f32) -> Vec3 {
    let length = vector.length();

    if length > max_length && length > 0.0 {
        vector * (max_length / length)
    } else {
        vector
    }
}

/// `forward` turned a quarter turn counterclockwise in the level plane.
fn perpendicular(forward: Vec3) -> Vec3 {
    Vec3::new(-forward.y, forward.x, 0.0)
}

fn is_finite(point: Vec3) -> bool {
    point.x.is_finite() && point.y.is_finite() && point.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{
        BLOCK_BREAK_SPEED, BirdPhase, BlockKind, BlockLayout, BlockState, Game, GravityPlanet,
        LevelLayout, MAX_PULL, MIN_PULL, Outcome, PIG_SCORE, SolidRock, UNUSED_BIRD_BONUS,
        circle_block_contact, launch_velocity,
    };
    use crate::math::Vec3;

    const FRAME: f32 = 1.0 / 60.0;

    fn test_layout() -> LevelLayout {
        LevelLayout {
            planets: vec![GravityPlanet {
                center: Vec3::new(3.0, 0.0, 0.0),
                body_radius: 0.5,
                atmosphere_radius: 2.0,
                gravity: 3.0,
            }],
            rocks: vec![SolidRock {
                center: Vec3::new(-3.0, -0.6, 0.0),
                radius: 0.3,
            }],
            slingshot_rest: Vec3::new(-3.0, 0.0, 0.0),
            waiting_spots: vec![super::BirdSpot {
                position: Vec3::new(-3.4, -0.5, 0.0),
                up: Vec3::new(-1.0, 0.0, 0.0),
            }],
            bird_count: 3,
            bird_radius: 0.12,
            pig_center: Vec3::new(3.0, 1.2, 0.0),
            pig_radius: 0.15,
            pig_support: Some(1),
            blocks: vec![
                BlockLayout {
                    kind: BlockKind::Frame,
                    center: Vec3::new(3.0, 0.6, 0.0),
                    half_width: 0.12,
                    half_height: 0.12,
                    angle: std::f32::consts::FRAC_PI_2,
                    support: None,
                },
                BlockLayout {
                    kind: BlockKind::Plank,
                    center: Vec3::new(3.0, 0.88, 0.0),
                    half_width: 0.16,
                    half_height: 0.04,
                    angle: std::f32::consts::FRAC_PI_2,
                    support: Some(0),
                },
            ],
            bounds_min: Vec3::new(-8.0, -6.0, 0.0),
            bounds_max: Vec3::new(9.0, 6.0, 0.0),
        }
    }

    fn run(game: &mut Game, seconds: f32) {
        let frames = (seconds / FRAME).ceil() as usize;
        for _ in 0..frames {
            game.update(FRAME);
        }
    }

    fn launch(game: &mut Game, pull: Vec3) -> bool {
        let rest = game.layout().slingshot_rest;
        assert!(game.try_grab(rest));
        game.aim_at(rest + pull);
        game.release()
    }

    #[test]
    fn game_starts_with_one_bird_loaded_and_the_rest_waiting() {
        let game = Game::new(test_layout());

        assert_eq!(game.birds_left(), 3);
        assert_eq!(game.outcome(), Outcome::Playing);
        assert!(game.loaded_bird().is_some());
        assert_eq!(
            game.birds()
                .iter()
                .filter(|bird| bird.phase == BirdPhase::Waiting)
                .count(),
            2
        );
        assert!(!game.is_animating());
    }

    #[test]
    fn no_gravity_outside_atmospheres_and_pull_to_center_inside() {
        let game = Game::new(test_layout());

        assert_eq!(game.gravity_at(Vec3::new(-2.0, 0.0, 0.0)), Vec3::ZERO);
        let pull = game.gravity_at(Vec3::new(4.5, 0.0, 0.0));
        assert!(pull.x < 0.0);
        assert!(pull.y.abs() < 1.0e-5);
        assert!((pull.length() - 3.0).abs() < 1.0e-4);
    }

    #[test]
    fn overlapping_atmospheres_add_their_pulls() {
        let mut layout = test_layout();
        layout.planets.push(GravityPlanet {
            center: Vec3::new(6.0, 0.0, 0.0),
            body_radius: 0.5,
            atmosphere_radius: 2.0,
            gravity: 3.0,
        });
        let game = Game::new(layout);

        // Halfway between both centers the pulls cancel out.
        assert!(game.gravity_at(Vec3::new(4.5, 0.0, 0.0)).length() < 1.0e-4);
        // Above the middle both pull down.
        let pull = game.gravity_at(Vec3::new(4.5, 1.0, 0.0));
        assert!(pull.y < -3.0);
    }

    #[test]
    fn grabbing_needs_a_click_near_the_loaded_bird() {
        let mut game = Game::new(test_layout());

        assert!(!game.try_grab(Vec3::new(0.0, 0.0, 0.0)));
        assert!(game.try_grab(Vec3::new(-3.1, 0.1, 0.0)));
        assert!(game.is_aiming());
    }

    #[test]
    fn pull_is_clamped_and_launch_goes_the_other_way() {
        let mut game = Game::new(test_layout());
        let rest = game.layout().slingshot_rest;
        game.try_grab(rest);
        game.aim_at(rest + Vec3::new(-3.0, 0.0, 0.0));

        let pull = game.aim_pull().unwrap();
        assert!((pull.length() - MAX_PULL).abs() < 1.0e-5);
        assert!(game.release());
        let bird = game.birds()[0];
        assert_eq!(bird.phase, BirdPhase::Flying);
        assert!(bird.velocity.x > 0.0);
        assert!(bird.velocity.approx_eq(launch_velocity(pull)));
    }

    #[test]
    fn short_pull_puts_the_bird_back() {
        let mut game = Game::new(test_layout());
        let rest = game.layout().slingshot_rest;
        game.try_grab(rest);
        game.aim_at(rest + Vec3::new(-MIN_PULL * 0.5, 0.0, 0.0));

        assert!(!game.release());
        assert_eq!(game.loaded_bird().unwrap().position, rest);
    }

    #[test]
    fn bird_flies_straight_outside_atmospheres() {
        let mut game = Game::new(test_layout());
        launch(&mut game, Vec3::new(-0.3, -0.3, 0.0));
        let start = game.birds()[0];
        game.update(0.1);
        let later = game.birds()[0];

        assert!(later.velocity.approx_eq(start.velocity));
    }

    #[test]
    fn bird_bends_towards_the_planet_inside_its_atmosphere() {
        let mut game = Game::new(test_layout());
        // Aimed above the planet: gravity bends it down.
        launch(&mut game, Vec3::new(-0.5, -0.12, 0.0));
        run(&mut game, 1.2);
        let bird = game.birds()[0];

        assert!(bird.phase == BirdPhase::Flying || bird.phase == BirdPhase::Spent);
        assert!(bird.velocity.y < 1.0);
    }

    #[test]
    fn birds_never_go_through_asteroids() {
        let mut game = Game::new(test_layout());
        launch(&mut game, Vec3::new(-0.6, 0.0, 0.0));
        let planet = game.layout().planets[0];

        for _ in 0..600 {
            game.update(FRAME);
            let bird = game.birds()[0];
            if bird.phase == BirdPhase::Flying {
                let distance = (bird.position - planet.center).length();
                assert!(distance >= planet.body_radius + game.layout().bird_radius - 0.02);
            }
        }
    }

    #[test]
    fn spent_bird_is_replaced_by_the_next_one() {
        let mut game = Game::new(test_layout());
        // Backwards and away: leaves the level.
        launch(&mut game, Vec3::new(0.7, 0.0, 0.0));
        run(&mut game, 3.0);

        assert_eq!(game.birds()[0].phase, BirdPhase::Spent);
        assert_eq!(game.birds_left(), 2);
        assert!(game.loaded_bird().is_some());
    }

    #[test]
    fn hitting_the_pig_wins_and_counts_unused_birds() {
        let mut layout = test_layout();
        layout.pig_support = None;
        layout.blocks.clear();
        layout.planets.clear();
        layout.pig_center = Vec3::new(0.0, 0.0, 0.0);
        let mut game = Game::new(layout);
        launch(&mut game, Vec3::new(-0.6, 0.0, 0.0));
        run(&mut game, 1.5);

        assert_eq!(game.outcome(), Outcome::Won);
        assert!(!game.pig().alive);
        assert_eq!(game.score(), PIG_SCORE + UNUSED_BIRD_BONUS * 2);
    }

    #[test]
    fn using_every_bird_without_hitting_the_pig_loses() {
        let mut game = Game::new(test_layout());

        for _ in 0..3 {
            launch(&mut game, Vec3::new(0.7, 0.0, 0.0));
            run(&mut game, 3.0);
        }
        run(&mut game, 2.0);

        assert_eq!(game.outcome(), Outcome::Lost);
        assert!(game.pig().alive);
        assert!(!game.is_animating());
    }

    #[test]
    fn fast_hit_breaks_a_block_and_frees_what_it_held() {
        let mut game = Game::new(test_layout());
        let mut layout = game.layout().clone();
        layout.planets.clear();
        // The frame hangs in the path of the bird.
        layout.blocks[0].center = Vec3::new(0.0, 0.0, 0.0);
        layout.blocks[1].center = Vec3::new(0.0, 0.28, 0.0);
        layout.pig_center = Vec3::new(0.0, 0.6, 0.0);
        game = Game::new(layout);
        launch(&mut game, Vec3::new(-0.6, 0.0, 0.0));
        assert!(launch_velocity(Vec3::new(-0.6, 0.0, 0.0)).length() > BLOCK_BREAK_SPEED);
        run(&mut game, 1.0);

        assert_eq!(game.blocks()[0].state, BlockState::Destroyed);
        assert_ne!(game.blocks()[1].state, BlockState::Attached);
        assert!(!game.pig().attached);
    }

    #[test]
    fn falling_pig_pops_on_the_asteroid() {
        let mut game = Game::new(test_layout());
        let mut layout = game.layout().clone();
        layout.pig_support = None;
        game = Game::new(layout);
        // Unsupported pig falls into the planet it floats over.
        assert!(!game.pig().attached);
        run(&mut game, 3.0);

        assert!(!game.pig().alive);
        assert_eq!(game.outcome(), Outcome::Won);
    }

    #[test]
    fn preview_starts_at_the_slingshot_and_follows_the_launch() {
        let mut game = Game::new(test_layout());
        let rest = game.layout().slingshot_rest;
        game.try_grab(rest);
        game.aim_at(rest + Vec3::new(-0.5, 0.0, 0.0));
        let dots = game.trajectory_preview();

        assert!(dots.len() > 5);
        assert!(dots[0].x > rest.x - 0.5);
        assert!(dots[dots.len() - 1].x > rest.x);
        assert!(dots.windows(2).all(|pair| pair[1].x >= pair[0].x - 1.0e-4));
    }

    #[test]
    fn circle_block_contact_pushes_out_of_the_nearest_side() {
        let game = Game::new(test_layout());
        let block = game.blocks()[0];
        // Frame at (3.0, 0.6), half size 0.12. A circle touching its left side.
        let (normal, depth) = circle_block_contact(Vec3::new(2.8, 0.6, 0.0), 0.1, block).unwrap();

        assert!(normal.x < -0.99);
        assert!((depth - 0.02).abs() < 1.0e-4);
        assert!(circle_block_contact(Vec3::new(2.5, 0.6, 0.0), 0.1, block).is_none());
    }

    #[test]
    fn restart_brings_every_bird_and_the_pig_back() {
        let mut game = Game::new(test_layout());
        launch(&mut game, Vec3::new(0.7, 0.0, 0.0));
        run(&mut game, 3.0);
        let revision = game.revision();

        game.restart();

        assert_eq!(game.birds_left(), 3);
        assert!(game.pig().alive);
        assert_eq!(game.score(), 0);
        assert!(game.revision() > revision);
    }

    #[test]
    fn huge_frames_are_clamped() {
        let mut game = Game::new(test_layout());
        launch(&mut game, Vec3::new(-0.3, 0.0, 0.0));
        let start = game.birds()[0].position;
        game.update(10.0);
        let travelled = (game.birds()[0].position - start).length();

        assert!(travelled < launch_velocity(Vec3::new(-0.3, 0.0, 0.0)).length() * 0.11);
    }
}
