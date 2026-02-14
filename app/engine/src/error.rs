//! Engine-specific errors

use thiserror::Error;

/// Engine-specific errors
#[derive(Debug, Error)]
pub enum EngineError {
    // === Capacity/Configuration Errors ===
    /// Invalid capacity (must be > 0)
    #[error("Invalid capacity: {0} (must be greater than 0)")]
    InvalidCapacity(usize),

    // === App Module Errors ===
    /// Application configuration error
    #[error("App configuration error: {0}")]
    AppConfig(String),

    /// Game loop error
    #[error("Game loop error: {0}")]
    GameLoop(String),

    // === Asset Module Errors ===
    /// Asset loading error
    #[error("Asset loading failed: {0}")]
    AssetLoad(String),

    /// Texture cache error
    #[error("Texture cache error: {0}")]
    TextureCache(String),

    /// Asset not found
    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    // === Audio Module Errors ===
    /// Audio initialization error
    #[error("Audio initialization failed: {0}")]
    AudioInit(String),

    /// BGM playback error
    #[error("BGM playback error: {0}")]
    BgmPlayback(String),

    /// SE playback error
    #[error("SE playback error: {0}")]
    SePlayback(String),

    /// Voice playback error
    #[error("Voice playback error: {0}")]
    VoicePlayback(String),

    // === Render Module Errors ===
    /// Renderer initialization error
    #[error("Renderer initialization failed: {0}")]
    RendererInit(String),

    /// Rendering error
    #[error("Rendering failed: {0}")]
    Rendering(String),

    /// Shader compilation error
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),

    /// Pipeline creation error
    #[error("Pipeline creation failed: {0}")]
    PipelineCreation(String),

    // === Text Module Errors ===
    /// Text layout error
    #[error("Text layout error: {0}")]
    TextLayout(String),

    /// Glyph cache error
    #[error("Glyph cache error: {0}")]
    GlyphCache(String),

    /// Font loading error
    #[error("Font loading failed: {0}")]
    FontLoad(String),

    // === Runtime Module Errors ===
    /// Scenario execution error
    #[error("Scenario execution error: {0}")]
    ScenarioExecution(String),

    /// State machine error
    #[error("State machine error: {0}")]
    StateMachine(String),

    /// Variable/Flag operation error
    #[error("Variable/Flag operation error: {0}")]
    VariableFlag(String),

    // === Save Module Errors ===
    /// Save operation error
    #[error("Save operation failed: {0}")]
    SaveOperation(String),

    /// Load operation error
    #[error("Load operation failed: {0}")]
    LoadOperation(String),

    /// Save data corruption
    #[error("Save data corrupted: {0}")]
    SaveDataCorruption(String),

    // === Input Module Errors ===
    /// Input processing error
    #[error("Input processing error: {0}")]
    InputProcessing(String),

    // === I/O Errors ===
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // === wgpu Errors ===
    /// wgpu surface error
    #[error("wgpu surface error: {0}")]
    WgpuSurface(#[from] wgpu::CreateSurfaceError),

    /// wgpu request device error
    #[error("wgpu request device error: {0}")]
    WgpuRequestDevice(#[from] wgpu::RequestDeviceError),

    /// wgpu surface texture error
    #[error("wgpu surface texture error: {0}")]
    WgpuSurfaceTexture(#[from] wgpu::SurfaceError),

    // === Generic Errors ===
    /// Generic engine error
    #[error("{0}")]
    Other(String),
}

/// Result type for engine operations
pub type EngineResult<T> = Result<T, EngineError>;

// Conversion from narrative_core::EngineError
impl From<narrative_core::EngineError> for EngineError {
    fn from(err: narrative_core::EngineError) -> Self {
        // Convert core error to engine error by wrapping in Other variant
        EngineError::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_capacity() {
        let err = EngineError::InvalidCapacity(0);
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid capacity"));
        assert!(msg.contains("0"));
    }

    #[test]
    fn test_app_errors() {
        let err = EngineError::AppConfig("Invalid config".to_string());
        assert!(format!("{}", err).contains("App configuration error"));

        let err = EngineError::GameLoop("Loop failed".to_string());
        assert!(format!("{}", err).contains("Game loop error"));
    }

    #[test]
    fn test_asset_errors() {
        let err = EngineError::AssetLoad("Failed to load".to_string());
        assert!(format!("{}", err).contains("Asset loading failed"));

        let err = EngineError::TextureCache("Cache full".to_string());
        assert!(format!("{}", err).contains("Texture cache error"));

        let err = EngineError::AssetNotFound("missing.png".to_string());
        assert!(format!("{}", err).contains("Asset not found"));
    }

    #[test]
    fn test_audio_errors() {
        let err = EngineError::AudioInit("Failed to initialize kira".to_string());
        assert!(format!("{}", err).contains("Audio initialization failed"));

        let err = EngineError::BgmPlayback("BGM error".to_string());
        assert!(format!("{}", err).contains("BGM playback error"));

        let err = EngineError::SePlayback("SE error".to_string());
        assert!(format!("{}", err).contains("SE playback error"));

        let err = EngineError::VoicePlayback("Voice error".to_string());
        assert!(format!("{}", err).contains("Voice playback error"));
    }

    #[test]
    fn test_render_errors() {
        let err = EngineError::RendererInit("wgpu not available".to_string());
        assert!(format!("{}", err).contains("Renderer initialization failed"));

        let err = EngineError::Rendering("Draw failed".to_string());
        assert!(format!("{}", err).contains("Rendering failed"));

        let err = EngineError::ShaderCompilation("Syntax error".to_string());
        assert!(format!("{}", err).contains("Shader compilation failed"));

        let err = EngineError::PipelineCreation("Invalid pipeline".to_string());
        assert!(format!("{}", err).contains("Pipeline creation failed"));
    }

    #[test]
    fn test_text_errors() {
        let err = EngineError::TextLayout("Layout failed".to_string());
        assert!(format!("{}", err).contains("Text layout error"));

        let err = EngineError::GlyphCache("Cache error".to_string());
        assert!(format!("{}", err).contains("Glyph cache error"));

        let err = EngineError::FontLoad("Font not found".to_string());
        assert!(format!("{}", err).contains("Font loading failed"));
    }

    #[test]
    fn test_runtime_errors() {
        let err = EngineError::ScenarioExecution("Invalid command".to_string());
        assert!(format!("{}", err).contains("Scenario execution error"));

        let err = EngineError::StateMachine("Invalid transition".to_string());
        assert!(format!("{}", err).contains("State machine error"));

        let err = EngineError::VariableFlag("Invalid variable".to_string());
        assert!(format!("{}", err).contains("Variable/Flag operation error"));
    }

    #[test]
    fn test_save_errors() {
        let err = EngineError::SaveOperation("Save failed".to_string());
        assert!(format!("{}", err).contains("Save operation failed"));

        let err = EngineError::LoadOperation("Load failed".to_string());
        assert!(format!("{}", err).contains("Load operation failed"));

        let err = EngineError::SaveDataCorruption("Checksum mismatch".to_string());
        assert!(format!("{}", err).contains("Save data corrupted"));
    }

    #[test]
    fn test_input_errors() {
        let err = EngineError::InputProcessing("Invalid input".to_string());
        assert!(format!("{}", err).contains("Input processing error"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let engine_err: EngineError = io_err.into();
        assert!(matches!(engine_err, EngineError::Io(_)));
    }

    #[test]
    fn test_other() {
        let err = EngineError::Other("Custom error".to_string());
        assert_eq!(format!("{}", err), "Custom error");
    }

    #[test]
    fn test_error_result() {
        let ok_result: EngineResult<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: EngineResult<i32> = Err(EngineError::Other("error".to_string()));
        assert!(err_result.is_err());
    }
}
