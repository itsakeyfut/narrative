//! Slot metadata for UI display

use super::SaveManager;
use chrono::{TimeZone, Utc};
use narrative_core::{EngineResult, SceneId};
use serde::{Deserialize, Serialize};

/// Slot metadata for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    /// Slot number (0-indexed)
    pub slot: usize,
    /// Whether this slot has save data
    pub exists: bool,
    /// Save timestamp (Unix seconds)
    pub timestamp: u64,
    /// Current scene ID
    pub scene_id: SceneId,
    /// Current scene name (for display)
    pub scene_name: String,
    /// Play time in seconds
    pub play_time_secs: u64,
    /// Thumbnail file path (relative to saves directory)
    pub thumbnail_path: Option<String>,
}

impl SlotInfo {
    /// Create empty slot info
    pub fn empty(slot: usize) -> Self {
        Self {
            slot,
            exists: false,
            timestamp: 0,
            scene_id: SceneId::new(""),
            scene_name: String::new(),
            play_time_secs: 0,
            thumbnail_path: None,
        }
    }

    /// Load slot info from SaveManager
    pub fn load(save_manager: &SaveManager, slot: usize) -> EngineResult<Self> {
        if !save_manager.slot_exists(slot) {
            return Ok(Self::empty(slot));
        }

        let save_data = save_manager.load(slot)?;

        Ok(Self {
            slot,
            exists: true,
            timestamp: save_data.timestamp,
            scene_id: save_data.current_scene.clone(),
            scene_name: save_data.current_scene.as_str().to_string(),
            play_time_secs: save_data.play_time_secs,
            thumbnail_path: save_data.thumbnail_path.clone(),
        })
    }

    /// Format timestamp as date/time string
    pub fn formatted_date(&self) -> String {
        if self.timestamp == 0 {
            return String::new();
        }

        // Use chrono for accurate date/time formatting
        let datetime = Utc
            .timestamp_opt(self.timestamp as i64, 0)
            .single()
            .unwrap_or_else(Utc::now);

        datetime.format("%Y/%m/%d %H:%M:%S").to_string()
    }

    /// Format play time as HH:MM:SS
    pub fn formatted_play_time(&self) -> String {
        let hours = self.play_time_secs / 3600;
        let minutes = (self.play_time_secs % 3600) / 60;
        let seconds = self.play_time_secs % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Get short scene name (truncated for grid layout)
    pub fn scene_name_short(&self) -> String {
        if self.scene_name.len() > 15 {
            format!("{}...", &self.scene_name[..12])
        } else {
            self.scene_name.clone()
        }
    }

    /// Format date in short format (MM/DD HH:MM)
    pub fn formatted_date_short(&self) -> String {
        if self.timestamp == 0 {
            return String::new();
        }

        // Use chrono for accurate date/time formatting
        let datetime = Utc
            .timestamp_opt(self.timestamp as i64, 0)
            .single()
            .unwrap_or_else(Utc::now);

        datetime.format("%m/%d %H:%M").to_string()
    }
}

/// List all slot information (0 to max_slots)
pub fn list_all_slots(save_manager: &SaveManager, max_slots: usize) -> Vec<SlotInfo> {
    (0..max_slots)
        .map(|slot| {
            SlotInfo::load(save_manager, slot).unwrap_or_else(|e| {
                tracing::warn!("Failed to load slot {}: {}", slot, e);
                SlotInfo::empty(slot)
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_info_empty() {
        let slot = SlotInfo::empty(5);
        assert_eq!(slot.slot, 5);
        assert!(!slot.exists);
        assert_eq!(slot.timestamp, 0);
        assert_eq!(slot.play_time_secs, 0);
    }

    #[test]
    fn test_formatted_play_time() {
        let slot = SlotInfo {
            slot: 0,
            exists: true,
            timestamp: 0,
            scene_id: SceneId::new("test"),
            scene_name: "Test Scene".to_string(),
            play_time_secs: 3661, // 1 hour, 1 minute, 1 second
            thumbnail_path: None,
        };

        assert_eq!(slot.formatted_play_time(), "01:01:01");
    }

    #[test]
    fn test_scene_name_short() {
        let mut slot = SlotInfo::empty(0);

        // Short name
        slot.scene_name = "Chapter 1".to_string();
        assert_eq!(slot.scene_name_short(), "Chapter 1");

        // Long name
        slot.scene_name = "A Very Long Scene Name That Should Be Truncated".to_string();
        assert!(slot.scene_name_short().ends_with("..."));
        assert!(slot.scene_name_short().len() <= 18); // 15 chars + "..."
    }
}
