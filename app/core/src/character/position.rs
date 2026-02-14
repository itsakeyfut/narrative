use serde::{Deserialize, Serialize};

/// Character position on screen
///
/// # Precision
///
/// The `Custom` variant stores position as `u8` (0-100) internally for efficient serialization.
/// This provides 101 discrete position levels with 1% precision (0.01 increments when converted to f32).
/// When creating custom positions via `custom()`, f32 values are rounded to the nearest percent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum CharacterPosition {
    /// Far left position (10% from left edge)
    FarLeft,
    /// Left position (25% from left edge)
    Left,
    /// Center position (50% from left edge)
    #[default]
    Center,
    /// Right position (75% from left edge)
    Right,
    /// Far right position (90% from left edge)
    FarRight,
    /// Custom position stored as percentage (0-100)
    ///
    /// Internally stores position as `u8` (0-100) for compact serialization.
    /// Convert to/from f32 (0.0-1.0) using `custom()` and `x_percent()`.
    /// Precision: 1% (0.01) increments, 101 discrete levels total.
    Custom(u8),
    /// Fixed pixel position from left edge at reference resolution (1280x720)
    ///
    /// This position is specified in pixels for the reference resolution (720p)
    /// and automatically scales proportionally with the actual screen size.
    /// For example, Fixed(150.0) means 150px from the left at 720p,
    /// and will scale to 225px at 1080p (1.5x scaling).
    Fixed(f32),
}

impl CharacterPosition {
    /// Create a custom position from a percentage (0.0-1.0)
    ///
    /// Values outside the 0.0-1.0 range are clamped.
    /// Input is rounded to the nearest 1% (0.01) due to internal u8 storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrative_core::CharacterPosition;
    ///
    /// let pos = CharacterPosition::custom(0.33);  // Stored as 33
    /// assert_eq!(pos.x_percent(), 0.33);
    ///
    /// let pos = CharacterPosition::custom(1.5);   // Clamped to 1.0, stored as 100
    /// assert_eq!(pos.x_percent(), 1.0);
    /// ```
    pub fn custom(x: f32) -> Self {
        let clamped = x.clamp(0.0, 1.0);
        Self::Custom((clamped * 100.0) as u8)
    }

    /// Get the x position as a percentage (0.0-1.0)
    ///
    /// Returns the normalized horizontal position from the left edge of the screen.
    /// For `Custom` positions, precision is limited to 0.01 increments.
    /// For `Fixed` positions, returns 0.0 (not applicable for percentage-based calculations).
    pub fn x_percent(self) -> f32 {
        match self {
            Self::FarLeft => 0.1,
            Self::Left => 0.25,
            Self::Center => 0.5,
            Self::Right => 0.75,
            Self::FarRight => 0.9,
            Self::Custom(percent) => percent as f32 / 100.0,
            Self::Fixed(_) => 0.0, // Not applicable for fixed pixel positions
        }
    }

    /// Get the position name
    pub fn name(&self) -> &str {
        match self {
            Self::FarLeft => "far_left",
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::FarRight => "far_right",
            Self::Custom(_) => "custom",
            Self::Fixed(_) => "fixed",
        }
    }
}

impl From<&str> for CharacterPosition {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "far_left" | "farleft" => Self::FarLeft,
            "left" => Self::Left,
            "center" | "middle" => Self::Center,
            "right" => Self::Right,
            "far_right" | "farright" => Self::FarRight,
            _ => Self::Center,
        }
    }
}

impl From<String> for CharacterPosition {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_default() {
        let pos = CharacterPosition::default();
        assert_eq!(pos, CharacterPosition::Center);
    }

    #[test]
    fn test_position_variants() {
        assert_eq!(CharacterPosition::Left, CharacterPosition::Left);
        assert_ne!(CharacterPosition::Left, CharacterPosition::Right);
        assert_ne!(CharacterPosition::FarLeft, CharacterPosition::Left);
    }

    #[test]
    fn test_position_custom() {
        let pos = CharacterPosition::custom(0.3);
        assert_eq!(pos, CharacterPosition::Custom(30));
    }

    #[test]
    fn test_position_custom_clamping_low() {
        let pos = CharacterPosition::custom(-0.5);
        assert_eq!(pos, CharacterPosition::Custom(0));
    }

    #[test]
    fn test_position_custom_clamping_high() {
        let pos = CharacterPosition::custom(1.5);
        assert_eq!(pos, CharacterPosition::Custom(100));
    }

    #[test]
    fn test_position_custom_valid_range() {
        let pos = CharacterPosition::custom(0.75);
        assert_eq!(pos, CharacterPosition::Custom(75));
    }

    #[test]
    fn test_position_x_percent_far_left() {
        assert_eq!(CharacterPosition::FarLeft.x_percent(), 0.1);
    }

    #[test]
    fn test_position_x_percent_left() {
        assert_eq!(CharacterPosition::Left.x_percent(), 0.25);
    }

    #[test]
    fn test_position_x_percent_center() {
        assert_eq!(CharacterPosition::Center.x_percent(), 0.5);
    }

    #[test]
    fn test_position_x_percent_right() {
        assert_eq!(CharacterPosition::Right.x_percent(), 0.75);
    }

    #[test]
    fn test_position_x_percent_far_right() {
        assert_eq!(CharacterPosition::FarRight.x_percent(), 0.9);
    }

    #[test]
    fn test_position_x_percent_custom() {
        let pos = CharacterPosition::Custom(60);
        assert_eq!(pos.x_percent(), 0.6);
    }

    #[test]
    fn test_position_name() {
        assert_eq!(CharacterPosition::FarLeft.name(), "far_left");
        assert_eq!(CharacterPosition::Left.name(), "left");
        assert_eq!(CharacterPosition::Center.name(), "center");
        assert_eq!(CharacterPosition::Right.name(), "right");
        assert_eq!(CharacterPosition::FarRight.name(), "far_right");
        assert_eq!(CharacterPosition::Custom(50).name(), "custom");
    }

    #[test]
    fn test_position_from_str_lowercase() {
        assert_eq!(CharacterPosition::from("left"), CharacterPosition::Left);
        assert_eq!(CharacterPosition::from("right"), CharacterPosition::Right);
        assert_eq!(CharacterPosition::from("center"), CharacterPosition::Center);
    }

    #[test]
    fn test_position_from_str_uppercase() {
        assert_eq!(CharacterPosition::from("LEFT"), CharacterPosition::Left);
        assert_eq!(CharacterPosition::from("CENTER"), CharacterPosition::Center);
    }

    #[test]
    fn test_position_from_str_far_variants() {
        assert_eq!(
            CharacterPosition::from("far_left"),
            CharacterPosition::FarLeft
        );
        assert_eq!(
            CharacterPosition::from("farleft"),
            CharacterPosition::FarLeft
        );
        assert_eq!(
            CharacterPosition::from("far_right"),
            CharacterPosition::FarRight
        );
        assert_eq!(
            CharacterPosition::from("farright"),
            CharacterPosition::FarRight
        );
    }

    #[test]
    fn test_position_from_str_middle_alias() {
        assert_eq!(CharacterPosition::from("middle"), CharacterPosition::Center);
    }

    #[test]
    fn test_position_from_str_unknown() {
        // Unknown strings default to Center
        assert_eq!(
            CharacterPosition::from("unknown"),
            CharacterPosition::Center
        );
    }

    #[test]
    fn test_position_from_string() {
        let pos = CharacterPosition::from("right".to_string());
        assert_eq!(pos, CharacterPosition::Right);
    }

    #[test]
    fn test_position_serialization() {
        let pos = CharacterPosition::Left;
        let serialized = serde_json::to_string(&pos).unwrap();
        let deserialized: CharacterPosition = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pos, deserialized);
    }

    #[test]
    fn test_position_custom_serialization() {
        let pos = CharacterPosition::Custom(45);
        let serialized = serde_json::to_string(&pos).unwrap();
        let deserialized: CharacterPosition = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pos, deserialized);
    }
}
