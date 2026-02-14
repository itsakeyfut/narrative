use serde::{Deserialize, Serialize};

/// Scene identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SceneId(pub String);

impl SceneId {
    /// Create a new scene ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the scene ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SceneId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SceneId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Character identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterId(pub String);

impl CharacterId {
    /// Create a new character ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the character ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for CharacterId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CharacterId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Asset reference - lightweight path reference
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetRef(pub String);

impl AssetRef {
    /// Create a new asset reference
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Get the asset path
    pub fn path(&self) -> &str {
        &self.0
    }
}

impl From<String> for AssetRef {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AssetRef {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Audio track identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioId(pub u32);

impl AudioId {
    /// Create a new audio ID
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the inner value
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Flag identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlagId(pub String);

impl FlagId {
    /// Create a new flag ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the flag name
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<String> for FlagId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for FlagId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Variable identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VariableId(pub String);

impl VariableId {
    /// Create a new variable ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the variable name
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<String> for VariableId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for VariableId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_id_creation() {
        let id = SceneId::new("scene_intro");
        assert_eq!(id.as_str(), "scene_intro");
    }

    #[test]
    fn test_scene_id_equality() {
        let id1 = SceneId::new("scene_1");
        let id2 = SceneId::new("scene_1");
        let id3 = SceneId::new("scene_2");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_scene_id_from_str() {
        let id: SceneId = "scene_start".into();
        assert_eq!(id.as_str(), "scene_start");
    }

    #[test]
    fn test_scene_id_from_string() {
        let name = String::from("scene_end");
        let id: SceneId = name.into();
        assert_eq!(id.as_str(), "scene_end");
    }

    #[test]
    fn test_scene_id_serialization() {
        let id = SceneId::new("scene_test");
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: SceneId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_character_id_creation() {
        let id = CharacterId::new("alice");
        assert_eq!(id.as_str(), "alice");
    }

    #[test]
    fn test_character_id_equality() {
        let id1 = CharacterId::new("bob");
        let id2 = CharacterId::new("bob");
        let id3 = CharacterId::new("charlie");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_character_id_from_str() {
        let id: CharacterId = "protagonist".into();
        assert_eq!(id.as_str(), "protagonist");
    }

    #[test]
    fn test_character_id_from_string() {
        let name = String::from("antagonist");
        let id: CharacterId = name.into();
        assert_eq!(id.as_str(), "antagonist");
    }

    #[test]
    fn test_asset_ref_creation() {
        let asset = AssetRef::new("path/to/asset.png");
        assert_eq!(asset.path(), "path/to/asset.png");
    }

    #[test]
    fn test_asset_ref_from_str() {
        let asset: AssetRef = "assets/image.jpg".into();
        assert_eq!(asset.path(), "assets/image.jpg");
    }

    #[test]
    fn test_asset_ref_from_string() {
        let path = String::from("assets/sound.wav");
        let asset: AssetRef = path.into();
        assert_eq!(asset.path(), "assets/sound.wav");
    }

    #[test]
    fn test_asset_ref_serialization() {
        let asset = AssetRef::new("test.png");
        let serialized = serde_json::to_string(&asset).unwrap();
        let deserialized: AssetRef = serde_json::from_str(&serialized).unwrap();
        assert_eq!(asset, deserialized);
    }

    #[test]
    fn test_audio_id_creation() {
        let id = AudioId::new(7);
        assert_eq!(id.get(), 7);
    }

    #[test]
    fn test_flag_id_creation() {
        let flag = FlagId::new("player_choice_a");
        assert_eq!(flag.name(), "player_choice_a");
    }

    #[test]
    fn test_flag_id_from_str() {
        let flag: FlagId = "has_key".into();
        assert_eq!(flag.name(), "has_key");
    }

    #[test]
    fn test_flag_id_from_string() {
        let name = String::from("completed_quest");
        let flag: FlagId = name.into();
        assert_eq!(flag.name(), "completed_quest");
    }

    #[test]
    fn test_flag_id_equality() {
        let flag1 = FlagId::new("test");
        let flag2 = FlagId::new("test");
        let flag3 = FlagId::new("other");
        assert_eq!(flag1, flag2);
        assert_ne!(flag1, flag3);
    }

    #[test]
    fn test_variable_id_creation() {
        let var = VariableId::new("player_health");
        assert_eq!(var.name(), "player_health");
    }

    #[test]
    fn test_variable_id_from_str() {
        let var: VariableId = "score".into();
        assert_eq!(var.name(), "score");
    }

    #[test]
    fn test_variable_id_from_string() {
        let name = String::from("affection_level");
        let var: VariableId = name.into();
        assert_eq!(var.name(), "affection_level");
    }

    #[test]
    fn test_variable_id_equality() {
        let var1 = VariableId::new("counter");
        let var2 = VariableId::new("counter");
        let var3 = VariableId::new("timer");
        assert_eq!(var1, var2);
        assert_ne!(var1, var3);
    }
}
