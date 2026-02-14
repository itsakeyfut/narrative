//! Save manager

use super::SaveData;
use narrative_core::EngineResult;
use std::fs;
use std::path::PathBuf;

/// Save file manager
pub struct SaveManager {
    save_directory: PathBuf,
}

impl SaveManager {
    /// Create a new save manager
    pub fn new(save_directory: PathBuf) -> Self {
        Self { save_directory }
    }

    /// Get the file path for a save slot
    fn slot_path(&self, slot: usize) -> PathBuf {
        self.save_directory.join(format!("slot_{:02}.ron", slot))
    }

    /// Ensure save directory exists
    fn ensure_save_directory(&self) -> EngineResult<()> {
        if !self.save_directory.exists() {
            fs::create_dir_all(&self.save_directory).map_err(|e| {
                narrative_core::EngineError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create save directory '{}': {}",
                        self.save_directory.display(),
                        e
                    ),
                ))
            })?;
        }
        Ok(())
    }

    /// Save game state to a slot
    ///
    /// This saves the game state to a RON file in the save directory using atomic write.
    /// The directory will be created if it doesn't exist.
    ///
    /// # Atomic Write Process
    /// To prevent data corruption during save (e.g., crash while writing):
    /// 1. Write to a temporary file (slot_XX.ron.tmp)
    /// 2. Atomically rename the temp file to the final file
    ///
    /// This ensures that the save file is either fully written or not changed at all.
    ///
    /// # Arguments
    /// * `slot` - The save slot number (e.g., 1 for slot_01.ron)
    /// * `data` - The save data to write
    ///
    /// # Errors
    /// Returns an error if:
    /// - The save directory cannot be created
    /// - The save data cannot be serialized
    /// - The temporary file cannot be written
    /// - The atomic rename fails
    pub fn save(&self, slot: usize, data: &SaveData) -> EngineResult<()> {
        // Ensure save directory exists
        self.ensure_save_directory()?;

        // Serialize to RON format with pretty printing
        let ron_config = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true)
            .enumerate_arrays(true);

        let serialized = ron::ser::to_string_pretty(data, ron_config).map_err(|e| {
            narrative_core::EngineError::Other(format!("Failed to serialize save data: {}", e))
        })?;

        // Atomic write: Write to temp file, then rename
        let final_path = self.slot_path(slot);
        let temp_path = final_path.with_extension("ron.tmp");

        // Write to temporary file
        fs::write(&temp_path, &serialized).map_err(|e| {
            narrative_core::EngineError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to write temporary save file '{}': {}",
                    temp_path.display(),
                    e
                ),
            ))
        })?;

        // Atomic rename to final location
        fs::rename(&temp_path, &final_path).map_err(|e| {
            // Clean up temp file on error
            let _ = fs::remove_file(&temp_path);
            narrative_core::EngineError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to rename save file from '{}' to '{}': {}",
                    temp_path.display(),
                    final_path.display(),
                    e
                ),
            ))
        })?;

        tracing::info!("Saved game to slot {} ({})", slot, final_path.display());
        Ok(())
    }

    /// Load game state from a slot
    ///
    /// # Arguments
    /// * `slot` - The save slot number to load from
    ///
    /// # Errors
    /// Returns an error if:
    /// - The save file doesn't exist
    /// - The file cannot be read
    /// - The save data cannot be deserialized
    pub fn load(&self, slot: usize) -> EngineResult<SaveData> {
        let path = self.slot_path(slot);

        // Check if file exists
        if !path.exists() {
            return Err(narrative_core::EngineError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Save slot {} not found at '{}'", slot, path.display()),
            )));
        }

        // Read file contents
        let contents = fs::read_to_string(&path).map_err(|e| {
            narrative_core::EngineError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read save file '{}': {}", path.display(), e),
            ))
        })?;

        // Deserialize from RON
        let save_data = ron::from_str::<SaveData>(&contents).map_err(|e| {
            narrative_core::EngineError::Other(format!(
                "Failed to deserialize save file '{}': {}",
                path.display(),
                e
            ))
        })?;

        tracing::info!("Loaded game from slot {} ({})", slot, path.display());
        Ok(save_data)
    }

    /// Check if a save slot exists
    ///
    /// # Arguments
    /// * `slot` - The save slot number to check
    ///
    /// # Returns
    /// `true` if the save file exists, `false` otherwise
    pub fn slot_exists(&self, slot: usize) -> bool {
        self.slot_path(slot).exists()
    }

    /// Delete a save slot
    ///
    /// # Arguments
    /// * `slot` - The save slot number to delete
    ///
    /// # Errors
    /// Returns an error if the file exists but cannot be deleted
    pub fn delete_slot(&self, slot: usize) -> EngineResult<()> {
        let path = self.slot_path(slot);

        // Only try to delete if file exists
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                narrative_core::EngineError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to delete save file '{}': {}", path.display(), e),
                ))
            })?;
            tracing::info!("Deleted save slot {} ({})", slot, path.display());
        }

        Ok(())
    }

    /// Get save directory
    pub fn save_directory(&self) -> &PathBuf {
        &self.save_directory
    }
}

impl Default for SaveManager {
    fn default() -> Self {
        Self::new(PathBuf::from("saves"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use narrative_core::SceneId;
    use tempfile::TempDir;

    fn create_test_save_data(slot: usize) -> SaveData {
        let mut save_data = SaveData::new(slot);
        save_data.timestamp = 1234567890;
        save_data.play_time_secs = 3600; // 1 hour
        save_data.current_scene = SceneId::new("test_scene");
        save_data.command_index = 42;
        save_data.flags.insert("flag_test".to_string(), true);
        save_data.variables.insert("score".to_string(), 100);
        save_data
            .read_scenes
            .push(SceneId::new("intro_scene".to_string()));
        save_data
    }

    #[test]
    fn test_save_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        assert_eq!(manager.save_directory(), temp_dir.path());
    }

    #[test]
    fn test_save_manager_default() {
        let manager = SaveManager::default();
        assert_eq!(manager.save_directory(), &PathBuf::from("saves"));
    }

    #[test]
    fn test_slot_path_formatting() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let path1 = manager.slot_path(1);
        let path5 = manager.slot_path(5);
        let path99 = manager.slot_path(99);

        assert!(path1.to_string_lossy().ends_with("slot_01.ron"));
        assert!(path5.to_string_lossy().ends_with("slot_05.ron"));
        assert!(path99.to_string_lossy().ends_with("slot_99.ron"));
    }

    #[test]
    fn test_save_and_load_basic() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let save_data = create_test_save_data(1);

        // Save
        manager.save(1, &save_data).unwrap();

        // Verify file exists
        assert!(manager.slot_exists(1));

        // Load
        let loaded_data = manager.load(1).unwrap();

        // Verify data matches
        assert_eq!(loaded_data.slot, save_data.slot);
        assert_eq!(loaded_data.timestamp, save_data.timestamp);
        assert_eq!(loaded_data.current_scene, save_data.current_scene);
        assert_eq!(loaded_data.command_index, save_data.command_index);
        assert_eq!(loaded_data.flags, save_data.flags);
        assert_eq!(loaded_data.variables, save_data.variables);
        assert_eq!(loaded_data.read_scenes, save_data.read_scenes);
    }

    #[test]
    fn test_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let save_dir = temp_dir.path().join("saves");
        let manager = SaveManager::new(save_dir.clone());

        // Directory should not exist yet
        assert!(!save_dir.exists());

        let save_data = create_test_save_data(1);
        manager.save(1, &save_data).unwrap();

        // Directory should be created
        assert!(save_dir.exists());
    }

    #[test]
    fn test_slot_exists_returns_false_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        assert!(!manager.slot_exists(1));
        assert!(!manager.slot_exists(99));
    }

    #[test]
    fn test_load_nonexistent_slot_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let result = manager.load(1);
        assert!(result.is_err());

        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("not found") || error_msg.contains("Not found"));
        }
    }

    #[test]
    fn test_delete_slot() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let save_data = create_test_save_data(1);
        manager.save(1, &save_data).unwrap();

        // Verify file exists
        assert!(manager.slot_exists(1));

        // Delete
        manager.delete_slot(1).unwrap();

        // Verify file is gone
        assert!(!manager.slot_exists(1));
    }

    #[test]
    fn test_delete_nonexistent_slot_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        // Deleting non-existent slot should not error
        let result = manager.delete_slot(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_slots() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        // Create multiple saves
        for slot in 1..=5 {
            let save_data = create_test_save_data(slot);
            manager.save(slot, &save_data).unwrap();
        }

        // Verify all exist
        for slot in 1..=5 {
            assert!(manager.slot_exists(slot));
        }

        // Load and verify each one
        for slot in 1..=5 {
            let loaded = manager.load(slot).unwrap();
            assert_eq!(loaded.slot, slot);
        }

        // Delete slot 3
        manager.delete_slot(3).unwrap();

        // Verify others still exist
        assert!(manager.slot_exists(1));
        assert!(manager.slot_exists(2));
        assert!(!manager.slot_exists(3));
        assert!(manager.slot_exists(4));
        assert!(manager.slot_exists(5));
    }

    #[test]
    fn test_overwrite_existing_slot() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        // Save initial data
        let mut save_data1 = create_test_save_data(1);
        save_data1.command_index = 10;
        manager.save(1, &save_data1).unwrap();

        let loaded1 = manager.load(1).unwrap();
        assert_eq!(loaded1.command_index, 10);

        // Overwrite with new data
        let mut save_data2 = create_test_save_data(1);
        save_data2.command_index = 99;
        manager.save(1, &save_data2).unwrap();

        // Verify new data
        let loaded2 = manager.load(1).unwrap();
        assert_eq!(loaded2.command_index, 99);
    }

    #[test]
    fn test_save_with_empty_flags_and_variables() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let mut save_data = SaveData::new(1);
        save_data.timestamp = 999;
        save_data.current_scene = SceneId::new("empty_scene");

        manager.save(1, &save_data).unwrap();
        let loaded = manager.load(1).unwrap();

        assert!(loaded.flags.is_empty());
        assert!(loaded.variables.is_empty());
        assert!(loaded.read_scenes.is_empty());
    }

    #[test]
    fn test_save_with_many_flags() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let mut save_data = SaveData::new(1);
        save_data.timestamp = 123;
        save_data.current_scene = SceneId::new("test");

        // Add many flags
        for i in 0..100 {
            save_data.flags.insert(format!("flag_{}", i), i % 2 == 0);
        }

        manager.save(1, &save_data).unwrap();
        let loaded = manager.load(1).unwrap();

        assert_eq!(loaded.flags.len(), 100);
        for i in 0..100 {
            assert_eq!(
                loaded.flags.get(&format!("flag_{}", i)),
                Some(&(i % 2 == 0))
            );
        }
    }

    #[test]
    fn test_save_with_scene_stack() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let mut save_data = SaveData::new(1);
        save_data.current_scene = SceneId::new("current");
        save_data
            .scene_stack
            .push((SceneId::new("main".to_string()), 5));
        save_data
            .scene_stack
            .push((SceneId::new("sub1".to_string()), 10));

        manager.save(1, &save_data).unwrap();
        let loaded = manager.load(1).unwrap();

        assert_eq!(loaded.scene_stack.len(), 2);
        assert_eq!(loaded.scene_stack[0].0, SceneId::new("main".to_string()));
        assert_eq!(loaded.scene_stack[0].1, 5);
        assert_eq!(loaded.scene_stack[1].0, SceneId::new("sub1".to_string()));
        assert_eq!(loaded.scene_stack[1].1, 10);
    }

    #[test]
    fn test_ron_format_is_human_readable() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let mut save_data = SaveData::new(1);
        save_data.timestamp = 12345;
        save_data.current_scene = SceneId::new("test_scene");
        save_data.command_index = 5;
        save_data.flags.insert("test_flag".to_string(), true);

        manager.save(1, &save_data).unwrap();

        // Read the raw file content
        let file_path = manager.slot_path(1);
        let content = std::fs::read_to_string(file_path).unwrap();

        // Verify it's RON format and readable
        assert!(content.contains("slot:"));
        assert!(content.contains("timestamp:"));
        assert!(content.contains("test_scene"));
        assert!(content.contains("test_flag"));
    }

    #[test]
    fn test_save_with_version_and_play_time() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let mut save_data = SaveData::new(1);
        save_data.play_time_secs = 7200; // 2 hours
        save_data.current_scene = SceneId::new("chapter2");

        manager.save(1, &save_data).unwrap();
        let loaded = manager.load(1).unwrap();

        assert_eq!(loaded.version, save_data.version);
        assert_eq!(loaded.play_time_secs, 7200);
        assert_eq!(loaded.current_scene, SceneId::new("chapter2"));
    }

    #[test]
    fn test_atomic_save_no_temp_file_remains() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let save_data = create_test_save_data(1);
        manager.save(1, &save_data).unwrap();

        // Verify temporary file was cleaned up
        let temp_path = manager.slot_path(1).with_extension("ron.tmp");
        assert!(
            !temp_path.exists(),
            "Temporary file should not exist after successful save"
        );

        // Verify final file exists
        assert!(manager.slot_exists(1));
    }

    #[test]
    fn test_save_preserves_all_new_fields() {
        use crate::save::SAVE_VERSION;

        let temp_dir = TempDir::new().unwrap();
        let manager = SaveManager::new(temp_dir.path().to_path_buf());

        let mut save_data = SaveData::new(3);
        save_data.version = SAVE_VERSION;
        save_data.timestamp = 9999999;
        save_data.play_time_secs = 12345;
        save_data.current_scene = SceneId::new("final_scene");
        save_data.command_index = 100;
        save_data
            .flags
            .insert("final_boss_defeated".to_string(), true);
        save_data
            .variables
            .insert("completion_rate".to_string(), 95);

        manager.save(3, &save_data).unwrap();
        let loaded = manager.load(3).unwrap();

        assert_eq!(loaded.version, SAVE_VERSION);
        assert_eq!(loaded.slot, 3);
        assert_eq!(loaded.timestamp, 9999999);
        assert_eq!(loaded.play_time_secs, 12345);
        assert_eq!(loaded.current_scene, SceneId::new("final_scene"));
        assert_eq!(loaded.command_index, 100);
        assert_eq!(loaded.flags.get("final_boss_defeated"), Some(&true));
        assert_eq!(loaded.variables.get("completion_rate"), Some(&95));
    }
}
