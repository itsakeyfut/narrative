//! Runtime module for scenario execution
//!
//! This module handles the execution of scenarios, including state management,
//! flag and variable storage, and scenario command execution.

mod executor;
mod flag_store;
mod state_machine;
mod variable_store;

pub use executor::{CommandExecutionResult, DisplayedCharacter, ScenarioRuntime};
pub use flag_store::FlagStore;
pub use narrative_core::{ReadHistory, TransitionKind};
pub use state_machine::{
    AppState, BacklogState, CgGalleryState, CgViewerState, ChoiceState, EffectKind, EffectState,
    InGameState, LayoutMode, LoadingState, MainMenuState, PauseMenuState, SaveLoadState,
    SettingsState, TransitionState, TypingState, WaitState, WaitingInputState,
};
pub use variable_store::VariableStore;
