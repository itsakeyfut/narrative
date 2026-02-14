use crate::types::Color;
use serde::{Deserialize, Serialize};

/// UI configuration for dialogue box and other UI elements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UiConfig {
    /// Dialogue box configuration
    #[serde(default)]
    pub dialogue_box: DialogueBoxConfig,
}

impl UiConfig {
    /// Create a new UI config with default settings
    pub fn new() -> Self {
        Self::default()
    }
}

/// Dialogue box configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogueBoxConfig {
    /// Background opacity (0.0 = fully transparent, 1.0 = fully opaque)
    #[serde(default = "default_opacity")]
    pub opacity: f32,

    /// Dialogue box height in pixels
    #[serde(default = "default_box_height")]
    pub height: f32,

    /// Text padding from box edges in pixels
    #[serde(default = "default_padding")]
    pub padding: f32,

    /// Speaker name font size in pixels
    #[serde(default = "default_speaker_font_size")]
    pub speaker_font_size: f32,

    /// Dialogue text font size in pixels
    #[serde(default = "default_text_font_size")]
    pub text_font_size: f32,

    /// Dialogue text line height in pixels
    #[serde(default = "default_line_height")]
    pub line_height: f32,

    /// Background color (RGB, alpha is controlled by opacity)
    #[serde(default = "default_background_color")]
    pub background_color: Color,

    /// Text color
    #[serde(default = "default_text_color")]
    pub text_color: Color,

    /// Speaker name color
    #[serde(default = "default_speaker_color")]
    pub speaker_color: Color,

    /// Corner radius for rounded corners (0.0 = sharp corners)
    #[serde(default)]
    pub corner_radius: f32,

    /// Show click indicator
    #[serde(default = "default_true")]
    pub show_click_indicator: bool,

    /// Click indicator blink speed (cycles per second)
    #[serde(default = "default_blink_speed")]
    pub click_indicator_blink_speed: f32,
}

impl DialogueBoxConfig {
    /// Create a new dialogue box config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the background color with opacity applied
    pub fn background_color_with_opacity(&self) -> Color {
        Color::new(
            self.background_color.r,
            self.background_color.g,
            self.background_color.b,
            self.opacity,
        )
    }
}

impl Default for DialogueBoxConfig {
    fn default() -> Self {
        Self {
            opacity: default_opacity(),
            height: default_box_height(),
            padding: default_padding(),
            speaker_font_size: default_speaker_font_size(),
            text_font_size: default_text_font_size(),
            line_height: default_line_height(),
            background_color: default_background_color(),
            text_color: default_text_color(),
            speaker_color: default_speaker_color(),
            corner_radius: 0.0,
            show_click_indicator: default_true(),
            click_indicator_blink_speed: default_blink_speed(),
        }
    }
}

fn default_opacity() -> f32 {
    0.8
}

fn default_box_height() -> f32 {
    200.0
}

fn default_padding() -> f32 {
    20.0
}

fn default_speaker_font_size() -> f32 {
    20.0
}

fn default_text_font_size() -> f32 {
    24.0
}

fn default_line_height() -> f32 {
    32.0
}

fn default_background_color() -> Color {
    Color::new(0.0, 0.0, 0.0, 1.0) // Black (alpha will be overridden by opacity)
}

fn default_text_color() -> Color {
    Color::WHITE
}

fn default_speaker_color() -> Color {
    Color::new(1.0, 0.9, 0.6, 1.0) // Light yellow
}

fn default_true() -> bool {
    true
}

fn default_blink_speed() -> f32 {
    2.0 // 2 blinks per second
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_config_new() {
        let config = UiConfig::new();
        assert_eq!(config.dialogue_box.opacity, 0.8);
        assert_eq!(config.dialogue_box.height, 200.0);
    }

    #[test]
    fn test_ui_config_default() {
        let config = UiConfig::default();
        assert_eq!(config.dialogue_box.opacity, 0.8);
    }

    #[test]
    fn test_dialogue_box_config_new() {
        let config = DialogueBoxConfig::new();
        assert_eq!(config.opacity, 0.8);
        assert_eq!(config.height, 200.0);
        assert_eq!(config.padding, 20.0);
        assert_eq!(config.speaker_font_size, 20.0);
        assert_eq!(config.text_font_size, 24.0);
        assert_eq!(config.line_height, 32.0);
        assert!(config.show_click_indicator);
        assert_eq!(config.click_indicator_blink_speed, 2.0);
    }

    #[test]
    fn test_dialogue_box_config_background_color_with_opacity() {
        let config = DialogueBoxConfig::new();
        let color = config.background_color_with_opacity();
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 0.8);
    }

    #[test]
    fn test_dialogue_box_config_custom_opacity() {
        let mut config = DialogueBoxConfig::new();
        config.opacity = 0.5;
        let color = config.background_color_with_opacity();
        assert_eq!(color.a, 0.5);
    }

    #[test]
    fn test_dialogue_box_config_serialization() {
        let config = DialogueBoxConfig::new();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: DialogueBoxConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_ui_config_serialization() {
        let config = UiConfig::new();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: UiConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }
}
