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

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();

        if length == 0.0 {
            return Self::ZERO;
        }

        self / length
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

impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl std::ops::Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3;

    const EPSILON: f32 = 0.0001;

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < EPSILON, "{left} != {right}");
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
    fn normalized_vector_has_unit_length() {
        let vector = Vec3::new(0.0, 3.0, 4.0).normalized();

        assert_near(vector.length(), 1.0);
        assert_near(vector.y, 0.6);
        assert_near(vector.z, 0.8);
    }
}
