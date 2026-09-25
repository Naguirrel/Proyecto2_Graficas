use crate::{basis::Basis3, math::Vec3};

const RADIAL_EPSILON: f32 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadialFrame {
    planet_center: Vec3,
    planet_radius: f32,
    outward: Vec3,
    basis: Basis3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadialFrameError {
    NonFiniteCenter,
    InvalidRadius,
    InvalidNormal,
}

impl RadialFrame {
    pub fn from_normal(
        planet_center: Vec3,
        planet_radius: f32,
        outward_normal: Vec3,
    ) -> Result<Self, RadialFrameError> {
        if !is_finite_vec3(planet_center) {
            return Err(RadialFrameError::NonFiniteCenter);
        }

        if !planet_radius.is_finite() || planet_radius <= 0.0 {
            return Err(RadialFrameError::InvalidRadius);
        }

        if !is_finite_vec3(outward_normal) || outward_normal.length_squared() <= RADIAL_EPSILON {
            return Err(RadialFrameError::InvalidNormal);
        }

        let outward = outward_normal.normalized();
        let auxiliary = stable_auxiliary_axis(outward);
        let tangent_x = outward.cross(auxiliary).normalized();
        let tangent_z = tangent_x.cross(outward).normalized();
        let basis = Basis3::new(tangent_x, outward, tangent_z)
            .map_err(|_| RadialFrameError::InvalidNormal)?;

        Ok(Self {
            planet_center,
            planet_radius,
            outward,
            basis,
        })
    }

    pub fn from_latitude_longitude(
        planet_center: Vec3,
        planet_radius: f32,
        latitude_radians: f32,
        longitude_radians: f32,
    ) -> Result<Self, RadialFrameError> {
        if !latitude_radians.is_finite() || !longitude_radians.is_finite() {
            return Err(RadialFrameError::InvalidNormal);
        }

        let latitude_cos = latitude_radians.cos();
        let normal = Vec3::new(
            latitude_cos * longitude_radians.cos(),
            latitude_radians.sin(),
            latitude_cos * longitude_radians.sin(),
        );

        Self::from_normal(planet_center, planet_radius, normal)
    }

    pub fn planet_center(self) -> Vec3 {
        self.planet_center
    }

    pub fn planet_radius(self) -> f32 {
        self.planet_radius
    }

    pub fn outward(self) -> Vec3 {
        self.outward
    }

    pub fn basis(self) -> Basis3 {
        self.basis
    }

    pub fn surface_point(self) -> Vec3 {
        self.planet_center + self.outward * self.planet_radius
    }

    pub fn position(self, tangent_x: f32, radial_y: f32, tangent_z: f32) -> Vec3 {
        self.local_to_world(Vec3::new(tangent_x, radial_y, tangent_z))
    }

    pub fn local_to_world(self, local: Vec3) -> Vec3 {
        self.surface_point() + self.basis.local_to_world_vector(local)
    }
}

fn stable_auxiliary_axis(outward: Vec3) -> Vec3 {
    if outward.y.abs() < 0.85 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    }
}

fn is_finite_vec3(vector: Vec3) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{RadialFrame, RadialFrameError};
    use crate::math::Vec3;

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < 0.0001, "{left} != {right}");
    }

    fn assert_vec_near(left: Vec3, right: Vec3) {
        assert!(left.approx_eq(right), "{left:?} != {right:?}");
    }

    fn assert_frame_is_orthonormal(frame: RadialFrame) {
        let basis = frame.basis();

        assert_near(basis.right().length(), 1.0);
        assert_near(basis.up().length(), 1.0);
        assert_near(basis.forward().length(), 1.0);
        assert_near(basis.right().dot(basis.up()), 0.0);
        assert_near(basis.right().dot(basis.forward()), 0.0);
        assert_near(basis.up().dot(basis.forward()), 0.0);
        assert_near(basis.right().cross(basis.up()).dot(basis.forward()), 1.0);
    }

    #[test]
    fn frame_from_normal_builds_finite_right_handed_basis() {
        let frame = RadialFrame::from_normal(Vec3::ZERO, 2.0, Vec3::new(1.0, 2.0, 3.0)).unwrap();

        assert_frame_is_orthonormal(frame);
        assert_near(frame.outward().length(), 1.0);
        assert_vec_near(frame.basis().up(), frame.outward());
    }

    #[test]
    fn frame_from_latitude_longitude_places_equator_point() {
        let frame =
            RadialFrame::from_latitude_longitude(Vec3::new(1.0, 2.0, 3.0), 2.5, 0.0, 0.0).unwrap();

        assert_frame_is_orthonormal(frame);
        assert_vec_near(frame.surface_point(), Vec3::new(3.5, 2.0, 3.0));
    }

    #[test]
    fn poles_use_stable_auxiliary_axes() {
        let north =
            RadialFrame::from_latitude_longitude(Vec3::ZERO, 2.0, std::f32::consts::FRAC_PI_2, 0.0)
                .unwrap();
        let south = RadialFrame::from_latitude_longitude(
            Vec3::ZERO,
            2.0,
            -std::f32::consts::FRAC_PI_2,
            0.0,
        )
        .unwrap();

        assert_frame_is_orthonormal(north);
        assert_frame_is_orthonormal(south);
        assert_vec_near(north.surface_point(), Vec3::new(0.0, 2.0, 0.0));
        assert_vec_near(south.surface_point(), Vec3::new(0.0, -2.0, 0.0));
    }

    #[test]
    fn local_y_elevation_moves_outside_planet() {
        let frame = RadialFrame::from_normal(Vec3::ZERO, 2.0, Vec3::new(0.0, 0.0, 1.0)).unwrap();
        let elevated = frame.position(0.4, 0.25, -0.2);

        assert!(elevated.length() > frame.planet_radius());
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        assert_eq!(
            RadialFrame::from_normal(Vec3::new(f32::NAN, 0.0, 0.0), 1.0, Vec3::new(0.0, 1.0, 0.0)),
            Err(RadialFrameError::NonFiniteCenter)
        );
        assert_eq!(
            RadialFrame::from_normal(Vec3::ZERO, 0.0, Vec3::new(0.0, 1.0, 0.0)),
            Err(RadialFrameError::InvalidRadius)
        );
        assert_eq!(
            RadialFrame::from_normal(Vec3::ZERO, 1.0, Vec3::ZERO),
            Err(RadialFrameError::InvalidNormal)
        );
        assert_eq!(
            RadialFrame::from_latitude_longitude(Vec3::ZERO, 1.0, f32::NAN, 0.0),
            Err(RadialFrameError::InvalidNormal)
        );
    }
}
