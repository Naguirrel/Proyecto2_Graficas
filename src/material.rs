use crate::{color::Color, math::Vec2, texture::WrapMode};

/// Surface parameters used by the CPU raytracer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material {
    /// Base diffuse color, stored as normalized RGB components.
    pub albedo: Color,
    /// Optional texture assigned by `Scene`.
    pub texture_id: Option<usize>,
    /// Per-material UV scale applied before sampling the texture.
    pub uv_scale: Vec2,
    /// Texture address mode.
    pub wrap_mode: WrapMode,
    /// Strength of future specular highlights. Expected range: `0.0..=1.0`.
    pub specular_strength: f32,
    /// Future specular exponent. Negative or non-finite values are clamped to `1.0`.
    pub shininess: f32,
    /// Future mirror contribution. Expected range: `0.0..=1.0`.
    pub reflectivity: f32,
    /// Future transmission contribution. Expected range: `0.0..=1.0`.
    pub transparency: f32,
    /// Future refraction ratio. Values below air or non-finite values become `1.0`.
    pub refractive_index: f32,
    /// Additive emitted color for future emissive surfaces.
    pub emission: Color,
}

impl Material {
    pub fn new(
        albedo: Color,
        specular_strength: f32,
        shininess: f32,
        reflectivity: f32,
        transparency: f32,
        refractive_index: f32,
        emission: Color,
    ) -> Self {
        Self {
            albedo: albedo.clamped(),
            texture_id: None,
            uv_scale: Vec2::new(1.0, 1.0),
            wrap_mode: WrapMode::Repeat,
            specular_strength: clamp_unit(specular_strength),
            shininess: positive_or(shininess, 1.0),
            reflectivity: clamp_unit(reflectivity),
            transparency: clamp_unit(transparency),
            refractive_index: valid_refractive_index(refractive_index),
            emission: emission.clamped(),
        }
    }

    pub fn diffuse(albedo: Color) -> Self {
        Self::new(albedo, 0.2, 16.0, 0.0, 0.0, 1.0, Color::BLACK)
    }

    pub fn with_texture(mut self, texture_id: usize, uv_scale: Vec2, wrap_mode: WrapMode) -> Self {
        self.texture_id = Some(texture_id);
        self.uv_scale = sanitize_uv_scale(uv_scale);
        self.wrap_mode = wrap_mode;
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::diffuse(Color::new(0.8, 0.8, 0.8))
    }
}

fn clamp_unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn positive_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        fallback
    }
}

fn valid_refractive_index(value: f32) -> f32 {
    if value.is_finite() && value >= 1.0 {
        value
    } else {
        1.0
    }
}

fn sanitize_uv_scale(uv_scale: Vec2) -> Vec2 {
    Vec2::new(
        positive_or(uv_scale.u, 1.0).max(0.0001),
        positive_or(uv_scale.v, 1.0).max(0.0001),
    )
}

#[cfg(test)]
mod tests {
    use super::Material;
    use crate::{color::Color, math::Vec2, texture::WrapMode};

    #[test]
    fn default_material_has_valid_values() {
        let material = Material::default();

        assert!((0.0..=1.0).contains(&material.albedo.r));
        assert!((0.0..=1.0).contains(&material.specular_strength));
        assert!(material.shininess >= 0.0);
        assert!((0.0..=1.0).contains(&material.reflectivity));
        assert!((0.0..=1.0).contains(&material.transparency));
        assert!(material.refractive_index >= 1.0);
        assert_eq!(material.texture_id, None);
        assert_eq!(material.uv_scale, Vec2::new(1.0, 1.0));
        assert_eq!(material.wrap_mode, WrapMode::Repeat);
    }

    #[test]
    fn untextured_material_keeps_albedo() {
        let material = Material::diffuse(Color::new(0.2, 0.3, 0.4));

        assert_eq!(material.texture_id, None);
        assert_eq!(material.albedo, Color::new(0.2, 0.3, 0.4));
    }

    #[test]
    fn textured_material_keeps_texture_id() {
        let material =
            Material::diffuse(Color::WHITE).with_texture(3, Vec2::new(2.0, 4.0), WrapMode::Clamp);

        assert_eq!(material.texture_id, Some(3));
        assert_eq!(material.uv_scale, Vec2::new(2.0, 4.0));
        assert_eq!(material.wrap_mode, WrapMode::Clamp);
    }

    #[test]
    fn invalid_uv_scale_is_sanitized() {
        let material = Material::diffuse(Color::WHITE).with_texture(
            0,
            Vec2::new(f32::NAN, f32::INFINITY),
            WrapMode::Repeat,
        );

        assert_eq!(material.uv_scale, Vec2::new(1.0, 1.0));
    }

    #[test]
    fn reflectivity_is_clamped_to_unit_range() {
        assert_eq!(
            Material::new(Color::WHITE, 0.0, 1.0, 2.0, 0.0, 1.0, Color::BLACK).reflectivity,
            1.0
        );
        assert_eq!(
            Material::new(Color::WHITE, 0.0, 1.0, -1.0, 0.0, 1.0, Color::BLACK).reflectivity,
            0.0
        );
    }

    #[test]
    fn transparency_is_clamped_to_unit_range() {
        assert_eq!(
            Material::new(Color::WHITE, 0.0, 1.0, 0.0, 4.0, 1.0, Color::BLACK).transparency,
            1.0
        );
        assert_eq!(
            Material::new(Color::WHITE, 0.0, 1.0, 0.0, -2.0, 1.0, Color::BLACK).transparency,
            0.0
        );
    }

    #[test]
    fn shininess_cannot_be_negative() {
        assert_eq!(
            Material::new(Color::WHITE, 0.0, -8.0, 0.0, 0.0, 1.0, Color::BLACK).shininess,
            1.0
        );
    }

    #[test]
    fn refractive_index_stays_valid_and_finite() {
        assert_eq!(
            Material::new(Color::WHITE, 0.0, 1.0, 0.0, 0.0, f32::NAN, Color::BLACK)
                .refractive_index,
            1.0
        );
        assert_eq!(
            Material::new(Color::WHITE, 0.0, 1.0, 0.0, 0.0, 0.2, Color::BLACK).refractive_index,
            1.0
        );
    }

    #[test]
    fn emission_and_albedo_are_clamped_colors() {
        let material = Material::new(
            Color::new(2.0, -1.0, 0.5),
            0.0,
            1.0,
            0.0,
            0.0,
            1.0,
            Color::new(-1.0, 0.25, 4.0),
        );

        assert_eq!(material.albedo, Color::new(1.0, 0.0, 0.5));
        assert_eq!(material.emission, Color::new(0.0, 0.25, 1.0));
    }
}
