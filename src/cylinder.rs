use crate::{
    basis::Basis3,
    intersection::Intersection,
    math::{Vec2, Vec3},
    ray::Ray,
};

const CYLINDER_EPSILON: f32 = 0.0001;
const TWO_PI: f32 = std::f32::consts::PI * 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cylinder {
    center: Vec3,
    radius: f32,
    half_height: f32,
    orientation: Basis3,
    material_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CylinderError {
    NonFiniteCenter,
    InvalidRadius,
    InvalidHalfHeight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CylinderSurface {
    Side,
    Cap,
}

#[derive(Debug, Clone, Copy)]
struct CylinderHitCandidate {
    distance: f32,
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
    surface: CylinderSurface,
}

impl Cylinder {
    /// Creates a closed finite cylinder centered at `center`.
    /// Its local Y axis is the cylinder axis and extends by `half_height`.
    pub fn new(
        center: Vec3,
        radius: f32,
        half_height: f32,
        orientation: Basis3,
        material_id: usize,
    ) -> Result<Self, CylinderError> {
        if !vec3_is_finite(center) {
            return Err(CylinderError::NonFiniteCenter);
        }

        if !radius.is_finite() || radius <= 0.0 {
            return Err(CylinderError::InvalidRadius);
        }

        if !half_height.is_finite() || half_height <= 0.0 {
            return Err(CylinderError::InvalidHalfHeight);
        }

        Ok(Self {
            center,
            radius,
            half_height,
            orientation,
            material_id,
        })
    }

    pub fn center(self) -> Vec3 {
        self.center
    }

    pub fn radius(self) -> f32 {
        self.radius
    }

    pub fn half_height(self) -> f32 {
        self.half_height
    }

    pub fn height(self) -> f32 {
        self.half_height * 2.0
    }

    pub fn orientation(self) -> Basis3 {
        self.orientation
    }

    pub fn material_id(self) -> usize {
        self.material_id
    }

    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        if !self.has_valid_geometry()
            || !vec3_is_finite(ray.origin)
            || !vec3_is_finite(ray.direction)
            || ray.direction == Vec3::ZERO
            || !t_min.is_finite()
            || !t_max.is_finite()
            || t_min > t_max
        {
            return None;
        }

        let local_ray = Ray {
            origin: self.world_to_local_point(ray.origin),
            direction: self.world_to_local_vector(ray.direction),
        };
        let mut best = None;

        self.record_side_hits(&local_ray, t_min, t_max, &mut best);
        self.record_cap_hit(&local_ray, self.half_height, t_min, t_max, &mut best);
        self.record_cap_hit(&local_ray, -self.half_height, t_min, t_max, &mut best);

        let best = best?;
        let position = self.local_to_world_point(best.position);
        let normal = self.local_to_world_vector(best.normal).normalized();

        if vec3_is_finite(position) && vec3_is_finite(normal) && normal != Vec3::ZERO {
            Some(Intersection::new(
                best.distance,
                position,
                normal,
                best.uv,
                self.material_id,
            ))
        } else {
            None
        }
    }

    fn has_valid_geometry(&self) -> bool {
        vec3_is_finite(self.center)
            && self.radius.is_finite()
            && self.radius > 0.0
            && self.half_height.is_finite()
            && self.half_height > 0.0
    }

    fn record_side_hits(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        best: &mut Option<CylinderHitCandidate>,
    ) {
        let a = ray.direction.x * ray.direction.x + ray.direction.z * ray.direction.z;

        if a <= CYLINDER_EPSILON {
            return;
        }

        let half_b = ray.origin.x * ray.direction.x + ray.origin.z * ray.direction.z;
        let c =
            ray.origin.x * ray.origin.x + ray.origin.z * ray.origin.z - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < -CYLINDER_EPSILON {
            return;
        }

        let root = discriminant.max(0.0).sqrt();
        let near = (-half_b - root) / a;
        let far = (-half_b + root) / a;

        self.record_side_hit_at(ray, near, t_min, t_max, best);
        if root > CYLINDER_EPSILON {
            self.record_side_hit_at(ray, far, t_min, t_max, best);
        }
    }

    fn record_side_hit_at(
        &self,
        ray: &Ray,
        distance: f32,
        t_min: f32,
        t_max: f32,
        best: &mut Option<CylinderHitCandidate>,
    ) {
        if !distance.is_finite() || distance < t_min || distance > t_max {
            return;
        }

        let position = ray.at(distance);
        if !vec3_is_finite(position)
            || position.y < -self.half_height - CYLINDER_EPSILON
            || position.y > self.half_height + CYLINDER_EPSILON
        {
            return;
        }

        let normal = Vec3::new(position.x, 0.0, position.z).normalized();
        if normal == Vec3::ZERO {
            return;
        }

        let uv = Vec2::new(
            (0.5 + position.z.atan2(position.x) / TWO_PI).rem_euclid(1.0),
            ((position.y + self.half_height) / self.height()).clamp(0.0, 1.0),
        );

        choose_best(
            best,
            CylinderHitCandidate {
                distance,
                position,
                normal,
                uv,
                surface: CylinderSurface::Side,
            },
        );
    }

    fn record_cap_hit(
        &self,
        ray: &Ray,
        cap_y: f32,
        t_min: f32,
        t_max: f32,
        best: &mut Option<CylinderHitCandidate>,
    ) {
        if ray.direction.y.abs() <= CYLINDER_EPSILON {
            return;
        }

        let distance = (cap_y - ray.origin.y) / ray.direction.y;
        if !distance.is_finite() || distance < t_min || distance > t_max {
            return;
        }

        let position = ray.at(distance);
        let radial_squared = position.x * position.x + position.z * position.z;
        let radius_squared = self.radius * self.radius;

        if !vec3_is_finite(position) || radial_squared > radius_squared + CYLINDER_EPSILON {
            return;
        }

        let normal = if cap_y > 0.0 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(0.0, -1.0, 0.0)
        };
        let uv = Vec2::new(
            (0.5 + position.x / (2.0 * self.radius)).clamp(0.0, 1.0),
            (0.5 + position.z / (2.0 * self.radius)).clamp(0.0, 1.0),
        );

        choose_best(
            best,
            CylinderHitCandidate {
                distance,
                position,
                normal,
                uv,
                surface: CylinderSurface::Cap,
            },
        );
    }

    fn local_to_world_point(&self, point: Vec3) -> Vec3 {
        self.center + self.orientation.local_to_world_vector(point)
    }

    fn world_to_local_point(&self, point: Vec3) -> Vec3 {
        self.orientation.world_to_local_vector(point - self.center)
    }

    fn local_to_world_vector(&self, vector: Vec3) -> Vec3 {
        self.orientation.local_to_world_vector(vector)
    }

    fn world_to_local_vector(&self, vector: Vec3) -> Vec3 {
        self.orientation.world_to_local_vector(vector)
    }
}

fn choose_best(best: &mut Option<CylinderHitCandidate>, candidate: CylinderHitCandidate) {
    let replace = match best {
        None => true,
        Some(current) => {
            candidate.distance < current.distance - CYLINDER_EPSILON
                || ((candidate.distance - current.distance).abs() <= CYLINDER_EPSILON
                    && candidate.surface == CylinderSurface::Cap
                    && current.surface == CylinderSurface::Side)
        }
    };

    if replace {
        *best = Some(candidate);
    }
}

fn vec3_is_finite(vector: Vec3) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{Cylinder, CylinderError, vec3_is_finite};
    use crate::{basis::Basis3, math::Vec3, ray::Ray};

    fn unit_cylinder() -> Cylinder {
        Cylinder::new(Vec3::ZERO, 1.0, 1.0, Basis3::identity(), 7).unwrap()
    }

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < 0.0001, "{left} != {right}");
    }

    fn assert_vec_near(left: Vec3, right: Vec3) {
        assert!(left.approx_eq(right), "{left:?} != {right:?}");
    }

    #[test]
    fn constructor_accepts_valid_cylinder() {
        let orientation = Basis3::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.75).unwrap();
        let cylinder = Cylinder::new(Vec3::new(1.0, 2.0, 3.0), 0.5, 1.25, orientation, 3).unwrap();

        assert_eq!(cylinder.center(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(cylinder.radius(), 0.5);
        assert_eq!(cylinder.half_height(), 1.25);
        assert_eq!(cylinder.height(), 2.5);
        assert_eq!(cylinder.orientation(), orientation);
        assert_eq!(cylinder.material_id(), 3);
    }

    #[test]
    fn constructor_rejects_non_finite_center() {
        assert_eq!(
            Cylinder::new(
                Vec3::new(f32::NAN, 0.0, 0.0),
                1.0,
                1.0,
                Basis3::identity(),
                0
            ),
            Err(CylinderError::NonFiniteCenter)
        );
    }

    #[test]
    fn constructor_rejects_invalid_radius() {
        for radius in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                Cylinder::new(Vec3::ZERO, radius, 1.0, Basis3::identity(), 0),
                Err(CylinderError::InvalidRadius)
            );
        }
    }

    #[test]
    fn constructor_rejects_invalid_half_height() {
        for half_height in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                Cylinder::new(Vec3::ZERO, 1.0, half_height, Basis3::identity(), 0),
                Err(CylinderError::InvalidHalfHeight)
            );
        }
    }

    #[test]
    fn frontal_ray_hits_side() {
        let ray = Ray::new(Vec3::new(0.0, 0.25, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
        assert_vec_near(hit.position, Vec3::new(0.0, 0.25, 1.0));
        assert_vec_near(hit.normal, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(hit.material_id, 7);
    }

    #[test]
    fn side_miss_returns_none() {
        let ray = Ray::new(Vec3::new(1.25, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_cylinder().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn tangent_ray_produces_valid_intersection() {
        let ray = Ray::new(Vec3::new(1.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 3.0);
        assert_vec_near(hit.position, Vec3::new(1.0, 0.0, 0.0));
        assert_vec_near(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn ray_originated_inside_finds_exit_side() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert_vec_near(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn non_normalized_ray_direction_preserves_parameter_convention() {
        let ray = Ray {
            origin: Vec3::new(0.0, 0.0, 3.0),
            direction: Vec3::new(0.0, 0.0, -2.0),
        };
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert_vec_near(hit.position, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn side_hit_outside_height_is_rejected() {
        let ray = Ray::new(Vec3::new(0.0, 1.25, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_cylinder().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn top_cap_hit_uses_upward_normal_and_planar_uv() {
        let ray = Ray::new(Vec3::new(0.25, 3.0, -0.5), Vec3::new(0.0, -1.0, 0.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
        assert_vec_near(hit.position, Vec3::new(0.25, 1.0, -0.5));
        assert_vec_near(hit.normal, Vec3::new(0.0, 1.0, 0.0));
        assert_near(hit.uv.u, 0.625);
        assert_near(hit.uv.v, 0.25);
    }

    #[test]
    fn bottom_cap_hit_uses_downward_normal_and_planar_uv() {
        let ray = Ray::new(Vec3::new(-0.25, -3.0, 0.5), Vec3::new(0.0, 1.0, 0.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
        assert_vec_near(hit.position, Vec3::new(-0.25, -1.0, 0.5));
        assert_vec_near(hit.normal, Vec3::new(0.0, -1.0, 0.0));
        assert_near(hit.uv.u, 0.375);
        assert_near(hit.uv.v, 0.75);
    }

    #[test]
    fn cap_hit_outside_radius_is_rejected() {
        let ray = Ray::new(Vec3::new(1.25, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0));

        assert!(unit_cylinder().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn cap_parallel_ray_does_not_divide_by_zero() {
        let ray = Ray::new(Vec3::new(0.0, 2.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_cylinder().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn inside_ray_can_exit_through_cap() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert_vec_near(hit.position, Vec3::new(0.0, 1.0, 0.0));
        assert_vec_near(hit.normal, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn rim_tie_prefers_cap_over_side() {
        let ray = Ray::new(Vec3::new(1.0, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(hit.position, Vec3::new(1.0, 1.0, 0.0));
        assert_vec_near(hit.normal, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn nearest_valid_surface_is_selected() {
        let ray = Ray::new(Vec3::new(0.0, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
        assert_vec_near(hit.normal, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn t_min_and_t_max_are_respected() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let cylinder = unit_cylinder();

        assert!(cylinder.intersect(&ray, 0.001, 1.0).is_none());
        assert_near(cylinder.intersect(&ray, 2.5, 100.0).unwrap().distance, 4.0);
    }

    #[test]
    fn non_finite_inputs_do_not_intersect() {
        let cylinder = unit_cylinder();
        let invalid_origin = Ray::new(Vec3::new(f32::NAN, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let invalid_direction = Ray {
            origin: Vec3::new(0.0, 0.0, 3.0),
            direction: Vec3::new(0.0, f32::INFINITY, -1.0),
        };

        assert!(cylinder.intersect(&invalid_origin, 0.001, 100.0).is_none());
        assert!(
            cylinder
                .intersect(&invalid_direction, 0.001, 100.0)
                .is_none()
        );
        assert!(
            cylinder
                .intersect(&Ray::new(Vec3::ZERO, Vec3::ZERO), 0.001, 100.0)
                .is_none()
        );
        assert!(
            cylinder
                .intersect(&Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)), 2.0, 1.0)
                .is_none()
        );
    }

    #[test]
    fn lateral_uv_uses_angle_and_height() {
        let ray = Ray::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.uv.u, 0.5);
        assert_near(hit.uv.v, 0.5);
    }

    #[test]
    fn lateral_uv_height_covers_bottom_and_top() {
        let bottom_ray = Ray::new(Vec3::new(0.0, -1.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let top_ray = Ray::new(Vec3::new(0.0, 1.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let cylinder = unit_cylinder();

        assert_near(
            cylinder.intersect(&bottom_ray, 0.001, 100.0).unwrap().uv.v,
            0.0,
        );
        assert_near(
            cylinder.intersect(&top_ray, 0.001, 100.0).unwrap().uv.v,
            1.0,
        );
    }

    #[test]
    fn lateral_uv_stays_inside_range_at_seam() {
        let ray = Ray::new(Vec3::new(-3.0, 0.0, 0.0001), Vec3::new(1.0, 0.0, 0.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert!((0.0..=1.0).contains(&hit.uv.u));
        assert!((0.0..=1.0).contains(&hit.uv.v));
    }

    #[test]
    fn oriented_cylinder_transforms_hit_position_and_normal_to_world() {
        let orientation =
            Basis3::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2).unwrap();
        let cylinder = Cylinder::new(Vec3::ZERO, 1.0, 1.0, orientation, 0).unwrap();
        let ray = Ray::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = cylinder.intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
        assert_vec_near(hit.position, Vec3::new(1.0, 0.0, 0.0));
        assert_vec_near(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn translated_rotated_cylinder_uses_world_space_ray() {
        let orientation =
            Basis3::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2).unwrap();
        let cylinder = Cylinder::new(Vec3::new(2.0, 0.5, -1.0), 0.5, 1.0, orientation, 0).unwrap();
        let ray = Ray::new(Vec3::new(2.0, 0.5, 2.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = cylinder.intersect(&ray, 0.001, 100.0).unwrap();

        assert!(vec3_is_finite(hit.position));
        assert!(vec3_is_finite(hit.normal));
        assert_near(hit.normal.length(), 1.0);
    }

    #[test]
    fn local_world_uvs_are_preserved_by_orientation() {
        let orientation = Basis3::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), 0.8).unwrap();
        let identity = unit_cylinder();
        let rotated = Cylinder::new(Vec3::new(1.0, 2.0, 3.0), 1.0, 1.0, orientation, 7).unwrap();
        let local_origin = Vec3::new(0.25, 0.4, 3.0);
        let local_direction = Vec3::new(0.0, 0.0, -1.0);
        let identity_ray = Ray {
            origin: local_origin,
            direction: local_direction,
        };
        let rotated_ray = Ray {
            origin: rotated.center() + orientation.local_to_world_vector(local_origin),
            direction: orientation.local_to_world_vector(local_direction),
        };
        let identity_hit = identity.intersect(&identity_ray, 0.001, 100.0).unwrap();
        let rotated_hit = rotated.intersect(&rotated_ray, 0.001, 100.0).unwrap();

        assert_near(identity_hit.uv.u, rotated_hit.uv.u);
        assert_near(identity_hit.uv.v, rotated_hit.uv.v);
    }

    #[test]
    fn valid_intersection_contains_no_nan_or_infinity() {
        let ray = Ray::new(Vec3::new(0.2, 3.0, 0.3), Vec3::new(0.0, -1.0, 0.0));
        let hit = unit_cylinder().intersect(&ray, 0.001, 100.0).unwrap();

        assert!(hit.distance.is_finite());
        assert!(vec3_is_finite(hit.position));
        assert!(vec3_is_finite(hit.normal));
        assert!(hit.uv.u.is_finite());
        assert!(hit.uv.v.is_finite());
    }
}
