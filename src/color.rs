/// RGB color stored as normalized floating-point components in `0.0..=1.0`.
/// Arithmetic may temporarily exceed that range; framebuffer conversion clamps
/// every channel safely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b)
    }

    pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b).clamped()
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        self * (1.0 - t) + other * t
    }

    pub fn component_mul(self, rhs: Self) -> Self {
        Self::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }

    pub fn clamped(self) -> Self {
        Self::new(
            clamp_channel(self.r),
            clamp_channel(self.g),
            clamp_channel(self.b),
        )
    }

    pub fn to_u32(self) -> u32 {
        let color = self.clamped();
        let r = channel_to_u8(color.r);
        let g = channel_to_u8(color.g);
        let b = channel_to_u8(color.b);

        ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }
}

impl std::ops::Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl std::ops::Mul<Color> for Color {
    type Output = Self;

    fn mul(self, rhs: Color) -> Self::Output {
        self.component_mul(rhs)
    }
}

impl std::ops::Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}

impl std::ops::AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

fn clamp_channel(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn channel_to_u8(value: f32) -> u8 {
    (value * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn converts_color_to_rgb_buffer_value() {
        assert_eq!(Color::new(1.0, 0.5, 0.0).to_u32(), 0xff8000);
    }

    #[test]
    fn clamps_values_outside_display_range() {
        let color = Color::new(-0.25, 1.25, f32::INFINITY);

        assert_eq!(color.clamped(), Color::new(0.0, 1.0, 0.0));
        assert_eq!(color.to_u32(), 0x00ff00);
    }

    #[test]
    fn multiplies_color_by_scalar() {
        let color = Color::new(0.2, 0.4, 0.8) * 0.5;

        assert_eq!(color, Color::new(0.1, 0.2, 0.4));
    }

    #[test]
    fn multiplies_color_components() {
        let left = Color::new(0.5, 0.25, 0.75);
        let right = Color::new(0.2, 0.8, 0.4);

        assert_eq!(left * right, Color::new(0.1, 0.2, 0.3));
    }

    #[test]
    fn from_f32_clamps_to_normalized_representation() {
        assert_eq!(Color::from_f32(1.5, -1.0, 0.25), Color::new(1.0, 0.0, 0.25));
    }
}
