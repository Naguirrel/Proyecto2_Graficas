use crate::{color::Color, math::Vec3};

/// Point light using practical scene units.
///
/// `intensity` is an artist-controlled multiplier applied with distance
/// attenuation by the renderer. It is clamped to a non-negative finite value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl PointLight {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Self {
            position: finite_position_or_origin(position),
            color: color.clamped(),
            intensity: safe_intensity(intensity),
        }
    }
}

fn finite_position_or_origin(position: Vec3) -> Vec3 {
    if position.x.is_finite() && position.y.is_finite() && position.z.is_finite() {
        position
    } else {
        Vec3::ZERO
    }
}

fn safe_intensity(intensity: f32) -> f32 {
    if intensity.is_finite() && intensity >= 0.0 {
        intensity
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::PointLight;
    use crate::{color::Color, math::Vec3};

    #[test]
    fn preserves_valid_position_and_color() {
        let light = PointLight::new(Vec3::new(1.0, 2.0, 3.0), Color::new(0.2, 0.4, 0.6), 3.0);

        assert_eq!(light.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(light.color, Color::new(0.2, 0.4, 0.6));
        assert_eq!(light.intensity, 3.0);
    }

    #[test]
    fn negative_intensity_becomes_safe() {
        let light = PointLight::new(Vec3::ZERO, Color::WHITE, -4.0);

        assert_eq!(light.intensity, 0.0);
    }

    #[test]
    fn non_finite_intensity_does_not_contaminate_render() {
        let light = PointLight::new(
            Vec3::new(f32::INFINITY, 0.0, 0.0),
            Color::new(2.0, -1.0, 0.5),
            f32::NAN,
        );

        assert_eq!(light.position, Vec3::ZERO);
        assert_eq!(light.color, Color::new(1.0, 0.0, 0.5));
        assert_eq!(light.intensity, 0.0);
    }
}
