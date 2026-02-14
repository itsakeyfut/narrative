use crate::error::EngineError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// BGM (Background Music) definition
///
/// Defines background music with playback settings and metadata.
///
/// # Example RON format
///
/// ```ron
/// BgmDef(
///     id: "bgm.dailylife.school",
///     name: "School Theme",
///     file_path: "assets/audio/music/dailylife/school_theme.ogg",
///     loop_enabled: true,
///     loop_start: Some(4.5),
///     loop_end: Some(120.0),
///     default_volume: 0.8,
///     fade_in_duration: 2.0,
///     fade_out_duration: 2.0,
///     meta: Some((
///         mood: "cheerful",
///         tempo: "moderate",
///         tags: ["school", "dailylife", "upbeat"],
///         duration_secs: Some(120.0),
///     )),
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgmDef {
    /// Unique BGM identifier (dot notation recommended, e.g., "bgm.dailylife.school")
    pub id: String,

    /// Display name
    pub name: String,

    /// Audio file path (relative to assets directory)
    pub file_path: String,

    /// Enable looping
    #[serde(default = "default_true")]
    pub loop_enabled: bool,

    /// Loop start position in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_start: Option<f32>,

    /// Loop end position in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_end: Option<f32>,

    /// Default volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub default_volume: f32,

    /// Fade in duration in seconds
    #[serde(default = "default_fade")]
    pub fade_in_duration: f32,

    /// Fade out duration in seconds
    #[serde(default = "default_fade")]
    pub fade_out_duration: f32,

    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<AudioMeta>,
}

impl BgmDef {
    /// Create a new BGM definition with minimal settings
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        file_path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            file_path: file_path.into(),
            loop_enabled: true,
            loop_start: None,
            loop_end: None,
            default_volume: default_volume(),
            fade_in_duration: default_fade(),
            fade_out_duration: default_fade(),
            meta: None,
        }
    }

    /// Set loop points
    pub fn with_loop(mut self, start: Option<f32>, end: Option<f32>) -> Self {
        self.loop_start = start;
        self.loop_end = end;
        self
    }

    /// Set volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.default_volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set fade durations
    pub fn with_fade(mut self, fade_in: f32, fade_out: f32) -> Self {
        self.fade_in_duration = fade_in;
        self.fade_out_duration = fade_out;
        self
    }

    /// Set metadata
    pub fn with_meta(mut self, meta: AudioMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Validate the BGM definition
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("BGM ID cannot be empty".to_string());
        }

        if self.file_path.is_empty() {
            return Err("BGM file path cannot be empty".to_string());
        }

        if self.default_volume < 0.0 || self.default_volume > 1.0 {
            return Err(format!(
                "BGM default volume must be 0.0-1.0, got {}",
                self.default_volume
            ));
        }

        Ok(())
    }
}

/// Audio metadata for categorization and mood tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioMeta {
    /// Mood/atmosphere (e.g., "cheerful", "melancholic", "tense")
    pub mood: String,

    /// Tempo (e.g., "slow", "moderate", "fast")
    pub tempo: String,

    /// Tags for searching and filtering
    pub tags: Vec<String>,

    /// Track duration in seconds (optional, for reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f32>,
}

impl AudioMeta {
    pub fn new(mood: impl Into<String>, tempo: impl Into<String>) -> Self {
        Self {
            mood: mood.into(),
            tempo: tempo.into(),
            tags: Vec::new(),
            duration_secs: None,
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration_secs = Some(duration);
        self
    }
}

/// BGM manifest - collection of BGM tracks
///
/// # Example RON format
///
/// ```ron
/// BgmManifest(
///     tracks: {
///         "bgm.dailylife.school": BgmDef(
///             id: "bgm.dailylife.school",
///             name: "School Theme",
///             file_path: "assets/audio/music/dailylife/school_theme.ogg",
///             // ... other fields
///         ),
///     },
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgmManifest {
    /// Map of BGM IDs to definitions
    pub tracks: HashMap<String, BgmDef>,
}

impl BgmManifest {
    /// Create a new empty manifest
    pub fn new() -> Self {
        Self {
            tracks: HashMap::new(),
        }
    }

    /// Add a BGM track
    pub fn add_track(mut self, bgm: BgmDef) -> Self {
        self.tracks.insert(bgm.id.clone(), bgm);
        self
    }

    /// Load manifest from a RON file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let manifest: Self = ron::from_str(&content).map_err(|e| EngineError::RonSer(e.into()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate all BGM tracks in the manifest
    pub fn validate(&self) -> Result<(), EngineError> {
        for (id, bgm) in &self.tracks {
            bgm.validate()
                .map_err(|e| EngineError::Other(format!("BGM '{}': {}", id, e)))?;

            if &bgm.id != id {
                return Err(EngineError::Other(format!(
                    "BGM map key '{}' does not match BGM id '{}'",
                    id, bgm.id
                )));
            }
        }
        Ok(())
    }

    /// Get a BGM track by ID
    pub fn get(&self, id: &str) -> Option<&BgmDef> {
        self.tracks.get(id)
    }

    /// Get all BGM IDs
    pub fn ids(&self) -> Vec<&str> {
        self.tracks.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for BgmManifest {
    fn default() -> Self {
        Self::new()
    }
}

fn default_true() -> bool {
    true
}

fn default_volume() -> f32 {
    1.0
}

fn default_fade() -> f32 {
    2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bgm_def_new() {
        let bgm = BgmDef::new("test.bgm", "Test BGM", "path/to/music.ogg");
        assert_eq!(bgm.id, "test.bgm");
        assert_eq!(bgm.name, "Test BGM");
        assert_eq!(bgm.file_path, "path/to/music.ogg");
        assert!(bgm.loop_enabled);
    }

    #[test]
    fn test_bgm_def_builder() {
        let bgm = BgmDef::new("bgm.test", "Test", "music.ogg")
            .with_loop(Some(4.0), Some(120.0))
            .with_volume(0.8)
            .with_fade(3.0, 2.5);

        assert_eq!(bgm.loop_start, Some(4.0));
        assert_eq!(bgm.loop_end, Some(120.0));
        assert_eq!(bgm.default_volume, 0.8);
        assert_eq!(bgm.fade_in_duration, 3.0);
        assert_eq!(bgm.fade_out_duration, 2.5);
    }

    #[test]
    fn test_bgm_def_validation() {
        let valid = BgmDef::new("valid", "Valid", "path.ogg");
        assert!(valid.validate().is_ok());

        let invalid_id = BgmDef::new("", "Name", "path.ogg");
        assert!(invalid_id.validate().is_err());

        let invalid_volume = BgmDef::new("test", "Test", "path.ogg").with_volume(1.5);
        // Volume is clamped in with_volume, so this should be valid
        assert!(invalid_volume.validate().is_ok());
        assert_eq!(invalid_volume.default_volume, 1.0);
    }

    #[test]
    fn test_audio_meta() {
        let meta = AudioMeta::new("cheerful", "moderate")
            .with_tags(vec!["school".to_string(), "upbeat".to_string()])
            .with_duration(120.0);

        assert_eq!(meta.mood, "cheerful");
        assert_eq!(meta.tempo, "moderate");
        assert_eq!(meta.tags.len(), 2);
        assert_eq!(meta.duration_secs, Some(120.0));
    }

    #[test]
    fn test_bgm_manifest_serialization() {
        let bgm = BgmDef::new("test.bgm", "Test", "music.ogg");
        let manifest = BgmManifest::new().add_track(bgm);

        let ron_str = ron::to_string(&manifest).unwrap();
        let deserialized: BgmManifest = ron::from_str(&ron_str).unwrap();

        assert_eq!(manifest, deserialized);
    }
}
