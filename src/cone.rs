use crate::{
    basis::Basis3,
    intersection::Intersection,
    math::{Vec2, Vec3},
    ray::Ray,
};

const CONE_EPSILON: f32 = 0.0001;
const TWO_PI: f32 = std::f32::consts::PI * 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cone {
    center: Vec3,
    base_radius: f32,
    half_height: f32,
    orientation: Basis3,
    material_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConeError {
    NonFiniteCenter,
    InvalidBaseRadius,
    InvalidHalfHeight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConeSurface {
    Side,
    Base,
}

#[derive(Debug, Clone, Copy)]
struct ConeHitCandidate {
    distance: f32,
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
    surface: ConeSurface,
}

impl Cone {
    /// Creates a closed finite cone centered at `center`.
    ///
    /// In local space the main axis is Y, the circular base is at
    /// `y = -half_height`, and the apex is at `y = +half_height`. The only cap
    /// is the base; the apex is a singular point on the lateral surface.
    pub fn new(
        center: Vec3,
        base_radius: f32,
        half_height: f32,
        orientation: Basis3,
        material_id: usize,
    ) -> Result<Self, ConeError> {
        if !vec3_is_finite(center) {
            return Err(ConeError::NonFiniteCenter);
        }

        if !base_radius.is_finite() || base_radius <= 0.0 {
            return Err(ConeError::InvalidBaseRadius);
        }

        if !half_height.is_finite() || half_height <= 0.0 {
            return Err(ConeError::InvalidHalfHeight);
        }

        Ok(Self {
            center,
            base_radius,
            half_height,
            orientation,
            material_id,
        })
    }

    pub fn center(self) -> Vec3 {
        self.center
    }

    pub fn base_radius(self) -> f32 {
        self.base_radius
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
        self.record_base_hit(&local_ray, t_min, t_max, &mut best);

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
            && self.base_radius.is_finite()
            && self.base_radius > 0.0
            && self.half_height.is_finite()
            && self.half_height > 0.0
    }

    fn slope_squared(&self) -> f32 {
        let slope = self.base_radius / self.height();
        slope * slope
    }

    fn record_side_hits(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        best: &mut Option<ConeHitCandidate>,
    ) {
        let slope_squared = self.slope_squared();
        let q = self.half_height - ray.origin.y;
        let a = ray.direction.x * ray.direction.x + ray.direction.z * ray.direction.z
            - slope_squared * ray.direction.y * ray.direction.y;
        let half_b = ray.origin.x * ray.direction.x
            + ray.origin.z * ray.direction.z
            + slope_squared * q * ray.direction.y;
        let c = ray.origin.x * ray.origin.x + ray.origin.z * ray.origin.z - slope_squared * q * q;

        if !a.is_finite() || !half_b.is_finite() || !c.is_finite() {
            return;
        }

        if a.abs() <= CONE_EPSILON {
            let denominator = 2.0 * half_b;
            if denominator.abs() > CONE_EPSILON {
                self.record_side_hit_at(ray, -c / denominator, t_min, t_max, best);
            }
            return;
        }

        let discriminant = half_b * half_b - a * c;
        if !discriminant.is_finite() || discriminant < -CONE_EPSILON {
            return;
        }

        let root = discriminant.max(0.0).sqrt();
        let first = (-half_b - root) / a;
        let second = (-half_b + root) / a;
        let near = first.min(second);
        let far = first.max(second);

        self.record_side_hit_at(ray, near, t_min, t_max, best);
        if root > CONE_EPSILON {
            self.record_side_hit_at(ray, far, t_min, t_max, best);
        }
    }

    fn record_side_hit_at(
        &self,
        ray: &Ray,
        distance: f32,
        t_min: f32,
        t_max: f32,
        best: &mut Option<ConeHitCandidate>,
    ) {
        if !distance.is_finite() || distance < t_min || distance > t_max {
            return;
        }

        let position = ray.at(distance);
        if !vec3_is_finite(position)
            || position.y < -self.half_height - CONE_EPSILON
            || position.y > self.half_height + CONE_EPSILON
        {
            return;
        }

        let normal = self.side_normal(position, ray.direction);
        if normal == Vec3::ZERO || !vec3_is_finite(normal) {
            return;
        }

        let radial = radial_direction(position, ray.direction);
        let uv = Vec2::new(
            (0.5 + radial.z.atan2(radial.x) / TWO_PI).rem_euclid(1.0),
            ((position.y + self.half_height) / self.height()).clamp(0.0, 1.0),
        );

        choose_best(
            best,
            ConeHitCandidate {
                distance,
                position,
                normal,
                uv,
                surface: ConeSurface::Side,
            },
        );
    }

    fn side_normal(&self, position: Vec3, ray_direction: Vec3) -> Vec3 {
        let slope_squared = self.slope_squared();
        let gradient = Vec3::new(
            position.x,
            slope_squared * (self.half_height - position.y),
            position.z,
        )
        .normalized();

        if gradient != Vec3::ZERO {
            return gradient;
        }

        let radial = radial_direction(position, ray_direction);
        Vec3::new(radial.x, 0.0, radial.z).normalized()
    }

    fn record_base_hit(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        best: &mut Option<ConeHitCandidate>,
    ) {
        if ray.direction.y.abs() <= CONE_EPSILON {
            return;
        }

        let distance = (-self.half_height - ray.origin.y) / ray.direction.y;
        if !distance.is_finite() || distance < t_min || distance > t_max {
            return;
        }

        let position = ray.at(distance);
        let radial_squared = position.x * position.x + position.z * position.z;
        let radius_squared = self.base_radius * self.base_radius;

        if !vec3_is_finite(position) || radial_squared > radius_squared + CONE_EPSILON {
            return;
        }

        let uv = Vec2::new(
            (0.5 + position.x / (2.0 * self.base_radius)).clamp(0.0, 1.0),
            (0.5 + position.z / (2.0 * self.base_radius)).clamp(0.0, 1.0),
        );

        choose_best(
            best,
            ConeHitCandidate {
                distance,
                position,
                normal: Vec3::new(0.0, -1.0, 0.0),
                uv,
                surface: ConeSurface::Base,
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

fn choose_best(best: &mut Option<ConeHitCandidate>, candidate: ConeHitCandidate) {
    // Base wins ties at the rim so the shared base/side edge is deterministic.
    let replace = match best {
        None => true,
        Some(current) => {
            candidate.distance < current.distance - CONE_EPSILON
                || ((candidate.distance - current.distance).abs() <= CONE_EPSILON
                    && candidate.surface == ConeSurface::Base
                    && current.surface == ConeSurface::Side)
        }
    };

    if replace {
        *best = Some(candidate);
    }
}

fn radial_direction(position: Vec3, ray_direction: Vec3) -> Vec3 {
    let radial = Vec3::new(position.x, 0.0, position.z).normalized();
    if radial != Vec3::ZERO {
        return radial;
    }

    let incoming_radial = Vec3::new(-ray_direction.x, 0.0, -ray_direction.z).normalized();
    if incoming_radial != Vec3::ZERO {
        return incoming_radial;
    }

    Vec3::new(1.0, 0.0, 0.0)
}

fn vec3_is_finite(vector: Vec3) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{Cone, ConeError, vec3_is_finite};
    use crate::{basis::Basis3, math::Vec3, ray::Ray};

    fn unit_cone() -> Cone {
        Cone::new(Vec3::ZERO, 1.0, 1.0, Basis3::identity(), 7).unwrap()
    }

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < 0.0001, "{left} != {right}");
    }

    fn assert_vec_near(left: Vec3, right: Vec3) {
        assert!(left.approx_eq(right), "{left:?} != {right:?}");
    }

    fn assert_uv_in_range(uv: crate::math::Vec2) {
        assert!(uv.u.is_finite());
        assert!(uv.v.is_finite());
        assert!((0.0..1.0).contains(&uv.u));
        assert!((0.0..=1.0).contains(&uv.v));
    }

    #[test]
    fn constructor_accepts_valid_cone() {
        let orientation = Basis3::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.75).unwrap();
        let cone = Cone::new(Vec3::new(1.0, 2.0, 3.0), 0.5, 1.25, orientation, 3).unwrap();

        assert_eq!(cone.center(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(cone.base_radius(), 0.5);
        assert_eq!(cone.half_height(), 1.25);
        assert_eq!(cone.height(), 2.5);
        assert_eq!(cone.orientation(), orientation);
        assert_eq!(cone.material_id(), 3);
    }

    #[test]
    fn constructor_rejects_non_finite_center() {
        assert_eq!(
            Cone::new(
                Vec3::new(f32::NAN, 0.0, 0.0),
                1.0,
                1.0,
                Basis3::identity(),
                0
            ),
            Err(ConeError::NonFiniteCenter)
        );
    }

    #[test]
    fn constructor_rejects_invalid_base_radius() {
        for base_radius in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                Cone::new(Vec3::ZERO, base_radius, 1.0, Basis3::identity(), 0),
                Err(ConeError::InvalidBaseRadius)
            );
        }
    }

    #[test]
    fn constructor_rejects_invalid_half_height() {
        for half_height in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                Cone::new(Vec3::ZERO, 1.0, half_height, Basis3::identity(), 0),
                Err(ConeError::InvalidHalfHeight)
            );
        }
    }

    #[test]
    fn frontal_ray_hits_side() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.5);
        assert_vec_near(hit.position, Vec3::new(0.0, 0.0, 0.5));
        assert_eq!(hit.material_id, 7);
    }

    #[test]
    fn missed_ray_returns_none() {
        let ray = Ray::new(Vec3::new(1.25, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_cone().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn tangent_ray_produces_valid_intersection() {
        let ray = Ray::new(Vec3::new(0.5, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 3.0);
        assert_vec_near(hit.position, Vec3::new(0.5, 0.0, 0.0));
        assert!(hit.normal.length() > 0.99);
    }

    #[test]
    fn ray_originated_inside_finds_exit_surface() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 0.5);
        assert_vec_near(hit.position, Vec3::new(0.0, 0.0, 0.5));
    }

    #[test]
    fn non_normalized_ray_direction_preserves_parameter_convention() {
        let ray = Ray {
            origin: Vec3::new(0.0, 0.0, 3.0),
            direction: Vec3::new(0.0, 0.0, -2.0),
        };
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.25);
        assert_vec_near(hit.position, Vec3::new(0.0, 0.0, 0.5));
    }

    #[test]
    fn side_hit_outside_height_is_rejected() {
        let ray = Ray::new(Vec3::new(0.0, 1.25, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_cone().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn t_min_and_t_max_are_respected() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let cone = unit_cone();

        assert!(cone.intersect(&ray, 0.001, 2.0).is_none());
        assert_near(cone.intersect(&ray, 3.0, 100.0).unwrap().distance, 3.5);
    }

    #[test]
    fn lateral_position_and_normal_are_correct() {
        let ray = Ray::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(hit.position, Vec3::new(0.5, 0.0, 0.0));
        assert!(hit.normal.x > 0.89, "{:?}", hit.normal);
        assert!(hit.normal.y > 0.44, "{:?}", hit.normal);
        assert_near(hit.normal.length(), 1.0);
    }

    #[test]
    fn implicit_radius_decreases_toward_apex() {
        let lower_ray = Ray::new(Vec3::new(0.0, -0.5, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let upper_ray = Ray::new(Vec3::new(0.0, 0.5, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let cone = unit_cone();
        let lower_hit = cone.intersect(&lower_ray, 0.001, 100.0).unwrap();
        let upper_hit = cone.intersect(&upper_ray, 0.001, 100.0).unwrap();

        assert!(lower_hit.position.z > upper_hit.position.z);
    }

    #[test]
    fn quadratic_degenerate_cases_are_stable() {
        let ray = Ray {
            origin: Vec3::new(0.25, -1.0, 0.0),
            direction: Vec3::new(0.5, 1.0, 0.0),
        };
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 0.75);
        assert!(vec3_is_finite(hit.position));
        assert!(vec3_is_finite(hit.normal));
    }

    #[test]
    fn base_hit_from_below_uses_exterior_normal_and_center_uv() {
        let ray = Ray::new(Vec3::new(0.0, -3.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
        assert_vec_near(hit.position, Vec3::new(0.0, -1.0, 0.0));
        assert_vec_near(hit.normal, Vec3::new(0.0, -1.0, 0.0));
        assert_near(hit.uv.u, 0.5);
        assert_near(hit.uv.v, 0.5);
    }

    #[test]
    fn inside_ray_can_exit_through_base() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, -1.0, 0.0));
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert_vec_near(hit.normal, Vec3::new(0.0, -1.0, 0.0));
    }

    #[test]
    fn base_parallel_ray_does_not_create_invalid_candidate() {
        let ray = Ray::new(Vec3::new(0.0, -1.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_cone().intersect(&ray, 0.001, 1.0).is_none());
    }

    #[test]
    fn base_hit_outside_radius_is_rejected() {
        let ray = Ray::new(Vec3::new(1.25, -3.0, 0.0), Vec3::new(0.0, 1.0, 0.0));

        assert!(unit_cone().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn nearest_candidate_and_base_rim_priority_are_deterministic() {
        let ray = Ray::new(Vec3::new(1.0, -3.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(hit.position, Vec3::new(1.0, -1.0, 0.0));
        assert_vec_near(hit.normal, Vec3::new(0.0, -1.0, 0.0));
    }

    #[test]
    fn apex_hit_is_finite_with_stable_normal_and_uv() {
        let ray = Ray::new(Vec3::new(0.0, 1.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(hit.position, Vec3::new(0.0, 1.0, 0.0));
        assert!(vec3_is_finite(hit.normal));
        assert_ne!(hit.normal, Vec3::ZERO);
        assert_uv_in_range(hit.uv);
        assert_near(hit.uv.v, 1.0);
    }

    #[test]
    fn no_artificial_top_cap_is_created() {
        let ray = Ray::new(Vec3::new(1.25, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0));

        assert!(unit_cone().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn ray_above_apex_without_entering_does_not_hit_extension() {
        let ray = Ray::new(Vec3::new(0.0, 1.25, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_cone().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn rotated_cone_transforms_hit_position_and_normal_to_world() {
        let orientation =
            Basis3::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2).unwrap();
        let cone = Cone::new(Vec3::ZERO, 1.0, 1.0, orientation, 0).unwrap();
        let ray = Ray::new(Vec3::new(3.0, 1.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = cone.intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(hit.position, Vec3::new(1.0, 1.0, 0.0));
        assert!(hit.normal.x > 0.44);
        assert_near(hit.normal.length(), 1.0);
    }

    #[test]
    fn translated_rotated_cone_returns_finite_world_hit() {
        let orientation =
            Basis3::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2).unwrap();
        let cone = Cone::new(Vec3::new(2.0, 0.5, -1.0), 0.5, 1.0, orientation, 0).unwrap();
        let ray = Ray::new(Vec3::new(2.0, 0.5, 2.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = cone.intersect(&ray, 0.001, 100.0).unwrap();

        assert!(vec3_is_finite(hit.position));
        assert!(vec3_is_finite(hit.normal));
        assert_near(hit.normal.length(), 1.0);
    }

    #[test]
    fn local_world_vectors_are_inverse_through_orientation() {
        let orientation = Basis3::from_axis_angle(Vec3::new(0.3, 1.0, 0.2), 0.8).unwrap();
        let cone = Cone::new(Vec3::new(1.0, 2.0, 3.0), 1.0, 1.0, orientation, 0).unwrap();
        let local = Vec3::new(0.25, -0.4, 0.5);
        let world = cone.center() + cone.orientation().local_to_world_vector(local);
        let local_again = cone
            .orientation()
            .world_to_local_vector(world - cone.center());

        assert_vec_near(local_again, local);
    }

    #[test]
    fn rotated_cone_preserves_local_uvs() {
        let orientation = Basis3::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), 0.8).unwrap();
        let identity = unit_cone();
        let rotated = Cone::new(Vec3::new(1.0, 2.0, 3.0), 1.0, 1.0, orientation, 7).unwrap();
        let local_origin = Vec3::new(0.2, 0.25, 3.0);
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
    fn lateral_uvs_cover_height_and_seam() {
        let bottom_ray = Ray::new(Vec3::new(0.0, -1.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let top_ray = Ray::new(Vec3::new(0.0, 0.999, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let seam_ray = Ray::new(Vec3::new(-3.0, 0.0, 0.0001), Vec3::new(1.0, 0.0, 0.0));
        let cone = unit_cone();
        let bottom = cone.intersect(&bottom_ray, 0.001, 100.0).unwrap();
        let top = cone.intersect(&top_ray, 0.001, 100.0).unwrap();
        let seam = cone.intersect(&seam_ray, 0.001, 100.0).unwrap();

        assert_near(bottom.uv.v, 0.0);
        assert!(top.uv.v > 0.999);
        assert_uv_in_range(seam.uv);
    }

    #[test]
    fn base_planar_uv_stays_in_range() {
        let ray = Ray::new(Vec3::new(0.5, -3.0, -0.5), Vec3::new(0.0, 1.0, 0.0));
        let hit = unit_cone().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.uv.u, 0.75);
        assert_near(hit.uv.v, 0.25);
        assert_uv_in_range(hit.uv);
    }

    #[test]
    fn non_finite_inputs_do_not_intersect() {
        let cone = unit_cone();
        let invalid_origin = Ray {
            origin: Vec3::new(f32::NAN, 0.0, 3.0),
            direction: Vec3::new(0.0, 0.0, -1.0),
        };
        let invalid_direction = Ray {
            origin: Vec3::new(0.0, 0.0, 3.0),
            direction: Vec3::new(0.0, f32::INFINITY, -1.0),
        };

        assert!(cone.intersect(&invalid_origin, 0.001, 100.0).is_none());
        assert!(cone.intersect(&invalid_direction, 0.001, 100.0).is_none());
        assert!(
            cone.intersect(&Ray::new(Vec3::ZERO, Vec3::ZERO), 0.001, 100.0)
                .is_none()
        );
        assert!(
            cone.intersect(&Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)), 2.0, 1.0)
                .is_none()
        );
    }
}
