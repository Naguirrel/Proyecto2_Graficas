use crate::{
    basis::Basis3, cube::intersect_box_bounds, intersection::Intersection, math::Vec3, ray::Ray,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrientedBox {
    center: Vec3,
    half_extents: Vec3,
    orientation: Basis3,
    material_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrientedBoxError {
    NonFiniteCenter,
    InvalidHalfExtents,
}

impl OrientedBox {
    pub fn new(
        center: Vec3,
        half_extents: Vec3,
        orientation: Basis3,
        material_id: usize,
    ) -> Result<Self, OrientedBoxError> {
        if !is_finite_vec3(center) {
            return Err(OrientedBoxError::NonFiniteCenter);
        }

        if !is_finite_vec3(half_extents)
            || half_extents.x <= 0.0
            || half_extents.y <= 0.0
            || half_extents.z <= 0.0
        {
            return Err(OrientedBoxError::InvalidHalfExtents);
        }

        Ok(Self {
            center,
            half_extents,
            orientation,
            material_id,
        })
    }

    pub fn center(self) -> Vec3 {
        self.center
    }

    pub fn half_extents(self) -> Vec3 {
        self.half_extents
    }

    pub fn orientation(self) -> Basis3 {
        self.orientation
    }

    pub fn material_id(self) -> usize {
        self.material_id
    }

    pub fn local_to_world_point(self, point: Vec3) -> Vec3 {
        self.center + self.orientation.local_to_world_vector(point)
    }

    pub fn world_to_local_point(self, point: Vec3) -> Vec3 {
        self.orientation.world_to_local_vector(point - self.center)
    }

    pub fn local_to_world_vector(self, vector: Vec3) -> Vec3 {
        self.orientation.local_to_world_vector(vector)
    }

    pub fn world_to_local_vector(self, vector: Vec3) -> Vec3 {
        self.orientation.world_to_local_vector(vector)
    }

    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        if !is_finite_vec3(ray.origin) || !is_finite_vec3(ray.direction) {
            return None;
        }

        let local_ray = Ray {
            origin: self.world_to_local_point(ray.origin),
            direction: self.world_to_local_vector(ray.direction),
        };
        let local_hit = intersect_box_bounds(
            &local_ray,
            -self.half_extents,
            self.half_extents,
            t_min,
            t_max,
        )?;
        let position = self.local_to_world_point(local_hit.position);
        let normal = self.local_to_world_vector(local_hit.normal).normalized();

        if !is_finite_vec3(position) || !is_finite_vec3(normal) {
            return None;
        }

        Some(Intersection::new(
            local_hit.distance,
            position,
            normal,
            local_hit.uv,
            self.material_id,
        ))
    }
}

fn is_finite_vec3(vector: Vec3) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{OrientedBox, OrientedBoxError};
    use crate::{basis::Basis3, math::Vec3, ray::Ray};

    const EPSILON: f32 = 0.0001;

    fn unit_box() -> OrientedBox {
        OrientedBox::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0), Basis3::identity(), 5).unwrap()
    }

    fn rotated_box() -> OrientedBox {
        OrientedBox::new(
            Vec3::ZERO,
            Vec3::new(1.0, 0.5, 0.75),
            Basis3::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2).unwrap(),
            8,
        )
        .unwrap()
    }

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < EPSILON, "{left} != {right}");
    }

    fn assert_vec_near(left: Vec3, right: Vec3) {
        assert!(left.approx_eq(right), "{left:?} != {right:?}");
    }

    #[test]
    fn constructor_accepts_valid_box() {
        let orientation = Basis3::identity();
        let oriented_box = OrientedBox::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(0.5, 1.0, 1.5),
            orientation,
            7,
        )
        .unwrap();

        assert_eq!(oriented_box.center(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(oriented_box.half_extents(), Vec3::new(0.5, 1.0, 1.5));
        assert_eq!(oriented_box.orientation(), orientation);
        assert_eq!(oriented_box.material_id(), 7);
    }

    #[test]
    fn constructor_rejects_non_finite_center() {
        assert_eq!(
            OrientedBox::new(
                Vec3::new(f32::INFINITY, 0.0, 0.0),
                Vec3::new(1.0, 1.0, 1.0),
                Basis3::identity(),
                0,
            ),
            Err(OrientedBoxError::NonFiniteCenter)
        );
    }

    #[test]
    fn constructor_rejects_invalid_half_extents() {
        assert_eq!(
            OrientedBox::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 1.0), Basis3::identity(), 0,),
            Err(OrientedBoxError::InvalidHalfExtents)
        );
        assert_eq!(
            OrientedBox::new(
                Vec3::ZERO,
                Vec3::new(1.0, f32::NAN, 1.0),
                Basis3::identity(),
                0,
            ),
            Err(OrientedBoxError::InvalidHalfExtents)
        );
    }

    #[test]
    fn identity_box_matches_axis_aligned_cube_hit() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_box().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
        assert_vec_near(hit.position, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(hit.material_id, 5);
    }

    #[test]
    fn rotated_box_transforms_hit_position_and_normal_to_world() {
        let ray = Ray::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = rotated_box().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.25);
        assert_vec_near(hit.position, Vec3::new(0.75, 0.0, 0.0));
        assert_vec_near(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn translated_rotated_box_uses_world_space_ray() {
        let orientation =
            Basis3::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2).unwrap();
        let oriented_box = OrientedBox::new(
            Vec3::new(2.0, 0.0, -1.0),
            Vec3::new(1.0, 0.5, 0.75),
            orientation,
            0,
        )
        .unwrap();
        let ray = Ray::new(Vec3::new(5.0, 0.0, -1.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = oriented_box.intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(hit.position, Vec3::new(2.75, 0.0, -1.0));
        assert_vec_near(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn ray_inside_box_uses_exit_surface() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
        let hit = unit_box().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert_eq!(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn t_min_and_t_max_are_respected() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let oriented_box = unit_box();

        assert!(oriented_box.intersect(&ray, 0.001, 1.5).is_none());
        assert_near(
            oriented_box.intersect(&ray, 2.5, 100.0).unwrap().distance,
            4.0,
        );
    }

    #[test]
    fn uv_is_computed_in_local_box_space() {
        let ray = Ray::new(Vec3::new(3.0, 0.25, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = rotated_box().intersect(&ray, 0.001, 100.0).unwrap();

        assert!(hit.uv.approx_eq(crate::math::Vec2::new(0.5, 0.75)));
    }

    #[test]
    fn miss_returns_none() {
        let ray = Ray::new(Vec3::new(3.0, 2.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));

        assert!(rotated_box().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn non_finite_ray_returns_none() {
        let ray = Ray {
            origin: Vec3::new(f32::NAN, 0.0, 0.0),
            direction: Vec3::new(1.0, 0.0, 0.0),
        };

        assert!(unit_box().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn non_normalized_ray_direction_preserves_parameter_convention() {
        let ray = Ray {
            origin: Vec3::new(0.0, 0.0, 3.0),
            direction: Vec3::new(0.0, 0.0, -2.0),
        };
        let hit = unit_box().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert_vec_near(hit.position, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn local_world_point_transforms_are_inverse() {
        let oriented_box = rotated_box();
        let local = Vec3::new(0.4, -0.2, 0.6);
        let world = oriented_box.local_to_world_point(local);

        assert_vec_near(oriented_box.world_to_local_point(world), local);
    }
}
