use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Builds a ray and normalizes its direction. A zero-length or non-finite
    /// direction becomes `Vec3::ZERO`, making the invalid ray explicit without
    /// introducing NaN or Inf values into later calculations.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalized(),
        }
    }

    pub fn at(self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[cfg(test)]
mod tests {
    use super::Ray;
    use crate::math::Vec3;

    #[test]
    fn normalizes_direction() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 3.0, 4.0));

        assert!(ray.direction.approx_eq(Vec3::new(0.0, 0.6, 0.8)));
    }

    #[test]
    fn at_zero_returns_origin() {
        let origin = Vec3::new(1.0, 2.0, 3.0);
        let ray = Ray::new(origin, Vec3::new(1.0, 0.0, 0.0));

        assert_eq!(ray.at(0.0), origin);
    }

    #[test]
    fn at_t_returns_point_along_ray() {
        let ray = Ray::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.0, 0.0, -2.0));

        assert!(ray.at(5.0).approx_eq(Vec3::new(1.0, 2.0, -2.0)));
    }

    #[test]
    fn zero_direction_stays_finite() {
        let ray = Ray::new(Vec3::ZERO, Vec3::ZERO);

        assert_eq!(ray.direction, Vec3::ZERO);
        assert!(ray.direction.x.is_finite());
        assert!(ray.direction.y.is_finite());
        assert!(ray.direction.z.is_finite());
    }
}
