//! Flag storage and management

use narrative_core::FlagId;
use std::collections::HashMap;

/// Storage for boolean flags
#[derive(Debug, Clone, Default)]
pub struct FlagStore {
    flags: HashMap<FlagId, bool>,
}

impl FlagStore {
    /// Create a new empty flag store
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a flag value
    pub fn set(&mut self, flag: FlagId, value: bool) {
        self.flags.insert(flag, value);
    }

    /// Get a flag value (defaults to false if not set)
    pub fn get(&self, flag: &FlagId) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }

    /// Check if a flag is set to true
    pub fn is_set(&self, flag: &FlagId) -> bool {
        self.get(flag)
    }

    /// Toggle a flag value (true -> false, false -> true)
    pub fn toggle(&mut self, flag: &FlagId) {
        let current = self.get(flag);
        self.set(flag.clone(), !current);
    }

    /// Clear all flags
    pub fn clear(&mut self) {
        self.flags.clear();
    }

    /// Convert flags to save data format (HashMap<String, bool>)
    pub fn to_save_format(&self) -> HashMap<String, bool> {
        self.flags
            .iter()
            .map(|(flag_id, value)| (flag_id.name().to_string(), *value))
            .collect()
    }

    /// Load flags from save data format (HashMap<String, bool>)
    pub fn from_save_format(data: &HashMap<String, bool>) -> Self {
        let flags = data
            .iter()
            .map(|(name, value)| (FlagId::new(name.clone()), *value))
            .collect();

        Self { flags }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_operations() {
        let mut store = FlagStore::new();
        let flag = FlagId::new("test_flag");

        assert!(!store.is_set(&flag));

        store.set(flag.clone(), true);
        assert!(store.is_set(&flag));

        store.set(flag.clone(), false);
        assert!(!store.is_set(&flag));
    }

    #[test]
    fn test_flag_default_value() {
        let store = FlagStore::new();
        let flag = FlagId::new("nonexistent_flag");

        // Default value should be false
        assert!(!store.get(&flag));
        assert!(!store.is_set(&flag));
    }

    #[test]
    fn test_multiple_flags() {
        let mut store = FlagStore::new();
        let flag1 = FlagId::new("flag_1");
        let flag2 = FlagId::new("flag_2");
        let flag3 = FlagId::new("flag_3");

        store.set(flag1.clone(), true);
        store.set(flag2.clone(), false);
        store.set(flag3.clone(), true);

        assert!(store.is_set(&flag1));
        assert!(!store.is_set(&flag2));
        assert!(store.is_set(&flag3));
    }

    #[test]
    fn test_flag_overwrite() {
        let mut store = FlagStore::new();
        let flag = FlagId::new("flag");

        store.set(flag.clone(), true);
        assert!(store.is_set(&flag));

        store.set(flag.clone(), false);
        assert!(!store.is_set(&flag));

        store.set(flag.clone(), true);
        assert!(store.is_set(&flag));
    }

    #[test]
    fn test_flag_clear() {
        let mut store = FlagStore::new();
        let flag1 = FlagId::new("flag_1");
        let flag2 = FlagId::new("flag_2");

        store.set(flag1.clone(), true);
        store.set(flag2.clone(), true);

        assert!(store.is_set(&flag1));
        assert!(store.is_set(&flag2));

        store.clear();

        assert!(!store.is_set(&flag1));
        assert!(!store.is_set(&flag2));
    }

    #[test]
    fn test_flag_clone() {
        let mut store1 = FlagStore::new();
        let flag = FlagId::new("flag");

        store1.set(flag.clone(), true);

        let store2 = store1.clone();
        assert!(store2.is_set(&flag));

        // Modifying clone shouldn't affect original
        drop(store1);
        assert!(store2.is_set(&flag));
    }

    #[test]
    fn test_flag_store_default() {
        let store = FlagStore::default();
        let flag = FlagId::new("any_flag");

        assert!(!store.is_set(&flag));
    }

    #[test]
    fn test_many_flags() {
        let mut store = FlagStore::new();

        // Add many flags
        for i in 0..100 {
            let flag = FlagId::new(format!("flag_{}", i));
            store.set(flag, i % 2 == 0);
        }

        // Verify flags
        for i in 0..100 {
            let flag = FlagId::new(format!("flag_{}", i));
            assert_eq!(store.is_set(&flag), i % 2 == 0);
        }
    }

    #[test]
    fn test_flag_toggle() {
        let mut store = FlagStore::new();
        let flag = FlagId::new("toggle_test");

        // Initially false
        assert!(!store.is_set(&flag));

        // Toggle to true
        store.toggle(&flag);
        assert!(store.is_set(&flag));

        // Toggle back to false
        store.toggle(&flag);
        assert!(!store.is_set(&flag));

        // Toggle again to true
        store.toggle(&flag);
        assert!(store.is_set(&flag));
    }

    #[test]
    fn test_toggle_multiple_flags() {
        let mut store = FlagStore::new();
        let flag1 = FlagId::new("flag_1");
        let flag2 = FlagId::new("flag_2");

        store.set(flag1.clone(), true);
        store.set(flag2.clone(), false);

        // Toggle both
        store.toggle(&flag1);
        store.toggle(&flag2);

        assert!(!store.is_set(&flag1));
        assert!(store.is_set(&flag2));
    }

    #[test]
    fn test_to_save_format() {
        let mut store = FlagStore::new();
        store.set(FlagId::new("flag_a"), true);
        store.set(FlagId::new("flag_b"), false);
        store.set(FlagId::new("flag_c"), true);

        let save_data = store.to_save_format();

        assert_eq!(save_data.len(), 3);
        assert_eq!(save_data.get("flag_a"), Some(&true));
        assert_eq!(save_data.get("flag_b"), Some(&false));
        assert_eq!(save_data.get("flag_c"), Some(&true));
    }

    #[test]
    fn test_from_save_format() {
        let mut save_data = HashMap::new();
        save_data.insert("flag_x".to_string(), true);
        save_data.insert("flag_y".to_string(), false);
        save_data.insert("flag_z".to_string(), true);

        let store = FlagStore::from_save_format(&save_data);

        assert!(store.is_set(&FlagId::new("flag_x")));
        assert!(!store.is_set(&FlagId::new("flag_y")));
        assert!(store.is_set(&FlagId::new("flag_z")));
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut store1 = FlagStore::new();
        store1.set(FlagId::new("completed_chapter_1"), true);
        store1.set(FlagId::new("unlocked_ending_a"), true);
        store1.set(FlagId::new("saw_secret_scene"), false);

        // Convert to save format
        let save_data = store1.to_save_format();

        // Load from save format
        let store2 = FlagStore::from_save_format(&save_data);

        // Verify all flags match
        assert_eq!(
            store1.is_set(&FlagId::new("completed_chapter_1")),
            store2.is_set(&FlagId::new("completed_chapter_1"))
        );
        assert_eq!(
            store1.is_set(&FlagId::new("unlocked_ending_a")),
            store2.is_set(&FlagId::new("unlocked_ending_a"))
        );
        assert_eq!(
            store1.is_set(&FlagId::new("saw_secret_scene")),
            store2.is_set(&FlagId::new("saw_secret_scene"))
        );
    }

    #[test]
    fn test_from_save_format_empty() {
        let save_data = HashMap::new();
        let store = FlagStore::from_save_format(&save_data);

        let flag = FlagId::new("any_flag");
        assert!(!store.is_set(&flag));
    }
}
