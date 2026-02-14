use crate::error::EngineError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Sound Effect definition
///
/// Defines a sound effect with playback settings and metadata.
///
/// # Example RON format
///
/// ```ron
/// SeDef(
///     id: "se.ui.click",
///     name: "Click Sound",
///     file_path: "assets/audio/sounds/click.wav",
///     default_volume: 1.0,
///     meta: Some((
///         category: "ui",
///         tags: ["button", "interaction"],
///         loop_enabled: Some(false),
///     )),
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeDef {
    /// Unique SE identifier (dot notation recommended, e.g., "se.ui.click")
    pub id: String,

    /// Display name
    pub name: String,

    /// Audio file path (relative to assets directory)
    pub file_path: String,

    /// Default volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub default_volume: f32,

    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<SeMeta>,
}

impl SeDef {
    /// Create a new sound effect definition
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        file_path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            file_path: file_path.into(),
            default_volume: default_volume(),
            meta: None,
        }
    }

    /// Set volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.default_volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set metadata
    pub fn with_meta(mut self, meta: SeMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Validate the SE definition
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("SE ID cannot be empty".to_string());
        }

        if self.file_path.is_empty() {
            return Err("SE file path cannot be empty".to_string());
        }

        if self.default_volume < 0.0 || self.default_volume > 1.0 {
            return Err(format!(
                "SE default volume must be 0.0-1.0, got {}",
                self.default_volume
            ));
        }

        Ok(())
    }
}

/// Sound effect metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeMeta {
    /// Category (e.g., "ui", "ambient", "action")
    pub category: String,

    /// Tags for searching and filtering
    pub tags: Vec<String>,

    /// Whether this SE should loop
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_enabled: Option<bool>,

    /// Duration in seconds (optional, for reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f32>,
}

impl SeMeta {
    pub fn new(category: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            tags: Vec::new(),
            loop_enabled: None,
            duration_secs: None,
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_loop(mut self, loop_enabled: bool) -> Self {
        self.loop_enabled = Some(loop_enabled);
        self
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration_secs = Some(duration);
        self
    }
}

/// Sound effect manifest - collection of sound effects
///
/// # Example RON format
///
/// ```ron
/// SeManifest(
///     sounds: {
///         "se.ui.click": SeDef(
///             id: "se.ui.click",
///             name: "Click Sound",
///             file_path: "assets/audio/sounds/click.wav",
///             default_volume: 1.0,
///             // ... other fields
///         ),
///     },
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeManifest {
    /// Map of SE IDs to definitions
    pub sounds: HashMap<String, SeDef>,
}

impl SeManifest {
    /// Create a new empty manifest
    pub fn new() -> Self {
        Self {
            sounds: HashMap::new(),
        }
    }

    /// Add a sound effect
    pub fn add_sound(mut self, se: SeDef) -> Self {
        self.sounds.insert(se.id.clone(), se);
        self
    }

    /// Load manifest from a RON file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let manifest: Self = ron::from_str(&content).map_err(|e| EngineError::RonSer(e.into()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate all sound effects in the manifest
    pub fn validate(&self) -> Result<(), EngineError> {
        for (id, se) in &self.sounds {
            se.validate()
                .map_err(|e| EngineError::Other(format!("SE '{}': {}", id, e)))?;

            if &se.id != id {
                return Err(EngineError::Other(format!(
                    "SE map key '{}' does not match SE id '{}'",
                    id, se.id
                )));
            }
        }
        Ok(())
    }

    /// Get a sound effect by ID
    pub fn get(&self, id: &str) -> Option<&SeDef> {
        self.sounds.get(id)
    }

    /// Get all SE IDs
    pub fn ids(&self) -> Vec<&str> {
        self.sounds.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for SeManifest {
    fn default() -> Self {
        Self::new()
    }
}

fn default_volume() -> f32 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_se_def_new() {
        let se = SeDef::new("se.test", "Test SE", "path/to/sound.wav");
        assert_eq!(se.id, "se.test");
        assert_eq!(se.name, "Test SE");
        assert_eq!(se.file_path, "path/to/sound.wav");
        assert_eq!(se.default_volume, 1.0);
    }

    #[test]
    fn test_se_def_builder() {
        let se = SeDef::new("se.test", "Test", "sound.wav").with_volume(0.7);
        assert_eq!(se.default_volume, 0.7);
    }

    #[test]
    fn test_se_def_validation() {
        let valid = SeDef::new("valid", "Valid", "path.wav");
        assert!(valid.validate().is_ok());

        let invalid_id = SeDef::new("", "Name", "path.wav");
        assert!(invalid_id.validate().is_err());
    }

    #[test]
    fn test_se_meta() {
        let meta = SeMeta::new("ui")
            .with_tags(vec!["button".to_string(), "click".to_string()])
            .with_loop(false)
            .with_duration(0.5);

        assert_eq!(meta.category, "ui");
        assert_eq!(meta.tags.len(), 2);
        assert_eq!(meta.loop_enabled, Some(false));
        assert_eq!(meta.duration_secs, Some(0.5));
    }

    #[test]
    fn test_se_manifest_serialization() {
        let se = SeDef::new("test.se", "Test", "sound.wav");
        let manifest = SeManifest::new().add_sound(se);

        let ron_str = ron::to_string(&manifest).unwrap();
        let deserialized: SeManifest = ron::from_str(&ron_str).unwrap();

        assert_eq!(manifest, deserialized);
    }
}
