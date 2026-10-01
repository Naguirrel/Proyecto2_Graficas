//! Rounded pig heads shared by the selector and all four levels.

use super::{SpaceBuildError, basis_with_up, facing_basis};
use crate::{basis::Basis3, cylinder::Cylinder, math::Vec3, scene::Scene, sphere::Sphere};

pub(super) const PIG_PART_COUNT: usize = 18;

#[derive(Clone, Copy)]
pub(super) struct PigMaterials {
    pub body: usize,
    pub ear: usize,
    pub snout: usize,
    pub detail: usize,
    pub eye: usize,
    pub pupil: usize,
}

pub(super) fn add_pig_head(
    scene: &mut Scene,
    center: Vec3,
    radius: f32,
    basis: Basis3,
    colors: PigMaterials,
) -> Result<(), SpaceBuildError> {
    let at = |point: Vec3| center + basis.local_to_world_vector(point * radius);
    let face = facing_basis(basis)?;
    let first = scene.object_count();
    scene.add_sphere(Sphere::new(center, radius, colors.body)?)?;

    // Slightly staggered discs form the oval snout without coplanar caps.
    for side in [-1.0, 1.0] {
        scene.add_cylinder(Cylinder::new(
            at(Vec3::new(side * 0.095, -0.18, 0.91 + side * 0.012)),
            radius * 0.285,
            radius * 0.16,
            face,
            colors.snout,
        )?)?;
        scene.add_cylinder(Cylinder::new(
            at(Vec3::new(side * 0.135, -0.18, 1.090)),
            radius * if side < 0.0 { 0.075 } else { 0.067 },
            radius * 0.018,
            face,
            colors.detail,
        )?)?;
    }

    for side in [-1.0, 1.0] {
        // Slightly uneven eyes and small pupils keep the pigs' goofy stare.
        let eye = Vec3::new(side * 0.43, 0.24 + side * 0.025, 0.83);
        let eye_radius = if side < 0.0 { 0.22 } else { 0.235 };
        scene.add_sphere(Sphere::new(at(eye), radius * eye_radius, colors.detail)?)?;
        scene.add_sphere(Sphere::new(
            at(eye + Vec3::new(0.0, 0.0, 0.07)),
            radius * (eye_radius - 0.022),
            colors.eye,
        )?)?;
        scene.add_sphere(Sphere::new(
            at(eye + Vec3::new(-side * 0.025, 0.01, 0.26)),
            radius * 0.068,
            colors.pupil,
        )?)?;

        // Round green ears with a darker inset, instead of pointed cones.
        let ear = Vec3::new(side * 0.43, 0.88 + side * 0.04, 0.025);
        scene.add_sphere(Sphere::new(at(ear), radius * 0.205, colors.ear)?)?;
        scene.add_sphere(Sphere::new(
            at(ear + Vec3::new(0.0, 0.028, 0.16)),
            radius * 0.086,
            colors.detail,
        )?)?;

        let brow_start = at(Vec3::new(side * 0.33, 0.51, 0.825));
        let brow_end = at(Vec3::new(side * 0.54, 0.54, 0.755));
        scene.add_cylinder(Cylinder::new(
            (brow_start + brow_end) * 0.5,
            radius * 0.028,
            (brow_end - brow_start).length() * 0.5,
            basis_with_up(brow_end - brow_start)?,
            colors.detail,
        )?)?;
    }

    scene.add_cylinder(Cylinder::new(
        at(Vec3::new(0.0, -0.48, 0.885)),
        radius * 0.025,
        radius * 0.17,
        basis_with_up(basis.right())?,
        colors.detail,
    )?)?;
    debug_assert_eq!(scene.object_count() - first, PIG_PART_COUNT);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PIG_PART_COUNT, PigMaterials, add_pig_head};
    use crate::{
        basis::Basis3, math::Vec3, ray::Ray, scene::Scene, space::register_space_materials,
    };

    #[test]
    fn pig_has_round_ears_and_two_visible_nostrils_in_any_frame() {
        for basis in [
            Basis3::identity(),
            Basis3::from_axis_angle(Vec3::new(0.3, 1.0, 0.2), 0.7).unwrap(),
        ] {
            let mut scene = Scene::new();
            let shared = register_space_materials(&mut scene).unwrap();
            let center = Vec3::new(2.0, -1.0, 0.5);
            let radius = 0.3;
            add_pig_head(
                &mut scene,
                center,
                radius,
                basis,
                PigMaterials {
                    body: shared.pig,
                    ear: shared.pig_ear,
                    snout: shared.snout,
                    detail: shared.pig_detail,
                    eye: shared.eye,
                    pupil: shared.pupil,
                },
            )
            .unwrap();
            assert_eq!(scene.object_count(), PIG_PART_COUNT);
            assert_eq!(scene.cone_count(), 0);
            assert_eq!(
                scene
                    .objects()
                    .iter()
                    .filter_map(|part| part.as_sphere())
                    .filter(|part| part.material_id() == shared.pig_ear)
                    .count(),
                2
            );
            for side in [-1.0, 1.0] {
                let origin = center
                    + basis.local_to_world_vector(Vec3::new(side * 0.135, -0.18, 3.0) * radius);
                let ray = Ray::new(origin, -basis.forward());
                let hit = scene.intersect(&ray, 0.001, 10.0).unwrap();
                assert_eq!(hit.material_id, shared.pig_detail);
            }
        }
    }
}
