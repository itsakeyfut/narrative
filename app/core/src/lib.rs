//! # Narrative Core
//!
//! Core type definitions for the Narrative Novel engine.
//!
//! This crate provides the fundamental types and data structures used throughout
//! the Narrative Novel engine, including:
//!
//! - **Scenario types**: Definitions for scenarios, scenes, and commands
//! - **Character types**: Character definitions, expressions, and positions
//! - **Configuration**: Game, graphics, audio, and text configuration
//! - **Type system**: ID types, colors, rectangles, transitions
//! - **Error handling**: Engine and scenario error types
//! - **Conditions**: Branching logic and variable operations
//!
//! ## File Format Support
//!
//! - **Scenarios**: TOML format (designed for non-programmers)
//! - **Configuration**: RON format (Rust-friendly configuration)
//! - **Save data**: RON format (type-safe serialization)
//! - **Assets**: RON manifest system (Veloren-inspired organization)
//!
//! ## Example
//!
//! ```rust
//! use narrative_core::{Scenario, ScenarioMetadata, Scene, ScenarioCommand, Dialogue, Speaker};
//!
//! let metadata = ScenarioMetadata::new("chapter_01", "Chapter 1");
//! let mut scenario = Scenario::new(metadata, "scene_01");
//!
//! let mut scene = Scene::new("scene_01", "Opening Scene");
//! scene.add_command(ScenarioCommand::Dialogue {
//!     dialogue: Dialogue::narrator("It was a dark and stormy night..."),
//! });
//!
//! scenario.add_scene("scene_01", scene);
//! ```

pub mod asset;
pub mod backlog;
pub mod cg_metadata;
pub mod character;
pub mod condition;
pub mod config;
pub mod error;
pub mod read_history;
pub mod scenario;
pub mod types;
pub mod unlocks;
pub mod variable;

// Re-export commonly used types
pub use asset::{
    AudioMeta, BackgroundDef, BackgroundManifest, BackgroundMeta, BgmDef, BgmManifest, SeDef,
    SeManifest, UiThemeDef, UiThemeManifest,
};
pub use backlog::{Backlog, BacklogEntry};
pub use cg_metadata::{CgId, CgMetadata, CgRegistry, CgVariation};
pub use character::{
    CharacterDef, CharacterManifest, CharacterPosition, CharacterRegistry, CharacterState,
    Expression,
};
pub use condition::{CompareOp, Condition};
pub use config::{
    AnimationSettings, AudioConfig, DialogueBoxConfig, GameConfig, GameMetadata, GraphicsConfig,
    PathConfig, SkipMode, TextConfig, TextSpeed, UiConfig, UserSettings,
};
pub use error::{
    ConfigError, ConfigResult, EngineError, EngineResult, ScenarioError, ScenarioResult,
};
pub use read_history::{DialogueId, ReadHistory};
pub use scenario::{
    Choice, ChoiceOption, Dialogue, Scenario, ScenarioCommand, ScenarioMetadata, Scene, Speaker,
    VariableValue,
};
pub use types::{
    AssetRef, AudioId, CharacterId, Color, FlagId, Point, Rect, SceneId, Size, SlideDirection,
    Transition, TransitionKind, VariableId, WipeDirection,
};
pub use unlocks::{UnlockData, UnlockError, UnlockResult, UnlockStatistics};
pub use variable::{Variable, VariableError, VariableOperation};
