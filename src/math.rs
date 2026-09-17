const EPSILON: f32 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn zero() -> Self {
        Self::ZERO
    }

    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Returns `Vec3::ZERO` for zero-length or non-finite vectors so ray
    /// directions never acquire NaN/Inf values from normalization.
    pub fn normalized(self) -> Self {
        let length = self.length();

        if length <= EPSILON || !length.is_finite() {
            return Self::ZERO;
        }

        self / length
    }

    pub fn reflect(self, normal: Self) -> Self {
        let normal = normal.normalized();
        self - normal * (2.0 * self.dot(normal))
    }

    pub fn approx_eq(self, rhs: Self) -> bool {
        (self.x - rhs.x).abs() < EPSILON
            && (self.y - rhs.y).abs() < EPSILON
            && (self.z - rhs.z).abs() < EPSILON
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl std::ops::Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl std::ops::Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl std::ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3;

    const TEST_EPSILON: f32 = 0.0001;

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < TEST_EPSILON, "{left} != {right}");
    }

    fn assert_vec_near(left: Vec3, right: Vec3) {
        assert!(left.approx_eq(right), "{left:?} != {right:?}");
    }

    #[test]
    fn adds_and_subtracts_vectors() {
        let left = Vec3::new(2.0, -1.0, 4.0);
        let right = Vec3::new(-3.0, 5.0, 0.5);

        assert_eq!(left + right, Vec3::new(-1.0, 4.0, 4.5));
        assert_eq!(left - right, Vec3::new(5.0, -6.0, 3.5));
    }

    #[test]
    fn dot_product_combines_components() {
        let left = Vec3::new(1.0, 2.0, 3.0);
        let right = Vec3::new(4.0, -5.0, 6.0);

        assert_near(left.dot(right), 12.0);
    }

    #[test]
    fn cross_product_returns_perpendicular_vector() {
        let x_axis = Vec3::new(1.0, 0.0, 0.0);
        let y_axis = Vec3::new(0.0, 1.0, 0.0);

        assert_eq!(x_axis.cross(y_axis), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn length_uses_squared_components() {
        let vector = Vec3::new(2.0, 3.0, 6.0);

        assert_near(vector.length_squared(), 49.0);
        assert_near(vector.length(), 7.0);
    }

    #[test]
    fn normalizes_vector_to_unit_length() {
        let vector = Vec3::new(0.0, 3.0, 4.0).normalized();

        assert_near(vector.length(), 1.0);
        assert_near(vector.y, 0.6);
        assert_near(vector.z, 0.8);
    }

    #[test]
    fn normalizing_zero_vector_returns_zero() {
        let vector = Vec3::ZERO.normalized();

        assert_eq!(vector, Vec3::ZERO);
        assert!(vector.x.is_finite());
        assert!(vector.y.is_finite());
        assert!(vector.z.is_finite());
    }

    #[test]
    fn reflects_vector_around_normal() {
        let incoming = Vec3::new(1.0, -1.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);

        assert_vec_near(incoming.reflect(normal), Vec3::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn supports_negation_and_assignment_operators() {
        let mut vector = Vec3::new(1.0, 2.0, 3.0);

        vector += Vec3::new(1.0, 0.0, -1.0);
        vector -= Vec3::new(0.0, 1.0, 1.0);
        vector *= 2.0;

        assert_eq!(vector, Vec3::new(4.0, 2.0, 2.0));
        assert_eq!(-vector, Vec3::new(-4.0, -2.0, -2.0));
    }
}
