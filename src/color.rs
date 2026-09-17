#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        self * (1.0 - t) + other * t
    }

    pub fn to_u32(self) -> u32 {
        let r = channel_to_u8(self.r);
        let g = channel_to_u8(self.g);
        let b = channel_to_u8(self.b);

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

fn channel_to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn converts_color_to_rgb_buffer_value() {
        assert_eq!(Color::new(1.0, 0.5, 0.0).to_u32(), 0xff8000);
    }
}
