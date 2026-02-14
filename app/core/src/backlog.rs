//! Backlog system for viewing past dialogues
//!
//! The backlog stores a history of displayed dialogues, allowing players
//! to review past conversations.

use crate::SceneId;
use crate::scenario::dialogue::Speaker;
use serde::{Deserialize, Serialize};

/// A single entry in the backlog
///
/// Represents a dialogue line that was displayed to the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BacklogEntry {
    /// Scene ID where this dialogue appeared
    pub scene_id: SceneId,
    /// Command index within the scene
    pub command_index: usize,
    /// Speaker of this dialogue
    pub speaker: Speaker,
    /// Dialogue text
    pub text: String,
}

impl BacklogEntry {
    /// Create a new backlog entry
    pub fn new(
        scene_id: SceneId,
        command_index: usize,
        speaker: Speaker,
        text: impl Into<String>,
    ) -> Self {
        Self {
            scene_id,
            command_index,
            speaker,
            text: text.into(),
        }
    }

    /// Get the speaker display name
    pub fn speaker_name(&self) -> &str {
        match &self.speaker {
            Speaker::Character(id) => id,
            Speaker::Narrator => "Narrator",
            Speaker::System => "System",
        }
    }
}

/// Backlog storage
///
/// Maintains a chronological history of displayed dialogues.
/// New entries are appended to the end, with the most recent at the tail.
///
/// # Examples
///
/// ```
/// use narrative_core::{Backlog, BacklogEntry, Speaker, SceneId};
///
/// let mut backlog = Backlog::new();
/// let entry = BacklogEntry::new(
///     SceneId::new("scene_01"),
///     0,
///     Speaker::character("alice"),
///     "Hello!"
/// );
///
/// backlog.add_entry(entry);
/// assert_eq!(backlog.len(), 1);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Backlog {
    /// Chronological list of dialogue entries
    entries: Vec<BacklogEntry>,
    /// Maximum number of entries to keep (0 = unlimited)
    max_entries: usize,
}

impl Backlog {
    /// Create a new empty backlog
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 0, // unlimited by default
        }
    }

    /// Create a backlog with a maximum entry limit
    ///
    /// When the limit is reached, oldest entries are removed.
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Add a new entry to the backlog
    ///
    /// If max_entries is set and exceeded, the oldest entry is removed.
    /// Duplicate entries (same scene_id and command_index) are not added.
    pub fn add_entry(&mut self, entry: BacklogEntry) {
        // Check for duplicates - don't add if the same dialogue is already in the backlog
        if self
            .entries
            .iter()
            .any(|e| e.scene_id == entry.scene_id && e.command_index == entry.command_index)
        {
            return; // Already exists, skip adding
        }

        self.entries.push(entry);

        // Enforce maximum size limit
        if self.max_entries > 0 && self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Get all entries in chronological order (oldest first)
    pub fn entries(&self) -> &[BacklogEntry] {
        &self.entries
    }

    /// Get entries in reverse chronological order (newest first)
    pub fn entries_reversed(&self) -> impl Iterator<Item = &BacklogEntry> {
        self.entries.iter().rev()
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the backlog is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get a specific entry by index
    pub fn get(&self, index: usize) -> Option<&BacklogEntry> {
        self.entries.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(scene: &str, index: usize, speaker: &str, text: &str) -> BacklogEntry {
        BacklogEntry::new(
            SceneId::new(scene),
            index,
            Speaker::character(speaker),
            text,
        )
    }

    #[test]
    fn test_backlog_creation() {
        let backlog = Backlog::new();
        assert_eq!(backlog.len(), 0);
        assert!(backlog.is_empty());
    }

    #[test]
    fn test_add_entry() {
        let mut backlog = Backlog::new();
        let entry = create_test_entry("scene_01", 0, "alice", "Hello!");

        backlog.add_entry(entry.clone());
        assert_eq!(backlog.len(), 1);
        assert_eq!(backlog.get(0), Some(&entry));
    }

    #[test]
    fn test_multiple_entries() {
        let mut backlog = Backlog::new();
        backlog.add_entry(create_test_entry("scene_01", 0, "alice", "First"));
        backlog.add_entry(create_test_entry("scene_01", 1, "bob", "Second"));
        backlog.add_entry(create_test_entry("scene_01", 2, "alice", "Third"));

        assert_eq!(backlog.len(), 3);
        assert_eq!(backlog.get(0).unwrap().text, "First");
        assert_eq!(backlog.get(1).unwrap().text, "Second");
        assert_eq!(backlog.get(2).unwrap().text, "Third");
    }

    #[test]
    fn test_max_entries() {
        let mut backlog = Backlog::with_max_entries(2);
        backlog.add_entry(create_test_entry("scene_01", 0, "alice", "First"));
        backlog.add_entry(create_test_entry("scene_01", 1, "bob", "Second"));
        backlog.add_entry(create_test_entry("scene_01", 2, "alice", "Third"));

        // Should only keep the last 2 entries
        assert_eq!(backlog.len(), 2);
        assert_eq!(backlog.get(0).unwrap().text, "Second");
        assert_eq!(backlog.get(1).unwrap().text, "Third");
    }

    #[test]
    fn test_entries_reversed() {
        let mut backlog = Backlog::new();
        backlog.add_entry(create_test_entry("scene_01", 0, "alice", "First"));
        backlog.add_entry(create_test_entry("scene_01", 1, "bob", "Second"));
        backlog.add_entry(create_test_entry("scene_01", 2, "alice", "Third"));

        let reversed: Vec<_> = backlog.entries_reversed().collect();
        assert_eq!(reversed.len(), 3);
        assert_eq!(reversed[0].text, "Third");
        assert_eq!(reversed[1].text, "Second");
        assert_eq!(reversed[2].text, "First");
    }

    #[test]
    fn test_clear() {
        let mut backlog = Backlog::new();
        backlog.add_entry(create_test_entry("scene_01", 0, "alice", "Hello"));
        assert_eq!(backlog.len(), 1);

        backlog.clear();
        assert_eq!(backlog.len(), 0);
        assert!(backlog.is_empty());
    }

    #[test]
    fn test_speaker_name() {
        let entry1 = BacklogEntry::new(
            SceneId::new("scene_01"),
            0,
            Speaker::character("alice"),
            "Hello",
        );
        assert_eq!(entry1.speaker_name(), "alice");

        let entry2 = BacklogEntry::new(
            SceneId::new("scene_01"),
            1,
            Speaker::Narrator,
            "The story begins",
        );
        assert_eq!(entry2.speaker_name(), "Narrator");

        let entry3 = BacklogEntry::new(
            SceneId::new("scene_01"),
            2,
            Speaker::System,
            "System message",
        );
        assert_eq!(entry3.speaker_name(), "System");
    }

    #[test]
    fn test_serialization() {
        let mut backlog = Backlog::new();
        backlog.add_entry(create_test_entry("scene_01", 0, "alice", "Hello"));
        backlog.add_entry(create_test_entry("scene_01", 1, "bob", "Hi"));

        let serialized = ron::to_string(&backlog).unwrap();
        let deserialized: Backlog = ron::from_str(&serialized).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized.get(0).unwrap().text, "Hello");
        assert_eq!(deserialized.get(1).unwrap().text, "Hi");
    }

    #[test]
    fn test_duplicate_entries_not_added() {
        let mut backlog = Backlog::new();
        let entry = create_test_entry("scene_01", 0, "alice", "Hello!");

        // Add the same entry multiple times
        backlog.add_entry(entry.clone());
        backlog.add_entry(entry.clone());
        backlog.add_entry(entry.clone());

        // Should only have one entry
        assert_eq!(backlog.len(), 1);
        assert_eq!(backlog.get(0).unwrap().text, "Hello!");
    }

    #[test]
    fn test_different_entries_added() {
        let mut backlog = Backlog::new();

        // Add entries with different command indices
        backlog.add_entry(create_test_entry("scene_01", 0, "alice", "First"));
        backlog.add_entry(create_test_entry("scene_01", 1, "alice", "Second"));
        backlog.add_entry(create_test_entry("scene_01", 2, "alice", "Third"));

        // All three should be added
        assert_eq!(backlog.len(), 3);
    }

    #[test]
    fn test_duplicate_prevention_with_max_entries() {
        let mut backlog = Backlog::with_max_entries(2);

        // Add entry multiple times
        let entry1 = create_test_entry("scene_01", 0, "alice", "First");
        backlog.add_entry(entry1.clone());
        backlog.add_entry(entry1.clone()); // Duplicate, should not add

        // Add different entries
        backlog.add_entry(create_test_entry("scene_01", 1, "bob", "Second"));
        backlog.add_entry(create_test_entry("scene_01", 2, "alice", "Third"));

        // Should have 2 entries (max limit), with duplicates prevented
        assert_eq!(backlog.len(), 2);
        assert_eq!(backlog.get(0).unwrap().text, "Second");
        assert_eq!(backlog.get(1).unwrap().text, "Third");
    }
}
