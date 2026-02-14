use super::{AudioConfig, GraphicsConfig, PathConfig, TextConfig};
use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Main game configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameConfig {
    /// Game metadata
    pub game: GameMetadata,
    /// Graphics settings
    #[serde(default)]
    pub graphics: GraphicsConfig,
    /// Audio settings
    #[serde(default)]
    pub audio: AudioConfig,
    /// Text settings
    #[serde(default)]
    pub text: TextConfig,
    /// Path settings
    #[serde(default)]
    pub paths: PathConfig,
}

impl GameConfig {
    /// Create a new game config with default settings
    pub fn new(game: GameMetadata) -> Self {
        Self {
            game,
            graphics: GraphicsConfig::default(),
            audio: AudioConfig::default(),
            text: TextConfig::default(),
            paths: PathConfig::default(),
        }
    }

    /// Load game configuration from a RON file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path.as_ref())?;
        let config: Self = ron::from_str(&contents)?;

        // Validate paths configuration (ensures all asset paths are relative and safe)
        config.paths.validate()?;

        Ok(config)
    }

    /// Save game configuration to a RON file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let path_ref = path.as_ref();

        // Validate paths before saving
        self.paths.validate()?;

        // Ensure parent directory exists
        if let Some(parent) = path_ref.parent() {
            fs::create_dir_all(parent)?;
        }

        let pretty_config = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .struct_names(true)
            .enumerate_arrays(false);

        let contents = ron::ser::to_string_pretty(self, pretty_config)?;
        fs::write(path_ref, contents)?;

        Ok(())
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self::new(GameMetadata::default())
    }
}

/// Game metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameMetadata {
    /// Game title
    pub title: String,
    /// Game version
    #[serde(default = "default_version")]
    pub version: String,
    /// Developer/author name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer: Option<String>,
    /// Window title (defaults to game title if not set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_title: Option<String>,
}

impl GameMetadata {
    /// Create new game metadata
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            version: default_version(),
            developer: None,
            window_title: None,
        }
    }

    /// Get the window title (falls back to game title)
    pub fn get_window_title(&self) -> &str {
        match &self.window_title {
            Some(title) => title.as_str(),
            None => &self.title,
        }
    }
}

impl Default for GameMetadata {
    fn default() -> Self {
        Self::new("Untitled Visual Novel")
    }
}

fn default_version() -> String {
    "0.1.0".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_metadata_new() {
        let metadata = GameMetadata::new("My Game");
        assert_eq!(metadata.title, "My Game");
        assert_eq!(metadata.version, "0.1.0");
        assert_eq!(metadata.developer, None);
        assert_eq!(metadata.window_title, None);
    }

    #[test]
    fn test_game_metadata_default() {
        let metadata = GameMetadata::default();
        assert_eq!(metadata.title, "Untitled Visual Novel");
        assert_eq!(metadata.version, "0.1.0");
    }

    #[test]
    fn test_game_metadata_get_window_title_fallback() {
        let metadata = GameMetadata::new("Test Game");
        assert_eq!(metadata.get_window_title(), "Test Game");
    }

    #[test]
    fn test_game_metadata_get_window_title_custom() {
        let mut metadata = GameMetadata::new("Test Game");
        metadata.window_title = Some("Custom Window Title".to_string());
        assert_eq!(metadata.get_window_title(), "Custom Window Title");
    }

    #[test]
    fn test_game_metadata_with_developer() {
        let mut metadata = GameMetadata::new("My Game");
        metadata.developer = Some("My Studio".to_string());
        assert_eq!(metadata.developer, Some("My Studio".to_string()));
    }

    #[test]
    fn test_game_metadata_with_version() {
        let mut metadata = GameMetadata::new("My Game");
        metadata.version = "1.0.0".to_string();
        assert_eq!(metadata.version, "1.0.0");
    }

    #[test]
    fn test_game_metadata_serialization() {
        let metadata = GameMetadata::new("Test");
        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: GameMetadata = serde_json::from_str(&serialized).unwrap();
        assert_eq!(metadata, deserialized);
    }

    #[test]
    fn test_game_config_new() {
        let metadata = GameMetadata::new("Config Test");
        let config = GameConfig::new(metadata.clone());
        assert_eq!(config.game.title, "Config Test");
        assert_eq!(config.graphics.width, 1280);
        assert_eq!(config.audio.master_volume, 1.0);
        assert_eq!(config.text.font_size, 24.0);
    }

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert_eq!(config.game.title, "Untitled Visual Novel");
        assert_eq!(config.graphics.width, 1280);
    }

    #[test]
    fn test_game_config_with_custom_metadata() {
        let mut metadata = GameMetadata::new("Adventure Game");
        metadata.developer = Some("Indie Dev".to_string());
        metadata.version = "2.0.0".to_string();

        let config = GameConfig::new(metadata);
        assert_eq!(config.game.title, "Adventure Game");
        assert_eq!(config.game.developer, Some("Indie Dev".to_string()));
        assert_eq!(config.game.version, "2.0.0");
    }

    #[test]
    fn test_game_config_serialization() {
        let config = GameConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: GameConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_game_config_all_subsystems() {
        let mut config = GameConfig::default();

        // Test graphics config is accessible
        config.graphics.width = 1920;
        assert_eq!(config.graphics.width, 1920);

        // Test audio config is accessible
        config.audio.master_volume = 0.8;
        assert_eq!(config.audio.master_volume, 0.8);

        // Test text config is accessible
        config.text.font_size = 32.0;
        assert_eq!(config.text.font_size, 32.0);

        // Test paths config is accessible
        config.paths.scenarios = "custom/scenarios/".into();
        assert_eq!(
            config.paths.scenarios,
            std::path::PathBuf::from("custom/scenarios/")
        );
    }

    #[test]
    fn test_game_config_ron_serialization() {
        let config = GameConfig::default();
        let ron_str = ron::to_string(&config).unwrap();
        let deserialized: GameConfig = ron::from_str(&ron_str).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_game_config_ron_pretty_serialization() {
        let config = GameConfig::default();
        let pretty_config = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .struct_names(true);
        let ron_str = ron::ser::to_string_pretty(&config, pretty_config).unwrap();
        let deserialized: GameConfig = ron::from_str(&ron_str).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_game_config_load_save() {
        // Create a temporary config file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let config = GameConfig::default();

        // Save config to file
        config.save_to_file(temp_file.path()).unwrap();

        // Load config from file
        let loaded_config = GameConfig::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config, loaded_config);
    }

    #[test]
    fn test_game_config_load_error_file_not_found() {
        let result = GameConfig::load_from_file("non_existent_file.ron");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Io(_)));
    }

    #[test]
    fn test_game_config_load_error_invalid_ron() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), b"invalid RON content").unwrap();

        let result = GameConfig::load_from_file(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::RonDe(_)));
    }

    #[test]
    fn test_game_config_with_all_fields() {
        let mut metadata = GameMetadata::new("Full Config Test");
        metadata.version = "2.5.0".to_string();
        metadata.developer = Some("Test Studio".to_string());
        metadata.window_title = Some("Test Window".to_string());

        let mut config = GameConfig::new(metadata);

        // Set graphics
        config.graphics.width = 1920;
        config.graphics.height = 1080;
        config.graphics.fullscreen = true;

        // Set audio
        config.audio.master_volume = 0.9;
        config.audio.bgm_volume = 0.7;

        // Set text
        config.text.font_size = 28.0;
        config.text.typewriter_speed = 60.0;

        // Set paths
        config.paths.scenarios = "data/scenarios/".into();

        // Verify all fields
        assert_eq!(config.game.title, "Full Config Test");
        assert_eq!(config.game.version, "2.5.0");
        assert_eq!(config.graphics.width, 1920);
        assert_eq!(config.audio.master_volume, 0.9);
        assert_eq!(config.text.font_size, 28.0);
        assert_eq!(
            config.paths.scenarios,
            std::path::PathBuf::from("data/scenarios/")
        );
    }

    #[test]
    fn test_game_config_load_validates_paths() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let mut config = GameConfig::default();

        #[cfg(unix)]
        {
            config.paths.scenarios = std::path::PathBuf::from("/absolute/path");
        }
        #[cfg(windows)]
        {
            config.paths.scenarios = std::path::PathBuf::from("C:\\absolute\\path");
        }

        // Save config with invalid paths
        let ron_str = ron::to_string(&config).unwrap();
        std::fs::write(temp_file.path(), ron_str).unwrap();

        // Loading should fail due to path validation
        let result = GameConfig::load_from_file(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidValue(_, _)
        ));
    }

    #[test]
    fn test_game_config_save_creates_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("subdir/config.ron");

        let config = GameConfig::default();
        config.save_to_file(&config_path).unwrap();

        assert!(config_path.exists());
        assert!(config_path.parent().unwrap().exists());
    }

    #[test]
    fn test_game_config_save_validates_paths() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let mut config = GameConfig::default();
        config.paths.scenarios = std::path::PathBuf::from("../../../etc/passwd");

        let result = config.save_to_file(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidValue(_, _)
        ));
    }
}
