//! Development asset configuration
//!
//! This module provides functionality to load default assets during development
//! for testing and prototyping purposes.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Development asset configuration
///
/// This configuration is loaded from `assets/config/dev_assets.ron` during development
/// to specify which assets should be loaded at startup for testing purposes.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DevAssetConfig {
    /// Background image path (optional - uses placeholder if None)
    pub background: Option<PathBuf>,
    /// Character image path (optional - uses placeholder if None)
    pub character: Option<PathBuf>,
}

impl DevAssetConfig {
    /// Load development asset configuration from RON file
    ///
    /// Returns `None` if the file doesn't exist or can't be parsed.
    /// This is intentional - development assets are optional.
    /// Errors are logged but not propagated.
    pub fn load() -> Option<Self> {
        let config_path = "assets/config/dev_assets.ron";

        let content = match std::fs::read_to_string(config_path) {
            Ok(content) => content,
            Err(e) => {
                tracing::debug!("assets/config/dev_assets.ron not found: {}", e);
                return None;
            }
        };

        match ron::from_str(&content) {
            Ok(config) => Some(config),
            Err(e) => {
                tracing::error!("Failed to parse assets/config/dev_assets.ron: {}", e);
                tracing::error!("Please check the RON syntax. Using default configuration.");
                None
            }
        }
    }

    /// Check if any assets are configured
    pub fn has_any_assets(&self) -> bool {
        self.background.is_some() || self.character.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DevAssetConfig::default();
        assert_eq!(config.background, None);
        assert_eq!(config.character, None);
        assert!(!config.has_any_assets());
    }

    #[test]
    fn test_has_any_assets() {
        let mut config = DevAssetConfig::default();
        assert!(!config.has_any_assets());

        config.background = Some(PathBuf::from("test.png"));
        assert!(config.has_any_assets());

        config.background = None;
        config.character = Some(PathBuf::from("test.png"));
        assert!(config.has_any_assets());
    }

    #[test]
    fn test_ron_serialization() {
        let config = DevAssetConfig {
            background: Some(PathBuf::from("assets/bg.png")),
            character: Some(PathBuf::from("assets/char.png")),
        };

        let serialized = ron::to_string(&config).unwrap();
        let deserialized: DevAssetConfig = ron::from_str(&serialized).unwrap();

        assert_eq!(config, deserialized);
    }
}
