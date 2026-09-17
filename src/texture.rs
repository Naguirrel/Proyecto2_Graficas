use std::{fs, path::Path};

use crate::{color::Color, math::Vec2};

/// Vertical UV convention: `v = 0.0` samples the bottom row of the image and
/// `v = 1.0` samples the top row. PPM rows are stored top-to-bottom, so the
/// sampler flips `v` when converting UVs to image row indices.
pub const FALLBACK_TEXTURE_COLOR: Color = Color::new(1.0, 0.0, 1.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMode {
    Repeat,
    Clamp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureError {
    InvalidDimensions,
    InvalidPixelCount { expected: usize, actual: usize },
    InvalidUtf8,
    InvalidMagic,
    MissingToken(&'static str),
    InvalidNumber { token: String },
    InvalidMaxValue,
    InsufficientData { expected: usize, actual: usize },
    TrailingData { extra_components: usize },
    Io(String),
}

impl Texture {
    pub fn new(width: usize, height: usize, pixels: Vec<Color>) -> Result<Self, TextureError> {
        let expected = pixel_count(width, height).ok_or(TextureError::InvalidDimensions)?;

        if pixels.len() != expected {
            return Err(TextureError::InvalidPixelCount {
                expected,
                actual: pixels.len(),
            });
        }

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixel(&self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }

        self.pixels.get(y * self.width + x).copied()
    }

    pub fn from_ppm_file(path: impl AsRef<Path>) -> Result<Self, TextureError> {
        let bytes = fs::read(path).map_err(|error| TextureError::Io(error.to_string()))?;

        Self::from_ppm_bytes(&bytes)
    }

    pub fn from_ppm_text(text: &str) -> Result<Self, TextureError> {
        Self::parse_p3(text)
    }

    pub fn from_ppm_bytes(bytes: &[u8]) -> Result<Self, TextureError> {
        let text = std::str::from_utf8(bytes).map_err(|_| TextureError::InvalidUtf8)?;

        Self::from_ppm_text(text)
    }

    pub fn sample(&self, uv: Vec2, wrap_mode: WrapMode) -> Color {
        if self.width == 0
            || self.height == 0
            || self.pixels.is_empty()
            || !uv.u.is_finite()
            || !uv.v.is_finite()
        {
            return FALLBACK_TEXTURE_COLOR;
        }

        let u = wrap_coordinate(uv.u, wrap_mode);
        let v = wrap_coordinate(uv.v, wrap_mode);
        let x = nearest_index(u, self.width);
        let y = nearest_index(1.0 - v, self.height);

        self.pixel(x, y).unwrap_or(FALLBACK_TEXTURE_COLOR)
    }

    fn parse_p3(text: &str) -> Result<Self, TextureError> {
        let mut tokens = PpmTokens::new(text);
        let magic = tokens.next_required("magic")?;

        if magic != "P3" {
            return Err(TextureError::InvalidMagic);
        }

        let width = parse_usize(tokens.next_required("width")?)?;
        let height = parse_usize(tokens.next_required("height")?)?;
        let expected_pixels = pixel_count(width, height).ok_or(TextureError::InvalidDimensions)?;
        let max_value = parse_u32(tokens.next_required("max_value")?)?;

        if max_value == 0 {
            return Err(TextureError::InvalidMaxValue);
        }

        let expected_components = expected_pixels
            .checked_mul(3)
            .ok_or(TextureError::InvalidDimensions)?;
        let mut components = Vec::with_capacity(expected_components);

        while let Some(token) = tokens.next() {
            components.push(parse_u32(token)?);
        }

        if components.len() < expected_components {
            return Err(TextureError::InsufficientData {
                expected: expected_components,
                actual: components.len(),
            });
        }

        if components.len() > expected_components {
            return Err(TextureError::TrailingData {
                extra_components: components.len() - expected_components,
            });
        }

        let mut pixels = Vec::with_capacity(expected_pixels);

        for rgb in components.chunks_exact(3) {
            pixels.push(Color::new(
                normalize_component(rgb[0], max_value)?,
                normalize_component(rgb[1], max_value)?,
                normalize_component(rgb[2], max_value)?,
            ));
        }

        Self::new(width, height, pixels)
    }
}

struct PpmTokens<'a> {
    text: &'a str,
    index: usize,
}

impl<'a> PpmTokens<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, index: 0 }
    }

    fn next_required(&mut self, field: &'static str) -> Result<&'a str, TextureError> {
        self.next().ok_or(TextureError::MissingToken(field))
    }

    fn next(&mut self) -> Option<&'a str> {
        let bytes = self.text.as_bytes();

        loop {
            while self.index < bytes.len() && bytes[self.index].is_ascii_whitespace() {
                self.index += 1;
            }

            if self.index >= bytes.len() {
                return None;
            }

            if bytes[self.index] != b'#' {
                break;
            }

            while self.index < bytes.len() && bytes[self.index] != b'\n' {
                self.index += 1;
            }
        }

        let start = self.index;

        while self.index < bytes.len()
            && !bytes[self.index].is_ascii_whitespace()
            && bytes[self.index] != b'#'
        {
            self.index += 1;
        }

        Some(&self.text[start..self.index])
    }
}

fn pixel_count(width: usize, height: usize) -> Option<usize> {
    if width == 0 || height == 0 {
        return None;
    }

    width.checked_mul(height)
}

fn parse_usize(token: &str) -> Result<usize, TextureError> {
    let value = parse_u32(token)?;

    usize::try_from(value).map_err(|_| TextureError::InvalidDimensions)
}

fn parse_u32(token: &str) -> Result<u32, TextureError> {
    token.parse().map_err(|_| TextureError::InvalidNumber {
        token: token.to_string(),
    })
}

fn normalize_component(component: u32, max_value: u32) -> Result<f32, TextureError> {
    if component > max_value {
        return Err(TextureError::InvalidMaxValue);
    }

    Ok(component as f32 / max_value as f32)
}

fn wrap_coordinate(value: f32, wrap_mode: WrapMode) -> f32 {
    match wrap_mode {
        WrapMode::Repeat => value.rem_euclid(1.0),
        WrapMode::Clamp => value.clamp(0.0, 1.0),
    }
}

fn nearest_index(value: f32, size: usize) -> usize {
    let max_index = size - 1;

    ((value * max_index as f32).round() as usize).min(max_index)
}

#[cfg(test)]
mod tests {
    use super::{FALLBACK_TEXTURE_COLOR, Texture, TextureError, WrapMode};
    use crate::{color::Color, math::Vec2};

    const DEMO_TEXTURE: &str = include_str!("../assets/textures/demo_checker.ppm");
    const MATERIAL_TEXTURE_PATHS: [&str; 5] = [
        "assets/textures/seat_fabric.ppm",
        "assets/textures/theater_carpet.ppm",
        "assets/textures/brushed_metal.ppm",
        "assets/textures/transparent_plastic.ppm",
        "assets/textures/popcorn_cardboard.ppm",
    ];

    fn assert_color_near(left: Color, right: Color) {
        assert!(
            (left.r - right.r).abs() < 0.0001
                && (left.g - right.g).abs() < 0.0001
                && (left.b - right.b).abs() < 0.0001,
            "{left:?} != {right:?}"
        );
    }

    fn corner_texture() -> Texture {
        Texture::new(
            2,
            2,
            vec![
                Color::new(1.0, 0.0, 0.0),
                Color::new(0.0, 1.0, 0.0),
                Color::new(0.0, 0.0, 1.0),
                Color::new(1.0, 1.0, 1.0),
            ],
        )
        .unwrap()
    }

    #[test]
    fn constructor_accepts_matching_dimensions_and_pixels() {
        let texture = Texture::new(2, 1, vec![Color::BLACK, Color::WHITE]).unwrap();

        assert_eq!(texture.width(), 2);
        assert_eq!(texture.height(), 1);
        assert_eq!(texture.pixel(1, 0), Some(Color::WHITE));
        assert_eq!(texture.pixel(2, 0), None);
    }

    #[test]
    fn constructor_rejects_wrong_pixel_count() {
        assert_eq!(
            Texture::new(2, 2, vec![Color::WHITE]).unwrap_err(),
            TextureError::InvalidPixelCount {
                expected: 4,
                actual: 1
            }
        );
    }

    #[test]
    fn parser_accepts_valid_p3() {
        let texture = Texture::from_ppm_text("P3 2 1 255 255 0 0 0 255 0").unwrap();

        assert_eq!(texture.width(), 2);
        assert_eq!(texture.height(), 1);
        assert_eq!(texture.pixel(0, 0), Some(Color::new(1.0, 0.0, 0.0)));
        assert_eq!(texture.pixel(1, 0), Some(Color::new(0.0, 1.0, 0.0)));
    }

    #[test]
    fn parser_accepts_comments() {
        let texture =
            Texture::from_ppm_text("P3\n# palette\n1 1\n255\n# pixel\n0 0 255\n").unwrap();

        assert_eq!(texture.pixel(0, 0), Some(Color::new(0.0, 0.0, 1.0)));
    }

    #[test]
    fn parser_accepts_whitespace_and_newlines() {
        let texture = Texture::from_ppm_text("P3\t1\n2\n255\n255\n255 255\n0 0 0\n").unwrap();

        assert_eq!(texture.width(), 1);
        assert_eq!(texture.height(), 2);
    }

    #[test]
    fn parser_rejects_invalid_magic_number() {
        assert_eq!(
            Texture::from_ppm_text("P6 1 1 255 0 0 0").unwrap_err(),
            TextureError::InvalidMagic
        );
    }

    #[test]
    fn parser_rejects_invalid_dimensions() {
        assert_eq!(
            Texture::from_ppm_text("P3 0 1 255 0 0 0").unwrap_err(),
            TextureError::InvalidDimensions
        );
    }

    #[test]
    fn parser_rejects_invalid_max_value() {
        assert_eq!(
            Texture::from_ppm_text("P3 1 1 0 0 0 0").unwrap_err(),
            TextureError::InvalidMaxValue
        );
        assert_eq!(
            Texture::from_ppm_text("P3 1 1 10 11 0 0").unwrap_err(),
            TextureError::InvalidMaxValue
        );
    }

    #[test]
    fn parser_rejects_insufficient_data() {
        assert_eq!(
            Texture::from_ppm_text("P3 2 1 255 255 0 0").unwrap_err(),
            TextureError::InsufficientData {
                expected: 6,
                actual: 3
            }
        );
    }

    #[test]
    fn parser_rejects_trailing_components() {
        assert_eq!(
            Texture::from_ppm_text("P3 1 1 255 255 0 0 1").unwrap_err(),
            TextureError::TrailingData {
                extra_components: 1
            }
        );
    }

    #[test]
    fn parser_rejects_non_numeric_components() {
        assert_eq!(
            Texture::from_ppm_text("P3 1 1 255 255 nope 0").unwrap_err(),
            TextureError::InvalidNumber {
                token: "nope".to_string()
            }
        );
    }

    #[test]
    fn parser_normalizes_colors_by_max_value() {
        let texture = Texture::from_ppm_text("P3 1 1 100 25 50 100").unwrap();

        assert_color_near(texture.pixel(0, 0).unwrap(), Color::new(0.25, 0.5, 1.0));
    }

    #[test]
    fn sampling_four_corners_returns_expected_pixels() {
        let texture = corner_texture();

        assert_eq!(
            texture.sample(Vec2::new(0.0, 1.0), WrapMode::Clamp),
            Color::new(1.0, 0.0, 0.0)
        );
        assert_eq!(
            texture.sample(Vec2::new(1.0, 1.0), WrapMode::Clamp),
            Color::new(0.0, 1.0, 0.0)
        );
        assert_eq!(
            texture.sample(Vec2::new(0.0, 0.0), WrapMode::Clamp),
            Color::new(0.0, 0.0, 1.0)
        );
        assert_eq!(
            texture.sample(Vec2::new(1.0, 0.0), WrapMode::Clamp),
            Color::new(1.0, 1.0, 1.0)
        );
    }

    #[test]
    fn vertical_orientation_uses_v_zero_as_image_bottom() {
        let texture = corner_texture();

        assert_eq!(
            texture.sample(Vec2::new(0.0, 0.0), WrapMode::Clamp),
            Color::new(0.0, 0.0, 1.0)
        );
        assert_eq!(
            texture.sample(Vec2::new(0.0, 1.0), WrapMode::Clamp),
            Color::new(1.0, 0.0, 0.0)
        );
    }

    #[test]
    fn repeat_wraps_uv_greater_than_one() {
        let texture = corner_texture();

        assert_eq!(
            texture.sample(Vec2::new(1.75, 1.75), WrapMode::Repeat),
            Color::new(0.0, 1.0, 0.0)
        );
    }

    #[test]
    fn repeat_wraps_negative_uv() {
        let texture = corner_texture();

        assert_eq!(
            texture.sample(Vec2::new(-0.25, -0.75), WrapMode::Repeat),
            Color::new(1.0, 1.0, 1.0)
        );
    }

    #[test]
    fn clamp_limits_uv_outside_range() {
        let texture = corner_texture();

        assert_eq!(
            texture.sample(Vec2::new(2.0, -1.0), WrapMode::Clamp),
            Color::new(1.0, 1.0, 1.0)
        );
    }

    #[test]
    fn one_uv_does_not_leave_pixel_buffer() {
        let texture = corner_texture();

        assert_eq!(
            texture.sample(Vec2::new(1.0, 1.0), WrapMode::Clamp),
            Color::new(0.0, 1.0, 0.0)
        );
    }

    #[test]
    fn non_finite_uv_returns_fallback_color() {
        let texture = corner_texture();

        assert_eq!(
            texture.sample(Vec2::new(f32::NAN, 0.0), WrapMode::Clamp),
            FALLBACK_TEXTURE_COLOR
        );
        assert_eq!(
            texture.sample(Vec2::new(0.0, f32::INFINITY), WrapMode::Repeat),
            FALLBACK_TEXTURE_COLOR
        );
    }

    #[test]
    fn one_by_one_texture_can_be_sampled() {
        let texture = Texture::new(1, 1, vec![Color::new(0.25, 0.5, 0.75)]).unwrap();

        assert_eq!(
            texture.sample(Vec2::new(42.0, -7.0), WrapMode::Repeat),
            Color::new(0.25, 0.5, 0.75)
        );
    }

    #[test]
    fn demo_texture_file_loads() {
        let texture = Texture::from_ppm_file("assets/textures/demo_checker.ppm").unwrap();

        assert_eq!(texture.width(), 4);
        assert_eq!(texture.height(), 4);
    }

    #[test]
    fn demo_texture_embedded_text_parses() {
        let texture = Texture::from_ppm_text(DEMO_TEXTURE).unwrap();

        assert_eq!(texture.width(), 4);
        assert_eq!(texture.height(), 4);
    }

    #[test]
    fn five_material_texture_files_load() {
        for path in MATERIAL_TEXTURE_PATHS {
            let texture = Texture::from_ppm_file(path).unwrap();

            assert!(texture.width() > 0);
            assert!(texture.height() > 0);
        }
    }

    #[test]
    fn five_material_texture_files_have_valid_pixels() {
        for path in MATERIAL_TEXTURE_PATHS {
            let texture = Texture::from_ppm_file(path).unwrap();

            assert!(texture.pixel(0, 0).is_some());
            assert!(
                texture
                    .pixel(texture.width() - 1, texture.height() - 1)
                    .is_some()
            );
            assert_eq!(texture.pixel(texture.width(), 0), None);
        }
    }
}
