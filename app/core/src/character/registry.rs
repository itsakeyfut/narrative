use super::{CharacterDef, CharacterManifest};
use crate::error::{EngineError, ScenarioError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Character registry - manages loaded character definitions
///
/// The registry loads character definitions from RON files and provides
/// lookup functionality by character ID. It can load from individual files
/// or from a manifest file that lists multiple characters.
pub struct CharacterRegistry {
    /// Map of character ID to character definition
    characters: HashMap<String, CharacterDef>,
    /// Base directory for resolving relative paths
    base_dir: PathBuf,
}

impl CharacterRegistry {
    /// Create a new empty character registry
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            base_dir: PathBuf::from("."),
        }
    }

    /// Create a registry with a specific base directory
    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            characters: HashMap::new(),
            base_dir: base_dir.into(),
        }
    }

    /// Set the base directory for resolving relative paths
    pub fn set_base_dir(&mut self, base_dir: impl Into<PathBuf>) {
        self.base_dir = base_dir.into();
    }

    /// Get the base directory
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Register a single character definition
    pub fn register(&mut self, character: CharacterDef) -> Result<(), EngineError> {
        // Validate before registering
        character.validate().map_err(EngineError::Other)?;

        if self.characters.contains_key(&character.id) {
            return Err(EngineError::Other(format!(
                "Character '{}' already registered",
                character.id
            )));
        }

        self.characters.insert(character.id.clone(), character);
        Ok(())
    }

    /// Load a character definition from a file
    pub fn load_character(&mut self, path: impl AsRef<Path>) -> Result<&CharacterDef, EngineError> {
        let full_path = self.base_dir.join(path.as_ref());
        let character = CharacterDef::load_from_file(&full_path)?;
        let id = character.id.clone();
        self.register(character)?;

        // Return the registered character
        self.characters.get(&id).ok_or_else(|| {
            EngineError::Other(format!(
                "Character '{}' was registered but not found in registry",
                id
            ))
        })
    }

    /// Load characters from a manifest file
    pub fn load_from_manifest(
        &mut self,
        manifest_path: impl AsRef<Path>,
    ) -> Result<usize, EngineError> {
        let full_manifest_path = self.base_dir.join(manifest_path.as_ref());
        let manifest = CharacterManifest::load_from_file(&full_manifest_path)?;

        // Get the directory containing the manifest for resolving relative paths
        let manifest_dir = full_manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."));

        let mut loaded_count = 0;
        for char_path in &manifest.characters {
            let full_char_path = manifest_dir.join(char_path);
            match CharacterDef::load_from_file(&full_char_path) {
                Ok(character) => {
                    self.register(character)?;
                    loaded_count += 1;
                }
                Err(e) => {
                    return Err(EngineError::Other(format!(
                        "Failed to load character from '{}': {}",
                        char_path, e
                    )));
                }
            }
        }

        Ok(loaded_count)
    }

    /// Get a character definition by ID
    pub fn get(&self, id: &str) -> Result<&CharacterDef, EngineError> {
        self.characters
            .get(id)
            .ok_or_else(|| ScenarioError::CharacterNotFound(id.to_string()).into())
    }

    /// Check if a character is registered
    pub fn contains(&self, id: &str) -> bool {
        self.characters.contains_key(id)
    }

    /// Get all registered character IDs
    pub fn character_ids(&self) -> Vec<&str> {
        self.characters.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of registered characters
    pub fn len(&self) -> usize {
        self.characters.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.characters.is_empty()
    }

    /// Clear all registered characters
    pub fn clear(&mut self) {
        self.characters.clear();
    }

    /// Validate file existence for all registered characters
    ///
    /// Checks that all expression sprite files exist relative to the base directory.
    /// Returns a list of missing files, or an empty vector if all files exist.
    pub fn validate_assets(&self) -> Vec<String> {
        let mut missing_files = Vec::new();

        for character in self.characters.values() {
            for (expression, sprite_path) in &character.expressions {
                let full_path = self.base_dir.join(sprite_path);
                if !full_path.exists() {
                    missing_files.push(format!(
                        "Character '{}' expression '{}': file not found at '{}'",
                        character.id,
                        expression,
                        full_path.display()
                    ));
                }
            }
        }

        missing_files
    }
}

impl Default for CharacterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_character() -> CharacterDef {
        CharacterDef::new("alice", "Alice", "normal")
            .with_expression("normal", "characters/alice/normal.png")
            .with_expression("happy", "characters/alice/happy.png")
            .with_color(255, 200, 200)
    }

    #[test]
    fn test_registry_new() {
        let registry = CharacterRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut registry = CharacterRegistry::new();
        let character = create_test_character();

        assert!(registry.register(character).is_ok());
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("alice"));
    }

    #[test]
    fn test_registry_register_duplicate() {
        let mut registry = CharacterRegistry::new();
        let character1 = create_test_character();
        let character2 = create_test_character();

        assert!(registry.register(character1).is_ok());
        assert!(registry.register(character2).is_err());
    }

    #[test]
    fn test_registry_get() {
        let mut registry = CharacterRegistry::new();
        let character = create_test_character();

        registry.register(character).unwrap();

        let retrieved = registry.get("alice").unwrap();
        assert_eq!(retrieved.id, "alice");
        assert_eq!(retrieved.name, "Alice");
    }

    #[test]
    fn test_registry_get_not_found() {
        let registry = CharacterRegistry::new();
        assert!(registry.get("nonexistent").is_err());
    }

    #[test]
    fn test_registry_character_ids() {
        let mut registry = CharacterRegistry::new();

        registry.register(create_test_character()).unwrap();
        registry
            .register(
                CharacterDef::new("bob", "Bob", "normal")
                    .with_expression("normal", "characters/bob/normal.png"),
            )
            .unwrap();

        let ids = registry.character_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"alice"));
        assert!(ids.contains(&"bob"));
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = CharacterRegistry::new();
        registry.register(create_test_character()).unwrap();

        assert_eq!(registry.len(), 1);
        registry.clear();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_with_base_dir() {
        let registry = CharacterRegistry::with_base_dir("/some/path");
        assert_eq!(registry.base_dir(), Path::new("/some/path"));
    }

    #[test]
    fn test_registry_set_base_dir() {
        let mut registry = CharacterRegistry::new();
        registry.set_base_dir("/new/path");
        assert_eq!(registry.base_dir(), Path::new("/new/path"));
    }

    #[test]
    fn test_registry_load_character() {
        let temp_dir = TempDir::new().unwrap();
        let char_path = temp_dir.path().join("alice.ron");

        // Create a character definition file
        let character = create_test_character();
        let ron_str = ron::to_string(&character).unwrap();
        std::fs::write(&char_path, ron_str).unwrap();

        // Load it through the registry
        let mut registry = CharacterRegistry::with_base_dir(temp_dir.path());
        assert!(registry.load_character("alice.ron").is_ok());
        assert!(registry.contains("alice"));
    }

    #[test]
    fn test_registry_load_from_manifest() {
        let temp_dir = TempDir::new().unwrap();

        // Create character files
        let alice = create_test_character();
        let alice_path = temp_dir.path().join("alice.ron");
        std::fs::write(&alice_path, ron::to_string(&alice).unwrap()).unwrap();

        let bob = CharacterDef::new("bob", "Bob", "normal")
            .with_expression("normal", "characters/bob/normal.png");
        let bob_path = temp_dir.path().join("bob.ron");
        std::fs::write(&bob_path, ron::to_string(&bob).unwrap()).unwrap();

        // Create manifest
        let manifest = CharacterManifest::new()
            .add_character("alice.ron")
            .add_character("bob.ron");
        let manifest_path = temp_dir.path().join("manifest.ron");
        std::fs::write(&manifest_path, ron::to_string(&manifest).unwrap()).unwrap();

        // Load from manifest
        let mut registry = CharacterRegistry::with_base_dir(temp_dir.path());
        let count = registry.load_from_manifest("manifest.ron").unwrap();

        assert_eq!(count, 2);
        assert!(registry.contains("alice"));
        assert!(registry.contains("bob"));
    }

    #[test]
    fn test_registry_validate_invalid_character() {
        let mut registry = CharacterRegistry::new();
        let invalid = CharacterDef::new("", "Name", "normal"); // Empty ID

        assert!(registry.register(invalid).is_err());
    }
}
