use std::{f32::consts::PI, path::Path};

use crate::{
    color::Color,
    math::{Vec2, Vec3},
    texture::{Texture, TextureError, WrapMode},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Skybox {
    texture: Texture,
    intensity: f32,
    horizontal_rotation: f32,
}

impl Skybox {
    pub fn new(texture: Texture) -> Self {
        Self {
            texture,
            intensity: 1.0,
            horizontal_rotation: 0.0,
        }
    }

    pub fn from_ppm_file(path: impl AsRef<Path>) -> Result<Self, TextureError> {
        Ok(Self::new(Texture::from_ppm_file(path)?))
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = sanitize_intensity(intensity);
        self
    }

    pub fn with_horizontal_rotation(mut self, horizontal_rotation: f32) -> Self {
        self.horizontal_rotation = sanitize_rotation(horizontal_rotation);
        self
    }

    pub fn sample_direction(&self, direction: Vec3) -> Color {
        let Some(uv) = self.direction_to_uv(direction) else {
            return Color::BLACK;
        };

        (self.texture.sample(uv, WrapMode::Clamp) * self.intensity).clamped()
    }

    pub(crate) fn direction_to_uv(&self, direction: Vec3) -> Option<Vec2> {
        let direction = direction.normalized();

        if direction == Vec3::ZERO {
            return None;
        }

        let u = (0.5 + direction.z.atan2(direction.x) / (2.0 * PI) + self.horizontal_rotation)
            .rem_euclid(1.0);
        let v = (0.5 + direction.y.clamp(-1.0, 1.0).asin() / PI).clamp(0.0, 1.0);

        Some(Vec2::new(u, v))
    }

    pub fn intensity(&self) -> f32 {
        self.intensity
    }

    pub fn horizontal_rotation(&self) -> f32 {
        self.horizontal_rotation
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}

fn sanitize_intensity(intensity: f32) -> f32 {
    if intensity.is_finite() {
        intensity.max(0.0)
    } else {
        1.0
    }
}

fn sanitize_rotation(horizontal_rotation: f32) -> f32 {
    if horizontal_rotation.is_finite() {
        horizontal_rotation.rem_euclid(1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::Skybox;
    use crate::{
        color::Color,
        math::Vec3,
        texture::{FALLBACK_TEXTURE_COLOR, Texture},
    };

    fn test_skybox() -> Skybox {
        Skybox::new(
            Texture::new(
                4,
                3,
                vec![
                    Color::new(0.1, 0.1, 0.5),
                    Color::new(0.2, 0.1, 0.5),
                    Color::new(0.3, 0.1, 0.5),
                    Color::new(0.1, 0.1, 0.5),
                    Color::new(0.1, 0.5, 0.1),
                    Color::new(0.2, 0.5, 0.1),
                    Color::new(0.3, 0.5, 0.1),
                    Color::new(0.1, 0.5, 0.1),
                    Color::new(0.5, 0.1, 0.1),
                    Color::new(0.5, 0.2, 0.1),
                    Color::new(0.5, 0.3, 0.1),
                    Color::new(0.5, 0.1, 0.1),
                ],
            )
            .unwrap(),
        )
    }

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < 0.0001, "{left} != {right}");
    }

    fn assert_color_near(left: Color, right: Color) {
        assert!(
            (left.r - right.r).abs() < 0.0001
                && (left.g - right.g).abs() < 0.0001
                && (left.b - right.b).abs() < 0.0001,
            "{left:?} != {right:?}"
        );
    }

    #[test]
    fn upward_direction_maps_to_top_zone() {
        let uv = test_skybox()
            .direction_to_uv(Vec3::new(0.0, 1.0, 0.0))
            .unwrap();

        assert!(uv.v > 0.99);
    }

    #[test]
    fn downward_direction_maps_to_bottom_zone() {
        let uv = test_skybox()
            .direction_to_uv(Vec3::new(0.0, -1.0, 0.0))
            .unwrap();

        assert!(uv.v < 0.01);
    }

    #[test]
    fn horizontal_direction_maps_near_middle_height() {
        let uv = test_skybox()
            .direction_to_uv(Vec3::new(1.0, 0.0, 0.0))
            .unwrap();

        assert_near(uv.v, 0.5);
    }

    #[test]
    fn cardinal_directions_have_coherent_u_values() {
        let skybox = test_skybox();

        assert_near(
            skybox.direction_to_uv(Vec3::new(1.0, 0.0, 0.0)).unwrap().u,
            0.5,
        );
        assert_near(
            skybox.direction_to_uv(Vec3::new(0.0, 0.0, 1.0)).unwrap().u,
            0.75,
        );
        assert_near(
            skybox.direction_to_uv(Vec3::new(0.0, 0.0, -1.0)).unwrap().u,
            0.25,
        );
        assert_near(
            skybox.direction_to_uv(Vec3::new(-1.0, 0.0, 0.0)).unwrap().u,
            0.0,
        );
    }

    #[test]
    fn horizontal_seam_repeats() {
        let skybox = test_skybox();
        let left = skybox.sample_direction(Vec3::new(-1.0, 0.0, 0.00001));
        let right = skybox.sample_direction(Vec3::new(-1.0, 0.0, -0.00001));

        assert_color_near(left, right);
    }

    #[test]
    fn horizontal_rotation_changes_u() {
        let base = test_skybox();
        let rotated = test_skybox().with_horizontal_rotation(0.25);

        assert_ne!(
            base.direction_to_uv(Vec3::new(1.0, 0.0, 0.0)).unwrap().u,
            rotated.direction_to_uv(Vec3::new(1.0, 0.0, 0.0)).unwrap().u
        );
    }

    #[test]
    fn horizontal_rotation_preserves_v() {
        let base = test_skybox();
        let rotated = test_skybox().with_horizontal_rotation(0.25);

        assert_near(
            base.direction_to_uv(Vec3::new(0.3, 0.4, 0.5)).unwrap().v,
            rotated.direction_to_uv(Vec3::new(0.3, 0.4, 0.5)).unwrap().v,
        );
    }

    #[test]
    fn zero_direction_returns_safe_fallback() {
        assert_eq!(test_skybox().sample_direction(Vec3::ZERO), Color::BLACK);
    }

    #[test]
    fn non_finite_direction_returns_safe_fallback() {
        assert_eq!(
            test_skybox().sample_direction(Vec3::new(f32::NAN, 0.0, 1.0)),
            Color::BLACK
        );
    }

    #[test]
    fn zero_intensity_produces_black() {
        let color = test_skybox()
            .with_intensity(0.0)
            .sample_direction(Vec3::new(1.0, 0.0, 0.0));

        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn positive_intensity_scales_color() {
        let skybox = test_skybox();
        let dim = test_skybox().with_intensity(0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);

        assert_color_near(
            dim.sample_direction(direction),
            skybox.sample_direction(direction) * 0.5,
        );
    }

    #[test]
    fn sampled_color_stays_in_unit_range() {
        let color = test_skybox()
            .with_intensity(3.0)
            .sample_direction(Vec3::new(1.0, 0.0, 0.0));

        assert!((0.0..=1.0).contains(&color.r));
        assert!((0.0..=1.0).contains(&color.g));
        assert!((0.0..=1.0).contains(&color.b));
    }

    #[test]
    fn invalid_settings_are_sanitized() {
        let skybox = test_skybox()
            .with_intensity(f32::NAN)
            .with_horizontal_rotation(f32::INFINITY);

        assert_near(skybox.intensity(), 1.0);
        assert_near(skybox.horizontal_rotation(), 0.0);
    }

    #[test]
    fn fallback_texture_color_is_not_returned_for_valid_direction() {
        assert_ne!(
            test_skybox().sample_direction(Vec3::new(1.0, 0.0, 0.0)),
            FALLBACK_TEXTURE_COLOR
        );
    }

    #[test]
    fn night_cinema_skybox_file_loads() {
        let skybox = Skybox::from_ppm_file("assets/textures/night_cinema_skybox.ppm").unwrap();

        assert_eq!(skybox.texture().width(), 16);
        assert_eq!(skybox.texture().height(), 8);
    }
}
