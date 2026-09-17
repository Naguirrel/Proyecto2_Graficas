use crate::{
    intersection::Intersection,
    math::{Vec2, Vec3},
    ray::Ray,
};

const SLAB_EPSILON: f32 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material_id: usize,
}

impl Cube {
    pub fn new(first_corner: Vec3, second_corner: Vec3, material_id: usize) -> Self {
        Self {
            min: Vec3::new(
                first_corner.x.min(second_corner.x),
                first_corner.y.min(second_corner.y),
                first_corner.z.min(second_corner.z),
            ),
            max: Vec3::new(
                first_corner.x.max(second_corner.x),
                first_corner.y.max(second_corner.y),
                first_corner.z.max(second_corner.z),
            ),
            material_id,
        }
    }

    /// Intersects this axis-aligned cube with a ray using the slab method.
    /// Non-finite inputs, invalid intervals, and zero-direction rays return
    /// `None` so no hit can carry NaN or Inf values into later shading.
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        if !self.has_finite_bounds()
            || !is_finite_vec3(ray.origin)
            || !is_finite_vec3(ray.direction)
            || ray.direction == Vec3::ZERO
            || !t_min.is_finite()
            || !t_max.is_finite()
            || t_min > t_max
        {
            return None;
        }

        let mut entry_distance = f32::NEG_INFINITY;
        let mut exit_distance = f32::INFINITY;
        let mut entry_normal = Vec3::ZERO;
        let mut exit_normal = Vec3::ZERO;

        for axis in Axis::ALL {
            let slab = slab_intersection(*axis, ray, self.min, self.max)?;

            if slab.near_distance > entry_distance + SLAB_EPSILON {
                entry_distance = slab.near_distance;
                entry_normal = slab.near_normal;
            }

            if slab.far_distance < exit_distance - SLAB_EPSILON {
                exit_distance = slab.far_distance;
                exit_normal = slab.far_normal;
            }

            if entry_distance > exit_distance + SLAB_EPSILON {
                return None;
            }
        }

        let (distance, normal) =
            if entry_distance >= t_min && entry_distance <= t_max && entry_distance.is_finite() {
                (entry_distance, entry_normal)
            } else if exit_distance >= t_min && exit_distance <= t_max && exit_distance.is_finite()
            {
                (exit_distance, exit_normal)
            } else {
                return None;
            };

        let position = ray.at(distance);

        if !distance.is_finite() || !is_finite_vec3(position) || !is_finite_vec3(normal) {
            return None;
        }

        Some(Intersection::new(
            distance,
            position,
            normal,
            self.uv_at(position, normal),
            self.material_id,
        ))
    }

    fn has_finite_bounds(&self) -> bool {
        is_finite_vec3(self.min) && is_finite_vec3(self.max)
    }

    /// UV orientation by face:
    /// +X uses u=-Z, v=+Y. -X uses u=+Z, v=+Y.
    /// +Y uses u=+X, v=-Z. -Y uses u=+X, v=+Z.
    /// +Z uses u=+X, v=+Y. -Z uses u=-X, v=+Y.
    fn uv_at(&self, position: Vec3, normal: Vec3) -> Vec2 {
        let x = normalized_coordinate(position.x, self.min.x, self.max.x);
        let y = normalized_coordinate(position.y, self.min.y, self.max.y);
        let z = normalized_coordinate(position.z, self.min.z, self.max.z);

        if normal == Vec3::new(1.0, 0.0, 0.0) {
            Vec2::new(1.0 - z, y)
        } else if normal == Vec3::new(-1.0, 0.0, 0.0) {
            Vec2::new(z, y)
        } else if normal == Vec3::new(0.0, 1.0, 0.0) {
            Vec2::new(x, 1.0 - z)
        } else if normal == Vec3::new(0.0, -1.0, 0.0) {
            Vec2::new(x, z)
        } else if normal == Vec3::new(0.0, 0.0, 1.0) {
            Vec2::new(x, y)
        } else if normal == Vec3::new(0.0, 0.0, -1.0) {
            Vec2::new(1.0 - x, y)
        } else {
            Vec2::ZERO
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SlabIntersection {
    near_distance: f32,
    far_distance: f32,
    near_normal: Vec3,
    far_normal: Vec3,
}

#[derive(Debug, Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    const ALL: &'static [Self] = &[Self::X, Self::Y, Self::Z];

    fn components(self, vector: Vec3) -> f32 {
        match self {
            Self::X => vector.x,
            Self::Y => vector.y,
            Self::Z => vector.z,
        }
    }

    fn min_normal(self) -> Vec3 {
        match self {
            Self::X => Vec3::new(-1.0, 0.0, 0.0),
            Self::Y => Vec3::new(0.0, -1.0, 0.0),
            Self::Z => Vec3::new(0.0, 0.0, -1.0),
        }
    }

    fn max_normal(self) -> Vec3 {
        match self {
            Self::X => Vec3::new(1.0, 0.0, 0.0),
            Self::Y => Vec3::new(0.0, 1.0, 0.0),
            Self::Z => Vec3::new(0.0, 0.0, 1.0),
        }
    }
}

fn slab_intersection(axis: Axis, ray: &Ray, min: Vec3, max: Vec3) -> Option<SlabIntersection> {
    let origin = axis.components(ray.origin);
    let direction = axis.components(ray.direction);
    let slab_min = axis.components(min);
    let slab_max = axis.components(max);

    if direction.abs() <= SLAB_EPSILON {
        if origin < slab_min || origin > slab_max {
            return None;
        }

        return Some(SlabIntersection {
            near_distance: f32::NEG_INFINITY,
            far_distance: f32::INFINITY,
            near_normal: Vec3::ZERO,
            far_normal: Vec3::ZERO,
        });
    }

    let inverse_direction = 1.0 / direction;
    let mut near_distance = (slab_min - origin) * inverse_direction;
    let mut far_distance = (slab_max - origin) * inverse_direction;
    let mut near_normal = axis.min_normal();
    let mut far_normal = axis.max_normal();

    if near_distance > far_distance {
        std::mem::swap(&mut near_distance, &mut far_distance);
        std::mem::swap(&mut near_normal, &mut far_normal);
    }

    Some(SlabIntersection {
        near_distance,
        far_distance,
        near_normal,
        far_normal,
    })
}

fn is_finite_vec3(vector: Vec3) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

fn normalized_coordinate(value: f32, min: f32, max: f32) -> f32 {
    let span = max - min;

    if !value.is_finite() || !min.is_finite() || !max.is_finite() || span.abs() <= SLAB_EPSILON {
        return 0.0;
    }

    ((value - min) / span).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::Cube;
    use crate::{
        math::{Vec2, Vec3},
        ray::Ray,
    };

    const EPSILON: f32 = 0.0001;

    fn unit_cube() -> Cube {
        Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0), 7)
    }

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < EPSILON, "{left} != {right}");
    }

    fn assert_vec_near(left: Vec3, right: Vec3) {
        assert!(left.approx_eq(right), "{left:?} != {right:?}");
    }

    fn assert_uv_near(left: Vec2, right: Vec2) {
        assert!(left.approx_eq(right), "{left:?} != {right:?}");
    }

    fn assert_valid_uv(uv: Vec2) {
        assert!(uv.u.is_finite());
        assert!(uv.v.is_finite());
        assert!((0.0..=1.0).contains(&uv.u));
        assert!((0.0..=1.0).contains(&uv.v));
    }

    #[test]
    fn frontal_ray_hits_cube() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_cube().intersect(&ray, 0.001, 100.0).is_some());
    }

    #[test]
    fn frontal_distance_is_correct() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
    }

    #[test]
    fn frontal_position_is_correct() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(hit.position, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn frontal_normal_is_correct() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn lateral_ray_returns_correct_normal() {
        let ray = Ray::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn vertical_ray_returns_correct_normal() {
        let ray = Ray::new(Vec3::new(0.0, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.normal, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn ray_missing_cube_returns_none() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 1.0, 0.0));

        assert!(unit_cube().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn parallel_ray_outside_slab_returns_none() {
        let ray = Ray::new(Vec3::new(2.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_cube().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn parallel_ray_inside_slabs_can_hit() {
        let ray = Ray::new(Vec3::new(0.5, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn ray_inside_cube_uses_exit_intersection() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert_vec_near(hit.position, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn material_id_is_copied_to_intersection() {
        let cube = Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0), 42);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = cube.intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.material_id, 42);
    }

    #[test]
    fn face_center_produces_center_uv() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_uv_near(hit.uv, Vec2::new(0.5, 0.5));
    }

    #[test]
    fn positive_x_face_produces_valid_uv() {
        let ray = Ray::new(Vec3::new(3.0, 0.25, 0.5), Vec3::new(-1.0, 0.0, 0.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.normal, Vec3::new(1.0, 0.0, 0.0));
        assert_valid_uv(hit.uv);
        assert_uv_near(hit.uv, Vec2::new(0.25, 0.625));
    }

    #[test]
    fn negative_x_face_produces_valid_uv() {
        let ray = Ray::new(Vec3::new(-3.0, 0.25, 0.5), Vec3::new(1.0, 0.0, 0.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.normal, Vec3::new(-1.0, 0.0, 0.0));
        assert_valid_uv(hit.uv);
        assert_uv_near(hit.uv, Vec2::new(0.75, 0.625));
    }

    #[test]
    fn positive_y_face_produces_valid_uv() {
        let ray = Ray::new(Vec3::new(0.5, 3.0, 0.25), Vec3::new(0.0, -1.0, 0.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.normal, Vec3::new(0.0, 1.0, 0.0));
        assert_valid_uv(hit.uv);
        assert_uv_near(hit.uv, Vec2::new(0.75, 0.375));
    }

    #[test]
    fn negative_y_face_produces_valid_uv() {
        let ray = Ray::new(Vec3::new(0.5, -3.0, 0.25), Vec3::new(0.0, 1.0, 0.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.normal, Vec3::new(0.0, -1.0, 0.0));
        assert_valid_uv(hit.uv);
        assert_uv_near(hit.uv, Vec2::new(0.75, 0.625));
    }

    #[test]
    fn positive_z_face_produces_valid_uv() {
        let ray = Ray::new(Vec3::new(0.5, 0.25, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
        assert_valid_uv(hit.uv);
        assert_uv_near(hit.uv, Vec2::new(0.75, 0.625));
    }

    #[test]
    fn negative_z_face_produces_valid_uv() {
        let ray = Ray::new(Vec3::new(0.5, 0.25, -3.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, -1.0));
        assert_valid_uv(hit.uv);
        assert_uv_near(hit.uv, Vec2::new(0.25, 0.625));
    }

    #[test]
    fn face_corners_cover_zero_and_one_uv() {
        let lower = Ray::new(Vec3::new(-1.0, -1.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let upper = Ray::new(Vec3::new(1.0, 1.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let lower_hit = unit_cube().intersect(&lower, 0.001, 100.0).unwrap();
        let upper_hit = unit_cube().intersect(&upper, 0.001, 100.0).unwrap();

        assert_uv_near(lower_hit.uv, Vec2::new(0.0, 0.0));
        assert_uv_near(upper_hit.uv, Vec2::new(1.0, 1.0));
    }

    #[test]
    fn opposite_faces_keep_documented_orientation() {
        let positive_x = unit_cube()
            .intersect(
                &Ray::new(Vec3::new(3.0, 0.0, 1.0), Vec3::new(-1.0, 0.0, 0.0)),
                0.001,
                100.0,
            )
            .unwrap();
        let negative_x = unit_cube()
            .intersect(
                &Ray::new(Vec3::new(-3.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
                0.001,
                100.0,
            )
            .unwrap();
        let positive_z = unit_cube()
            .intersect(
                &Ray::new(Vec3::new(1.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0)),
                0.001,
                100.0,
            )
            .unwrap();
        let negative_z = unit_cube()
            .intersect(
                &Ray::new(Vec3::new(1.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0)),
                0.001,
                100.0,
            )
            .unwrap();

        assert_near(positive_x.uv.u, 0.0);
        assert_near(negative_x.uv.u, 1.0);
        assert_near(positive_z.uv.u, 1.0);
        assert_near(negative_z.uv.u, 0.0);
        assert_near(positive_x.uv.v, negative_x.uv.v);
        assert_near(positive_z.uv.v, negative_z.uv.v);
    }

    #[test]
    fn translated_cube_uses_local_uv() {
        let cube = Cube::new(Vec3::new(2.0, 4.0, 6.0), Vec3::new(4.0, 6.0, 8.0), 0);
        let ray = Ray::new(Vec3::new(3.0, 5.0, 10.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = cube.intersect(&ray, 0.001, 100.0).unwrap();

        assert_uv_near(hit.uv, Vec2::new(0.5, 0.5));
    }

    #[test]
    fn non_uniform_cube_uses_axis_lengths_for_uv() {
        let cube = Cube::new(Vec3::new(-2.0, -1.0, -4.0), Vec3::new(2.0, 3.0, 6.0), 0);
        let ray = Ray::new(Vec3::new(0.0, 1.0, 8.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = cube.intersect(&ray, 0.001, 100.0).unwrap();

        assert_uv_near(hit.uv, Vec2::new(0.5, 0.5));
    }

    #[test]
    fn inverted_corner_cube_keeps_local_uv() {
        let cube = Cube::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(-2.0, -2.0, -2.0), 0);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = cube.intersect(&ray, 0.001, 100.0).unwrap();

        assert_uv_near(hit.uv, Vec2::new(0.5, 0.5));
    }

    #[test]
    fn ray_inside_cube_gets_exit_face_uv() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.5, 0.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.normal, Vec3::new(1.0, 0.0, 0.0));
        assert_uv_near(hit.uv, Vec2::new(0.5, 0.75));
    }

    #[test]
    fn short_t_max_discards_far_hit() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_cube().intersect(&ray, 0.001, 1.5).is_none());
    }

    #[test]
    fn t_min_after_entry_selects_exit_if_valid() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cube().intersect(&ray, 2.5, 100.0).unwrap();

        assert_near(hit.distance, 4.0);
        assert_vec_near(hit.position, Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn inverted_corners_are_normalized() {
        let cube = Cube::new(Vec3::new(1.0, -2.0, 3.0), Vec3::new(-1.0, 2.0, -3.0), 0);

        assert_eq!(cube.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(cube.max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn valid_hit_contains_only_finite_values() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert!(hit.distance.is_finite());
        assert!(hit.position.x.is_finite());
        assert!(hit.position.y.is_finite());
        assert!(hit.position.z.is_finite());
        assert!(hit.normal.x.is_finite());
        assert!(hit.normal.y.is_finite());
        assert!(hit.normal.z.is_finite());
        assert_valid_uv(hit.uv);
    }

    #[test]
    fn edge_hit_uses_deterministic_outer_normal() {
        let ray = Ray::new(Vec3::new(2.0, 2.0, 0.0), Vec3::new(-1.0, -1.0, 0.0));
        let hit = unit_cube().intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(hit.position, Vec3::new(1.0, 1.0, 0.0));
        assert_eq!(hit.normal, Vec3::new(1.0, 0.0, 0.0));
        assert_valid_uv(hit.uv);
        assert_uv_near(hit.uv, Vec2::new(0.5, 1.0));
    }

    #[test]
    fn degenerate_dimensions_do_not_produce_nan_uv() {
        let cube = Cube::new(Vec3::new(0.0, -1.0, -1.0), Vec3::new(0.0, 1.0, 1.0), 0);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = cube.intersect(&ray, 0.001, 100.0).unwrap();

        assert_valid_uv(hit.uv);
    }

    #[test]
    fn non_finite_values_return_none() {
        let cube = Cube::new(
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(f32::INFINITY, 1.0, 1.0),
            0,
        );
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));

        assert!(cube.intersect(&ray, 0.001, 100.0).is_none());
    }
}
