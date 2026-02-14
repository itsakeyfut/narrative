//! Engine configuration

use narrative_core::EngineResult;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Window configuration
    #[serde(default)]
    pub window: WindowConfig,
    /// Graphics configuration
    #[serde(default)]
    pub graphics: GraphicsConfig,
    /// Audio configuration
    #[serde(default)]
    pub audio: AudioConfig,
    /// Gameplay configuration
    #[serde(default)]
    pub gameplay: GameplayConfig,
    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,
    /// Development configuration
    #[serde(default)]
    pub development: DevelopmentConfig,
    /// Asset base path
    #[serde(default = "default_asset_path")]
    pub asset_path: PathBuf,
    /// Save directory
    #[serde(default = "default_save_path")]
    pub save_path: PathBuf,
    /// Start scenario path
    #[serde(default = "default_start_scenario")]
    pub start_scenario: PathBuf,
}

fn default_asset_path() -> PathBuf {
    PathBuf::from("assets")
}

fn default_save_path() -> PathBuf {
    PathBuf::from("saves")
}

fn default_start_scenario() -> PathBuf {
    PathBuf::from("assets/scenarios/chapter_01.toml")
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window title
    #[serde(default = "default_window_title")]
    pub title: String,
    /// Window width
    #[serde(default = "default_window_width")]
    pub width: u32,
    /// Window height
    #[serde(default = "default_window_height")]
    pub height: u32,
    /// Resizable window
    #[serde(default = "default_true")]
    pub resizable: bool,
    /// Fullscreen mode
    #[serde(default)]
    pub fullscreen: bool,
}

fn default_window_title() -> String {
    "Narrative Novel".to_string()
}

fn default_window_width() -> u32 {
    1280
}

fn default_window_height() -> u32 {
    720
}

fn default_true() -> bool {
    true
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: default_window_title(),
            width: default_window_width(),
            height: default_window_height(),
            resizable: true,
            fullscreen: false,
        }
    }
}

impl WindowConfig {
    /// Validate window configuration values
    pub fn validate(&self) -> Result<(), String> {
        const MIN_WIDTH: u32 = 800;
        const MIN_HEIGHT: u32 = 600;
        const MAX_WIDTH: u32 = 7680; // 8K width
        const MAX_HEIGHT: u32 = 4320; // 8K height

        if self.width < MIN_WIDTH {
            return Err(format!(
                "window.width must be at least {}, got {}",
                MIN_WIDTH, self.width
            ));
        }

        if self.width > MAX_WIDTH {
            return Err(format!(
                "window.width must be at most {}, got {}",
                MAX_WIDTH, self.width
            ));
        }

        if self.height < MIN_HEIGHT {
            return Err(format!(
                "window.height must be at least {}, got {}",
                MIN_HEIGHT, self.height
            ));
        }

        if self.height > MAX_HEIGHT {
            return Err(format!(
                "window.height must be at most {}, got {}",
                MAX_HEIGHT, self.height
            ));
        }

        Ok(())
    }
}

/// Anti-aliasing setting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AntiAliasing {
    /// No anti-aliasing
    None,
    /// 2x multi-sampling
    #[serde(rename = "2x")]
    X2,
    /// 4x multi-sampling (default)
    #[serde(rename = "4x")]
    #[default]
    X4,
    /// 8x multi-sampling
    #[serde(rename = "8x")]
    X8,
}

/// Graphics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    /// VSync enabled
    #[serde(default = "default_true")]
    pub vsync: bool,
    /// Target FPS
    #[serde(default = "default_target_fps")]
    pub target_fps: u32,
    /// Anti-aliasing setting
    #[serde(default)]
    pub anti_aliasing: AntiAliasing,
    /// Character texture cache capacity (number of textures)
    #[serde(default = "default_character_cache_capacity")]
    pub character_cache_capacity: usize,
}

fn default_target_fps() -> u32 {
    60
}

fn default_character_cache_capacity() -> usize {
    75 // 50-100 range, using middle value
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            vsync: true,
            target_fps: 60,
            anti_aliasing: AntiAliasing::default(),
            character_cache_capacity: default_character_cache_capacity(),
        }
    }
}

impl GraphicsConfig {
    /// Validate graphics configuration values
    pub fn validate(&self) -> Result<(), String> {
        const MIN_FPS: u32 = 15;
        const MAX_FPS: u32 = 240;
        const MIN_CACHE_CAPACITY: usize = 10;
        const MAX_CACHE_CAPACITY: usize = 500;

        if self.target_fps < MIN_FPS || self.target_fps > MAX_FPS {
            return Err(format!(
                "graphics.target_fps must be {}-{}, got {}",
                MIN_FPS, MAX_FPS, self.target_fps
            ));
        }

        if self.character_cache_capacity < MIN_CACHE_CAPACITY
            || self.character_cache_capacity > MAX_CACHE_CAPACITY
        {
            return Err(format!(
                "graphics.character_cache_capacity must be {}-{}, got {}",
                MIN_CACHE_CAPACITY, MAX_CACHE_CAPACITY, self.character_cache_capacity
            ));
        }

        Ok(())
    }
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Master volume (0.0-1.0)
    #[serde(default = "default_volume")]
    pub master_volume: f32,
    /// Music volume (0.0-1.0)
    #[serde(default = "default_music_volume")]
    pub music_volume: f32,
    /// Sound effects volume (0.0-1.0)
    #[serde(default = "default_volume")]
    pub sound_volume: f32,
    /// Voice volume (0.0-1.0)
    #[serde(default = "default_volume")]
    pub voice_volume: f32,
    /// Audio enabled (mute when false)
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_volume() -> f32 {
    1.0
}

fn default_music_volume() -> f32 {
    0.8
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sound_volume: 1.0,
            voice_volume: 1.0,
            enabled: true,
        }
    }
}

impl AudioConfig {
    /// Validate audio configuration values
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.master_volume) {
            return Err(format!(
                "audio.master_volume must be 0.0-1.0, got {}",
                self.master_volume
            ));
        }
        if !(0.0..=1.0).contains(&self.music_volume) {
            return Err(format!(
                "audio.music_volume must be 0.0-1.0, got {}",
                self.music_volume
            ));
        }
        if !(0.0..=1.0).contains(&self.sound_volume) {
            return Err(format!(
                "audio.sound_volume must be 0.0-1.0, got {}",
                self.sound_volume
            ));
        }
        if !(0.0..=1.0).contains(&self.voice_volume) {
            return Err(format!(
                "audio.voice_volume must be 0.0-1.0, got {}",
                self.voice_volume
            ));
        }
        Ok(())
    }

    /// Get the effective music volume (master * music)
    pub fn effective_music_volume(&self) -> f32 {
        if self.enabled {
            self.master_volume * self.music_volume
        } else {
            0.0
        }
    }

    /// Get the effective sound volume (master * sound)
    pub fn effective_sound_volume(&self) -> f32 {
        if self.enabled {
            self.master_volume * self.sound_volume
        } else {
            0.0
        }
    }

    /// Get the effective voice volume (master * voice)
    pub fn effective_voice_volume(&self) -> f32 {
        if self.enabled {
            self.master_volume * self.voice_volume
        } else {
            0.0
        }
    }

    /// Set master volume (clamped to 0.0-1.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set music volume (clamped to 0.0-1.0)
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    /// Set sound volume (clamped to 0.0-1.0)
    pub fn set_sound_volume(&mut self, volume: f32) {
        self.sound_volume = volume.clamp(0.0, 1.0);
    }

    /// Set voice volume (clamped to 0.0-1.0)
    pub fn set_voice_volume(&mut self, volume: f32) {
        self.voice_volume = volume.clamp(0.0, 1.0);
    }

    /// Toggle mute (enable/disable audio)
    pub fn toggle_mute(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Set mute state
    pub fn set_mute(&mut self, muted: bool) {
        self.enabled = !muted;
    }

    /// Check if audio is muted
    pub fn is_muted(&self) -> bool {
        !self.enabled
    }
}

/// Gameplay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayConfig {
    /// Text speed (characters per second)
    #[serde(default = "default_text_speed")]
    pub text_speed: f32,
    /// Auto-advance speed (seconds per line)
    #[serde(default = "default_auto_advance_speed")]
    pub auto_advance_speed: f32,
    /// Auto mode enabled (automatically advance text)
    #[serde(default)]
    pub auto_mode_enabled: bool,
    /// Wait for voice to finish before auto-advancing
    #[serde(default = "default_true")]
    pub auto_wait_for_voice: bool,
    /// Skip mode (disabled, read_only, or all)
    #[serde(default)]
    pub skip_mode: narrative_core::SkipMode,
    /// Skip mode currently enabled (runtime state)
    #[serde(default)]
    pub skip_mode_enabled: bool,
    /// Stop skip mode at choices
    #[serde(default = "default_true")]
    pub skip_stop_at_choices: bool,
    /// Enable quick save
    #[serde(default = "default_true")]
    pub enable_quick_save: bool,
    /// Maximum save slots
    #[serde(default = "default_max_save_slots")]
    pub max_save_slots: usize,
}

fn default_text_speed() -> f32 {
    30.0
}

fn default_auto_advance_speed() -> f32 {
    2.0
}

fn default_max_save_slots() -> usize {
    20
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            text_speed: 30.0,
            auto_advance_speed: 2.0,
            auto_mode_enabled: false,
            auto_wait_for_voice: true,
            skip_mode: narrative_core::SkipMode::default(),
            skip_mode_enabled: false,
            skip_stop_at_choices: true,
            enable_quick_save: true,
            max_save_slots: 20,
        }
    }
}

impl GameplayConfig {
    /// Validate gameplay configuration values
    pub fn validate(&self) -> Result<(), String> {
        const MIN_TEXT_SPEED: f32 = 1.0;
        const MAX_TEXT_SPEED: f32 = 200.0;
        const MIN_AUTO_SPEED: f32 = 0.1;
        const MAX_AUTO_SPEED: f32 = 10.0;
        const MIN_SAVE_SLOTS: usize = 1;
        const MAX_SAVE_SLOTS: usize = 100;

        if self.text_speed < MIN_TEXT_SPEED || self.text_speed > MAX_TEXT_SPEED {
            return Err(format!(
                "gameplay.text_speed must be {}-{}, got {}",
                MIN_TEXT_SPEED, MAX_TEXT_SPEED, self.text_speed
            ));
        }

        if self.auto_advance_speed < MIN_AUTO_SPEED || self.auto_advance_speed > MAX_AUTO_SPEED {
            return Err(format!(
                "gameplay.auto_advance_speed must be {}-{}, got {}",
                MIN_AUTO_SPEED, MAX_AUTO_SPEED, self.auto_advance_speed
            ));
        }

        if self.max_save_slots < MIN_SAVE_SLOTS || self.max_save_slots > MAX_SAVE_SLOTS {
            return Err(format!(
                "gameplay.max_save_slots must be {}-{}, got {}",
                MIN_SAVE_SLOTS, MAX_SAVE_SLOTS, self.max_save_slots
            ));
        }

        Ok(())
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Dialogue font size
    #[serde(default = "default_dialogue_font_size")]
    pub dialogue_font_size: u32,
    /// UI font size
    #[serde(default = "default_ui_font_size")]
    pub ui_font_size: u32,
    /// Dialogue box opacity (0.0-1.0)
    #[serde(default = "default_dialogue_box_opacity")]
    pub dialogue_box_opacity: f32,
    /// Choice highlight color (RGBA)
    #[serde(default = "default_choice_highlight_color")]
    pub choice_highlight_color: [f32; 4],
    /// Title screen BGM path
    #[serde(default)]
    pub title_bgm: Option<String>,
}

fn default_dialogue_font_size() -> u32 {
    24
}

fn default_ui_font_size() -> u32 {
    18
}

fn default_dialogue_box_opacity() -> f32 {
    0.8
}

fn default_choice_highlight_color() -> [f32; 4] {
    [1.0, 1.0, 0.0, 1.0]
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            dialogue_font_size: 24,
            ui_font_size: 18,
            dialogue_box_opacity: 0.8,
            choice_highlight_color: [1.0, 1.0, 0.0, 1.0],
            title_bgm: None,
        }
    }
}

impl UiConfig {
    /// Validate UI configuration values
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.dialogue_box_opacity) {
            return Err(format!(
                "ui.dialogue_box_opacity must be 0.0-1.0, got {}",
                self.dialogue_box_opacity
            ));
        }

        for (i, &component) in self.choice_highlight_color.iter().enumerate() {
            if !(0.0..=1.0).contains(&component) {
                return Err(format!(
                    "ui.choice_highlight_color[{}] must be 0.0-1.0, got {}",
                    i, component
                ));
            }
        }

        Ok(())
    }
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevelopmentConfig {
    /// Debug mode enabled
    #[serde(default)]
    pub debug_mode: bool,
    /// Show FPS counter
    #[serde(default)]
    pub show_fps: bool,
    /// Hot reload enabled
    #[serde(default)]
    pub hot_reload: bool,
}

impl EngineConfig {
    /// Create a new engine configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> EngineResult<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let config: EngineConfig = toml::from_str(&content)?;

        // Validate all configuration sections
        config.validate()?;

        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> EngineResult<()> {
        let content = toml::to_string_pretty(self)?;

        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path.as_ref(), content)?;

        Ok(())
    }

    /// Validate all configuration sections
    pub fn validate(&self) -> EngineResult<()> {
        self.window
            .validate()
            .map_err(narrative_core::ConfigError::Other)?;
        self.graphics
            .validate()
            .map_err(narrative_core::ConfigError::Other)?;
        self.audio
            .validate()
            .map_err(narrative_core::ConfigError::Other)?;
        self.gameplay
            .validate()
            .map_err(narrative_core::ConfigError::Other)?;
        self.ui
            .validate()
            .map_err(narrative_core::ConfigError::Other)?;
        Ok(())
    }

    // Backward compatibility helpers
    /// Get window title
    pub fn window_title(&self) -> &str {
        &self.window.title
    }

    /// Get window width
    pub fn window_width(&self) -> u32 {
        self.window.width
    }

    /// Get window height
    pub fn window_height(&self) -> u32 {
        self.window.height
    }

    /// Get target FPS
    pub fn target_fps(&self) -> u32 {
        self.graphics.target_fps
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            graphics: GraphicsConfig::default(),
            audio: AudioConfig::default(),
            gameplay: GameplayConfig::default(),
            ui: UiConfig::default(),
            development: DevelopmentConfig::default(),
            asset_path: default_asset_path(),
            save_path: default_save_path(),
            start_scenario: default_start_scenario(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = EngineConfig::default();

        assert_eq!(config.window_title(), "Narrative Novel");
        assert_eq!(config.window_width(), 1280);
        assert_eq!(config.window_height(), 720);
        assert_eq!(config.target_fps(), 60);
        assert_eq!(config.asset_path, PathBuf::from("assets"));
        assert_eq!(config.save_path, PathBuf::from("saves"));
    }

    #[test]
    fn test_config_new() {
        let config = EngineConfig::new();

        assert_eq!(config.window_title(), "Narrative Novel");
        assert_eq!(config.target_fps(), 60);
    }

    #[test]
    fn test_config_clone() {
        let config1 = EngineConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.window.title, config2.window.title);
        assert_eq!(config1.window.width, config2.window.width);
        assert_eq!(config1.window.height, config2.window.height);
    }

    #[test]
    fn test_config_modify() {
        let mut config = EngineConfig::default();

        config.window.title = "My Game".to_string();
        config.window.width = 1920;
        config.window.height = 1080;
        config.graphics.target_fps = 144;

        assert_eq!(config.window_title(), "My Game");
        assert_eq!(config.window_width(), 1920);
        assert_eq!(config.window_height(), 1080);
        assert_eq!(config.target_fps(), 144);
    }

    #[test]
    fn test_config_paths() {
        let mut config = EngineConfig::default();

        config.asset_path = PathBuf::from("/custom/assets");
        config.save_path = PathBuf::from("/custom/saves");

        assert_eq!(config.asset_path, PathBuf::from("/custom/assets"));
        assert_eq!(config.save_path, PathBuf::from("/custom/saves"));
    }

    #[test]
    fn test_config_serialization() {
        let mut config = EngineConfig::default();
        config.window.title = "Test Game".to_string();
        config.window.width = 800;
        config.window.height = 600;
        config.graphics.target_fps = 30;
        config.asset_path = PathBuf::from("test_assets");
        config.save_path = PathBuf::from("test_saves");
        config.start_scenario = PathBuf::from("assets/scenarios/test.toml");

        // Test TOML serialization
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: EngineConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.window_title(), "Test Game");
        assert_eq!(deserialized.window_width(), 800);
        assert_eq!(deserialized.window_height(), 600);
        assert_eq!(deserialized.target_fps(), 30);
    }

    #[test]
    fn test_config_debug() {
        let config = EngineConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("EngineConfig"));
        assert!(debug_str.contains("Narrative Novel"));
    }

    #[test]
    fn test_config_load_nonexistent() {
        let result = EngineConfig::load("nonexistent_file.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_aspect_ratio() {
        let config = EngineConfig::default();

        let aspect_ratio = config.window_width() as f32 / config.window_height() as f32;
        assert!((aspect_ratio - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn test_window_config() {
        let window = WindowConfig::default();
        assert_eq!(window.title, "Narrative Novel");
        assert_eq!(window.width, 1280);
        assert_eq!(window.height, 720);
        assert!(window.resizable);
        assert!(!window.fullscreen);
    }

    #[test]
    fn test_graphics_config() {
        let graphics = GraphicsConfig::default();
        assert!(graphics.vsync);
        assert_eq!(graphics.target_fps, 60);
        assert_eq!(graphics.anti_aliasing, AntiAliasing::X4);
        assert_eq!(graphics.character_cache_capacity, 75);
    }

    #[test]
    fn test_graphics_config_character_cache_capacity() {
        let mut graphics = GraphicsConfig::default();
        assert_eq!(graphics.character_cache_capacity, 75);

        // Valid capacity
        graphics.character_cache_capacity = 50;
        assert!(graphics.validate().is_ok());

        graphics.character_cache_capacity = 100;
        assert!(graphics.validate().is_ok());
    }

    #[test]
    fn test_graphics_config_character_cache_capacity_too_low() {
        let mut graphics = GraphicsConfig::default();
        graphics.character_cache_capacity = 9;
        assert!(graphics.validate().is_err());
    }

    #[test]
    fn test_graphics_config_character_cache_capacity_too_high() {
        let mut graphics = GraphicsConfig::default();
        graphics.character_cache_capacity = 501;
        assert!(graphics.validate().is_err());
    }

    #[test]
    fn test_audio_config() {
        let audio = AudioConfig::default();
        assert_eq!(audio.master_volume, 1.0);
        assert_eq!(audio.music_volume, 0.8);
        assert_eq!(audio.sound_volume, 1.0);
        assert_eq!(audio.voice_volume, 1.0);
        assert!(audio.enabled);
    }

    #[test]
    fn test_gameplay_config() {
        let gameplay = GameplayConfig::default();
        assert_eq!(gameplay.text_speed, 30.0);
        assert_eq!(gameplay.auto_advance_speed, 2.0);
        assert_eq!(gameplay.skip_mode, narrative_core::SkipMode::ReadOnly);
        assert!(gameplay.enable_quick_save);
        assert_eq!(gameplay.max_save_slots, 20);
    }

    #[test]
    fn test_ui_config() {
        let ui = UiConfig::default();
        assert_eq!(ui.dialogue_font_size, 24);
        assert_eq!(ui.ui_font_size, 18);
        assert_eq!(ui.dialogue_box_opacity, 0.8);
        assert_eq!(ui.choice_highlight_color, [1.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn test_development_config() {
        let dev = DevelopmentConfig::default();
        assert!(!dev.debug_mode);
        assert!(!dev.show_fps);
        assert!(!dev.hot_reload);
    }

    #[test]
    fn test_window_validation_success() {
        let window = WindowConfig::default();
        assert!(window.validate().is_ok());
    }

    #[test]
    fn test_window_validation_min_width() {
        let mut window = WindowConfig::default();
        window.width = 799;
        assert!(window.validate().is_err());
    }

    #[test]
    fn test_window_validation_min_height() {
        let mut window = WindowConfig::default();
        window.height = 599;
        assert!(window.validate().is_err());
    }

    #[test]
    fn test_graphics_validation_success() {
        let graphics = GraphicsConfig::default();
        assert!(graphics.validate().is_ok());
    }

    #[test]
    fn test_graphics_validation_fps_too_low() {
        let mut graphics = GraphicsConfig::default();
        graphics.target_fps = 10;
        assert!(graphics.validate().is_err());
    }

    #[test]
    fn test_graphics_validation_fps_too_high() {
        let mut graphics = GraphicsConfig::default();
        graphics.target_fps = 300;
        assert!(graphics.validate().is_err());
    }

    #[test]
    fn test_audio_validation_success() {
        let audio = AudioConfig::default();
        assert!(audio.validate().is_ok());
    }

    #[test]
    fn test_audio_validation_invalid_master_volume() {
        let mut audio = AudioConfig::default();
        audio.master_volume = 1.5;
        assert!(audio.validate().is_err());
    }

    #[test]
    fn test_audio_validation_negative_volume() {
        let mut audio = AudioConfig::default();
        audio.music_volume = -0.1;
        assert!(audio.validate().is_err());
    }

    #[test]
    fn test_gameplay_validation_success() {
        let gameplay = GameplayConfig::default();
        assert!(gameplay.validate().is_ok());
    }

    #[test]
    fn test_gameplay_validation_text_speed_too_high() {
        let mut gameplay = GameplayConfig::default();
        gameplay.text_speed = 300.0;
        assert!(gameplay.validate().is_err());
    }

    #[test]
    fn test_gameplay_validation_invalid_save_slots() {
        let mut gameplay = GameplayConfig::default();
        gameplay.max_save_slots = 0;
        assert!(gameplay.validate().is_err());
    }

    #[test]
    fn test_ui_validation_success() {
        let ui = UiConfig::default();
        assert!(ui.validate().is_ok());
    }

    #[test]
    fn test_ui_validation_invalid_opacity() {
        let mut ui = UiConfig::default();
        ui.dialogue_box_opacity = 1.5;
        assert!(ui.validate().is_err());
    }

    #[test]
    fn test_ui_validation_invalid_color() {
        let mut ui = UiConfig::default();
        ui.choice_highlight_color = [2.0, 0.5, 0.5, 1.0];
        assert!(ui.validate().is_err());
    }

    #[test]
    fn test_engine_config_validation() {
        let config = EngineConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_anti_aliasing_serialization() {
        // Test serialization within a struct (TOML doesn't support bare enum serialization)
        #[derive(Serialize, Deserialize)]
        struct TestConfig {
            aa: AntiAliasing,
        }

        let test_none = TestConfig {
            aa: AntiAliasing::None,
        };
        let toml_str = toml::to_string(&test_none).unwrap();
        assert!(toml_str.contains("aa = \"none\""));

        let test_x2 = TestConfig {
            aa: AntiAliasing::X2,
        };
        let toml_str = toml::to_string(&test_x2).unwrap();
        assert!(toml_str.contains("aa = \"2x\""));

        let test_x4 = TestConfig {
            aa: AntiAliasing::X4,
        };
        let toml_str = toml::to_string(&test_x4).unwrap();
        assert!(toml_str.contains("aa = \"4x\""));

        let test_x8 = TestConfig {
            aa: AntiAliasing::X8,
        };
        let toml_str = toml::to_string(&test_x8).unwrap();
        assert!(toml_str.contains("aa = \"8x\""));

        // Test deserialization
        let parsed: TestConfig = toml::from_str("aa = \"none\"").unwrap();
        assert_eq!(parsed.aa, AntiAliasing::None);

        let parsed: TestConfig = toml::from_str("aa = \"2x\"").unwrap();
        assert_eq!(parsed.aa, AntiAliasing::X2);

        let parsed: TestConfig = toml::from_str("aa = \"4x\"").unwrap();
        assert_eq!(parsed.aa, AntiAliasing::X4);

        let parsed: TestConfig = toml::from_str("aa = \"8x\"").unwrap();
        assert_eq!(parsed.aa, AntiAliasing::X8);
    }

    #[test]
    fn test_audio_config_effective_music_volume() {
        let mut audio = AudioConfig::default();
        audio.master_volume = 0.8;
        audio.music_volume = 0.5;
        assert!((audio.effective_music_volume() - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_audio_config_effective_music_volume_muted() {
        let mut audio = AudioConfig::default();
        audio.master_volume = 0.8;
        audio.music_volume = 0.5;
        audio.enabled = false;
        assert_eq!(audio.effective_music_volume(), 0.0);
    }

    #[test]
    fn test_audio_config_effective_sound_volume() {
        let mut audio = AudioConfig::default();
        audio.master_volume = 0.7;
        audio.sound_volume = 0.6;
        assert!((audio.effective_sound_volume() - 0.42).abs() < 0.001);
    }

    #[test]
    fn test_audio_config_effective_voice_volume() {
        let mut audio = AudioConfig::default();
        audio.master_volume = 0.9;
        audio.voice_volume = 0.8;
        assert!((audio.effective_voice_volume() - 0.72).abs() < 0.001);
    }

    #[test]
    fn test_audio_config_set_master_volume() {
        let mut audio = AudioConfig::default();
        audio.set_master_volume(0.75);
        assert_eq!(audio.master_volume, 0.75);
    }

    #[test]
    fn test_audio_config_set_master_volume_clamping() {
        let mut audio = AudioConfig::default();
        audio.set_master_volume(1.5);
        assert_eq!(audio.master_volume, 1.0);
        audio.set_master_volume(-0.5);
        assert_eq!(audio.master_volume, 0.0);
    }

    #[test]
    fn test_audio_config_set_music_volume() {
        let mut audio = AudioConfig::default();
        audio.set_music_volume(0.6);
        assert_eq!(audio.music_volume, 0.6);
    }

    #[test]
    fn test_audio_config_set_sound_volume() {
        let mut audio = AudioConfig::default();
        audio.set_sound_volume(0.4);
        assert_eq!(audio.sound_volume, 0.4);
    }

    #[test]
    fn test_audio_config_set_voice_volume() {
        let mut audio = AudioConfig::default();
        audio.set_voice_volume(0.85);
        assert_eq!(audio.voice_volume, 0.85);
    }

    #[test]
    fn test_audio_config_toggle_mute() {
        let mut audio = AudioConfig::default();
        assert!(audio.enabled);
        audio.toggle_mute();
        assert!(!audio.enabled);
        audio.toggle_mute();
        assert!(audio.enabled);
    }

    #[test]
    fn test_audio_config_set_mute() {
        let mut audio = AudioConfig::default();
        audio.set_mute(true);
        assert!(!audio.enabled);
        assert!(audio.is_muted());
        audio.set_mute(false);
        assert!(audio.enabled);
        assert!(!audio.is_muted());
    }

    #[test]
    fn test_audio_config_is_muted() {
        let mut audio = AudioConfig::default();
        assert!(!audio.is_muted());
        audio.enabled = false;
        assert!(audio.is_muted());
    }
}
