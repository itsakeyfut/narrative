//! # Narrative Engine
//!
//! Core engine for the Narrative Novel visual novel engine.
//!
//! This crate provides the core functionality for running visual novel games,
//! including:
//!
//! - **Runtime**: Scenario execution, state management, flags, and variables
//! - **Rendering**: 2D GPU-accelerated rendering with wgpu
//! - **Text**: Text layout and rendering with cosmic-text
//! - **Audio**: Audio playback with kira (BGM, SE, voice)
//! - **Input**: Keyboard, mouse, and gamepad input handling
//! - **Save/Load**: Game state persistence
//! - **Assets**: Asset loading and caching
//! - **App**: Game loop and configuration
//!
//! ## Architecture
//!
//! The engine is built with a modular architecture:
//!
//! ```text
//! narrative-core (types)
//!        ↓
//! narrative-engine (this crate)
//!   ├── runtime    - Scenario execution
//!   ├── render     - 2D rendering (wgpu)
//!   ├── text       - Text rendering (cosmic-text)
//!   ├── audio      - Audio (kira)
//!   ├── input      - Input handling
//!   ├── save       - Save/load
//!   ├── asset      - Asset management
//!   └── app        - Application loop
//! ```
//!
//! ## Example
//!
//! ```rust,no_run
//! use narrative_engine::{EngineConfig, GameLoop};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = EngineConfig::default();
//!     let mut game_loop = GameLoop::new();
//!     game_loop.run()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Phase 0.1 Status
//!
//! This is the initial crate structure. Core functionality will be implemented
//! in subsequent phases:
//!
//! - **Phase 0.2**: wgpu renderer setup, sprite pipeline
//! - **Phase 0.3**: cosmic-text integration, glyph cache
//! - **Phase 0.4**: Runtime state machine, scenario executor
//! - **Phase 0.5**: Game loop, asset loading

pub mod app;
pub mod asset;
pub mod audio;
pub mod error;
pub mod input;
pub mod render;
pub mod runtime;
pub mod save;
pub mod text;
pub mod ui;

// Re-export commonly used types
pub use app::{EngineConfig, GameLoop};
pub use asset::{AssetLoader, TextureCache, TextureHandle};
pub use audio::{AudioManager, BgmPlayer, SePlayer, VoicePlayer};
pub use error::{EngineError, EngineResult};
pub use input::{InputHandler, InputState, KeyCode, Modifiers, MouseButton};
pub use render::{RenderBatch, RenderCommand, Renderer, SpritePipeline, SpriteVertex};
pub use runtime::{
    AppState, ChoiceState, EffectKind, EffectState, FlagStore, InGameState, LoadingState,
    MainMenuState, PauseMenuState, ReadHistory, SaveLoadState, ScenarioRuntime, SettingsState,
    TransitionKind, TransitionState, TypingState, VariableStore, WaitState, WaitingInputState,
};
pub use save::{SAVE_VERSION, SaveData, SaveManager, SavedCharacterDisplay, generate_thumbnail};
pub use text::{GlyphCache, TextLayout, TextureAtlas, TypewriterEffect};
pub use ui::UiComponent;

// Re-export narrative-core for convenience
pub use narrative_core;

/// Engine initialization helper
///
/// This function will be implemented in Phase 0.5 to initialize all engine
/// subsystems in the correct order.
pub fn init(_config: EngineConfig) -> narrative_core::EngineResult<Engine> {
    // TODO: Phase 0.5 - engine initialization
    Ok(Engine {
        audio: AudioManager::new()
            .map_err(|e| narrative_core::EngineError::Other(e.to_string()))?,
        input: InputHandler::default(),
        assets: AssetLoader::default(),
        save: SaveManager::default(),
    })
}

/// Main engine struct combining all subsystems
///
/// This will be properly initialized in Phase 0.5.
/// Note: Renderer is not included here as it requires window context
pub struct Engine {
    /// Audio subsystem
    pub audio: AudioManager,
    /// Input subsystem
    pub input: InputHandler,
    /// Asset subsystem
    pub assets: AssetLoader,
    /// Save subsystem
    pub save: SaveManager,
}

impl Engine {
    /// Create a new engine instance (stub implementation)
    pub fn new(_config: EngineConfig) -> narrative_core::EngineResult<Self> {
        Ok(Self {
            audio: AudioManager::new()
                .map_err(|e| narrative_core::EngineError::Other(e.to_string()))?,
            input: InputHandler::default(),
            assets: AssetLoader::default(),
            save: SaveManager::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let config = EngineConfig::default();
        let engine = Engine::new(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_init() {
        let config = EngineConfig::default();
        let engine = init(config);
        assert!(engine.is_ok());
    }
}
