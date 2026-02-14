use thiserror::Error;

/// Core engine errors
#[derive(Debug, Error)]
pub enum EngineError {
    /// Scenario-related error
    #[error("Scenario error: {0}")]
    Scenario(#[from] ScenarioError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error (TOML)
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// Deserialization error (TOML)
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// Serialization error (RON)
    #[error("RON serialization error: {0}")]
    RonSer(#[from] ron::Error),

    /// Asset not found
    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    /// Invalid asset reference
    #[error("Invalid asset reference: {0}")]
    InvalidAssetRef(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Configuration-specific errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// I/O error during config file operations
    #[error("Config I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// RON deserialization error
    #[error("Failed to parse RON config: {0}")]
    RonDe(#[from] ron::error::SpannedError),

    /// RON serialization error
    #[error("Failed to serialize config to RON: {0}")]
    RonSer(#[from] ron::Error),

    /// TOML deserialization error
    #[error("Failed to parse TOML config: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("Failed to serialize config to TOML: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// Invalid configuration value
    #[error("Invalid config value for {0}: {1}")]
    InvalidValue(String, String),

    /// Missing required configuration field
    #[error("Missing required config field: {0}")]
    MissingField(String),

    /// Generic configuration error
    #[error("{0}")]
    Other(String),
}

/// Scenario-specific errors
#[derive(Debug, Error)]
pub enum ScenarioError {
    /// Scene not found
    #[error("Scene not found: {0}")]
    SceneNotFound(String),

    /// Character not found
    #[error("Character not found: {0}")]
    CharacterNotFound(String),

    /// Invalid scene ID
    #[error("Invalid scene ID: {0}")]
    InvalidSceneId(String),

    /// Invalid character ID
    #[error("Invalid character ID: {0}")]
    InvalidCharacterId(String),

    /// Invalid command index
    #[error("Invalid command index: {0} (max: {1})")]
    InvalidCommandIndex(usize, usize),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid field value
    #[error("Invalid field value for {0}: {1}")]
    InvalidFieldValue(String, String),

    /// Circular reference detected
    #[error("Circular reference detected: {0}")]
    CircularReference(String),

    /// Invalid condition
    #[error("Invalid condition: {0}")]
    InvalidCondition(String),

    /// Variable not found
    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    /// Flag not found
    #[error("Flag not found: {0}")]
    FlagNotFound(String),

    /// Type mismatch
    #[error("Type mismatch: expected {0}, got {1}")]
    TypeMismatch(String, String),

    /// Generic scenario error
    #[error("{0}")]
    Other(String),
}

/// Result type for engine operations
pub type EngineResult<T> = Result<T, EngineError>;

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Result type for scenario operations
pub type ScenarioResult<T> = Result<T, ScenarioError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_error_scenario() {
        let scenario_err = ScenarioError::SceneNotFound("test_scene".to_string());
        let engine_err: EngineError = scenario_err.into();
        assert!(matches!(engine_err, EngineError::Scenario(_)));
    }

    #[test]
    fn test_engine_error_config() {
        let config_err = ConfigError::Other("Invalid configuration".to_string());
        let err: EngineError = config_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Configuration error"));
        assert!(msg.contains("Invalid configuration"));
    }

    #[test]
    fn test_engine_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let engine_err: EngineError = io_err.into();
        assert!(matches!(engine_err, EngineError::Io(_)));
    }

    #[test]
    fn test_engine_error_asset_not_found() {
        let err = EngineError::AssetNotFound("sprites/missing.png".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Asset not found"));
        assert!(msg.contains("sprites/missing.png"));
    }

    #[test]
    fn test_engine_error_invalid_asset_ref() {
        let err = EngineError::InvalidAssetRef("invalid://path".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid asset reference"));
    }

    #[test]
    fn test_engine_error_other() {
        let err = EngineError::Other("Custom error message".to_string());
        let msg = format!("{}", err);
        assert_eq!(msg, "Custom error message");
    }

    #[test]
    fn test_scenario_error_scene_not_found() {
        let err = ScenarioError::SceneNotFound("intro".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Scene not found"));
        assert!(msg.contains("intro"));
    }

    #[test]
    fn test_scenario_error_character_not_found() {
        let err = ScenarioError::CharacterNotFound("alice".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Character not found"));
        assert!(msg.contains("alice"));
    }

    #[test]
    fn test_scenario_error_invalid_scene_id() {
        let err = ScenarioError::InvalidSceneId("bad id".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid scene ID"));
    }

    #[test]
    fn test_scenario_error_invalid_character_id() {
        let err = ScenarioError::InvalidCharacterId("123".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid character ID"));
    }

    #[test]
    fn test_scenario_error_invalid_command_index() {
        let err = ScenarioError::InvalidCommandIndex(10, 5);
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid command index"));
        assert!(msg.contains("10"));
        assert!(msg.contains("5"));
    }

    #[test]
    fn test_scenario_error_missing_field() {
        let err = ScenarioError::MissingField("title".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Missing required field"));
        assert!(msg.contains("title"));
    }

    #[test]
    fn test_scenario_error_invalid_field_value() {
        let err = ScenarioError::InvalidFieldValue("age".to_string(), "not a number".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid field value"));
        assert!(msg.contains("age"));
        assert!(msg.contains("not a number"));
    }

    #[test]
    fn test_scenario_error_circular_reference() {
        let err = ScenarioError::CircularReference("scene_a -> scene_b -> scene_a".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Circular reference"));
    }

    #[test]
    fn test_scenario_error_invalid_condition() {
        let err = ScenarioError::InvalidCondition("malformed condition".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid condition"));
    }

    #[test]
    fn test_scenario_error_variable_not_found() {
        let err = ScenarioError::VariableNotFound("score".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Variable not found"));
        assert!(msg.contains("score"));
    }

    #[test]
    fn test_scenario_error_flag_not_found() {
        let err = ScenarioError::FlagNotFound("completed_quest".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Flag not found"));
        assert!(msg.contains("completed_quest"));
    }

    #[test]
    fn test_scenario_error_type_mismatch() {
        let err = ScenarioError::TypeMismatch("int".to_string(), "string".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Type mismatch"));
        assert!(msg.contains("int"));
        assert!(msg.contains("string"));
    }

    #[test]
    fn test_scenario_error_other() {
        let err = ScenarioError::Other("Custom scenario error".to_string());
        let msg = format!("{}", err);
        assert_eq!(msg, "Custom scenario error");
    }

    #[test]
    fn test_engine_result_type() {
        let ok_result: EngineResult<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: EngineResult<i32> = Err(EngineError::Other("error".to_string()));
        assert!(err_result.is_err());
    }

    #[test]
    fn test_scenario_result_type() {
        let ok_result: ScenarioResult<String> = Ok("success".to_string());
        assert_eq!(ok_result.unwrap(), "success");

        let err_result: ScenarioResult<String> =
            Err(ScenarioError::Other("scenario error".to_string()));
        assert!(err_result.is_err());
    }
}
