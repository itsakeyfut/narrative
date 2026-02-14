//! Save data

use narrative_core::{CharacterPosition, ReadHistory, SceneId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current save data format version
pub const SAVE_VERSION: u32 = 1;

/// Save data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// Save data format version
    #[serde(default = "default_version")]
    pub version: u32,
    /// Save slot number
    pub slot: usize,
    /// Save timestamp (Unix timestamp in seconds)
    pub timestamp: u64,
    /// Play time in seconds
    #[serde(default)]
    pub play_time_secs: u64,
    /// Current scene
    pub current_scene: SceneId,
    /// Current command index
    pub command_index: usize,
    /// Flag states
    pub flags: HashMap<String, bool>,
    /// Variable states
    pub variables: HashMap<String, i64>,
    /// Read history (deprecated, use read_history instead)
    #[serde(default)]
    pub read_scenes: Vec<SceneId>,
    /// Read history (dialogue-level tracking for skip functionality)
    #[serde(default)]
    pub read_history: ReadHistory,
    /// Call/Return stack for subroutine tracking
    #[serde(default)]
    pub scene_stack: Vec<(SceneId, usize)>,
    /// Display state: current background
    #[serde(default)]
    pub current_background: Option<String>,
    /// Display state: current CG (event graphics)
    #[serde(default)]
    pub current_cg: Option<String>,
    /// Display state: displayed characters
    #[serde(default)]
    pub displayed_characters: HashMap<String, SavedCharacterDisplay>,
    /// Thumbnail file path (relative to save directory)
    #[serde(default)]
    pub thumbnail_path: Option<String>,
}

/// Serialized character display state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedCharacterDisplay {
    /// Character ID
    pub character_id: String,
    /// Sprite asset path
    pub sprite: String,
    /// Position on screen
    pub position: CharacterPosition,
}

/// Default version for serde
fn default_version() -> u32 {
    SAVE_VERSION
}

impl SaveData {
    /// Create a new save data
    pub fn new(slot: usize) -> Self {
        Self {
            version: SAVE_VERSION,
            slot,
            timestamp: 0,
            play_time_secs: 0,
            current_scene: SceneId::new(""),
            command_index: 0,
            flags: HashMap::new(),
            variables: HashMap::new(),
            read_scenes: Vec::new(),
            read_history: ReadHistory::new(),
            scene_stack: Vec::new(),
            current_background: None,
            current_cg: None,
            displayed_characters: HashMap::new(),
            thumbnail_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_data_creation() {
        let save = SaveData::new(1);

        assert_eq!(save.version, SAVE_VERSION);
        assert_eq!(save.slot, 1);
        assert_eq!(save.timestamp, 0);
        assert_eq!(save.play_time_secs, 0);
        assert_eq!(save.command_index, 0);
        assert!(save.flags.is_empty());
        assert!(save.variables.is_empty());
        assert!(save.read_scenes.is_empty());
    }

    #[test]
    fn test_save_data_clone() {
        let mut save1 = SaveData::new(1);
        save1.timestamp = 12345;
        save1.flags.insert("flag1".to_string(), true);

        let save2 = save1.clone();
        assert_eq!(save2.slot, 1);
        assert_eq!(save2.timestamp, 12345);
        assert_eq!(save2.flags.get("flag1"), Some(&true));
    }

    #[test]
    fn test_save_data_serialization() {
        let mut save = SaveData::new(5);
        save.timestamp = 99999;
        save.current_scene = SceneId::new("scene_01");
        save.command_index = 10;
        save.flags.insert("completed_intro".to_string(), true);
        save.variables.insert("score".to_string(), 100);
        save.read_scenes.push(SceneId::new("scene_01"));

        // Test RON serialization
        let serialized = ron::to_string(&save).unwrap();
        let deserialized: SaveData = ron::from_str(&serialized).unwrap();

        assert_eq!(deserialized.slot, 5);
        assert_eq!(deserialized.timestamp, 99999);
        assert_eq!(deserialized.command_index, 10);
        assert_eq!(deserialized.flags.get("completed_intro"), Some(&true));
        assert_eq!(deserialized.variables.get("score"), Some(&100));
        assert_eq!(deserialized.read_scenes.len(), 1);
    }

    #[test]
    fn test_save_data_with_multiple_flags() {
        let mut save = SaveData::new(1);

        save.flags.insert("flag_a".to_string(), true);
        save.flags.insert("flag_b".to_string(), false);
        save.flags.insert("flag_c".to_string(), true);

        assert_eq!(save.flags.len(), 3);
        assert_eq!(save.flags.get("flag_a"), Some(&true));
        assert_eq!(save.flags.get("flag_b"), Some(&false));
        assert_eq!(save.flags.get("flag_c"), Some(&true));
    }

    #[test]
    fn test_save_data_with_variables() {
        let mut save = SaveData::new(1);

        save.variables.insert("health".to_string(), 100);
        save.variables.insert("mana".to_string(), 50);
        save.variables.insert("gold".to_string(), 9999);

        assert_eq!(save.variables.len(), 3);
        assert_eq!(save.variables.get("health"), Some(&100));
        assert_eq!(save.variables.get("mana"), Some(&50));
        assert_eq!(save.variables.get("gold"), Some(&9999));
    }

    #[test]
    fn test_save_data_read_history() {
        let mut save = SaveData::new(1);

        save.read_scenes.push(SceneId::new("intro"));
        save.read_scenes.push(SceneId::new("chapter_01"));
        save.read_scenes.push(SceneId::new("chapter_02"));

        assert_eq!(save.read_scenes.len(), 3);
    }

    #[test]
    fn test_save_data_with_scene_stack() {
        let mut save = SaveData::new(1);

        // Simulate Call stack
        save.scene_stack.push((SceneId::new("main".to_string()), 5));
        save.scene_stack
            .push((SceneId::new("sub1".to_string()), 10));
        save.scene_stack.push((SceneId::new("sub2".to_string()), 3));

        assert_eq!(save.scene_stack.len(), 3);
        assert_eq!(save.scene_stack[0].0, SceneId::new("main".to_string()));
        assert_eq!(save.scene_stack[0].1, 5);
        assert_eq!(save.scene_stack[2].0, SceneId::new("sub2".to_string()));
        assert_eq!(save.scene_stack[2].1, 3);

        // Test serialization with scene_stack
        let serialized = ron::to_string(&save).unwrap();
        let deserialized: SaveData = ron::from_str(&serialized).unwrap();

        assert_eq!(deserialized.scene_stack.len(), 3);
        assert_eq!(
            deserialized.scene_stack[0].0,
            SceneId::new("main".to_string())
        );
        assert_eq!(deserialized.scene_stack[0].1, 5);
    }

    #[test]
    fn test_save_data_version() {
        let save = SaveData::new(1);
        assert_eq!(save.version, SAVE_VERSION);
        assert_eq!(save.version, 1);
    }

    #[test]
    fn test_save_data_play_time() {
        let mut save = SaveData::new(1);
        save.play_time_secs = 3600; // 1 hour

        assert_eq!(save.play_time_secs, 3600);

        // Test serialization with play_time
        let serialized = ron::to_string(&save).unwrap();
        let deserialized: SaveData = ron::from_str(&serialized).unwrap();

        assert_eq!(deserialized.play_time_secs, 3600);
    }

    #[test]
    fn test_save_data_with_default_version() {
        // Test that default values work correctly for new fields
        let mut save = SaveData::new(1);
        save.play_time_secs = 0;

        // Serialize and deserialize
        let serialized = ron::to_string(&save).unwrap();
        let deserialized: SaveData = ron::from_str(&serialized).unwrap();

        // Should preserve version even after round-trip
        assert_eq!(deserialized.version, SAVE_VERSION);
        assert_eq!(deserialized.play_time_secs, 0);
    }

    #[test]
    fn test_backward_compatibility_old_save_format() {
        // Simulate old save format (without display state fields)
        // This tests that old saves can still be loaded
        // Note: Using actual RON format with SceneId as a newtype struct
        let old_save_ron = r#"(
            version: 1,
            slot: 1,
            timestamp: 1234567890,
            play_time_secs: 3600,
            current_scene: ("scene_01"),
            command_index: 10,
            flags: {
                "flag1": true,
                "flag2": false,
            },
            variables: {
                "score": 100,
            },
            read_scenes: [("intro"), ("scene_01")],
            scene_stack: [],
        )"#;

        // Deserialize old format - should use defaults for new fields
        let loaded: SaveData = ron::from_str(old_save_ron).unwrap();

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.slot, 1);
        assert_eq!(loaded.timestamp, 1234567890);
        assert_eq!(loaded.play_time_secs, 3600);
        assert_eq!(loaded.current_scene, SceneId::new("scene_01"));
        assert_eq!(loaded.command_index, 10);
        assert_eq!(loaded.flags.get("flag1"), Some(&true));
        assert_eq!(loaded.variables.get("score"), Some(&100));
        assert_eq!(loaded.read_scenes.len(), 2);

        // New fields should use defaults
        assert!(loaded.current_background.is_none());
        assert!(loaded.displayed_characters.is_empty());
    }

    #[test]
    fn test_save_data_with_display_state() {
        let mut save = SaveData::new(1);
        save.timestamp = 999;
        save.current_scene = SceneId::new("test");
        save.current_background = Some("bg_room".to_string());
        save.displayed_characters.insert(
            "alice".to_string(),
            SavedCharacterDisplay {
                character_id: "alice".to_string(),
                sprite: "alice_happy".to_string(),
                position: CharacterPosition::Left,
            },
        );

        // Serialize and deserialize
        let serialized = ron::to_string(&save).unwrap();
        let deserialized: SaveData = ron::from_str(&serialized).unwrap();

        // Verify display state was preserved
        assert_eq!(deserialized.current_background.as_ref().unwrap(), "bg_room");
        assert_eq!(deserialized.displayed_characters.len(), 1);
        assert_eq!(
            deserialized
                .displayed_characters
                .get("alice")
                .unwrap()
                .sprite,
            "alice_happy"
        );
        assert_eq!(
            deserialized
                .displayed_characters
                .get("alice")
                .unwrap()
                .position,
            CharacterPosition::Left
        );
    }
}
