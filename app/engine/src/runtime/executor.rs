//! Scenario runtime executor
//!
//! This module provides the main execution engine for scenarios, handling
//! command execution, state management, and scene transitions.

use super::{FlagStore, ReadHistory, VariableStore};
use crate::asset::AssetLoader;
use crate::error::{EngineError, EngineResult};
use narrative_core::{
    AssetRef, Backlog, BacklogEntry, CharacterPosition, ChoiceOption, FlagId, Scenario,
    ScenarioCommand, Scene, SceneId, Transition, UnlockData, VariableId,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Maximum call stack depth to prevent infinite recursion
const MAX_CALL_STACK_DEPTH: usize = 100;

/// Scenario runtime for executing scenarios
///
/// This is the main execution engine that manages scenario state,
/// executes commands, and handles transitions between scenes.
pub struct ScenarioRuntime {
    /// The loaded scenario data
    scenario: Scenario,
    /// Current scene being executed
    current_scene: Option<SceneId>,
    /// Current command index within the scene
    command_index: usize,
    /// Flag storage for boolean flags
    flag_store: FlagStore,
    /// Variable storage for typed variables
    variable_store: VariableStore,
    /// Read history tracking
    read_history: ReadHistory,
    /// Backlog of displayed dialogues
    backlog: Backlog,
    /// Scene navigation stack for Call/Return commands
    ///
    /// - Call: Push current scene and position to stack, jump to target scene
    /// - Return: Pop from stack and return to previous scene position
    ///
    /// This enables subroutine-like scene navigation patterns.
    scene_stack: Vec<(SceneId, usize)>,
    /// Currently displayed characters (for UI rendering)
    displayed_characters: HashMap<String, DisplayedCharacter>,
    /// Dirty flag for displayed characters changes
    displayed_characters_dirty: bool,
    /// Current background asset
    current_background: Option<AssetRef>,
    /// Current CG (event graphics) asset
    current_cg: Option<AssetRef>,
    /// Global unlock data (shared across saves)
    unlock_data: Option<Arc<Mutex<UnlockData>>>,
}

/// Information about a displayed character
#[derive(Debug, Clone)]
pub struct DisplayedCharacter {
    /// Character ID
    pub character_id: String,
    /// Sprite asset path
    pub sprite: AssetRef,
    /// Position on screen
    pub position: CharacterPosition,
    /// Transition effect
    pub transition: Transition,
}

/// Result of executing a command
#[derive(Debug, Clone, PartialEq)]
pub enum CommandExecutionResult {
    /// Continue to next command
    Continue,
    /// Scene was changed (jumped), with optional exit and entry transitions
    SceneChanged {
        exit_transition: Option<Transition>,
        entry_transition: Option<Transition>,
    },
    /// Display choices to the player
    ShowChoices(Vec<ChoiceOption>),
    /// Wait for a duration (in seconds)
    Wait(f32),
    /// Scenario has ended
    End,
}

mod command_execution;
mod display_state;
mod execution_support;
mod flow_control;
mod lifecycle;
mod persistence;
mod state;

#[cfg(test)]
mod tests;
