use serde::{Deserialize, Serialize};

/// RGBA color (0.0 - 1.0 range)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color from RGBA components (0.0 - 1.0)
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new color from RGB (alpha = 1.0)
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// Create a color from 8-bit RGB values (0-255)
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba8(r, g, b, 255)
    }

    /// Create a color from 8-bit RGBA values (0-255)
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Create a color from hexadecimal (e.g., 0xRRGGBB)
    pub fn hex(hex: u32) -> Self {
        Self::rgb8(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        )
    }

    /// Create a color from hexadecimal with alpha (e.g., 0xRRGGBBAA)
    pub fn hex_alpha(hex: u32) -> Self {
        Self::rgba8(
            ((hex >> 24) & 0xFF) as u8,
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        )
    }

    // Common colors
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0, 1.0);
    pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0, 1.0);
    pub const CYAN: Self = Self::new(0.0, 1.0, 1.0, 1.0);
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let color = Color::new(0.5, 0.6, 0.7, 0.8);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.6);
        assert_eq!(color.b, 0.7);
        assert_eq!(color.a, 0.8);
    }

    #[test]
    fn test_color_rgb() {
        let color = Color::rgb(1.0, 0.5, 0.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_rgb8() {
        let color = Color::rgb8(255, 128, 0);
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5019).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_rgba8() {
        let color = Color::rgba8(255, 128, 64, 192);
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5019).abs() < 0.01);
        assert!((color.b - 0.251).abs() < 0.01);
        assert!((color.a - 0.753).abs() < 0.01);
    }

    #[test]
    fn test_color_hex() {
        let color = Color::hex(0xFF8000); // Orange
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5019).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_color_hex_alpha() {
        let color = Color::hex_alpha(0xFF8000C0);
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5019).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
        assert!((color.a - 0.753).abs() < 0.01);
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::TRANSPARENT, Color::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0, 1.0));
        assert_eq!(Color::YELLOW, Color::new(1.0, 1.0, 0.0, 1.0));
        assert_eq!(Color::MAGENTA, Color::new(1.0, 0.0, 1.0, 1.0));
        assert_eq!(Color::CYAN, Color::new(0.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn test_color_default() {
        let color = Color::default();
        assert_eq!(color, Color::WHITE);
    }

    #[test]
    fn test_color_serialization() {
        let color = Color::rgb(0.5, 0.6, 0.7);
        let serialized = serde_json::to_string(&color).unwrap();
        let deserialized: Color = serde_json::from_str(&serialized).unwrap();
        assert_eq!(color, deserialized);
    }

    #[test]
    fn test_color_equality() {
        let color1 = Color::rgb(1.0, 0.5, 0.0);
        let color2 = Color::rgb(1.0, 0.5, 0.0);
        let color3 = Color::rgb(0.0, 0.5, 1.0);
        assert_eq!(color1, color2);
        assert_ne!(color1, color3);
    }
}
