//! User settings with RON persistence
//!
//! This module provides a RON-based settings system for user preferences.
//! Settings are persisted to `assets/config/settings.ron`.

use super::{AudioConfig, SkipMode, TextSpeed};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Layout mode for save/load menu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SaveMenuLayoutMode {
    /// List layout (1 column, 6 slots per page, detailed view)
    #[default]
    List,
    /// Grid layout (3 columns, 9 slots per page, compact view)
    Grid,
}

/// User settings (persisted to assets/config/settings.ron)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UserSettings {
    /// Audio settings
    pub audio: AudioSettings,
    /// Text settings
    pub text: TextSettings,
    /// Display settings
    pub display: DisplaySettings,
    /// Skip settings
    pub skip: SkipSettings,
    /// Animation settings
    pub animation: AnimationSettings,
}

impl UserSettings {
    /// Create new user settings with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load settings from RON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, SettingsError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| SettingsError::IoError(e.to_string()))?;
        let settings: UserSettings =
            ron::from_str(&content).map_err(|e| SettingsError::ParseError(e.to_string()))?;
        Ok(settings)
    }

    /// Save settings to RON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), SettingsError> {
        let content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| SettingsError::SerializeError(e.to_string()))?;

        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| SettingsError::IoError(e.to_string()))?;
        }

        std::fs::write(path.as_ref(), content)
            .map_err(|e| SettingsError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Convert to AudioConfig (for compatibility with existing engine)
    pub fn to_audio_config(&self) -> AudioConfig {
        AudioConfig {
            master_volume: self.audio.master_volume,
            bgm_volume: self.audio.bgm_volume,
            se_volume: self.audio.se_volume,
            voice_volume: self.audio.voice_volume,
            enabled: true,
        }
    }

    /// Update from AudioConfig (for compatibility with existing engine)
    pub fn update_from_audio_config(&mut self, audio: &AudioConfig) {
        self.audio.master_volume = audio.master_volume;
        self.audio.bgm_volume = audio.bgm_volume;
        self.audio.se_volume = audio.se_volume;
        self.audio.voice_volume = audio.voice_volume;
    }
}

/// Audio settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub master_volume: f32,
    /// BGM volume (0.0 - 1.0)
    #[serde(default = "default_music_volume")]
    pub bgm_volume: f32,
    /// SE volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub se_volume: f32,
    /// Voice volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub voice_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: default_volume(),
            bgm_volume: default_music_volume(),
            se_volume: default_volume(),
            voice_volume: default_volume(),
        }
    }
}

fn default_volume() -> f32 {
    1.0
}

fn default_music_volume() -> f32 {
    0.7
}

/// Text settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextSettings {
    /// Text speed preset
    #[serde(default)]
    pub speed: TextSpeed,
    /// Auto-advance wait time in seconds
    #[serde(default = "default_auto_wait")]
    pub auto_wait: f32,
}

impl Default for TextSettings {
    fn default() -> Self {
        Self {
            speed: TextSpeed::default(),
            auto_wait: default_auto_wait(),
        }
    }
}

fn default_auto_wait() -> f32 {
    2.0
}

/// Display settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// Fullscreen mode
    #[serde(default)]
    pub fullscreen: bool,
    /// Window resolution (width, height)
    #[serde(default = "default_resolution")]
    pub resolution: (u32, u32),
    /// Save/Load menu layout preference
    #[serde(default)]
    pub save_menu_layout: SaveMenuLayoutMode,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            resolution: default_resolution(),
            save_menu_layout: SaveMenuLayoutMode::default(),
        }
    }
}

fn default_resolution() -> (u32, u32) {
    (1280, 720)
}

/// Common display resolutions
pub const COMMON_RESOLUTIONS: &[(u32, u32, &str)] = &[
    (1280, 720, "1280x720 (720p HD)"),
    (1920, 1080, "1920x1080 (1080p Full HD)"),
    (2560, 1440, "2560x1440 (1440p 2K)"),
    (3840, 2160, "3840x2160 (2160p 4K UHD)"),
];

impl DisplaySettings {
    /// Get display name for current resolution
    pub fn resolution_display_name(&self) -> String {
        for &(width, height, name) in COMMON_RESOLUTIONS {
            if self.resolution == (width, height) {
                return name.to_string();
            }
        }
        // Custom resolution
        format!("{}x{} (Custom)", self.resolution.0, self.resolution.1)
    }
}

/// Skip settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkipSettings {
    /// Skip mode
    #[serde(default)]
    pub mode: SkipMode,
    /// Stop at choices
    #[serde(default = "default_true")]
    pub stop_at_choices: bool,
}

impl Default for SkipSettings {
    fn default() -> Self {
        Self {
            mode: SkipMode::default(),
            stop_at_choices: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Animation settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationSettings {
    /// Global animation enable/disable
    #[serde(default = "default_animations_enabled")]
    pub enabled: bool,
    /// UI animation speed multiplier (0.5 = half speed, 2.0 = double speed)
    #[serde(default = "default_animation_speed")]
    pub speed: f32,
    /// Respect system reduced motion preference
    #[serde(default)]
    pub respect_system_preference: bool,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            speed: 1.0,
            respect_system_preference: false,
        }
    }
}

fn default_animations_enabled() -> bool {
    true
}

fn default_animation_speed() -> f32 {
    1.0
}

/// Settings error types
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsError {
    /// IO error
    IoError(String),
    /// Parse error
    ParseError(String),
    /// Serialize error
    SerializeError(String),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::IoError(msg) => write!(f, "IO error: {}", msg),
            SettingsError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SettingsError::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
        }
    }
}

impl std::error::Error for SettingsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_settings_default() {
        let settings = UserSettings::default();
        assert_eq!(settings.audio.master_volume, 1.0);
        assert_eq!(settings.audio.bgm_volume, 0.7);
        assert_eq!(settings.text.speed, TextSpeed::Normal);
        assert_eq!(settings.text.auto_wait, 2.0);
        assert!(!settings.display.fullscreen);
        assert_eq!(settings.display.resolution, (1280, 720));
        assert_eq!(settings.skip.mode, SkipMode::ReadOnly);
        assert!(settings.skip.stop_at_choices);
        assert!(settings.animation.enabled);
        assert_eq!(settings.animation.speed, 1.0);
        assert!(!settings.animation.respect_system_preference);
    }

    #[test]
    fn test_user_settings_serialization() {
        let settings = UserSettings::default();
        let serialized =
            ron::ser::to_string_pretty(&settings, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: UserSettings = ron::from_str(&serialized).unwrap();
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_audio_settings_default() {
        let audio = AudioSettings::default();
        assert_eq!(audio.master_volume, 1.0);
        assert_eq!(audio.bgm_volume, 0.7);
        assert_eq!(audio.se_volume, 1.0);
        assert_eq!(audio.voice_volume, 1.0);
    }

    #[test]
    fn test_text_settings_default() {
        let text = TextSettings::default();
        assert_eq!(text.speed, TextSpeed::Normal);
        assert_eq!(text.auto_wait, 2.0);
    }

    #[test]
    fn test_display_settings_default() {
        let display = DisplaySettings::default();
        assert!(!display.fullscreen);
        assert_eq!(display.resolution, (1280, 720));
    }

    #[test]
    fn test_skip_settings_default() {
        let skip = SkipSettings::default();
        assert_eq!(skip.mode, SkipMode::ReadOnly);
        assert!(skip.stop_at_choices);
    }

    #[test]
    fn test_animation_settings_default() {
        let animation = AnimationSettings::default();
        assert!(animation.enabled);
        assert_eq!(animation.speed, 1.0);
        assert!(!animation.respect_system_preference);
    }

    #[test]
    fn test_user_settings_to_audio_config() {
        let settings = UserSettings::default();
        let audio_config = settings.to_audio_config();
        assert_eq!(audio_config.master_volume, 1.0);
        assert_eq!(audio_config.bgm_volume, 0.7);
        assert_eq!(audio_config.se_volume, 1.0);
        assert_eq!(audio_config.voice_volume, 1.0);
    }

    #[test]
    fn test_user_settings_update_from_audio_config() {
        let mut settings = UserSettings::default();
        let mut audio_config = AudioConfig::new();
        audio_config.set_master_volume(0.8);
        audio_config.set_bgm_volume(0.6);
        audio_config.set_se_volume(0.9);
        audio_config.set_voice_volume(0.7);

        settings.update_from_audio_config(&audio_config);

        assert_eq!(settings.audio.master_volume, 0.8);
        assert_eq!(settings.audio.bgm_volume, 0.6);
        assert_eq!(settings.audio.se_volume, 0.9);
        assert_eq!(settings.audio.voice_volume, 0.7);
    }

    #[test]
    fn test_settings_error_display() {
        let io_err = SettingsError::IoError("file not found".to_string());
        assert_eq!(io_err.to_string(), "IO error: file not found");

        let parse_err = SettingsError::ParseError("invalid RON".to_string());
        assert_eq!(parse_err.to_string(), "Parse error: invalid RON");

        let serialize_err = SettingsError::SerializeError("cannot serialize".to_string());
        assert_eq!(
            serialize_err.to_string(),
            "Serialize error: cannot serialize"
        );
    }
}
