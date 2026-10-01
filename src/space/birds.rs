//! The three space birds: Super Red, the purple Lazer Bird and Fire Bomb.
//! Level one moves them with its game; the
//! static levels place them at their slingshot, one loaded in the pouch and
//! the others waiting on the ground next to it.

use super::{
    BLUE_MOON_SLINGSHOT_POUCH, Songbird, SongbirdMaterials, SpaceBuildError, SpaceMaterials,
    add_feather, basis_with_up, slingshot_local_point, voxel::VoxelBody,
};
use crate::{
    basis::Basis3,
    color::Color,
    crystal::{Crystal, CrystalShape},
    cylinder::Cylinder,
    material::Material,
    math::Vec3,
    oriented_box::OrientedBox,
    radial::RadialFrame,
    scene::Scene,
    sphere::Sphere,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BirdKind {
    Red,
    Lazer,
    Bomb,
}

pub(super) const BIRD_KINDS: [BirdKind; BIRD_COUNT] =
    [BirdKind::Red, BirdKind::Lazer, BirdKind::Bomb];
#[cfg(test)]
pub(super) const LAZER_PRISM_PARTS: usize = 2;

pub(super) const BIRD_COUNT: usize = 3;
/// Bird radius for a slingshot of scale 1; birds grow with their slingshot.
pub(super) const BIRD_RADIUS_PER_SLINGSHOT_SCALE: f32 = 0.12;
/// The loaded bird sits this far over the pouch, for a slingshot of scale 1.
pub(super) const BIRD_POUCH_LIFT: f32 = 0.07;
/// Height of a standing bird's center over the ground, in body radii.
pub(super) const STANDING_HEIGHT: f32 = 1.14;
/// Birds look a little towards the default camera so their faces show.
const FACE_THE_CAMERA: Vec3 = Vec3::new(0.0, 0.0, 0.55);

/// Body, belly and crest/brow colors from the space character designs.
pub(super) const BIRD_PALETTES: [(Color, Color, Color); BIRD_COUNT] = [
    (
        Color::new(0.88, 0.045, 0.065),
        Color::new(0.96, 0.82, 0.65),
        Color::new(0.68, 0.02, 0.035),
    ),
    (
        Color::new(0.48, 0.13, 0.72),
        Color::new(0.78, 0.59, 0.90),
        Color::new(0.18, 0.035, 0.28),
    ),
    (
        Color::new(0.045, 0.045, 0.06),
        Color::new(0.38, 0.40, 0.44),
        Color::new(0.88, 0.13, 0.035),
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
            0.22,
            24.0,
            0.0,
            0.0,
            1.0,
            color * 0.14,
        ))
    };
    let beak = bird_material(scene, Color::new(1.0, 0.64, 0.12))?;
    let accent = bird_material(scene, Color::new(1.0, 0.84, 0.10))?;
    let mut birds = [SongbirdMaterials {
        body: 0,
        belly: 0,
        feather: 0,
        beak,
        accent,
        eye: space.eye,
        pupil: space.pupil,
        kind: BirdKind::Red,
    }; BIRD_COUNT];
    for ((bird, (body, belly, feather)), kind) in
        birds.iter_mut().zip(BIRD_PALETTES).zip(BIRD_KINDS)
    {
        bird.body = bird_material(scene, body)?;
        bird.belly = bird_material(scene, belly)?;
        bird.feather = bird_material(scene, feather)?;
        bird.kind = kind;
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
            },
            materials[palette],
        )?;
    }

    Ok(scene.object_count() - first)
}

/// Compact, limbless silhouettes, with the mask and angry brows visible from
/// the launch camera. Character details stay in the bird's radial frame.
pub(super) fn add_songbird(
    scene: &mut Scene,
    bird: Songbird,
    colors: SongbirdMaterials,
) -> Result<(), SpaceBuildError> {
    let up = bird.up.normalized();
    let forward = (bird.forward - up * bird.forward.dot(up)).normalized();
    let right = up.cross(forward);
    let basis = Basis3::new(right, up, forward)?;
    let at = |point: Vec3| bird.center + basis.local_to_world_vector(point * bird.radius);
    let triangular = colors.kind == BirdKind::Lazer;
    let face_z = if triangular { 0.50 } else { 0.86 };
    let eye_y = if triangular { 0.03 } else { 0.22 };

    if triangular {
        // A triangular prism keeps Lazer's silhouette angular from the front
        // and gives it real depth when the orbital camera turns around it.
        let triangle_basis = Basis3::new(-up, forward, -right)?;
        for (base, radius, depth, material) in [
            (Vec3::new(0.0, -0.40, -0.40), 1.48, 0.88, colors.body),
            (Vec3::new(0.0, -0.60, 0.485), 0.70, 0.025, colors.belly),
        ] {
            scene.add_crystal(Crystal::new(
                at(base),
                CrystalShape {
                    radius: radius * bird.radius,
                    body_height: depth * bird.radius,
                    tip_height: 0.0,
                    tip_cut: 0.0,
                    base_tip_height: 0.0,
                    sides: 3,
                },
                triangle_basis,
                material,
            )?)?;
        }
    } else {
        scene.add_sphere(Sphere::new(bird.center, bird.radius, colors.body)?)?;
        scene.add_sphere(Sphere::new(
            at(Vec3::new(0.0, -0.25, 0.08)),
            bird.radius * 0.90,
            colors.belly,
        )?)?;
    }

    for side in [-1.0, 1.0] {
        let eye_x = if triangular { 0.26 } else { 0.30 };
        let eye = Vec3::new(side * eye_x, eye_y, face_z);
        // Black rims form Super Red's mask and Lazer's goggles.
        for (offset, radius, material) in [
            (Vec3::ZERO, 0.285, colors.pupil),
            (Vec3::new(0.0, -0.01, 0.15), 0.225, colors.eye),
            (Vec3::new(-side * 0.035, -0.025, 0.36), 0.10, colors.pupil),
        ] {
            scene.add_sphere(Sphere::new(
                at(eye + offset),
                bird.radius * radius,
                material,
            )?)?;
        }
        let (sin, cos) = (side * 0.28_f32).sin_cos();
        let brow_basis = Basis3::new(right * cos + up * sin, up * cos - right * sin, forward)?;
        scene.add_oriented_box(OrientedBox::new(
            at(Vec3::new(side * eye_x, eye_y + 0.20, face_z + 0.37)),
            Vec3::new(0.30, 0.07, 0.06) * bird.radius,
            brow_basis,
            if colors.kind == BirdKind::Bomb {
                colors.feather
            } else {
                colors.pupil
            },
        )?)?;

        if colors.kind != BirdKind::Bomb {
            let start = at(Vec3::new(side * 0.54, eye_y, face_z));
            let end = at(Vec3::new(side * 0.85, eye_y + 0.025, face_z - 0.46));
            scene.add_cylinder(Cylinder::new(
                (start + end) * 0.5,
                bird.radius * 0.045,
                (end - start).length() * 0.5,
                basis_with_up(end - start)?,
                colors.pupil,
            )?)?;
        }
    }

    for (base, tip, radius) in [
        (
            Vec3::new(0.0, eye_y - 0.26, face_z + 0.02),
            Vec3::new(0.0, eye_y - 0.32, face_z + 0.65),
            0.235,
        ),
        (
            Vec3::new(0.0, eye_y - 0.40, face_z),
            Vec3::new(0.0, eye_y - 0.46, face_z + 0.43),
            0.13,
        ),
    ] {
        add_feather(scene, at(base), at(tip), bird.radius * radius, colors.beak)?;
    }
    for tip in [
        Vec3::new(-0.30, 0.10, -1.28),
        Vec3::new(0.0, 0.22, -1.40),
        Vec3::new(0.30, 0.10, -1.28),
    ] {
        add_feather(
            scene,
            at(Vec3::new(0.0, -0.04, -0.74)),
            at(tip),
            bird.radius * 0.11,
            colors.pupil,
        )?;
    }

    if colors.kind == BirdKind::Bomb {
        let start = at(Vec3::new(0.0, 0.87, 0.0));
        let end = at(Vec3::new(0.12, 1.30, 0.0));
        scene.add_cylinder(Cylinder::new(
            (start + end) * 0.5,
            bird.radius * 0.075,
            (end - start).length() * 0.5,
            basis_with_up(end - start)?,
            colors.pupil,
        )?)?;
        add_feather(
            scene,
            end,
            at(Vec3::new(0.06, 1.59, 0.0)),
            bird.radius * 0.12,
            colors.accent,
        )?;
        scene.add_sphere(Sphere::new(
            at(Vec3::new(0.0, 0.63, 0.75)),
            bird.radius * 0.12,
            colors.eye,
        )?)?;
    } else {
        for (base, tip, radius) in [
            (
                Vec3::new(-0.10, 0.83, 0.0),
                Vec3::new(-0.28, 1.34, -0.08),
                0.17,
            ),
            (
                Vec3::new(0.12, 0.87, -0.04),
                Vec3::new(0.04, 1.22, -0.22),
                0.13,
            ),
        ] {
            add_feather(
                scene,
                at(base),
                at(tip),
                bird.radius * radius,
                colors.feather,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        BIRD_COUNT, BIRD_RADIUS_PER_SLINGSHOT_SCALE, BirdKind, SlingshotBirds, Songbird,
        add_slingshot_birds, add_songbird, register_bird_materials,
    };
    use crate::{
        basis::Basis3,
        math::Vec3,
        radial::RadialFrame,
        ray::Ray,
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
    fn space_birds_keep_distinct_silhouettes_and_visible_faces_when_rotated() {
        for basis in [
            Basis3::identity(),
            Basis3::from_axis_angle(Vec3::new(0.3, 1.0, 0.2), 0.7).unwrap(),
        ] {
            for palette in 0..BIRD_COUNT {
                let mut scene = Scene::new();
                let shared = register_space_materials(&mut scene).unwrap();
                let colors = register_bird_materials(&mut scene, shared).unwrap()[palette];
                let center = Vec3::new(2.0, -1.0, 0.5);
                let radius = 0.3;
                add_songbird(
                    &mut scene,
                    Songbird {
                        center,
                        radius,
                        forward: basis.forward(),
                        up: basis.up(),
                        palette,
                    },
                    colors,
                )
                .unwrap();

                let body = scene
                    .objects()
                    .iter()
                    .find(|p| p.material_id() == colors.body)
                    .unwrap();
                if colors.kind == BirdKind::Lazer {
                    assert_eq!(body.as_crystal().unwrap().sides(), 3);
                } else {
                    let sphere = body.as_sphere().unwrap();
                    assert!(sphere.center().approx_eq(center));
                    assert!((sphere.radius() - radius).abs() < 1.0e-6);
                }
                assert_eq!(
                    scene
                        .objects()
                        .iter()
                        .filter(|p| p.material_id() == colors.accent)
                        .count(),
                    usize::from(colors.kind == BirdKind::Bomb),
                );

                let (eye_x, eye_y) = if colors.kind == BirdKind::Lazer {
                    (0.26, 0.03)
                } else {
                    (0.30, 0.22)
                };
                for side in [-1.0, 1.0] {
                    for (point, material) in [
                        (
                            Vec3::new(side * (eye_x + 0.10), eye_y - 0.05, 3.0),
                            colors.eye,
                        ),
                        (
                            Vec3::new(side * (eye_x - 0.035), eye_y - 0.025, 3.0),
                            colors.pupil,
                        ),
                    ] {
                        let ray = Ray::new(
                            center + basis.local_to_world_vector(point * radius),
                            -basis.forward(),
                        );
                        let hit = scene.intersect(&ray, 0.001, 10.0).unwrap();
                        assert_eq!(
                            hit.material_id, material,
                            "{:?} face is obscured",
                            colors.kind
                        );
                    }
                }
            }
        }
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
