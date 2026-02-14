use crate::types::AssetRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Text speed preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextSpeed {
    Slow,
    #[default]
    Normal,
    Fast,
    Instant,
}

impl fmt::Display for TextSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextSpeed::Slow => write!(f, "Slow"),
            TextSpeed::Normal => write!(f, "Normal"),
            TextSpeed::Fast => write!(f, "Fast"),
            TextSpeed::Instant => write!(f, "Instant"),
        }
    }
}

impl FromStr for TextSpeed {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "slow" => Ok(TextSpeed::Slow),
            "normal" => Ok(TextSpeed::Normal),
            "fast" => Ok(TextSpeed::Fast),
            "instant" => Ok(TextSpeed::Instant),
            _ => Err(format!("Unknown text speed: {}", s)),
        }
    }
}

/// Text rendering configuration
///
/// This struct contains two related but distinct text speed settings:
///
/// 1. **typewriter_speed**: The base typewriter effect speed in characters per second.
///    This is used when the typewriter effect is enabled and no specific preset is selected.
///    Use `character_delay()` to get the delay per character in seconds.
///
/// 2. **speeds**: User-selectable text speed presets that map preset names to
///    character delays in seconds. These are intended for in-game text speed options
///    that players can choose. Use `get_speed_delay()` to retrieve a specific preset.
///
/// The `default_speed` field indicates which preset should be used by default.
///
/// # Example
///
/// ```
/// use narrative_core::TextConfig;
///
/// let config = TextConfig::default();
///
/// // Get the base typewriter delay
/// let base_delay = config.character_delay();
///
/// // Get a specific preset delay
/// if let Some(delay) = config.get_speed_delay("Fast") {
///     println!("Fast preset delay: {} seconds", delay);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextConfig {
    /// Default font asset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_font: Option<AssetRef>,

    /// Default font size in points
    #[serde(default = "default_font_size")]
    pub font_size: f32,

    /// Typewriter effect speed (characters per second)
    ///
    /// This is the base speed used when typewriter effect is enabled.
    /// For user-selectable speed options, use the `speeds` presets instead.
    #[serde(default = "default_typewriter_speed")]
    pub typewriter_speed: f32,

    /// Enable typewriter effect
    #[serde(default = "default_true")]
    pub typewriter_enabled: bool,

    /// Line spacing multiplier (1.0 = single spacing, 1.5 = 1.5x spacing)
    #[serde(default = "default_line_spacing")]
    pub line_spacing: f32,

    /// Auto-advance delay in seconds (0 = disabled)
    ///
    /// When set to a value > 0, text will automatically advance after this delay.
    /// Use `calculate_auto_wait()` to calculate delays based on text length.
    #[serde(default)]
    pub auto_advance_delay: f32,

    /// Default text speed preset
    ///
    /// This indicates which speed preset from the `speeds` map should be used by default.
    #[serde(default)]
    pub default_speed: TextSpeed,

    /// Text speed presets (maps preset name to delay per character in seconds)
    ///
    /// These are user-selectable speed options. Each preset maps a name to a
    /// character delay in seconds. For example:
    /// - "Slow": 0.08 seconds per character
    /// - "Normal": 0.04 seconds per character
    /// - "Fast": 0.02 seconds per character
    /// - "Instant": 0.0 seconds per character
    #[serde(default = "default_speeds")]
    pub speeds: HashMap<String, f32>,

    /// Base auto-advance wait time in seconds
    ///
    /// This is the minimum time to wait before auto-advancing, regardless of text length.
    #[serde(default = "default_auto_wait_base")]
    pub auto_wait_base: f32,

    /// Additional auto-advance wait time per character in seconds
    ///
    /// This is added to `auto_wait_base` for each character in the text,
    /// allowing longer texts to have proportionally longer auto-advance delays.
    #[serde(default = "default_auto_wait_per_char")]
    pub auto_wait_per_char: f32,
}

impl TextConfig {
    /// Create a new text config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the default font
    pub fn with_font(mut self, font: impl Into<AssetRef>) -> Self {
        self.default_font = Some(font.into());
        self
    }

    /// Get the character delay in seconds (inverse of speed)
    pub fn character_delay(&self) -> f32 {
        if self.typewriter_speed > 0.0 {
            1.0 / self.typewriter_speed
        } else {
            0.0
        }
    }

    /// Check if auto-advance is enabled
    pub fn is_auto_advance_enabled(&self) -> bool {
        self.auto_advance_delay > 0.0
    }

    /// Get the character delay for a specific speed preset
    pub fn get_speed_delay(&self, speed: &str) -> Option<f32> {
        self.speeds.get(speed).copied()
    }

    /// Calculate total auto-wait time for a given text length
    pub fn calculate_auto_wait(&self, char_count: usize) -> f32 {
        self.auto_wait_base + (char_count as f32 * self.auto_wait_per_char)
    }
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            default_font: None,
            font_size: default_font_size(),
            typewriter_speed: default_typewriter_speed(),
            typewriter_enabled: default_true(),
            line_spacing: default_line_spacing(),
            auto_advance_delay: 0.0,
            default_speed: TextSpeed::default(),
            speeds: default_speeds(),
            auto_wait_base: default_auto_wait_base(),
            auto_wait_per_char: default_auto_wait_per_char(),
        }
    }
}

fn default_font_size() -> f32 {
    24.0
}

fn default_typewriter_speed() -> f32 {
    30.0 // 30 characters per second
}

fn default_true() -> bool {
    true
}

fn default_line_spacing() -> f32 {
    1.2
}

fn default_speeds() -> HashMap<String, f32> {
    let mut speeds = HashMap::new();
    speeds.insert("Slow".to_string(), 0.08);
    speeds.insert("Normal".to_string(), 0.04);
    speeds.insert("Fast".to_string(), 0.02);
    speeds.insert("Instant".to_string(), 0.0);
    speeds
}

fn default_auto_wait_base() -> f32 {
    1.5
}

fn default_auto_wait_per_char() -> f32 {
    0.05
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_config_new() {
        let config = TextConfig::new();
        assert_eq!(config.default_font, None);
        assert_eq!(config.font_size, 24.0);
        assert_eq!(config.typewriter_speed, 30.0);
        assert!(config.typewriter_enabled);
        assert_eq!(config.line_spacing, 1.2);
        assert_eq!(config.auto_advance_delay, 0.0);
    }

    #[test]
    fn test_text_config_default() {
        let config = TextConfig::default();
        assert_eq!(config.default_font, None);
        assert_eq!(config.font_size, 24.0);
    }

    #[test]
    fn test_text_config_with_font() {
        let config = TextConfig::new().with_font("fonts/main.ttf");
        assert_eq!(config.default_font, Some(AssetRef::from("fonts/main.ttf")));
    }

    #[test]
    fn test_text_config_character_delay() {
        let config = TextConfig::new(); // 30 chars/sec
        let delay = config.character_delay();
        assert!((delay - 0.03333).abs() < 0.0001);
    }

    #[test]
    fn test_text_config_character_delay_fast() {
        let mut config = TextConfig::new();
        config.typewriter_speed = 60.0; // 60 chars/sec
        let delay = config.character_delay();
        assert!((delay - 0.01667).abs() < 0.0001);
    }

    #[test]
    fn test_text_config_character_delay_instant() {
        let mut config = TextConfig::new();
        config.typewriter_speed = 0.0;
        assert_eq!(config.character_delay(), 0.0);
    }

    #[test]
    fn test_text_config_is_auto_advance_enabled_false() {
        let config = TextConfig::new();
        assert!(!config.is_auto_advance_enabled());
    }

    #[test]
    fn test_text_config_is_auto_advance_enabled_true() {
        let mut config = TextConfig::new();
        config.auto_advance_delay = 2.0;
        assert!(config.is_auto_advance_enabled());
    }

    #[test]
    fn test_text_config_serialization() {
        let config = TextConfig::new();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: TextConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_text_config_custom_font_size() {
        let mut config = TextConfig::new();
        config.font_size = 32.0;
        assert_eq!(config.font_size, 32.0);
    }

    #[test]
    fn test_text_config_custom_line_spacing() {
        let mut config = TextConfig::new();
        config.line_spacing = 1.5;
        assert_eq!(config.line_spacing, 1.5);
    }

    #[test]
    fn test_text_config_typewriter_disabled() {
        let mut config = TextConfig::new();
        config.typewriter_enabled = false;
        assert!(!config.typewriter_enabled);
    }

    #[test]
    fn test_text_config_with_auto_advance() {
        let mut config = TextConfig::new();
        config.auto_advance_delay = 3.5;
        assert_eq!(config.auto_advance_delay, 3.5);
        assert!(config.is_auto_advance_enabled());
    }

    #[test]
    fn test_text_speed_display() {
        assert_eq!(TextSpeed::Slow.to_string(), "Slow");
        assert_eq!(TextSpeed::Normal.to_string(), "Normal");
        assert_eq!(TextSpeed::Fast.to_string(), "Fast");
        assert_eq!(TextSpeed::Instant.to_string(), "Instant");
    }

    #[test]
    fn test_text_speed_from_str() {
        assert_eq!("Slow".parse::<TextSpeed>().unwrap(), TextSpeed::Slow);
        assert_eq!("Normal".parse::<TextSpeed>().unwrap(), TextSpeed::Normal);
        assert_eq!("Fast".parse::<TextSpeed>().unwrap(), TextSpeed::Fast);
        assert_eq!("Instant".parse::<TextSpeed>().unwrap(), TextSpeed::Instant);
    }

    #[test]
    fn test_text_speed_from_str_lowercase() {
        assert_eq!("slow".parse::<TextSpeed>().unwrap(), TextSpeed::Slow);
        assert_eq!("normal".parse::<TextSpeed>().unwrap(), TextSpeed::Normal);
        assert_eq!("fast".parse::<TextSpeed>().unwrap(), TextSpeed::Fast);
        assert_eq!("instant".parse::<TextSpeed>().unwrap(), TextSpeed::Instant);
    }

    #[test]
    fn test_text_speed_from_str_invalid() {
        assert!("invalid".parse::<TextSpeed>().is_err());
        assert!("".parse::<TextSpeed>().is_err());
    }

    #[test]
    fn test_text_speed_default() {
        assert_eq!(TextSpeed::default(), TextSpeed::Normal);
    }
}
