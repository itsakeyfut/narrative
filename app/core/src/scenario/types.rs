use super::{Choice, Dialogue};
use crate::character::{CharacterDef, CharacterPosition, Expression};
use crate::types::{AssetRef, Transition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete scenario containing all scenes and metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario metadata
    pub metadata: ScenarioMetadata,
    /// Character definitions
    pub characters: Vec<CharacterDef>,
    /// All scenes in the scenario (scene_id -> Scene)
    pub scenes: HashMap<String, Scene>,
    /// Starting scene ID
    pub start_scene: String,
}

impl Scenario {
    /// Create a new scenario
    pub fn new(metadata: ScenarioMetadata, start_scene: impl Into<String>) -> Self {
        Self {
            metadata,
            characters: Vec::new(),
            scenes: HashMap::new(),
            start_scene: start_scene.into(),
        }
    }

    /// Add a character definition
    pub fn add_character(&mut self, character: CharacterDef) {
        self.characters.push(character);
    }

    /// Add a scene
    pub fn add_scene(&mut self, scene_id: impl Into<String>, scene: Scene) {
        self.scenes.insert(scene_id.into(), scene);
    }

    /// Get a scene by ID
    pub fn get_scene(&self, scene_id: &str) -> Option<&Scene> {
        self.scenes.get(scene_id)
    }

    /// Get the starting scene
    pub fn get_start_scene(&self) -> Option<&Scene> {
        self.get_scene(&self.start_scene)
    }
}

/// Scenario metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioMetadata {
    /// Scenario ID
    pub id: String,
    /// Display title
    pub title: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Author name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Version string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl ScenarioMetadata {
    /// Create new scenario metadata
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            author: None,
            version: None,
        }
    }
}

/// A scene contains a sequence of commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    /// Scene ID
    pub id: String,
    /// Display title (for save/load UI)
    pub title: String,
    /// Commands to execute in this scene
    pub commands: Vec<ScenarioCommand>,
    /// Optional entry transition when entering this scene
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_transition: Option<Transition>,
    /// Optional exit transition when leaving this scene
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_transition: Option<Transition>,
}

impl Scene {
    /// Create a new scene
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            commands: Vec::new(),
            entry_transition: None,
            exit_transition: None,
        }
    }

    /// Set the entry transition for this scene
    pub fn with_entry_transition(mut self, transition: Transition) -> Self {
        self.entry_transition = Some(transition);
        self
    }

    /// Set the exit transition for this scene
    pub fn with_exit_transition(mut self, transition: Transition) -> Self {
        self.exit_transition = Some(transition);
        self
    }

    /// Add a command to this scene
    pub fn add_command(&mut self, command: ScenarioCommand) {
        self.commands.push(command);
    }

    /// Get the number of commands
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
}

/// Commands that can be executed in a scenario
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScenarioCommand {
    /// Show dialogue text
    Dialogue { dialogue: Dialogue },

    /// Show background image
    ShowBackground {
        asset: AssetRef,
        #[serde(default)]
        transition: Transition,
    },

    /// Hide background
    HideBackground {
        #[serde(default)]
        transition: Transition,
    },

    /// Show a CG (event graphics)
    ShowCG {
        asset: AssetRef,
        #[serde(default)]
        transition: Transition,
    },

    /// Hide CG
    HideCG {
        #[serde(default)]
        transition: Transition,
    },

    /// Show a character
    ShowCharacter {
        character_id: String,
        sprite: AssetRef,
        position: CharacterPosition,
        #[serde(skip_serializing_if = "Option::is_none")]
        expression: Option<Expression>,
        #[serde(default)]
        transition: Transition,
    },

    /// Hide a character
    HideCharacter {
        character_id: String,
        #[serde(default)]
        transition: Transition,
    },

    /// Move a character to a different position
    MoveCharacter {
        character_id: String,
        position: CharacterPosition,
        duration: f32,
    },

    /// Change character expression
    ChangeExpression {
        character_id: String,
        expression: Expression,
    },

    /// Change character sprite
    ChangeSprite {
        character_id: String,
        sprite: AssetRef,
    },

    /// Play background music
    PlayBgm {
        asset: AssetRef,
        #[serde(default = "default_volume")]
        volume: f32,
        #[serde(default)]
        fade_in: f32,
    },

    /// Stop background music
    StopBgm {
        #[serde(default)]
        fade_out: f32,
    },

    /// Play sound effect
    PlaySe {
        asset: AssetRef,
        #[serde(default = "default_volume")]
        volume: f32,
    },

    /// Play voice
    PlayVoice {
        asset: AssetRef,
        #[serde(default = "default_volume")]
        volume: f32,
    },

    /// Present choices to the player
    ShowChoice { choice: Choice },

    /// Jump to another scene
    JumpToScene { scene_id: String },

    /// Set a flag
    SetFlag { flag_name: String, value: bool },

    /// Set a variable
    SetVariable {
        variable_name: String,
        value: VariableValue,
    },

    /// Modify a variable using an operation
    ModifyVariable {
        variable_name: String,
        operation: crate::variable::VariableOperation,
    },

    /// Wait for a duration (in seconds)
    Wait { duration: f32 },

    /// Call a scene as a subroutine
    ///
    /// Pushes current scene and next command index to scene_stack,
    /// then jumps to the target scene. When Return is encountered,
    /// execution resumes from return_scene at the next command.
    Call {
        scene_id: String,
        return_scene: String,
    },

    /// Return from a subroutine call
    ///
    /// Pops the previous scene and command index from scene_stack
    /// and returns to the saved return_scene. Returns error if stack is empty.
    Return,

    /// Conditional branching
    ///
    /// Executes then_commands if condition is true, else_commands otherwise.
    /// The commands within then/else blocks are executed inline (not as separate scenes).
    If {
        condition: crate::condition::Condition,
        then_commands: Vec<ScenarioCommand>,
        #[serde(default)]
        else_commands: Vec<ScenarioCommand>,
    },

    /// End the scenario
    End,
}

/// Variable value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariableValue {
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
}

// Helper function for default volume
fn default_volume() -> f32 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::ChoiceOption;

    #[test]
    fn test_scenario_metadata_new() {
        let metadata = ScenarioMetadata::new("test_scenario", "Test Scenario");
        assert_eq!(metadata.id, "test_scenario");
        assert_eq!(metadata.title, "Test Scenario");
        assert_eq!(metadata.description, None);
        assert_eq!(metadata.author, None);
        assert_eq!(metadata.version, None);
    }

    #[test]
    fn test_scenario_new() {
        let metadata = ScenarioMetadata::new("game1", "My Game");
        let scenario = Scenario::new(metadata.clone(), "intro");
        assert_eq!(scenario.metadata.id, "game1");
        assert_eq!(scenario.start_scene, "intro");
        assert!(scenario.characters.is_empty());
        assert!(scenario.scenes.is_empty());
    }

    #[test]
    fn test_scenario_add_character() {
        let metadata = ScenarioMetadata::new("test", "Test");
        let mut scenario = Scenario::new(metadata, "start");

        let character = CharacterDef::new("alice", "Alice", "sprites/alice.png");
        scenario.add_character(character.clone());

        assert_eq!(scenario.characters.len(), 1);
        assert_eq!(scenario.characters[0].id, "alice");
    }

    #[test]
    fn test_scenario_add_scene() {
        let metadata = ScenarioMetadata::new("test", "Test");
        let mut scenario = Scenario::new(metadata, "start");

        let scene = Scene::new("scene1", "First Scene");
        scenario.add_scene("scene1", scene);

        assert_eq!(scenario.scenes.len(), 1);
        assert!(scenario.scenes.contains_key("scene1"));
    }

    #[test]
    fn test_scenario_get_scene() {
        let metadata = ScenarioMetadata::new("test", "Test");
        let mut scenario = Scenario::new(metadata, "start");

        let scene = Scene::new("scene1", "First Scene");
        scenario.add_scene("scene1", scene);

        let retrieved = scenario.get_scene("scene1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "scene1");
    }

    #[test]
    fn test_scenario_get_scene_not_found() {
        let metadata = ScenarioMetadata::new("test", "Test");
        let scenario = Scenario::new(metadata, "start");

        let retrieved = scenario.get_scene("nonexistent");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_scenario_get_start_scene() {
        let metadata = ScenarioMetadata::new("test", "Test");
        let mut scenario = Scenario::new(metadata, "intro");

        let scene = Scene::new("intro", "Introduction");
        scenario.add_scene("intro", scene);

        let start = scenario.get_start_scene();
        assert!(start.is_some());
        assert_eq!(start.unwrap().id, "intro");
    }

    #[test]
    fn test_scene_new() {
        let scene = Scene::new("test_scene", "Test Scene");
        assert_eq!(scene.id, "test_scene");
        assert_eq!(scene.title, "Test Scene");
        assert!(scene.commands.is_empty());
    }

    #[test]
    fn test_scene_add_command() {
        let mut scene = Scene::new("test", "Test");
        let dialogue = Dialogue::narrator("Hello");
        scene.add_command(ScenarioCommand::Dialogue { dialogue });

        assert_eq!(scene.command_count(), 1);
    }

    #[test]
    fn test_scene_command_count() {
        let mut scene = Scene::new("test", "Test");
        assert_eq!(scene.command_count(), 0);

        scene.add_command(ScenarioCommand::Dialogue {
            dialogue: Dialogue::narrator("Test"),
        });
        scene.add_command(ScenarioCommand::Wait { duration: 1.0 });

        assert_eq!(scene.command_count(), 2);
    }

    #[test]
    fn test_scenario_command_dialogue() {
        let dialogue = Dialogue::character("alice", "Hello!");
        let cmd = ScenarioCommand::Dialogue {
            dialogue: dialogue.clone(),
        };

        if let ScenarioCommand::Dialogue { dialogue: d } = cmd {
            assert_eq!(d.text, "Hello!");
        } else {
            panic!("Expected Dialogue command");
        }
    }

    #[test]
    fn test_scenario_command_show_background() {
        let cmd = ScenarioCommand::ShowBackground {
            asset: "bg/room.png".into(),
            transition: Transition::fade(),
        };

        if let ScenarioCommand::ShowBackground { asset, transition } = cmd {
            assert_eq!(asset, AssetRef::from("bg/room.png"));
            assert_eq!(transition.duration, 0.5);
        } else {
            panic!("Expected ShowBackground command");
        }
    }

    #[test]
    fn test_scenario_command_show_character() {
        let cmd = ScenarioCommand::ShowCharacter {
            character_id: "alice".to_string(),
            sprite: "sprites/alice.png".into(),
            position: CharacterPosition::Center,
            expression: Some(Expression::Happy),
            transition: Transition::instant(),
        };

        if let ScenarioCommand::ShowCharacter {
            character_id,
            expression,
            ..
        } = cmd
        {
            assert_eq!(character_id, "alice");
            assert_eq!(expression, Some(Expression::Happy));
        } else {
            panic!("Expected ShowCharacter command");
        }
    }

    #[test]
    fn test_scenario_command_play_bgm() {
        let cmd = ScenarioCommand::PlayBgm {
            asset: "music/theme.ogg".into(),
            volume: 0.8,
            fade_in: 1.0,
        };

        if let ScenarioCommand::PlayBgm {
            volume, fade_in, ..
        } = cmd
        {
            assert_eq!(volume, 0.8);
            assert_eq!(fade_in, 1.0);
        } else {
            panic!("Expected PlayBgm command");
        }
    }

    #[test]
    fn test_scenario_command_show_choice() {
        let option = ChoiceOption::new("Option 1", "scene_1");
        let choice = Choice::new(vec![option]);
        let cmd = ScenarioCommand::ShowChoice { choice };

        if let ScenarioCommand::ShowChoice { choice: c } = cmd {
            assert_eq!(c.options.len(), 1);
        } else {
            panic!("Expected ShowChoice command");
        }
    }

    #[test]
    fn test_scenario_command_set_flag() {
        let cmd = ScenarioCommand::SetFlag {
            flag_name: "completed_intro".to_string(),
            value: true,
        };

        if let ScenarioCommand::SetFlag { flag_name, value } = cmd {
            assert_eq!(flag_name, "completed_intro");
            assert!(value);
        } else {
            panic!("Expected SetFlag command");
        }
    }

    #[test]
    fn test_scenario_command_set_variable() {
        let cmd = ScenarioCommand::SetVariable {
            variable_name: "score".to_string(),
            value: VariableValue::Int(100),
        };

        if let ScenarioCommand::SetVariable {
            variable_name,
            value,
        } = cmd
        {
            assert_eq!(variable_name, "score");
            assert_eq!(value, VariableValue::Int(100));
        } else {
            panic!("Expected SetVariable command");
        }
    }

    #[test]
    fn test_scenario_command_jump_to_scene() {
        let cmd = ScenarioCommand::JumpToScene {
            scene_id: "next_scene".to_string(),
        };

        if let ScenarioCommand::JumpToScene { scene_id } = cmd {
            assert_eq!(scene_id, "next_scene");
        } else {
            panic!("Expected JumpToScene command");
        }
    }

    #[test]
    fn test_scenario_command_end() {
        let cmd = ScenarioCommand::End;
        assert!(matches!(cmd, ScenarioCommand::End));
    }

    #[test]
    fn test_variable_value_variants() {
        let bool_val = VariableValue::Bool(true);
        let int_val = VariableValue::Int(42);
        let float_val = VariableValue::Float(3.14);
        let string_val = VariableValue::String("hello".to_string());

        assert_eq!(bool_val, VariableValue::Bool(true));
        assert_eq!(int_val, VariableValue::Int(42));
        assert_eq!(float_val, VariableValue::Float(3.14));
        assert_eq!(string_val, VariableValue::String("hello".to_string()));
    }

    #[test]
    fn test_scenario_serialization() {
        let metadata = ScenarioMetadata::new("test", "Test");
        let scenario = Scenario::new(metadata, "start");

        let serialized = serde_json::to_string(&scenario).unwrap();
        let deserialized: Scenario = serde_json::from_str(&serialized).unwrap();
        assert_eq!(scenario, deserialized);
    }

    #[test]
    fn test_scene_serialization() {
        let scene = Scene::new("test", "Test");
        let serialized = serde_json::to_string(&scene).unwrap();
        let deserialized: Scene = serde_json::from_str(&serialized).unwrap();
        assert_eq!(scene, deserialized);
    }

    #[test]
    fn test_dialogue_command_with_escape_animation_toml() {
        use crate::character::CharacterAnimation;

        let toml_str = r#"
type = "Dialogue"
dialogue = {
    speaker = { Character = "bob" },
    text = "Run!",
    animation = { type = "escape", direction = "right", preset = "medium" }
}
"#;
        let cmd: ScenarioCommand = toml::from_str(toml_str).unwrap();

        match cmd {
            ScenarioCommand::Dialogue { dialogue } => {
                assert_eq!(dialogue.text, "Run!");
                assert!(dialogue.animation.is_some());
                let animation = dialogue.animation.unwrap();
                assert!(animation.is_keyframe_based());
                assert!(animation.is_active());
            }
            _ => panic!("Expected Dialogue command"),
        }
    }

    #[test]
    fn test_dialogue_command_with_faint_animation_toml() {
        let toml_str = r#"
type = "Dialogue"
dialogue = {
    speaker = { Character = "alice" },
    text = "I feel weak...",
    animation = { type = "faint", preset = "small" }
}
"#;
        let cmd: ScenarioCommand = toml::from_str(toml_str).unwrap();

        match cmd {
            ScenarioCommand::Dialogue { dialogue } => {
                assert_eq!(dialogue.text, "I feel weak...");
                assert!(dialogue.animation.is_some());
                assert!(dialogue.animation.unwrap().is_keyframe_based());
            }
            _ => panic!("Expected Dialogue command"),
        }
    }
}
