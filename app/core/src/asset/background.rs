use crate::error::EngineError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Background image definition
///
/// Defines a background with multiple variants (time of day, weather, etc.)
/// and associated metadata for categorization and searching.
///
/// # Example RON format
///
/// ```ron
/// BackgroundDef(
///     variants: {
///         "noon_sunny": "assets/backgrounds/bg_classroom_noon_sunny.png",
///         "evening_cloudy": "assets/backgrounds/bg_classroom_evening_cloudy.png",
///     },
///     default_variant: "noon_sunny",
///     meta: Some((
///         location: "school",
///         scene_type: "indoor",
///         tags: ["classroom", "daytime"],
///     )),
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundDef {
    /// Map of variant names to image file paths
    /// Example: { "noon_sunny": "path/to/image.png", "night_rain": "path/to/night.png" }
    pub variants: HashMap<String, String>,

    /// Default variant to use when none is specified
    pub default_variant: String,

    /// Optional metadata for categorization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<BackgroundMeta>,
}

impl BackgroundDef {
    /// Create a new background definition with a single variant
    pub fn new(default_variant: impl Into<String>, path: impl Into<String>) -> Self {
        let variant_name = default_variant.into();
        let mut variants = HashMap::new();
        variants.insert(variant_name.clone(), path.into());

        Self {
            variants,
            default_variant: variant_name,
            meta: None,
        }
    }

    /// Add a variant to the background
    pub fn with_variant(mut self, variant: impl Into<String>, path: impl Into<String>) -> Self {
        self.variants.insert(variant.into(), path.into());
        self
    }

    /// Set metadata
    pub fn with_meta(mut self, meta: BackgroundMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Get the path for a specific variant
    pub fn get_variant(&self, variant: &str) -> Option<&str> {
        self.variants.get(variant).map(|s| s.as_str())
    }

    /// Get the default variant path
    pub fn get_default(&self) -> Option<&str> {
        self.get_variant(&self.default_variant)
    }

    /// Validate the background definition
    pub fn validate(&self) -> Result<(), String> {
        if self.variants.is_empty() {
            return Err("Background must have at least one variant".to_string());
        }

        if !self.variants.contains_key(&self.default_variant) {
            return Err(format!(
                "Default variant '{}' not found in variants",
                self.default_variant
            ));
        }

        Ok(())
    }
}

/// Background metadata for categorization and filtering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundMeta {
    /// Location category (e.g., "school", "city", "nature")
    pub location: String,

    /// Scene type (e.g., "indoor", "outdoor")
    pub scene_type: String,

    /// Tags for searching and filtering
    pub tags: Vec<String>,
}

impl BackgroundMeta {
    pub fn new(location: impl Into<String>, scene_type: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            scene_type: scene_type.into(),
            tags: Vec::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Background manifest - defines a collection of backgrounds
///
/// # Example RON format
///
/// ```ron
/// BackgroundManifest(
///     backgrounds: {
///         "bg.school.classroom": BackgroundDef(
///             variants: {
///                 "noon_sunny": "assets/backgrounds/bg_classroom_noon_sunny.png",
///             },
///             default_variant: "noon_sunny",
///             meta: Some((
///                 location: "school",
///                 scene_type: "indoor",
///                 tags: ["classroom"],
///             )),
///         ),
///     },
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundManifest {
    /// Map of background IDs to definitions
    /// IDs use dot notation (e.g., "bg.school.classroom", "bg.city.park")
    pub backgrounds: HashMap<String, BackgroundDef>,
}

impl BackgroundManifest {
    /// Create a new empty manifest
    pub fn new() -> Self {
        Self {
            backgrounds: HashMap::new(),
        }
    }

    /// Add a background definition
    pub fn add_background(mut self, id: impl Into<String>, def: BackgroundDef) -> Self {
        self.backgrounds.insert(id.into(), def);
        self
    }

    /// Load manifest from a RON file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let manifest: Self = ron::from_str(&content).map_err(|e| EngineError::RonSer(e.into()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate all backgrounds in the manifest
    pub fn validate(&self) -> Result<(), EngineError> {
        for (id, bg) in &self.backgrounds {
            bg.validate()
                .map_err(|e| EngineError::Other(format!("Background '{}': {}", id, e)))?;
        }
        Ok(())
    }

    /// Get a background by ID
    pub fn get(&self, id: &str) -> Option<&BackgroundDef> {
        self.backgrounds.get(id)
    }

    /// Get all background IDs
    pub fn ids(&self) -> Vec<&str> {
        self.backgrounds.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for BackgroundManifest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_def_new() {
        let bg = BackgroundDef::new("default", "path/to/bg.png");
        assert_eq!(bg.variants.len(), 1);
        assert_eq!(bg.default_variant, "default");
        assert_eq!(bg.get_default(), Some("path/to/bg.png"));
    }

    #[test]
    fn test_background_def_with_variants() {
        let bg = BackgroundDef::new("noon", "noon.png")
            .with_variant("evening", "evening.png")
            .with_variant("night", "night.png");

        assert_eq!(bg.variants.len(), 3);
        assert_eq!(bg.get_variant("evening"), Some("evening.png"));
    }

    #[test]
    fn test_background_def_validation() {
        let valid = BackgroundDef::new("default", "path.png");
        assert!(valid.validate().is_ok());

        let invalid = BackgroundDef {
            variants: HashMap::new(),
            default_variant: "default".to_string(),
            meta: None,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_background_manifest_serialization() {
        let bg = BackgroundDef::new("default", "bg.png");
        let manifest = BackgroundManifest::new().add_background("test.bg", bg);

        let ron_str = ron::to_string(&manifest).unwrap();
        let deserialized: BackgroundManifest = ron::from_str(&ron_str).unwrap();

        assert_eq!(manifest, deserialized);
    }
}
