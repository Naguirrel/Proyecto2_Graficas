use std::f32::consts::PI;

use crate::{
    intersection::Intersection,
    math::{Vec2, Vec3},
    ray::Ray,
};

const SPHERE_EPSILON: f32 = 0.0001;

/// Sphere primitive with equirectangular UVs.
///
/// UV orientation follows `Texture`: `v = 0.0` is the bottom pole,
/// `v = 1.0` is the top pole, and `u` wraps horizontally in `[0, 1)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sphere {
    center: Vec3,
    radius: f32,
    material_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphereError {
    NonFiniteCenter,
    InvalidRadius,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material_id: usize) -> Result<Self, SphereError> {
        if !is_finite_vec3(center) {
            return Err(SphereError::NonFiniteCenter);
        }

        if !radius.is_finite() || radius <= 0.0 {
            return Err(SphereError::InvalidRadius);
        }

        Ok(Self {
            center,
            radius,
            material_id,
        })
    }

    pub fn center(&self) -> Vec3 {
        self.center
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn material_id(&self) -> usize {
        self.material_id
    }

    /// Intersects this sphere with a ray using the quadratic equation.
    /// Non-finite inputs, invalid intervals, and zero-direction rays return
    /// `None` so no hit can carry NaN or Inf values into later shading.
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        if !self.is_valid()
            || !is_finite_vec3(ray.origin)
            || !is_finite_vec3(ray.direction)
            || ray.direction == Vec3::ZERO
            || !t_min.is_finite()
            || !t_max.is_finite()
            || t_min > t_max
        {
            return None;
        }

        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);

        if !a.is_finite() || a <= SPHERE_EPSILON {
            return None;
        }

        let half_b = oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if !half_b.is_finite() || !c.is_finite() || !discriminant.is_finite() {
            return None;
        }

        if discriminant < -SPHERE_EPSILON {
            return None;
        }

        let sqrt_discriminant = discriminant.max(0.0).sqrt();
        let near = (-half_b - sqrt_discriminant) / a;
        let far = (-half_b + sqrt_discriminant) / a;
        let distance = select_distance(near, far, t_min, t_max)?;
        let position = ray.at(distance);

        if !distance.is_finite() || !is_finite_vec3(position) {
            return None;
        }

        let normal = ((position - self.center) / self.radius).normalized();
        let uv = Self::uv_from_normal(normal);

        if normal == Vec3::ZERO || !is_finite_vec3(normal) || !uv.u.is_finite() || !uv.v.is_finite()
        {
            return None;
        }

        Some(Intersection::new(
            distance,
            position,
            normal,
            uv,
            self.material_id,
        ))
    }

    /// Converts a normalized local sphere direction to equirectangular UVs.
    /// `v = 0.0` maps to the bottom pole and `v = 1.0` maps to the top pole.
    pub fn uv_from_normal(normal: Vec3) -> Vec2 {
        if !is_finite_vec3(normal) || normal == Vec3::ZERO {
            return Vec2::ZERO;
        }

        let n = normal.normalized();
        let u = (0.5 + n.z.atan2(n.x) / (2.0 * PI)).rem_euclid(1.0);
        let v = (0.5 + n.y.clamp(-1.0, 1.0).asin() / PI).clamp(0.0, 1.0);

        Vec2::new(u, v)
    }

    fn is_valid(&self) -> bool {
        is_finite_vec3(self.center) && self.radius.is_finite() && self.radius > 0.0
    }
}

fn select_distance(near: f32, far: f32, t_min: f32, t_max: f32) -> Option<f32> {
    if near.is_finite() && near >= t_min && near <= t_max {
        Some(near)
    } else if far.is_finite() && far >= t_min && far <= t_max {
        Some(far)
    } else {
        None
    }
}

fn is_finite_vec3(vector: Vec3) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{Sphere, SphereError};
    use crate::{
        math::{Vec2, Vec3},
        ray::Ray,
    };

    const EPSILON: f32 = 0.0001;

    fn unit_sphere() -> Sphere {
        Sphere::new(Vec3::ZERO, 1.0, 7).unwrap()
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
        assert!((0.0..1.0).contains(&uv.u));
        assert!((0.0..=1.0).contains(&uv.v));
    }

    fn assert_finite_hit(hit: crate::intersection::Intersection) {
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
    fn constructor_accepts_valid_center_and_radius() {
        let sphere = Sphere::new(Vec3::new(1.0, 2.0, 3.0), 2.5, 4).unwrap();

        assert_eq!(sphere.center(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(sphere.radius(), 2.5);
        assert_eq!(sphere.material_id(), 4);
    }

    #[test]
    fn constructor_rejects_zero_radius() {
        assert_eq!(
            Sphere::new(Vec3::ZERO, 0.0, 0),
            Err(SphereError::InvalidRadius)
        );
    }

    #[test]
    fn constructor_rejects_negative_radius() {
        assert_eq!(
            Sphere::new(Vec3::ZERO, -1.0, 0),
            Err(SphereError::InvalidRadius)
        );
    }

    #[test]
    fn constructor_rejects_nan_radius() {
        assert_eq!(
            Sphere::new(Vec3::ZERO, f32::NAN, 0),
            Err(SphereError::InvalidRadius)
        );
    }

    #[test]
    fn constructor_rejects_infinite_radius() {
        assert_eq!(
            Sphere::new(Vec3::ZERO, f32::INFINITY, 0),
            Err(SphereError::InvalidRadius)
        );
    }

    #[test]
    fn constructor_rejects_non_finite_center() {
        assert_eq!(
            Sphere::new(Vec3::new(0.0, f32::INFINITY, 0.0), 1.0, 0),
            Err(SphereError::NonFiniteCenter)
        );
    }

    #[test]
    fn frontal_ray_hits_sphere() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(unit_sphere().intersect(&ray, 0.001, 100.0).is_some());
    }

    #[test]
    fn nearest_positive_root_is_selected() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_sphere().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
    }

    #[test]
    fn missed_ray_returns_none() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 1.0, 0.0));

        assert!(unit_sphere().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn tangent_ray_produces_valid_intersection() {
        let ray = Ray::new(Vec3::new(1.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_sphere().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 3.0);
        assert_vec_near(hit.position, Vec3::new(1.0, 0.0, 0.0));
        assert_vec_near(hit.normal, Vec3::new(1.0, 0.0, 0.0));
        assert_finite_hit(hit);
    }

    #[test]
    fn ray_originated_inside_finds_exit_surface() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
        let hit = unit_sphere().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert_vec_near(hit.position, Vec3::new(1.0, 0.0, 0.0));
        assert_vec_near(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn ray_pointing_away_returns_none() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, 1.0));

        assert!(unit_sphere().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn non_normalized_ray_direction_works() {
        let ray = Ray {
            origin: Vec3::new(0.0, 0.0, 3.0),
            direction: Vec3::new(0.0, 0.0, -2.0),
        };
        let hit = unit_sphere().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert_vec_near(hit.position, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn surface_origin_avoids_immediate_hit_when_below_t_min() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));

        assert!(unit_sphere().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn hit_position_is_correct() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_sphere().intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(hit.position, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn distance_follows_ray_parameter_convention() {
        let ray = Ray {
            origin: Vec3::new(0.0, 0.0, 3.0),
            direction: Vec3::new(0.0, 0.0, -2.0),
        };
        let hit = unit_sphere().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert_vec_near(ray.at(hit.distance), hit.position);
    }

    #[test]
    fn outward_normal_is_correct() {
        let ray = Ray::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = unit_sphere().intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn normal_is_normalized() {
        let ray = Ray::new(Vec3::new(0.0, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = unit_sphere().intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.normal.length(), 1.0);
    }

    #[test]
    fn material_id_is_preserved() {
        let sphere = Sphere::new(Vec3::ZERO, 1.0, 42).unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = sphere.intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.material_id, 42);
    }

    #[test]
    fn top_pole_produces_v_near_one() {
        let uv = Sphere::uv_from_normal(Vec3::new(0.0, 1.0, 0.0));

        assert_near(uv.v, 1.0);
        assert_valid_uv(uv);
    }

    #[test]
    fn bottom_pole_produces_v_near_zero() {
        let uv = Sphere::uv_from_normal(Vec3::new(0.0, -1.0, 0.0));

        assert_near(uv.v, 0.0);
        assert_valid_uv(uv);
    }

    #[test]
    fn equator_produces_v_near_half() {
        let uv = Sphere::uv_from_normal(Vec3::new(1.0, 0.0, 0.0));

        assert_near(uv.v, 0.5);
        assert_valid_uv(uv);
    }

    #[test]
    fn cardinal_directions_produce_coherent_u_values() {
        assert_uv_near(
            Sphere::uv_from_normal(Vec3::new(1.0, 0.0, 0.0)),
            Vec2::new(0.5, 0.5),
        );
        assert_uv_near(
            Sphere::uv_from_normal(Vec3::new(0.0, 0.0, 1.0)),
            Vec2::new(0.75, 0.5),
        );
        assert_uv_near(
            Sphere::uv_from_normal(Vec3::new(-1.0, 0.0, 0.0)),
            Vec2::new(0.0, 0.5),
        );
        assert_uv_near(
            Sphere::uv_from_normal(Vec3::new(0.0, 0.0, -1.0)),
            Vec2::new(0.25, 0.5),
        );
    }

    #[test]
    fn horizontal_seam_stays_inside_range() {
        let uv = Sphere::uv_from_normal(Vec3::new(-1.0, 0.0, -0.00001));

        assert_valid_uv(uv);
    }

    #[test]
    fn all_intersection_uvs_are_finite() {
        let rays = [
            Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0)),
            Ray::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
            Ray::new(Vec3::new(0.0, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
        ];

        for ray in rays {
            let hit = unit_sphere().intersect(&ray, 0.001, 100.0).unwrap();

            assert_valid_uv(hit.uv);
        }
    }

    #[test]
    fn non_finite_inputs_do_not_intersect() {
        let sphere = unit_sphere();
        let invalid_origin = Ray {
            origin: Vec3::new(f32::NAN, 0.0, 0.0),
            direction: Vec3::new(1.0, 0.0, 0.0),
        };
        let invalid_direction = Ray {
            origin: Vec3::ZERO,
            direction: Vec3::new(f32::INFINITY, 0.0, 0.0),
        };

        assert!(sphere.intersect(&invalid_origin, 0.001, 100.0).is_none());
        assert!(sphere.intersect(&invalid_direction, 0.001, 100.0).is_none());
    }

    #[test]
    fn valid_intersection_contains_no_nan_or_infinity() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = unit_sphere().intersect(&ray, 0.001, 100.0).unwrap();

        assert_finite_hit(hit);
    }
}
