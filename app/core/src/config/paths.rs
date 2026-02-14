use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Path configuration for asset directories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathConfig {
    /// Scenario files directory
    #[serde(default = "default_scenarios_path")]
    pub scenarios: PathBuf,
    /// Asset files directory (images, fonts, etc.)
    #[serde(default = "default_assets_path")]
    pub assets: PathBuf,
    /// Save files directory
    #[serde(default = "default_saves_path")]
    pub saves: PathBuf,
    /// Character definitions directory
    #[serde(default = "default_characters_path")]
    pub characters: PathBuf,
}

impl PathConfig {
    /// Create a new path config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate all paths in the configuration
    ///
    /// Checks that all paths are:
    /// - Relative paths (not absolute)
    /// - Do not contain path traversal attempts (..)
    pub fn validate(&self) -> Result<(), ConfigError> {
        Self::validate_path(&self.scenarios, "scenarios")?;
        Self::validate_path(&self.assets, "assets")?;
        Self::validate_path(&self.saves, "saves")?;
        Self::validate_path(&self.characters, "characters")?;
        Ok(())
    }

    /// Validate a single path
    fn validate_path(path: &Path, field_name: &str) -> Result<(), ConfigError> {
        // Check if path is absolute
        if path.is_absolute() {
            return Err(ConfigError::InvalidValue(
                field_name.to_string(),
                format!("Absolute paths are not allowed: {}", path.display()),
            ));
        }

        // Check for path traversal attempts
        for component in path.components() {
            if component.as_os_str() == ".." {
                return Err(ConfigError::InvalidValue(
                    field_name.to_string(),
                    format!("Path traversal is not allowed: {}", path.display()),
                ));
            }
        }

        Ok(())
    }

    /// Get the full path to a scenario file
    pub fn scenario_path(&self, scenario: impl AsRef<Path>) -> PathBuf {
        self.scenarios.join(scenario)
    }

    /// Get the full path to an asset file
    pub fn asset_path(&self, asset: impl AsRef<Path>) -> PathBuf {
        self.assets.join(asset)
    }

    /// Get the full path to a save file
    pub fn save_path(&self, save: impl AsRef<Path>) -> PathBuf {
        self.saves.join(save)
    }

    /// Get the full path to a character definition file
    pub fn character_path(&self, character: impl AsRef<Path>) -> PathBuf {
        self.characters.join(character)
    }
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            scenarios: default_scenarios_path(),
            assets: default_assets_path(),
            saves: default_saves_path(),
            characters: default_characters_path(),
        }
    }
}

fn default_scenarios_path() -> PathBuf {
    PathBuf::from("assets/scenarios/")
}

fn default_assets_path() -> PathBuf {
    PathBuf::from("assets/")
}

fn default_saves_path() -> PathBuf {
    PathBuf::from("saves/")
}

fn default_characters_path() -> PathBuf {
    PathBuf::from("characters/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_config_new() {
        let config = PathConfig::new();
        assert_eq!(config.scenarios, PathBuf::from("assets/scenarios/"));
        assert_eq!(config.assets, PathBuf::from("assets/"));
        assert_eq!(config.saves, PathBuf::from("saves/"));
        assert_eq!(config.characters, PathBuf::from("characters/"));
    }

    #[test]
    fn test_path_config_default() {
        let config = PathConfig::default();
        assert_eq!(config.scenarios, PathBuf::from("assets/scenarios/"));
        assert_eq!(config.assets, PathBuf::from("assets/"));
        assert_eq!(config.saves, PathBuf::from("saves/"));
        assert_eq!(config.characters, PathBuf::from("characters/"));
    }

    #[test]
    fn test_path_config_scenario_path() {
        let config = PathConfig::new();
        let path = config.scenario_path("chapter_01.toml");
        assert_eq!(path, PathBuf::from("assets/scenarios/chapter_01.toml"));
    }

    #[test]
    fn test_path_config_asset_path() {
        let config = PathConfig::new();
        let path = config.asset_path("images/bg_room.png");
        assert_eq!(path, PathBuf::from("assets/images/bg_room.png"));
    }

    #[test]
    fn test_path_config_save_path() {
        let config = PathConfig::new();
        let path = config.save_path("save_001.ron");
        assert_eq!(path, PathBuf::from("saves/save_001.ron"));
    }

    #[test]
    fn test_path_config_character_path() {
        let config = PathConfig::new();
        let path = config.character_path("alice.toml");
        assert_eq!(path, PathBuf::from("characters/alice.toml"));
    }

    #[test]
    fn test_path_config_custom_paths() {
        let mut config = PathConfig::new();
        config.scenarios = PathBuf::from("custom/scenarios/");
        config.assets = PathBuf::from("custom/assets/");

        assert_eq!(config.scenarios, PathBuf::from("custom/scenarios/"));
        assert_eq!(config.assets, PathBuf::from("custom/assets/"));
    }

    #[test]
    fn test_path_config_serialization() {
        let config = PathConfig::new();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: PathConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_path_config_nested_paths() {
        let config = PathConfig::new();
        let path = config.asset_path("images/backgrounds/room/day.png");
        assert_eq!(
            path,
            PathBuf::from("assets/images/backgrounds/room/day.png")
        );
    }

    #[test]
    fn test_path_config_validate_success() {
        let config = PathConfig::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_path_config_validate_absolute_path() {
        let mut config = PathConfig::new();
        #[cfg(unix)]
        {
            config.scenarios = PathBuf::from("/absolute/path");
        }
        #[cfg(windows)]
        {
            config.scenarios = PathBuf::from("C:\\absolute\\path");
        }
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidValue(_, _)
        ));
    }

    #[test]
    fn test_path_config_validate_path_traversal() {
        let mut config = PathConfig::new();
        config.assets = PathBuf::from("../../../etc/passwd");
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidValue(_, _)
        ));
    }

    #[test]
    fn test_path_config_validate_relative_with_parent() {
        let mut config = PathConfig::new();
        config.saves = PathBuf::from("data/../saves");
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidValue(_, _)
        ));
    }

    #[test]
    fn test_path_config_validate_nested_relative_path() {
        let mut config = PathConfig::new();
        config.scenarios = PathBuf::from("data/scenarios/chapter01");
        assert!(config.validate().is_ok());
    }
}
