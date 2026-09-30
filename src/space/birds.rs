//! The three birds shared by the space levels: blue, yellow and pink
//! songbirds (see `add_songbird`). Level one moves them with its game; the
//! static levels place them at their slingshot, one loaded in the pouch and
//! the others waiting on the ground next to it.

use super::{
    BLUE_MOON_SLINGSHOT_POUCH, Songbird, SongbirdMaterials, SpaceBuildError, SpaceMaterials,
    add_songbird, slingshot_local_point, voxel::VoxelBody,
};
use crate::{color::Color, material::Material, math::Vec3, radial::RadialFrame, scene::Scene};

pub(super) const BIRD_COUNT: usize = 3;
/// Bird radius for a slingshot of scale 1; birds grow with their slingshot.
pub(super) const BIRD_RADIUS_PER_SLINGSHOT_SCALE: f32 = 0.12;
/// The loaded bird sits this far over the pouch, for a slingshot of scale 1.
pub(super) const BIRD_POUCH_LIFT: f32 = 0.07;
/// Height of a standing bird's center over the ground, in body radii.
pub(super) const STANDING_HEIGHT: f32 = 1.14;
/// Birds look a little towards the default camera so their faces show.
const FACE_THE_CAMERA: Vec3 = Vec3::new(0.0, 0.0, 0.55);

/// Bird colors: body, belly and wing/tail feathers.
const BIRD_PALETTES: [(Color, Color, Color); BIRD_COUNT] = [
    (
        Color::new(0.20, 0.52, 0.95),
        Color::new(0.90, 0.95, 1.0),
        Color::new(0.08, 0.26, 0.66),
    ),
    (
        Color::new(1.0, 0.80, 0.14),
        Color::new(1.0, 0.96, 0.78),
        Color::new(0.86, 0.50, 0.04),
    ),
    (
        Color::new(0.95, 0.38, 0.66),
        Color::new(1.0, 0.88, 0.93),
        Color::new(0.60, 0.16, 0.44),
    ),
];

/// Registers the materials of the three birds. Birds keep a little emission,
/// like the other characters, so they stay readable away from the lights.
pub(super) fn register_bird_materials(
    scene: &mut Scene,
    space: SpaceMaterials,
) -> Result<[SongbirdMaterials; BIRD_COUNT], SpaceBuildError> {
    let bird_material = |scene: &mut Scene, color: Color| {
        scene.add_material(Material::new(
            color,
            0.45,
            40.0,
            0.0,
            0.0,
            1.0,
            color * 0.14,
        ))
    };
    let beak = bird_material(scene, Color::new(1.0, 0.64, 0.12))?;
    let feet = bird_material(scene, Color::new(0.95, 0.55, 0.12))?;
    let mut birds = [SongbirdMaterials {
        body: 0,
        belly: 0,
        feather: 0,
        beak,
        feet,
        eye: space.eye,
        pupil: space.pupil,
    }; BIRD_COUNT];
    for (bird, (body, belly, feather)) in birds.iter_mut().zip(BIRD_PALETTES) {
        bird.body = bird_material(scene, body)?;
        bird.belly = bird_material(scene, belly)?;
        bird.feather = bird_material(scene, feather)?;
    }

    Ok(birds)
}

/// A slingshot planted in `frame` and the ground its birds wait on.
#[derive(Debug, Clone, Copy)]
pub(super) struct SlingshotBirds<'a> {
    pub frame: RadialFrame,
    pub yaw_radians: f32,
    pub scale: f32,
    /// Where the loaded bird looks: towards the pigs.
    pub aim: Vec3,
    pub ground: &'a VoxelBody,
    /// Directions from the ground's first ball center where the other birds
    /// stand, in turn order.
    pub waiting_directions: &'a [Vec3],
}

impl SlingshotBirds<'_> {
    pub(super) fn bird_radius(&self) -> f32 {
        BIRD_RADIUS_PER_SLINGSHOT_SCALE * self.scale
    }

    /// Center of the bird loaded in the pouch.
    pub(super) fn loaded_center(&self) -> Vec3 {
        self.frame.local_to_world(slingshot_local_point(
            BLUE_MOON_SLINGSHOT_POUCH,
            self.yaw_radians,
            self.scale,
        )) + self.frame.outward() * (BIRD_POUCH_LIFT * self.scale)
    }

    /// Center and up direction of each waiting bird.
    pub(super) fn waiting_spots(&self) -> Vec<(Vec3, Vec3)> {
        let center = self.ground.origin();

        self.waiting_directions
            .iter()
            .map(|&direction| {
                let up = direction.normalized();
                let ground = self.ground.surface_distance(center, up);

                (
                    center + up * (ground + self.bird_radius() * STANDING_HEIGHT),
                    up,
                )
            })
            .collect()
    }
}

/// Adds the loaded bird and the waiting birds. Returns how many primitives
/// were added.
pub(super) fn add_slingshot_birds(
    scene: &mut Scene,
    slingshot: SlingshotBirds<'_>,
    materials: [SongbirdMaterials; BIRD_COUNT],
) -> Result<usize, SpaceBuildError> {
    let first = scene.object_count();
    let radius = slingshot.bird_radius();
    let up = slingshot.frame.outward();

    add_songbird(
        scene,
        Songbird {
            center: slingshot.loaded_center(),
            radius,
            forward: (slingshot.aim.normalized() + FACE_THE_CAMERA).normalized(),
            up,
            palette: 0,
            flying: false,
            crest: true,
            legs: false,
        },
        materials[0],
    )?;

    for (index, (center, up)) in slingshot.waiting_spots().into_iter().enumerate() {
        let palette = (index + 1) % BIRD_COUNT;
        add_songbird(
            scene,
            Songbird {
                center,
                radius,
                // Waiting birds look at the slingshot and the camera.
                forward: ((slingshot.loaded_center() - center).normalized() + FACE_THE_CAMERA)
                    .normalized(),
                up,
                palette,
                flying: false,
                crest: true,
                legs: true,
            },
            materials[palette],
        )?;
    }

    Ok(scene.object_count() - first)
}

#[cfg(test)]
mod tests {
    use super::{
        BIRD_COUNT, BIRD_RADIUS_PER_SLINGSHOT_SCALE, SlingshotBirds, add_slingshot_birds,
        register_bird_materials,
    };
    use crate::{
        math::Vec3,
        radial::RadialFrame,
        scene::Scene,
        space::{
            register_space_materials,
            voxel::{VoxelBall, VoxelBody},
        },
    };

    fn rock() -> VoxelBody {
        VoxelBody::ball(VoxelBall::new(Vec3::ZERO, 0.5, 0), 0.1, 7.0).unwrap()
    }

    #[test]
    fn birds_sit_in_the_pouch_and_stand_on_the_ground() {
        let ground = rock();
        let top = ground.surface_distance(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0));
        let frame = RadialFrame::from_normal(Vec3::ZERO, top, Vec3::new(0.0, 1.0, 0.0)).unwrap();
        let waiting = [Vec3::new(-1.0, 0.2, 0.3), Vec3::new(-0.6, -0.8, 0.3)];
        let slingshot = SlingshotBirds {
            frame,
            yaw_radians: 0.0,
            scale: 1.5,
            aim: Vec3::new(1.0, 1.0, 0.0),
            ground: &ground,
            waiting_directions: &waiting,
        };
        let radius = slingshot.bird_radius();

        assert!((radius - BIRD_RADIUS_PER_SLINGSHOT_SCALE * 1.5).abs() < 1.0e-6);
        // The pouch is over the ground, so the loaded bird does not touch it.
        assert!(slingshot.loaded_center().y > top + radius);
        for ((center, up), direction) in slingshot.waiting_spots().into_iter().zip(waiting) {
            let ground_distance = ground.surface_distance(Vec3::ZERO, direction);
            assert!(up.approx_eq(direction.normalized()));
            assert!(center.length() > ground_distance + radius);
            assert!(center.length() < ground_distance + radius * 1.5);
        }
    }

    #[test]
    fn every_bird_is_added_with_its_colors() {
        let mut scene = Scene::new();
        let space = register_space_materials(&mut scene).unwrap();
        let materials = register_bird_materials(&mut scene, space).unwrap();
        let ground = rock();
        let top = ground.surface_distance(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0));
        let frame = RadialFrame::from_normal(Vec3::ZERO, top, Vec3::new(0.0, 1.0, 0.0)).unwrap();
        let waiting = [Vec3::new(-1.0, 0.2, 0.3), Vec3::new(-0.6, -0.8, 0.3)];
        let parts = add_slingshot_birds(
            &mut scene,
            SlingshotBirds {
                frame,
                yaw_radians: 0.0,
                scale: 1.0,
                aim: Vec3::new(1.0, 0.0, 0.0),
                ground: &ground,
                waiting_directions: &waiting,
            },
            materials,
        )
        .unwrap();

        assert_eq!(parts, scene.object_count());
        for bird in materials.iter().take(BIRD_COUNT) {
            let bodies = scene
                .objects()
                .iter()
                .filter(|object| object.material_id() == bird.body)
                .count();
            assert_eq!(bodies, 1);
        }
    }
}
