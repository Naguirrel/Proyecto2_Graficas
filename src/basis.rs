use crate::math::Vec3;

const ORTHONORMAL_EPSILON: f32 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Basis3 {
    right: Vec3,
    up: Vec3,
    forward: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis3Error {
    NonFiniteAxis,
    ZeroAxis,
    NonUnitAxis,
    NonPerpendicularAxes,
    LeftHandedAxes,
    NonFiniteAngle,
}

impl Basis3 {
    pub fn identity() -> Self {
        Self {
            right: Vec3::new(1.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            forward: Vec3::new(0.0, 0.0, 1.0),
        }
    }

    pub fn new(right: Vec3, up: Vec3, forward: Vec3) -> Result<Self, Basis3Error> {
        validate_axis(right)?;
        validate_axis(up)?;
        validate_axis(forward)?;

        if !is_unit(right) || !is_unit(up) || !is_unit(forward) {
            return Err(Basis3Error::NonUnitAxis);
        }

        if right.dot(up).abs() > ORTHONORMAL_EPSILON
            || right.dot(forward).abs() > ORTHONORMAL_EPSILON
            || up.dot(forward).abs() > ORTHONORMAL_EPSILON
        {
            return Err(Basis3Error::NonPerpendicularAxes);
        }

        if right.cross(up).dot(forward) < 1.0 - ORTHONORMAL_EPSILON {
            return Err(Basis3Error::LeftHandedAxes);
        }

        Ok(Self { right, up, forward })
    }

    pub fn from_axis_angle(axis: Vec3, angle_radians: f32) -> Result<Self, Basis3Error> {
        if !angle_radians.is_finite() {
            return Err(Basis3Error::NonFiniteAngle);
        }

        validate_axis(axis)?;

        let axis = axis.normalized();
        let (sin_angle, cos_angle) = angle_radians.sin_cos();
        let rotate = |vector: Vec3| {
            vector * cos_angle
                + axis.cross(vector) * sin_angle
                + axis * (axis.dot(vector) * (1.0 - cos_angle))
        };

        Self::new(
            rotate(Vec3::new(1.0, 0.0, 0.0)),
            rotate(Vec3::new(0.0, 1.0, 0.0)),
            rotate(Vec3::new(0.0, 0.0, 1.0)),
        )
    }

    pub fn right(self) -> Vec3 {
        self.right
    }

    pub fn up(self) -> Vec3 {
        self.up
    }

    pub fn forward(self) -> Vec3 {
        self.forward
    }

    pub fn local_to_world_vector(self, vector: Vec3) -> Vec3 {
        self.right * vector.x + self.up * vector.y + self.forward * vector.z
    }

    pub fn world_to_local_vector(self, vector: Vec3) -> Vec3 {
        Vec3::new(
            vector.dot(self.right),
            vector.dot(self.up),
            vector.dot(self.forward),
        )
    }
}

fn validate_axis(axis: Vec3) -> Result<(), Basis3Error> {
    if !is_finite_vec3(axis) {
        return Err(Basis3Error::NonFiniteAxis);
    }

    if axis == Vec3::ZERO {
        return Err(Basis3Error::ZeroAxis);
    }

    Ok(())
}

fn is_unit(axis: Vec3) -> bool {
    (axis.length_squared() - 1.0).abs() <= ORTHONORMAL_EPSILON
}

fn is_finite_vec3(vector: Vec3) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{Basis3, Basis3Error};
    use crate::math::Vec3;

    fn assert_vec_near(left: Vec3, right: Vec3) {
        assert!(left.approx_eq(right), "{left:?} != {right:?}");
    }

    #[test]
    fn identity_preserves_local_vectors() {
        let basis = Basis3::identity();
        let vector = Vec3::new(1.0, 2.0, 3.0);

        assert_eq!(basis.local_to_world_vector(vector), vector);
        assert_eq!(basis.world_to_local_vector(vector), vector);
    }

    #[test]
    fn constructor_accepts_right_handed_orthonormal_axes() {
        let basis = Basis3::new(
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        assert_eq!(basis.right(), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(basis.up(), Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(basis.forward(), Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn constructor_rejects_non_finite_axes() {
        assert_eq!(
            Basis3::new(
                Vec3::new(f32::NAN, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ),
            Err(Basis3Error::NonFiniteAxis)
        );
    }

    #[test]
    fn constructor_rejects_zero_axis() {
        assert_eq!(
            Basis3::new(
                Vec3::ZERO,
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0)
            ),
            Err(Basis3Error::ZeroAxis)
        );
    }

    #[test]
    fn constructor_rejects_non_unit_axes() {
        assert_eq!(
            Basis3::new(
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ),
            Err(Basis3Error::NonUnitAxis)
        );
    }

    #[test]
    fn constructor_rejects_non_perpendicular_axes() {
        assert_eq!(
            Basis3::new(
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.2, 0.98, 0.0).normalized(),
                Vec3::new(0.0, 0.0, 1.0),
            ),
            Err(Basis3Error::NonPerpendicularAxes)
        );
    }

    #[test]
    fn constructor_rejects_left_handed_axes() {
        assert_eq!(
            Basis3::new(
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0),
            ),
            Err(Basis3Error::LeftHandedAxes)
        );
    }

    #[test]
    fn axis_angle_rotates_around_y_axis() {
        let basis =
            Basis3::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2).unwrap();

        assert_vec_near(
            basis.local_to_world_vector(Vec3::new(1.0, 0.0, 0.0)),
            Vec3::new(0.0, 0.0, -1.0),
        );
        assert_vec_near(
            basis.local_to_world_vector(Vec3::new(0.0, 0.0, 1.0)),
            Vec3::new(1.0, 0.0, 0.0),
        );
    }

    #[test]
    fn axis_angle_accepts_non_unit_axis() {
        let basis = Basis3::from_axis_angle(Vec3::new(0.0, 2.0, 0.0), 0.0).unwrap();

        assert_eq!(basis, Basis3::identity());
    }

    #[test]
    fn axis_angle_rejects_invalid_inputs() {
        assert_eq!(
            Basis3::from_axis_angle(Vec3::ZERO, 0.0),
            Err(Basis3Error::ZeroAxis)
        );
        assert_eq!(
            Basis3::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), f32::INFINITY),
            Err(Basis3Error::NonFiniteAngle)
        );
    }

    #[test]
    fn local_world_transforms_are_inverse_for_vectors() {
        let basis = Basis3::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.7).unwrap();
        let local = Vec3::new(0.4, -0.8, 1.7);
        let world = basis.local_to_world_vector(local);

        assert_vec_near(basis.world_to_local_vector(world), local);
    }
}
