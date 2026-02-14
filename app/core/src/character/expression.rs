use serde::{Deserialize, Serialize};

/// Character expression/emotion
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Expression {
    /// Normal/neutral expression
    #[default]
    Normal,
    /// Happy/smiling
    Happy,
    /// Sad/crying
    Sad,
    /// Angry
    Angry,
    /// Surprised/shocked
    Surprised,
    /// Embarrassed/blushing
    Embarrassed,
    /// Confused/puzzled
    Confused,
    /// Determined/serious
    Determined,
    /// Worried/anxious
    Worried,
    /// Excited
    Excited,
    /// Shy
    Shy,
    /// Thinking/pondering
    Thinking,
    /// Custom expression (user-defined name)
    Custom(String),
}

impl Expression {
    /// Create a custom expression
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(name.into())
    }

    /// Get the expression name as a string
    pub fn name(&self) -> &str {
        match self {
            Self::Normal => "normal",
            Self::Happy => "happy",
            Self::Sad => "sad",
            Self::Angry => "angry",
            Self::Surprised => "surprised",
            Self::Embarrassed => "embarrassed",
            Self::Confused => "confused",
            Self::Determined => "determined",
            Self::Worried => "worried",
            Self::Excited => "excited",
            Self::Shy => "shy",
            Self::Thinking => "thinking",
            Self::Custom(name) => name,
        }
    }
}

impl From<&str> for Expression {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "normal" => Self::Normal,
            "happy" => Self::Happy,
            "sad" => Self::Sad,
            "angry" => Self::Angry,
            "surprised" => Self::Surprised,
            "embarrassed" => Self::Embarrassed,
            "confused" => Self::Confused,
            "determined" => Self::Determined,
            "worried" => Self::Worried,
            "excited" => Self::Excited,
            "shy" => Self::Shy,
            "thinking" => Self::Thinking,
            _ => Self::Custom(s.to_string()),
        }
    }
}

impl From<String> for Expression {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_default() {
        let expr = Expression::default();
        assert_eq!(expr, Expression::Normal);
    }

    #[test]
    fn test_expression_variants() {
        assert_eq!(Expression::Normal, Expression::Normal);
        assert_ne!(Expression::Happy, Expression::Sad);
        assert_ne!(Expression::Angry, Expression::Surprised);
    }

    #[test]
    fn test_expression_custom() {
        let expr = Expression::custom("smirk");
        assert_eq!(expr, Expression::Custom("smirk".to_string()));
    }

    #[test]
    fn test_expression_name_normal() {
        assert_eq!(Expression::Normal.name(), "normal");
        assert_eq!(Expression::Happy.name(), "happy");
        assert_eq!(Expression::Sad.name(), "sad");
    }

    #[test]
    fn test_expression_name_custom() {
        let expr = Expression::custom("playful");
        assert_eq!(expr.name(), "playful");
    }

    #[test]
    fn test_expression_name_all_variants() {
        assert_eq!(Expression::Angry.name(), "angry");
        assert_eq!(Expression::Surprised.name(), "surprised");
        assert_eq!(Expression::Embarrassed.name(), "embarrassed");
        assert_eq!(Expression::Confused.name(), "confused");
        assert_eq!(Expression::Determined.name(), "determined");
        assert_eq!(Expression::Worried.name(), "worried");
        assert_eq!(Expression::Excited.name(), "excited");
        assert_eq!(Expression::Shy.name(), "shy");
        assert_eq!(Expression::Thinking.name(), "thinking");
    }

    #[test]
    fn test_expression_from_str_lowercase() {
        assert_eq!(Expression::from("happy"), Expression::Happy);
        assert_eq!(Expression::from("sad"), Expression::Sad);
        assert_eq!(Expression::from("angry"), Expression::Angry);
    }

    #[test]
    fn test_expression_from_str_uppercase() {
        assert_eq!(Expression::from("HAPPY"), Expression::Happy);
        assert_eq!(Expression::from("SAD"), Expression::Sad);
        assert_eq!(Expression::from("NORMAL"), Expression::Normal);
    }

    #[test]
    fn test_expression_from_str_mixedcase() {
        assert_eq!(Expression::from("HaPpY"), Expression::Happy);
        assert_eq!(Expression::from("SaD"), Expression::Sad);
    }

    #[test]
    fn test_expression_from_str_unknown() {
        let expr = Expression::from("unknown_expression");
        assert_eq!(expr, Expression::Custom("unknown_expression".to_string()));
    }

    #[test]
    fn test_expression_from_string() {
        let expr = Expression::from("excited".to_string());
        assert_eq!(expr, Expression::Excited);
    }

    #[test]
    fn test_expression_from_string_custom() {
        let expr = Expression::from("custom_emotion".to_string());
        assert_eq!(expr, Expression::Custom("custom_emotion".to_string()));
    }

    #[test]
    fn test_expression_serialization() {
        let expr = Expression::Happy;
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn test_expression_custom_serialization() {
        let expr = Expression::custom("gleeful");
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn test_expression_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Expression::Happy);
        set.insert(Expression::Sad);
        set.insert(Expression::Happy); // duplicate
        assert_eq!(set.len(), 2);
    }
}
