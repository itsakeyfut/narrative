use super::CharacterPosition;
use crate::types::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sprite rendering mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum SpriteMode {
    /// Single integrated sprite per expression
    #[default]
    Integrated,

    /// Layered sprite composition (base + expression overlays)
    /// NOT YET IMPLEMENTED - Data structure only
    ///
    /// TODO(layered-sprites): Implement layered rendering
    /// Required changes:
    /// - CharacterSpriteElement: Support multiple texture layers
    /// - Renderer: Add multi-texture compositing in shader
    /// - Cache: Store Vec<u64> of texture IDs per character
    Layered {
        poses: HashMap<String, String>,
        expressions: HashMap<String, HashMap<String, Vec<String>>>,
    },
}

/// Character definition loaded from RON or TOML files
///
/// Defines a character's metadata, available expressions (sprite mappings),
/// default settings, and optional voice prefix for audio playback.
///
/// # Example RON format
///
/// ```ron
/// CharacterDef(
///     id: "alice",
///     name: "Alice",
///     color: Some((255, 200, 200)),
///     expressions: {
///         "normal": "characters/alice/normal.png",
///         "happy": "characters/alice/happy.png",
///         "sad": "characters/alice/sad.png",
///     },
///     default_expression: "normal",
///     default_position: Center,
///     voice_prefix: Some("voice/alice/"),
///     sprite_offset: Some((0.0, 75.0)), // (x_offset, y_offset) for sprite positioning adjustment
/// )
/// ```
///
/// # Example TOML format (in scenario files)
///
/// ```toml
/// [[characters]]
/// id = "alice"
/// name = "Alice"
/// default_expression = "normal"
///
/// [characters.expressions]
/// normal = "characters/alice/normal.png"
/// happy = "characters/alice/happy.png"
/// sad = "characters/alice/sad.png"
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterDef {
    /// Character unique identifier (e.g., "alice", "bob")
    pub id: String,
    /// Display name (e.g., "Alice", "Bob")
    pub name: String,
    /// Optional text color for dialogue (RGB tuple 0-255)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<(u8, u8, u8)>,
    /// Map of expression names to sprite file paths
    /// Example: { "normal": "characters/alice/normal.png", "happy": "characters/alice/happy.png" }
    pub expressions: HashMap<String, String>,
    /// Default expression name (must exist in expressions map)
    pub default_expression: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Default position when character appears
    #[serde(default)]
    pub default_position: CharacterPosition,
    /// Optional voice file prefix path (e.g., "voice/alice/")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_prefix: Option<String>,
    /// Optional sprite position offset (x, y) in pixels at reference resolution (1280x720)
    /// Used to compensate for sprite image padding/margins
    /// Positive Y moves sprite down (useful for bottom padding compensation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprite_offset: Option<(f32, f32)>,
    /// Optional sprite scale multiplier (default: 1.0)
    /// Values > 1.0 make sprite larger, < 1.0 make it smaller
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprite_scale: Option<f32>,
    /// Sprite rendering mode (Integrated or Layered)
    #[serde(default)]
    pub sprite_mode: SpriteMode,
}

impl CharacterDef {
    /// Create a new character definition with minimal required fields
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        default_expression: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            color: None,
            expressions: HashMap::new(),
            default_expression: default_expression.into(),
            description: None,
            default_position: CharacterPosition::default(),
            voice_prefix: None,
            sprite_offset: None,
            sprite_scale: None,
            sprite_mode: SpriteMode::default(),
        }
    }

    /// Add an expression sprite to the character
    pub fn with_expression(
        mut self,
        expression: impl Into<String>,
        sprite_path: impl Into<String>,
    ) -> Self {
        self.expressions
            .insert(expression.into(), sprite_path.into());
        self
    }

    /// Set the default position
    pub fn with_position(mut self, position: CharacterPosition) -> Self {
        self.default_position = position;
        self
    }

    /// Set the text color
    pub fn with_color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = Some((r, g, b));
        self
    }

    /// Set the voice prefix
    pub fn with_voice_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.voice_prefix = Some(prefix.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the sprite offset for positioning adjustment
    pub fn with_sprite_offset(mut self, x_offset: f32, y_offset: f32) -> Self {
        self.sprite_offset = Some((x_offset, y_offset));
        self
    }

    /// Set the sprite scale multiplier
    pub fn with_sprite_scale(mut self, scale: f32) -> Self {
        self.sprite_scale = Some(scale);
        self
    }

    /// Get the text color as a Color struct (converts from RGB tuple)
    pub fn get_color(&self) -> Option<Color> {
        self.color.map(|(r, g, b)| Color::rgb8(r, g, b))
    }

    /// Get the sprite path for a given expression
    pub fn get_expression_sprite(&self, expression: &str) -> Option<&str> {
        self.expressions.get(expression).map(|s| s.as_str())
    }

    /// Get the default expression sprite path
    pub fn get_default_sprite(&self) -> Option<&str> {
        self.get_expression_sprite(&self.default_expression)
    }

    /// Validate sprite mode configuration
    ///
    /// Note: Layered sprite mode is accepted but not yet implemented for rendering.
    /// The validation passes with a warning message returned in the Result.
    pub fn validate_sprite_mode(&self) -> Result<(), String> {
        match &self.sprite_mode {
            SpriteMode::Integrated => Ok(()),
            SpriteMode::Layered { .. } => {
                // Layered mode is structurally valid but rendering not yet implemented
                // Log this at the application level when loading characters
                Ok(())
            }
        }
    }

    /// Validate the character definition
    ///
    /// Checks:
    /// - ID is not empty
    /// - Name is not empty
    /// - At least one expression is defined
    /// - Default expression exists in expressions map
    /// - Sprite mode is valid
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Character ID cannot be empty".to_string());
        }

        if self.name.is_empty() {
            return Err("Character name cannot be empty".to_string());
        }

        if self.expressions.is_empty() {
            return Err(format!(
                "Character '{}' has no expressions defined",
                self.id
            ));
        }

        if !self.expressions.contains_key(&self.default_expression) {
            return Err(format!(
                "Character '{}' default expression '{}' not found in expressions map",
                self.id, self.default_expression
            ));
        }

        self.validate_sprite_mode()?;

        Ok(())
    }

    /// Load character definition from a RON file
    pub fn load_from_file(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, crate::error::EngineError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let def: Self =
            ron::from_str(&content).map_err(|e| crate::error::EngineError::RonSer(e.into()))?;
        def.validate().map_err(crate::error::EngineError::Other)?;
        Ok(def)
    }
}

/// Character state in a scene (runtime state)
///
/// Tracks the current display state of a character during gameplay,
/// including position, expression, visibility, and visual properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterState {
    /// Character ID (references CharacterDef)
    pub character_id: String,
    /// Current expression name
    pub expression: String,
    /// Current position
    pub position: CharacterPosition,
    /// Visibility flag
    pub visible: bool,
    /// Opacity (0.0-1.0)
    pub opacity: f32,
    /// Z-order for layering (higher values render on top)
    pub z_order: i32,
}

impl CharacterState {
    /// Create a new character state with default values
    pub fn new(
        character_id: impl Into<String>,
        expression: impl Into<String>,
        position: CharacterPosition,
    ) -> Self {
        Self {
            character_id: character_id.into(),
            expression: expression.into(),
            position,
            visible: true,
            opacity: 1.0,
            z_order: 0,
        }
    }

    /// Create from a CharacterDef with default values
    pub fn from_def(def: &CharacterDef) -> Self {
        Self::new(&def.id, &def.default_expression, def.default_position)
    }

    /// Show the character
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the character
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Set the expression
    pub fn set_expression(&mut self, expression: impl Into<String>) {
        self.expression = expression.into();
    }

    /// Set the position
    pub fn set_position(&mut self, position: CharacterPosition) {
        self.position = position;
    }

    /// Set the opacity (clamped to 0.0-1.0)
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Set the z-order (higher values render on top)
    pub fn set_z_order(&mut self, z_order: i32) {
        self.z_order = z_order;
    }
}

/// Character manifest - defines list of character definition files to load
///
/// # Example RON format
///
/// ```ron
/// CharacterManifest(
///     characters: [
///         "characters/alice.ron",
///         "characters/bob.ron",
///         "characters/carol.ron",
///     ],
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterManifest {
    /// List of character definition file paths (relative to manifest location)
    pub characters: Vec<String>,
}

impl CharacterManifest {
    /// Create a new empty manifest
    pub fn new() -> Self {
        Self {
            characters: Vec::new(),
        }
    }

    /// Add a character file path
    pub fn add_character(mut self, path: impl Into<String>) -> Self {
        self.characters.push(path.into());
        self
    }

    /// Load manifest from a RON file
    pub fn load_from_file(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, crate::error::EngineError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let manifest: Self =
            ron::from_str(&content).map_err(|e| crate::error::EngineError::RonSer(e.into()))?;
        Ok(manifest)
    }
}

impl Default for CharacterManifest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_def_new() {
        let def = CharacterDef::new("alice", "Alice", "normal");
        assert_eq!(def.id, "alice");
        assert_eq!(def.name, "Alice");
        assert_eq!(def.default_expression, "normal");
        assert_eq!(def.color, None);
        assert!(def.expressions.is_empty());
        assert_eq!(def.voice_prefix, None);
    }

    #[test]
    fn test_character_def_builder() {
        let def = CharacterDef::new("bob", "Bob", "normal")
            .with_expression("normal", "characters/bob/normal.png")
            .with_expression("happy", "characters/bob/happy.png")
            .with_color(255, 200, 200)
            .with_position(CharacterPosition::Left)
            .with_voice_prefix("voice/bob/");

        assert_eq!(def.id, "bob");
        assert_eq!(def.name, "Bob");
        assert_eq!(def.expressions.len(), 2);
        assert_eq!(
            def.expressions.get("normal"),
            Some(&"characters/bob/normal.png".to_string())
        );
        assert_eq!(def.color, Some((255, 200, 200)));
        assert_eq!(def.default_position, CharacterPosition::Left);
        assert_eq!(def.voice_prefix, Some("voice/bob/".to_string()));
    }

    #[test]
    fn test_character_def_get_color() {
        let def = CharacterDef::new("charlie", "Charlie", "normal").with_color(128, 64, 32);

        let color = def.get_color().unwrap();
        assert!((color.r - 0.502).abs() < 0.01);
        assert!((color.g - 0.251).abs() < 0.01);
        assert!((color.b - 0.125).abs() < 0.01);
    }

    #[test]
    fn test_character_def_get_expression_sprite() {
        let def = CharacterDef::new("dave", "Dave", "normal")
            .with_expression("normal", "characters/dave/normal.png")
            .with_expression("happy", "characters/dave/happy.png");

        assert_eq!(
            def.get_expression_sprite("normal"),
            Some("characters/dave/normal.png")
        );
        assert_eq!(
            def.get_expression_sprite("happy"),
            Some("characters/dave/happy.png")
        );
        assert_eq!(def.get_expression_sprite("sad"), None);
    }

    #[test]
    fn test_character_def_get_default_sprite() {
        let def = CharacterDef::new("eve", "Eve", "normal")
            .with_expression("normal", "characters/eve/normal.png")
            .with_expression("happy", "characters/eve/happy.png");

        assert_eq!(def.get_default_sprite(), Some("characters/eve/normal.png"));
    }

    #[test]
    fn test_character_def_validate_success() {
        let def = CharacterDef::new("frank", "Frank", "normal")
            .with_expression("normal", "characters/frank/normal.png");

        assert!(def.validate().is_ok());
    }

    #[test]
    fn test_character_def_validate_empty_id() {
        let def = CharacterDef::new("", "Name", "normal").with_expression("normal", "path.png");

        assert!(def.validate().is_err());
    }

    #[test]
    fn test_character_def_validate_empty_name() {
        let def = CharacterDef::new("id", "", "normal").with_expression("normal", "path.png");

        assert!(def.validate().is_err());
    }

    #[test]
    fn test_character_def_validate_no_expressions() {
        let def = CharacterDef::new("george", "George", "normal");

        assert!(def.validate().is_err());
    }

    #[test]
    fn test_character_def_validate_default_not_found() {
        let def = CharacterDef::new("helen", "Helen", "sad").with_expression("normal", "path.png");

        let result = def.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("default expression 'sad' not found")
        );
    }

    #[test]
    fn test_character_def_ron_serialization() {
        let def = CharacterDef::new("ivan", "Ivan", "normal")
            .with_expression("normal", "characters/ivan/normal.png")
            .with_expression("happy", "characters/ivan/happy.png")
            .with_color(255, 255, 255)
            .with_voice_prefix("voice/ivan/");

        let ron_str = ron::to_string(&def).unwrap();
        let deserialized: CharacterDef = ron::from_str(&ron_str).unwrap();

        assert_eq!(def, deserialized);
    }

    #[test]
    fn test_character_state_new() {
        let state = CharacterState::new("alice", "normal", CharacterPosition::Center);

        assert_eq!(state.character_id, "alice");
        assert_eq!(state.expression, "normal");
        assert_eq!(state.position, CharacterPosition::Center);
        assert!(state.visible);
        assert_eq!(state.opacity, 1.0);
        assert_eq!(state.z_order, 0);
    }

    #[test]
    fn test_character_state_from_def() {
        let def = CharacterDef::new("bob", "Bob", "happy")
            .with_expression("happy", "characters/bob/happy.png")
            .with_position(CharacterPosition::Right);

        let state = CharacterState::from_def(&def);

        assert_eq!(state.character_id, "bob");
        assert_eq!(state.expression, "happy");
        assert_eq!(state.position, CharacterPosition::Right);
    }

    #[test]
    fn test_character_state_visibility() {
        let mut state = CharacterState::new("charlie", "normal", CharacterPosition::Left);

        assert!(state.visible);
        state.hide();
        assert!(!state.visible);
        state.show();
        assert!(state.visible);
    }

    #[test]
    fn test_character_state_set_expression() {
        let mut state = CharacterState::new("dave", "normal", CharacterPosition::Center);

        state.set_expression("happy");
        assert_eq!(state.expression, "happy");

        state.set_expression("sad".to_string());
        assert_eq!(state.expression, "sad");
    }

    #[test]
    fn test_character_state_set_position() {
        let mut state = CharacterState::new("eve", "normal", CharacterPosition::Left);

        state.set_position(CharacterPosition::Right);
        assert_eq!(state.position, CharacterPosition::Right);
    }

    #[test]
    fn test_character_state_set_opacity() {
        let mut state = CharacterState::new("frank", "normal", CharacterPosition::Center);

        state.set_opacity(0.5);
        assert_eq!(state.opacity, 0.5);
    }

    #[test]
    fn test_character_state_set_opacity_clamping() {
        let mut state = CharacterState::new("george", "normal", CharacterPosition::Center);

        state.set_opacity(-0.5);
        assert_eq!(state.opacity, 0.0);

        state.set_opacity(1.5);
        assert_eq!(state.opacity, 1.0);
    }

    #[test]
    fn test_character_state_set_z_order() {
        let mut state = CharacterState::new("helen", "normal", CharacterPosition::Center);

        state.set_z_order(10);
        assert_eq!(state.z_order, 10);

        state.set_z_order(-5);
        assert_eq!(state.z_order, -5);
    }

    #[test]
    fn test_character_state_serialization() {
        let state = CharacterState::new("ivan", "happy", CharacterPosition::FarLeft);
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: CharacterState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(state, deserialized);
    }

    #[test]
    fn test_character_manifest_new() {
        let manifest = CharacterManifest::new();
        assert!(manifest.characters.is_empty());
    }

    #[test]
    fn test_character_manifest_default() {
        let manifest = CharacterManifest::default();
        assert!(manifest.characters.is_empty());
    }

    #[test]
    fn test_character_manifest_add_character() {
        let manifest = CharacterManifest::new()
            .add_character("characters/alice.ron")
            .add_character("characters/bob.ron");

        assert_eq!(manifest.characters.len(), 2);
        assert_eq!(manifest.characters[0], "characters/alice.ron");
        assert_eq!(manifest.characters[1], "characters/bob.ron");
    }

    #[test]
    fn test_character_manifest_ron_serialization() {
        let manifest = CharacterManifest::new()
            .add_character("characters/alice.ron")
            .add_character("characters/bob.ron");

        let ron_str = ron::to_string(&manifest).unwrap();
        let deserialized: CharacterManifest = ron::from_str(&ron_str).unwrap();

        assert_eq!(manifest, deserialized);
    }

    #[test]
    fn test_sprite_mode_default() {
        assert_eq!(SpriteMode::default(), SpriteMode::Integrated);
    }

    #[test]
    fn test_character_def_backward_compatibility() {
        // Test that existing TOML/RON without sprite_mode field loads as Integrated
        let toml_str = r#"
            id = "test"
            name = "Test"
            default_expression = "normal"

            [expressions]
            normal = "test.png"
        "#;

        let def: CharacterDef = toml::from_str(toml_str).unwrap();
        assert_eq!(def.sprite_mode, SpriteMode::Integrated);
    }

    #[test]
    fn test_layered_mode_validation() {
        let mut def =
            CharacterDef::new("test", "Test", "normal").with_expression("normal", "test.png");

        def.sprite_mode = SpriteMode::Layered {
            poses: HashMap::new(),
            expressions: HashMap::new(),
        };

        // Should validate successfully but with a warning (not tested here)
        assert!(def.validate().is_ok());
    }
}
