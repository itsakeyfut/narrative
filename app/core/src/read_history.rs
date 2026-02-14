//! Read history tracking
//!
//! Tracks which dialogue lines have been read by the player.
//! Used for "skip read text" functionality in visual novels.

use crate::SceneId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Unique identifier for a dialogue line
///
/// Combines scene ID and command index to uniquely identify a dialogue line.
/// This enables dialogue-level read tracking for skip functionality.
///
/// # Examples
///
/// ```
/// use narrative_core::{DialogueId, SceneId};
///
/// let id = DialogueId::new(SceneId::new("scene_01"), 5);
/// assert_eq!(id.scene_id, SceneId::new("scene_01"));
/// assert_eq!(id.command_index, 5);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DialogueId {
    /// Scene ID
    pub scene_id: SceneId,
    /// Command index within the scene
    pub command_index: usize,
}

impl DialogueId {
    /// Create a new dialogue ID
    pub fn new(scene_id: SceneId, command_index: usize) -> Self {
        Self {
            scene_id,
            command_index,
        }
    }
}

/// Read history tracker
///
/// Tracks which dialogue lines have been read by the player.
/// Uses dialogue-level granularity (SceneId + command_index) for precise skip control.
///
/// # Examples
///
/// ```
/// use narrative_core::{ReadHistory, SceneId};
///
/// let mut history = ReadHistory::new();
/// let scene_id = SceneId::new("scene_01");
///
/// // Mark dialogue as read
/// history.mark_read(scene_id.clone(), 0);
/// assert!(history.is_read(&scene_id, 0));
/// assert!(!history.is_read(&scene_id, 1)); // Different command index
///
/// // Get total read count
/// assert_eq!(history.read_count(), 1);
/// ```
///
/// # Performance
///
/// - `mark_read()`: O(1) average case (HashSet insertion)
/// - `is_read()`: O(1) average case (HashSet lookup)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadHistory {
    /// Set of read dialogue IDs
    read_dialogues: HashSet<DialogueId>,
}

impl ReadHistory {
    /// Create a new empty read history
    pub fn new() -> Self {
        Self {
            read_dialogues: HashSet::new(),
        }
    }

    /// Mark a dialogue as read
    pub fn mark_read(&mut self, scene_id: SceneId, command_index: usize) {
        self.read_dialogues
            .insert(DialogueId::new(scene_id, command_index));
    }

    /// Check if a dialogue has been read
    pub fn is_read(&self, scene_id: &SceneId, command_index: usize) -> bool {
        let id = DialogueId {
            scene_id: scene_id.clone(),
            command_index,
        };
        self.read_dialogues.contains(&id)
    }

    /// Get the total number of read dialogues
    pub fn read_count(&self) -> usize {
        self.read_dialogues.len()
    }

    /// Clear all read history
    pub fn clear(&mut self) {
        self.read_dialogues.clear();
    }

    /// Get an iterator over all read dialogue IDs
    pub fn iter(&self) -> impl Iterator<Item = &DialogueId> {
        self.read_dialogues.iter()
    }
}

impl Default for ReadHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_history_creation() {
        let history = ReadHistory::new();
        assert_eq!(history.read_count(), 0);
    }

    #[test]
    fn test_mark_read() {
        let mut history = ReadHistory::new();
        let scene_id = SceneId::new("scene_01");

        history.mark_read(scene_id.clone(), 0);
        assert!(history.is_read(&scene_id, 0));
        assert_eq!(history.read_count(), 1);
    }

    #[test]
    fn test_is_not_read() {
        let history = ReadHistory::new();
        let scene_id = SceneId::new("scene_01");

        assert!(!history.is_read(&scene_id, 0));
    }

    #[test]
    fn test_multiple_dialogues() {
        let mut history = ReadHistory::new();
        let scene_id = SceneId::new("scene_01");

        history.mark_read(scene_id.clone(), 0);
        history.mark_read(scene_id.clone(), 1);
        history.mark_read(scene_id.clone(), 2);

        assert!(history.is_read(&scene_id, 0));
        assert!(history.is_read(&scene_id, 1));
        assert!(history.is_read(&scene_id, 2));
        assert!(!history.is_read(&scene_id, 3));
        assert_eq!(history.read_count(), 3);
    }

    #[test]
    fn test_multiple_scenes() {
        let mut history = ReadHistory::new();
        let scene1 = SceneId::new("scene_01");
        let scene2 = SceneId::new("scene_02");

        history.mark_read(scene1.clone(), 0);
        history.mark_read(scene2.clone(), 0);

        assert!(history.is_read(&scene1, 0));
        assert!(history.is_read(&scene2, 0));
        assert!(!history.is_read(&scene1, 1));
        assert_eq!(history.read_count(), 2);
    }

    #[test]
    fn test_duplicate_marks() {
        let mut history = ReadHistory::new();
        let scene_id = SceneId::new("scene_01");

        history.mark_read(scene_id.clone(), 0);
        history.mark_read(scene_id.clone(), 0);
        history.mark_read(scene_id.clone(), 0);

        // Should only count once
        assert_eq!(history.read_count(), 1);
    }

    #[test]
    fn test_clear() {
        let mut history = ReadHistory::new();
        let scene_id = SceneId::new("scene_01");

        history.mark_read(scene_id.clone(), 0);
        history.mark_read(scene_id.clone(), 1);
        assert_eq!(history.read_count(), 2);

        history.clear();
        assert_eq!(history.read_count(), 0);
        assert!(!history.is_read(&scene_id, 0));
    }

    #[test]
    fn test_serialization() {
        let mut history = ReadHistory::new();
        let scene_id = SceneId::new("scene_01");

        history.mark_read(scene_id.clone(), 0);
        history.mark_read(scene_id.clone(), 5);

        // Serialize to RON
        let serialized = ron::to_string(&history).unwrap();
        let deserialized: ReadHistory = ron::from_str(&serialized).unwrap();

        assert_eq!(deserialized.read_count(), 2);
        assert!(deserialized.is_read(&scene_id, 0));
        assert!(deserialized.is_read(&scene_id, 5));
    }

    #[test]
    fn test_default() {
        let history = ReadHistory::default();
        assert_eq!(history.read_count(), 0);
    }

    #[test]
    fn test_dialogue_id_equality() {
        let id1 = DialogueId::new(SceneId::new("scene_01"), 0);
        let id2 = DialogueId::new(SceneId::new("scene_01"), 0);
        let id3 = DialogueId::new(SceneId::new("scene_01"), 1);
        let id4 = DialogueId::new(SceneId::new("scene_02"), 0);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id1, id4);
    }

    #[test]
    fn test_iter() {
        let mut history = ReadHistory::new();
        let scene_id = SceneId::new("scene_01");

        history.mark_read(scene_id.clone(), 0);
        history.mark_read(scene_id.clone(), 1);

        let count = history.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_clone() {
        let mut history1 = ReadHistory::new();
        history1.mark_read(SceneId::new("scene_01"), 0);

        let history2 = history1.clone();
        assert_eq!(history2.read_count(), 1);
        assert!(history2.is_read(&SceneId::new("scene_01"), 0));
    }
}
