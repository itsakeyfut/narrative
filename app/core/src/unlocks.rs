//! Global unlock tracking system
//!
//! This module manages persistent unlock state that persists across all save slots,
//! including unlocked CGs, BGM tracks, achievements, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when working with unlock data
#[derive(Debug, Error)]
pub enum UnlockError {
    /// IO error when reading/writing unlock file
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// RON serialization/deserialization error
    #[error("RON error: {0}")]
    Ron(String),
}

/// Result type for unlock operations
pub type UnlockResult<T> = Result<T, UnlockError>;

/// Global unlock database that persists across all save slots
///
/// This structure tracks permanent unlocks like CG gallery, BGM collection,
/// achievements, etc. Unlike save data which is slot-specific, this data
/// is shared across all playthroughs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnlockData {
    /// Version of the unlock data format
    pub version: u32,

    /// Set of unlocked CG IDs
    pub unlocked_cgs: HashSet<String>,

    /// Set of unlocked BGM IDs (for future BGM gallery)
    pub unlocked_bgm: HashSet<String>,

    /// Set of unlocked achievements (for future achievement system)
    pub unlocked_achievements: HashSet<String>,

    /// Statistics and counters
    pub statistics: UnlockStatistics,
}

/// Statistics and counters for unlock tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UnlockStatistics {
    /// Total number of times the game has been completed
    pub completion_count: u32,

    /// Total playtime across all saves (in seconds)
    pub total_playtime_secs: u64,

    /// Endings reached (ending_id -> count)
    pub endings_reached: std::collections::HashMap<String, u32>,
}

impl Default for UnlockData {
    fn default() -> Self {
        Self {
            version: 1,
            unlocked_cgs: HashSet::new(),
            unlocked_bgm: HashSet::new(),
            unlocked_achievements: HashSet::new(),
            statistics: UnlockStatistics::default(),
        }
    }
}

impl UnlockData {
    /// Create a new empty unlock database
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a CG is unlocked
    pub fn is_cg_unlocked(&self, cg_id: &str) -> bool {
        self.unlocked_cgs.contains(cg_id)
    }

    /// Unlock a CG
    pub fn unlock_cg(&mut self, cg_id: impl Into<String>) -> bool {
        self.unlocked_cgs.insert(cg_id.into())
    }

    /// Get the number of unlocked CGs
    pub fn unlocked_cg_count(&self) -> usize {
        self.unlocked_cgs.len()
    }

    /// Get unlock rate for CGs (0.0 to 1.0)
    ///
    /// Returns 1.0 if total_cgs is 0 (edge case: no CGs defined means 100% completion)
    pub fn cg_unlock_rate(&self, total_cgs: usize) -> f32 {
        if total_cgs == 0 {
            return 1.0; // Edge case: no CGs means 100% completion
        }
        self.unlocked_cg_count() as f32 / total_cgs as f32
    }

    /// Check if a BGM is unlocked
    pub fn is_bgm_unlocked(&self, bgm_id: &str) -> bool {
        self.unlocked_bgm.contains(bgm_id)
    }

    /// Unlock a BGM track
    pub fn unlock_bgm(&mut self, bgm_id: impl Into<String>) -> bool {
        self.unlocked_bgm.insert(bgm_id.into())
    }

    /// Load unlock data from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> UnlockResult<Self> {
        let path = path.as_ref();

        // If file doesn't exist, return default data
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)?;
        let data: UnlockData =
            ron::from_str(&contents).map_err(|e| UnlockError::Ron(e.to_string()))?;

        Ok(data)
    }

    /// Save unlock data to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> UnlockResult<()> {
        let path = path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize to RON format with pretty printing
        let ron_config = ron::ser::PrettyConfig::default()
            .depth_limit(4)
            .indentor("  ".to_string());
        let contents = ron::ser::to_string_pretty(self, ron_config)
            .map_err(|e| UnlockError::Ron(e.to_string()))?;

        // Write to temporary file first for atomic operation
        let temp_path = path.with_extension("ron.tmp");
        fs::write(&temp_path, contents)?;

        // Atomic rename
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Get the default unlock file path
    pub fn default_path() -> PathBuf {
        PathBuf::from("saves/cg/unlocks.ron")
    }

    /// Load unlock data from the default path
    pub fn load_default() -> UnlockResult<Self> {
        Self::load_from_file(Self::default_path())
    }

    /// Save unlock data to the default path
    pub fn save_default(&self) -> UnlockResult<()> {
        self.save_to_file(Self::default_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_unlock_data_new() {
        let data = UnlockData::new();
        assert_eq!(data.version, 1);
        assert_eq!(data.unlocked_cg_count(), 0);
        assert_eq!(data.unlocked_bgm.len(), 0);
    }

    #[test]
    fn test_unlock_cg() {
        let mut data = UnlockData::new();

        assert!(!data.is_cg_unlocked("cg_01"));
        assert!(data.unlock_cg("cg_01"));
        assert!(data.is_cg_unlocked("cg_01"));
        assert_eq!(data.unlocked_cg_count(), 1);

        // Unlocking again returns false (already unlocked)
        assert!(!data.unlock_cg("cg_01"));
        assert_eq!(data.unlocked_cg_count(), 1);
    }

    #[test]
    fn test_cg_unlock_rate() {
        let mut data = UnlockData::new();

        assert_eq!(data.cg_unlock_rate(0), 1.0); // Edge case: no CGs
        assert_eq!(data.cg_unlock_rate(10), 0.0);

        data.unlock_cg("cg_01");
        data.unlock_cg("cg_02");
        data.unlock_cg("cg_03");

        assert_eq!(data.cg_unlock_rate(10), 0.3);
        assert_eq!(data.cg_unlock_rate(3), 1.0);
    }

    #[test]
    fn test_unlock_bgm() {
        let mut data = UnlockData::new();

        assert!(!data.is_bgm_unlocked("bgm_01"));
        assert!(data.unlock_bgm("bgm_01"));
        assert!(data.is_bgm_unlocked("bgm_01"));
    }

    #[test]
    fn test_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("unlocks.ron");

        // Create and save data
        let mut data = UnlockData::new();
        data.unlock_cg("cg_01");
        data.unlock_cg("cg_02");
        data.unlock_bgm("bgm_01");
        data.statistics.completion_count = 5;

        data.save_to_file(&path).unwrap();

        // Load and verify
        let loaded = UnlockData::load_from_file(&path).unwrap();
        assert_eq!(loaded, data);
        assert_eq!(loaded.unlocked_cg_count(), 2);
        assert_eq!(loaded.unlocked_bgm.len(), 1);
        assert_eq!(loaded.statistics.completion_count, 5);
    }

    #[test]
    fn test_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.ron");

        // Loading nonexistent file returns default
        let data = UnlockData::load_from_file(&path).unwrap();
        assert_eq!(data, UnlockData::default());
    }

    #[test]
    fn test_default_path() {
        let path = UnlockData::default_path();
        assert_eq!(path, PathBuf::from("saves/cg/unlocks.ron"));
    }
}
