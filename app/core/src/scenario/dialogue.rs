use crate::character::{CharacterAnimation, Expression};
use serde::{Deserialize, Serialize};

/// Speaker in a dialogue
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Speaker {
    /// A named character (character ID)
    Character(String),
    /// Narrator (no character)
    #[default]
    Narrator,
    /// System message
    System,
}

impl Speaker {
    /// Create a character speaker
    pub fn character(id: impl Into<String>) -> Self {
        Self::Character(id.into())
    }

    /// Check if this is a character speaker
    pub fn is_character(&self) -> bool {
        matches!(self, Self::Character(_))
    }

    /// Check if this is the narrator
    pub fn is_narrator(&self) -> bool {
        matches!(self, Self::Narrator)
    }

    /// Check if this is a system message
    pub fn is_system(&self) -> bool {
        matches!(self, Self::System)
    }

    /// Get the character ID if this is a character speaker
    pub fn character_id(&self) -> Option<&str> {
        match self {
            Self::Character(id) => Some(id),
            _ => None,
        }
    }
}

impl From<&str> for Speaker {
    fn from(s: &str) -> Self {
        match s {
            "narrator" | "Narrator" | "" => Self::Narrator,
            "system" | "System" => Self::System,
            id => Self::Character(id.to_string()),
        }
    }
}

impl From<String> for Speaker {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

/// Dialogue line
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dialogue {
    /// Speaker
    pub speaker: Speaker,
    /// Dialogue text
    pub text: String,
    /// Optional expression for the speaker
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<Expression>,
    /// Optional character animation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<CharacterAnimation>,
}

impl Dialogue {
    /// Create a new dialogue line
    pub fn new(speaker: impl Into<Speaker>, text: impl Into<String>) -> Self {
        Self {
            speaker: speaker.into(),
            text: text.into(),
            expression: None,
            animation: None,
        }
    }

    /// Create a narrator line
    pub fn narrator(text: impl Into<String>) -> Self {
        Self::new(Speaker::Narrator, text)
    }

    /// Create a character dialogue
    pub fn character(character_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(Speaker::character(character_id), text)
    }

    /// Set the expression
    pub fn with_expression(mut self, expression: Expression) -> Self {
        self.expression = Some(expression);
        self
    }

    /// Set the character animation
    pub fn with_animation(mut self, animation: CharacterAnimation) -> Self {
        self.animation = Some(animation);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speaker_default() {
        let speaker = Speaker::default();
        assert_eq!(speaker, Speaker::Narrator);
    }

    #[test]
    fn test_speaker_character() {
        let speaker = Speaker::character("alice");
        assert_eq!(speaker, Speaker::Character("alice".to_string()));
    }

    #[test]
    fn test_speaker_is_character() {
        assert!(Speaker::Character("bob".to_string()).is_character());
        assert!(!Speaker::Narrator.is_character());
        assert!(!Speaker::System.is_character());
    }

    #[test]
    fn test_speaker_is_narrator() {
        assert!(Speaker::Narrator.is_narrator());
        assert!(!Speaker::Character("alice".to_string()).is_narrator());
        assert!(!Speaker::System.is_narrator());
    }

    #[test]
    fn test_speaker_is_system() {
        assert!(Speaker::System.is_system());
        assert!(!Speaker::Narrator.is_system());
        assert!(!Speaker::Character("bob".to_string()).is_system());
    }

    #[test]
    fn test_speaker_character_id() {
        let speaker = Speaker::Character("charlie".to_string());
        assert_eq!(speaker.character_id(), Some("charlie"));

        assert_eq!(Speaker::Narrator.character_id(), None);
        assert_eq!(Speaker::System.character_id(), None);
    }

    #[test]
    fn test_speaker_from_str_narrator() {
        assert_eq!(Speaker::from("narrator"), Speaker::Narrator);
        assert_eq!(Speaker::from("Narrator"), Speaker::Narrator);
        assert_eq!(Speaker::from(""), Speaker::Narrator);
    }

    #[test]
    fn test_speaker_from_str_system() {
        assert_eq!(Speaker::from("system"), Speaker::System);
        assert_eq!(Speaker::from("System"), Speaker::System);
    }

    #[test]
    fn test_speaker_from_str_character() {
        assert_eq!(
            Speaker::from("alice"),
            Speaker::Character("alice".to_string())
        );
        assert_eq!(Speaker::from("Bob"), Speaker::Character("Bob".to_string()));
    }

    #[test]
    fn test_speaker_from_string() {
        let speaker = Speaker::from("dave".to_string());
        assert_eq!(speaker, Speaker::Character("dave".to_string()));
    }

    #[test]
    fn test_speaker_serialization() {
        let speaker = Speaker::Character("eve".to_string());
        let serialized = serde_json::to_string(&speaker).unwrap();
        let deserialized: Speaker = serde_json::from_str(&serialized).unwrap();
        assert_eq!(speaker, deserialized);
    }

    #[test]
    fn test_dialogue_new() {
        let dialogue = Dialogue::new("alice", "Hello, world!");
        assert_eq!(dialogue.speaker, Speaker::Character("alice".to_string()));
        assert_eq!(dialogue.text, "Hello, world!");
        assert_eq!(dialogue.expression, None);
    }

    #[test]
    fn test_dialogue_narrator() {
        let dialogue = Dialogue::narrator("The story begins...");
        assert_eq!(dialogue.speaker, Speaker::Narrator);
        assert_eq!(dialogue.text, "The story begins...");
        assert_eq!(dialogue.expression, None);
    }

    #[test]
    fn test_dialogue_character() {
        let dialogue = Dialogue::character("bob", "I'm Bob!");
        assert_eq!(dialogue.speaker, Speaker::Character("bob".to_string()));
        assert_eq!(dialogue.text, "I'm Bob!");
        assert_eq!(dialogue.expression, None);
    }

    #[test]
    fn test_dialogue_with_expression() {
        let dialogue =
            Dialogue::character("charlie", "I'm happy!").with_expression(Expression::Happy);
        assert_eq!(dialogue.expression, Some(Expression::Happy));
    }

    #[test]
    fn test_dialogue_serialization() {
        let dialogue = Dialogue::character("dave", "Test message");
        let serialized = serde_json::to_string(&dialogue).unwrap();
        let deserialized: Dialogue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(dialogue, deserialized);
    }

    #[test]
    fn test_dialogue_with_expression_serialization() {
        let dialogue = Dialogue::character("eve", "Excited!").with_expression(Expression::Excited);
        let serialized = serde_json::to_string(&dialogue).unwrap();
        let deserialized: Dialogue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(dialogue, deserialized);
    }

    #[test]
    fn test_dialogue_builder_chain() {
        let dialogue = Dialogue::new("frank", "Greetings").with_expression(Expression::Confused);

        assert_eq!(dialogue.speaker, Speaker::Character("frank".to_string()));
        assert_eq!(dialogue.text, "Greetings");
        assert_eq!(dialogue.expression, Some(Expression::Confused));
    }

    #[test]
    fn test_speaker_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Speaker::Narrator);
        set.insert(Speaker::System);
        set.insert(Speaker::Narrator); // duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_dialogue_with_animation() {
        let dialogue = Dialogue::character("sakura", "Wow!")
            .with_expression(Expression::Surprised)
            .with_animation(CharacterAnimation::shake());
        assert_eq!(dialogue.expression, Some(Expression::Surprised));
        assert!(dialogue.animation.is_some());
        assert!(dialogue.animation.as_ref().unwrap().is_active());
    }

    #[test]
    fn test_dialogue_animation_serialization() {
        let dialogue = Dialogue::character("alice", "I'm so happy!")
            .with_animation(CharacterAnimation::jump());
        let serialized = serde_json::to_string(&dialogue).unwrap();
        let deserialized: Dialogue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(dialogue, deserialized);
    }

    #[test]
    fn test_dialogue_with_escape_animation_toml() {
        let toml_str = r#"
speaker = { Character = "bob" }
text = "Run away!"
animation = { type = "escape", direction = "right", preset = "large" }
"#;
        let dialogue: Dialogue = toml::from_str(toml_str).unwrap();
        assert_eq!(dialogue.speaker, Speaker::Character("bob".to_string()));
        assert_eq!(dialogue.text, "Run away!");
        assert!(dialogue.animation.is_some());
        assert!(dialogue.animation.as_ref().unwrap().is_keyframe_based());
    }

    #[test]
    fn test_dialogue_with_faint_animation_toml() {
        let toml_str = r#"
speaker = { Character = "alice" }
text = "I feel dizzy..."
animation = { type = "faint", preset = "medium" }
"#;
        let dialogue: Dialogue = toml::from_str(toml_str).unwrap();
        assert_eq!(dialogue.speaker, Speaker::Character("alice".to_string()));
        assert_eq!(dialogue.text, "I feel dizzy...");
        assert!(dialogue.animation.is_some());
        assert!(dialogue.animation.as_ref().unwrap().is_keyframe_based());
    }
}
