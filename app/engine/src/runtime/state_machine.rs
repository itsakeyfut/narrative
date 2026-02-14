//! Game state machine
//!
//! This module implements the state machine for the visual novel engine,
//! including application-level states and detailed in-game states.
//!
//! # Implementation Status
//!
//! ✅ **Implemented (Phase 0.4)**:
//! - `AppState` and `InGameState` definitions
//! - State data structures (TypingState, ChoiceState, etc.)
//! - State helper methods and validation
//!
//! 📋 **TODO (Future Phases)**:
//!
//! The following components are defined in `docs/design/engine/runtime.md`
//! but not yet implemented. They will be added in future phases:
//!
//! - **Phase 0.5+**: RuntimeContext (flags, variables, read_history, scene_stack)
//! - **Phase 0.5+**: DisplayState (background, characters, dialogue_box, current_bgm)
//! - **Phase 0.5+**: CharacterDisplay, DialogueBoxState, BgmState
//! - **Phase 0.5+**: RuntimeSettings (text_speed, auto_mode, auto_wait_time)
//! - **Phase 0.6+**: ScenarioCommand enum (Dialogue, ShowCharacter, Jump, etc.)
//! - **Phase 0.6+**: ScenarioRuntime::update() and state transition logic
//! - **Phase 0.6+**: Command execution (execute_command, update_typing, etc.)
//!
//! See `docs/design/engine/runtime.md` for full design details.

use narrative_core::{CharacterId, ChoiceOption, SceneId, TransitionKind};
use std::sync::Arc;

// =============================================================================
// Top-level Application State
// =============================================================================

/// Application-level state
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Loading initial resources
    Loading(LoadingState),
    /// Main menu
    MainMenu(MainMenuState),
    /// In-game (playing scenario)
    InGame(InGameState),
    /// Settings menu
    Settings(SettingsState),
}

impl Default for AppState {
    fn default() -> Self {
        Self::Loading(LoadingState::default())
    }
}

/// Loading state
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoadingState {
    /// Loading progress (0.0 to 1.0)
    pub progress: f32,
    /// Current task being loaded
    pub current_task: String,
    /// Total number of tasks
    pub total_tasks: usize,
    /// Number of completed tasks
    pub completed_tasks: usize,
}

/// Main menu state
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MainMenuState {
    /// Currently selected menu item index
    pub selected_item: usize,
    /// Whether continue option is available (save exists)
    pub has_continue: bool,
}

/// Settings menu state
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SettingsState {
    /// Currently selected setting index
    pub selected_item: usize,
}

// =============================================================================
// In-Game States
// =============================================================================

/// In-game state with detailed substates
#[derive(Debug, Clone, PartialEq)]
pub enum InGameState {
    /// Text display with typewriter effect
    Typing(TypingState),
    /// Waiting for player input
    WaitingInput(WaitingInputState),
    /// Showing choice options
    ShowingChoices(ChoiceState),
    /// Scene transition animation
    Transition(TransitionState),
    /// Playing visual effect
    PlayingEffect(EffectState),
    /// Waiting for duration (Wait command)
    Waiting(WaitState),
    /// Pause menu
    PauseMenu(PauseMenuState),
    /// Save/load menu
    SaveLoadMenu(SaveLoadState),
    /// Backlog (dialogue history viewer)
    Backlog(BacklogState),
    /// CG gallery (unlocked CG collection)
    CgGallery(CgGalleryState),
    /// CG viewer (full-size CG display)
    CgViewer(CgViewerState),
}

/// Typewriter text display state
#[derive(Debug, Clone, PartialEq)]
pub struct TypingState {
    /// Current scene ID
    pub scene_id: SceneId,
    /// Current command index in the scene
    pub command_index: usize,
    /// Speaker name (None for narrator)
    pub speaker: Option<String>,
    /// Dialogue text to display (Arc<str> for efficient cloning during rendering)
    pub text: Arc<str>,
    /// Current character index being displayed
    pub char_index: usize,
    /// Elapsed time since last character
    pub elapsed: f32,
    /// Auto mode enabled
    pub auto_mode: bool,
    /// Skip mode enabled
    pub skip_mode: bool,
}

/// Waiting for input state
#[derive(Debug, Clone, PartialEq)]
pub struct WaitingInputState {
    /// Current scene ID
    pub scene_id: SceneId,
    /// Current command index
    pub command_index: usize,
    /// Time elapsed waiting (for auto mode)
    pub auto_wait_elapsed: f32,
    /// Skip mode enabled
    pub skip_mode: bool,
}

/// Choice selection state
#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceState {
    /// Current scene ID
    pub scene_id: SceneId,
    /// Current command index
    pub command_index: usize,
    /// Available choice options
    pub choices: Vec<ChoiceOption>,
    /// Currently selected choice index
    pub selected: usize,
    /// Whether a choice has been confirmed
    pub confirmed: bool,
}

/// Scene transition state
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionState {
    /// Source scene ID
    pub from_scene: SceneId,
    /// Destination scene ID
    pub to_scene: SceneId,
    /// Transition type
    pub kind: TransitionKind,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Total transition duration
    pub duration: f32,
}

/// Visual effect playback state
#[derive(Debug, Clone, PartialEq)]
pub struct EffectState {
    /// Effect type
    pub kind: EffectKind,
    /// Elapsed time
    pub elapsed: f32,
    /// Effect duration
    pub duration: f32,
}

/// Effect types
#[derive(Debug, Clone, PartialEq)]
pub enum EffectKind {
    /// Screen shake
    Shake { intensity: f32 },
    /// Screen flash
    Flash { color: [f32; 4] },
    /// Character animation
    CharacterAnimation { character_id: CharacterId },
}

/// Wait state (for Wait command)
#[derive(Debug, Clone, PartialEq)]
pub struct WaitState {
    /// Remaining wait time in seconds
    pub remaining: f32,
}

impl Default for WaitState {
    fn default() -> Self {
        Self { remaining: 0.0 }
    }
}

/// Pause menu state
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PauseMenuState {
    /// Currently selected menu item
    pub selected_item: usize,
}

/// Layout mode for save/load menu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutMode {
    /// List layout (1 column, 6 slots per page, detailed view)
    #[default]
    List,
    /// Grid layout (3 columns, 9 slots per page, compact view)
    Grid,
}

/// Save/Load menu state
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SaveLoadState {
    /// Whether in save mode (true) or load mode (false)
    pub is_save_mode: bool,
    /// Currently selected save slot (global index across all pages)
    pub selected_slot: usize,
    /// Current page (0-indexed)
    pub current_page: usize,
    /// Layout mode (List or Grid)
    pub layout_mode: LayoutMode,
}

/// Backlog state
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BacklogState {
    /// Current scroll position in the backlog
    pub scroll_offset: f32,
    /// Currently hovered entry index (if any)
    pub hovered_index: Option<usize>,
}

/// CG Gallery state
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CgGalleryState {
    /// Currently selected CG index
    pub selected_cg: usize,
    /// Current page (0-indexed)
    pub current_page: usize,
    /// Number of CGs per page (depends on layout)
    pub cgs_per_page: usize,
    /// Total number of CGs in the gallery
    pub total_cgs: usize,
}

impl CgGalleryState {
    /// Create a new CG gallery state
    pub fn new(total_cgs: usize) -> Self {
        Self {
            selected_cg: 0,
            current_page: 0,
            cgs_per_page: 9, // 3x3 grid by default
            total_cgs,
        }
    }

    /// Get the total number of pages
    pub fn total_pages(&self) -> usize {
        if self.total_cgs == 0 {
            1
        } else {
            (self
                .total_cgs
                .saturating_add(self.cgs_per_page)
                .saturating_sub(1))
                / self.cgs_per_page
        }
    }

    /// Get the index of the first CG on the current page
    pub fn first_cg_on_page(&self) -> usize {
        self.current_page.saturating_mul(self.cgs_per_page)
    }

    /// Get the index of the last CG on the current page
    pub fn last_cg_on_page(&self) -> usize {
        let last = self.first_cg_on_page().saturating_add(self.cgs_per_page);
        last.min(self.total_cgs)
    }

    /// Check if we can go to the next page
    pub fn can_next_page(&self) -> bool {
        self.current_page.saturating_add(1) < self.total_pages()
    }

    /// Check if we can go to the previous page
    pub fn can_prev_page(&self) -> bool {
        self.current_page > 0
    }
}

/// CG Viewer state (full-size CG display)
#[derive(Debug, Clone, PartialEq)]
pub struct CgViewerState {
    /// ID of the CG being viewed
    pub cg_id: String,
    /// Currently displayed variation index (0 = main image)
    pub variation_index: usize,
    /// Total number of variations (including main image)
    pub total_variations: usize,
}

impl CgViewerState {
    /// Create a new CG viewer state
    pub fn new(cg_id: impl Into<String>, total_variations: usize) -> Self {
        Self {
            cg_id: cg_id.into(),
            variation_index: 0,
            total_variations,
        }
    }

    /// Check if we can show the next variation
    pub fn can_next_variation(&self) -> bool {
        self.variation_index.saturating_add(1) < self.total_variations
    }

    /// Check if we can show the previous variation
    pub fn can_prev_variation(&self) -> bool {
        self.variation_index > 0
    }

    /// Move to the next variation
    pub fn next_variation(&mut self) {
        if self.can_next_variation() {
            self.variation_index = self.variation_index.saturating_add(1);
        }
    }

    /// Move to the previous variation
    pub fn prev_variation(&mut self) {
        if self.can_prev_variation() {
            self.variation_index = self.variation_index.saturating_sub(1);
        }
    }
}

// =============================================================================
// AppState Implementation
// =============================================================================

impl AppState {
    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading(_))
    }

    /// Check if in main menu
    pub fn is_main_menu(&self) -> bool {
        matches!(self, Self::MainMenu(_))
    }

    /// Check if in-game
    pub fn is_in_game(&self) -> bool {
        matches!(self, Self::InGame(_))
    }

    /// Check if in settings
    pub fn is_settings(&self) -> bool {
        matches!(self, Self::Settings(_))
    }

    /// Get in-game state if currently in-game
    pub fn in_game_state(&self) -> Option<&InGameState> {
        match self {
            Self::InGame(state) => Some(state),
            _ => None,
        }
    }

    /// Get mutable in-game state if currently in-game
    pub fn in_game_state_mut(&mut self) -> Option<&mut InGameState> {
        match self {
            Self::InGame(state) => Some(state),
            _ => None,
        }
    }
}

// =============================================================================
// State Data Structure Implementations
// =============================================================================

impl LoadingState {
    /// Set loading progress with automatic clamping to 0.0-1.0 range
    ///
    /// This prevents UI rendering bugs from invalid progress values.
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Create a new loading state with progress
    pub fn with_progress(mut self, progress: f32) -> Self {
        self.set_progress(progress);
        self
    }
}

impl TypingState {
    /// Create a new typing state
    pub fn new(
        scene_id: SceneId,
        command_index: usize,
        speaker: Option<String>,
        text: String,
    ) -> Self {
        Self {
            scene_id,
            command_index,
            speaker,
            text: Arc::from(text),
            char_index: 0,
            elapsed: 0.0,
            auto_mode: false,
            skip_mode: false,
        }
    }

    /// Get the total character count of the dialogue text
    pub fn text_length(&self) -> usize {
        self.text.chars().count()
    }

    /// Check if all text has been displayed
    pub fn is_complete(&self) -> bool {
        self.char_index >= self.text_length()
    }
}

impl TransitionState {
    /// Get transition progress as a safe ratio (0.0 to 1.0)
    ///
    /// Returns 0.0 if duration is zero to prevent division by zero.
    pub fn progress_ratio(&self) -> f32 {
        if self.duration <= 0.0 {
            0.0
        } else {
            (self.progress / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Update transition progress by delta time
    ///
    /// Automatically clamps progress to not exceed duration.
    pub fn update(&mut self, delta: f32) {
        self.progress = (self.progress + delta).min(self.duration);
    }

    /// Check if transition is complete
    pub fn is_complete(&self) -> bool {
        self.progress >= self.duration
    }
}

impl EffectState {
    /// Create a new effect state
    pub fn new(kind: EffectKind, duration: f32) -> Self {
        Self {
            kind,
            elapsed: 0.0,
            duration,
        }
    }

    /// Update effect elapsed time
    ///
    /// Automatically clamps elapsed to not exceed duration.
    /// Returns true if effect is complete.
    pub fn update(&mut self, delta: f32) -> bool {
        self.elapsed = (self.elapsed + delta).min(self.duration);
        self.is_complete()
    }

    /// Check if effect is complete
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Get effect progress as a ratio (0.0 to 1.0)
    pub fn progress_ratio(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0 // Instant effect
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }
}

impl WaitState {
    /// Create a new wait state with duration
    pub fn new(duration: f32) -> Self {
        Self {
            remaining: duration,
        }
    }

    /// Update remaining wait time
    ///
    /// Returns true if wait is complete.
    pub fn update(&mut self, delta: f32) -> bool {
        self.remaining = (self.remaining - delta).max(0.0);
        self.is_complete()
    }

    /// Check if wait is complete
    pub fn is_complete(&self) -> bool {
        self.remaining <= 0.0
    }
}

impl ChoiceState {
    /// Check if the current selection is valid
    pub fn is_valid_selection(&self) -> bool {
        self.selected < self.choices.len()
    }

    /// Get the currently selected choice if selection is valid
    pub fn selected_choice(&self) -> Option<&ChoiceOption> {
        if self.is_valid_selection() {
            self.choices.get(self.selected)
        } else {
            None
        }
    }
}

// =============================================================================
// InGameState Implementation
// =============================================================================

impl InGameState {
    /// Check if currently typing
    pub fn is_typing(&self) -> bool {
        matches!(self, Self::Typing(_))
    }

    /// Check if waiting for input
    pub fn is_waiting_input(&self) -> bool {
        matches!(self, Self::WaitingInput(_))
    }

    /// Check if showing choices
    pub fn is_showing_choices(&self) -> bool {
        matches!(self, Self::ShowingChoices(_))
    }

    /// Check if in transition
    pub fn is_transition(&self) -> bool {
        matches!(self, Self::Transition(_))
    }

    /// Check if playing effect
    pub fn is_playing_effect(&self) -> bool {
        matches!(self, Self::PlayingEffect(_))
    }

    /// Check if waiting
    pub fn is_waiting(&self) -> bool {
        matches!(self, Self::Waiting(_))
    }

    /// Check if in pause menu
    pub fn is_pause_menu(&self) -> bool {
        matches!(self, Self::PauseMenu(_))
    }

    /// Check if in save/load menu
    pub fn is_save_load_menu(&self) -> bool {
        matches!(self, Self::SaveLoadMenu(_))
    }

    /// Check if viewing backlog
    pub fn is_backlog(&self) -> bool {
        matches!(self, Self::Backlog(_))
    }

    /// Get current scene ID if available
    pub fn current_scene(&self) -> Option<&SceneId> {
        match self {
            Self::Typing(state) => Some(&state.scene_id),
            Self::WaitingInput(state) => Some(&state.scene_id),
            Self::ShowingChoices(state) => Some(&state.scene_id),
            Self::Transition(state) => Some(&state.to_scene),
            _ => None,
        }
    }

    /// Get current command index if available
    pub fn command_index(&self) -> Option<usize> {
        match self {
            Self::Typing(state) => Some(state.command_index),
            Self::WaitingInput(state) => Some(state.command_index),
            Self::ShowingChoices(state) => Some(state.command_index),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use narrative_core::SlideDirection;

    // =============================================================================
    // AppState Tests
    // =============================================================================

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(state.is_loading());
    }

    #[test]
    fn test_app_state_is_loading() {
        let state = AppState::Loading(LoadingState::default());
        assert!(state.is_loading());
        assert!(!state.is_main_menu());
        assert!(!state.is_in_game());
        assert!(!state.is_settings());
    }

    #[test]
    fn test_app_state_is_main_menu() {
        let state = AppState::MainMenu(MainMenuState::default());
        assert!(!state.is_loading());
        assert!(state.is_main_menu());
        assert!(!state.is_in_game());
        assert!(!state.is_settings());
    }

    #[test]
    fn test_app_state_is_in_game() {
        let typing_state = TypingState {
            scene_id: SceneId::new("test_scene"),
            command_index: 0,
            speaker: None,
            text: Arc::from("Hello"),
            char_index: 0,
            elapsed: 0.0,
            auto_mode: false,
            skip_mode: false,
        };
        let state = AppState::InGame(InGameState::Typing(typing_state));
        assert!(!state.is_loading());
        assert!(!state.is_main_menu());
        assert!(state.is_in_game());
        assert!(!state.is_settings());
    }

    #[test]
    fn test_app_state_is_settings() {
        let state = AppState::Settings(SettingsState::default());
        assert!(!state.is_loading());
        assert!(!state.is_main_menu());
        assert!(!state.is_in_game());
        assert!(state.is_settings());
    }

    #[test]
    fn test_app_state_in_game_state() {
        let typing_state = TypingState {
            scene_id: SceneId::new("test"),
            command_index: 0,
            speaker: Some("Alice".to_string()),
            text: Arc::from("Test"),
            char_index: 0,
            elapsed: 0.0,
            auto_mode: false,
            skip_mode: false,
        };
        let state = AppState::InGame(InGameState::Typing(typing_state.clone()));

        assert!(state.in_game_state().is_some());
        assert!(matches!(
            state.in_game_state(),
            Some(InGameState::Typing(_))
        ));

        let loading_state = AppState::Loading(LoadingState::default());
        assert!(loading_state.in_game_state().is_none());
    }

    // =============================================================================
    // InGameState Tests
    // =============================================================================

    #[test]
    fn test_in_game_state_is_typing() {
        let state = InGameState::Typing(TypingState {
            scene_id: SceneId::new("test"),
            command_index: 0,
            speaker: None,
            text: Arc::from("Test"),
            char_index: 0,
            elapsed: 0.0,
            auto_mode: false,
            skip_mode: false,
        });
        assert!(state.is_typing());
        assert!(!state.is_waiting_input());
        assert!(!state.is_showing_choices());
    }

    #[test]
    fn test_in_game_state_is_waiting_input() {
        let state = InGameState::WaitingInput(WaitingInputState {
            scene_id: SceneId::new("test"),
            command_index: 0,
            auto_wait_elapsed: 0.0,
            skip_mode: false,
        });
        assert!(!state.is_typing());
        assert!(state.is_waiting_input());
        assert!(!state.is_showing_choices());
    }

    #[test]
    fn test_in_game_state_is_showing_choices() {
        let state = InGameState::ShowingChoices(ChoiceState {
            scene_id: SceneId::new("test"),
            command_index: 0,
            choices: vec![],
            selected: 0,
            confirmed: false,
        });
        assert!(!state.is_typing());
        assert!(!state.is_waiting_input());
        assert!(state.is_showing_choices());
    }

    #[test]
    fn test_in_game_state_is_transition() {
        let state = InGameState::Transition(TransitionState {
            from_scene: SceneId::new("scene1"),
            to_scene: SceneId::new("scene2"),
            kind: TransitionKind::Fade,
            progress: 0.0,
            duration: 1.0,
        });
        assert!(state.is_transition());
        assert!(!state.is_typing());
    }

    #[test]
    fn test_in_game_state_is_playing_effect() {
        let state = InGameState::PlayingEffect(EffectState {
            kind: EffectKind::Shake { intensity: 1.0 },
            elapsed: 0.0,
            duration: 0.5,
        });
        assert!(state.is_playing_effect());
        assert!(!state.is_transition());
    }

    #[test]
    fn test_in_game_state_is_waiting() {
        let state = InGameState::Waiting(WaitState { remaining: 2.0 });
        assert!(state.is_waiting());
        assert!(!state.is_typing());
    }

    #[test]
    fn test_in_game_state_is_pause_menu() {
        let state = InGameState::PauseMenu(PauseMenuState::default());
        assert!(state.is_pause_menu());
        assert!(!state.is_save_load_menu());
    }

    #[test]
    fn test_in_game_state_is_save_load_menu() {
        let state = InGameState::SaveLoadMenu(SaveLoadState {
            is_save_mode: true,
            selected_slot: 0,
            current_page: 0,
            layout_mode: LayoutMode::List,
        });
        assert!(state.is_save_load_menu());
        assert!(!state.is_pause_menu());
    }

    #[test]
    fn test_in_game_state_current_scene() {
        let scene_id = SceneId::new("test_scene");

        let typing_state = InGameState::Typing(TypingState {
            scene_id: scene_id.clone(),
            command_index: 0,
            speaker: None,
            text: Arc::from("Test"),
            char_index: 0,
            elapsed: 0.0,
            auto_mode: false,
            skip_mode: false,
        });
        assert_eq!(typing_state.current_scene(), Some(&scene_id));

        let waiting_state = InGameState::WaitingInput(WaitingInputState {
            scene_id: scene_id.clone(),
            command_index: 0,
            auto_wait_elapsed: 0.0,
            skip_mode: false,
        });
        assert_eq!(waiting_state.current_scene(), Some(&scene_id));

        let pause_state = InGameState::PauseMenu(PauseMenuState::default());
        assert_eq!(pause_state.current_scene(), None);
    }

    #[test]
    fn test_in_game_state_command_index() {
        let typing_state = InGameState::Typing(TypingState {
            scene_id: SceneId::new("test"),
            command_index: 42,
            speaker: None,
            text: Arc::from("Test"),
            char_index: 0,
            elapsed: 0.0,
            auto_mode: false,
            skip_mode: false,
        });
        assert_eq!(typing_state.command_index(), Some(42));

        let pause_state = InGameState::PauseMenu(PauseMenuState::default());
        assert_eq!(pause_state.command_index(), None);
    }

    // =============================================================================
    // State Data Structure Tests
    // =============================================================================

    #[test]
    fn test_loading_state_default() {
        let state = LoadingState::default();
        assert_eq!(state.progress, 0.0);
        assert_eq!(state.current_task, "");
        assert_eq!(state.total_tasks, 0);
        assert_eq!(state.completed_tasks, 0);
    }

    #[test]
    fn test_loading_state_set_progress() {
        let mut state = LoadingState::default();

        // Normal progress
        state.set_progress(0.5);
        assert_eq!(state.progress, 0.5);

        // Clamp upper bound
        state.set_progress(1.5);
        assert_eq!(state.progress, 1.0);

        // Clamp lower bound
        state.set_progress(-0.5);
        assert_eq!(state.progress, 0.0);
    }

    #[test]
    fn test_loading_state_with_progress() {
        let state = LoadingState::default().with_progress(0.75);
        assert_eq!(state.progress, 0.75);

        let state = LoadingState::default().with_progress(2.0);
        assert_eq!(state.progress, 1.0);
    }

    #[test]
    fn test_main_menu_state_default() {
        let state = MainMenuState::default();
        assert_eq!(state.selected_item, 0);
        assert!(!state.has_continue);
    }

    #[test]
    fn test_transition_kind_equality() {
        assert_eq!(TransitionKind::Fade, TransitionKind::Fade);
        assert_ne!(TransitionKind::Fade, TransitionKind::Crossfade);
        assert_ne!(
            TransitionKind::Slide(SlideDirection::Left),
            TransitionKind::Slide(SlideDirection::Right)
        );
    }

    #[test]
    fn test_transition_state_progress_ratio() {
        // Normal transition
        let state = TransitionState {
            from_scene: SceneId::new("scene1"),
            to_scene: SceneId::new("scene2"),
            kind: TransitionKind::Fade,
            progress: 0.5,
            duration: 2.0,
        };
        assert_eq!(state.progress_ratio(), 0.25); // 0.5 / 2.0

        // Zero duration (prevent division by zero)
        let state = TransitionState {
            from_scene: SceneId::new("scene1"),
            to_scene: SceneId::new("scene2"),
            kind: TransitionKind::None,
            progress: 1.0,
            duration: 0.0,
        };
        assert_eq!(state.progress_ratio(), 0.0);

        // Progress exceeds duration (clamp to 1.0)
        let state = TransitionState {
            from_scene: SceneId::new("scene1"),
            to_scene: SceneId::new("scene2"),
            kind: TransitionKind::Fade,
            progress: 3.0,
            duration: 2.0,
        };
        assert_eq!(state.progress_ratio(), 1.0);
    }

    #[test]
    fn test_typing_state_new() {
        let state = TypingState::new(
            SceneId::new("test_scene"),
            5,
            Some("Alice".to_string()),
            "Hello, world!".to_string(),
        );

        assert_eq!(state.scene_id, SceneId::new("test_scene"));
        assert_eq!(state.command_index, 5);
        assert_eq!(state.speaker, Some("Alice".to_string()));
        assert_eq!(&*state.text, "Hello, world!");
        assert_eq!(state.char_index, 0);
        assert_eq!(state.elapsed, 0.0);
        assert!(!state.auto_mode);
    }

    #[test]
    fn test_typing_state_is_complete() {
        let mut state = TypingState::new(SceneId::new("test"), 0, None, "Test".to_string());

        assert!(!state.is_complete());

        state.char_index = 4; // "Test" has 4 characters
        assert!(state.is_complete());
    }

    #[test]
    fn test_typing_state_text_length() {
        let state = TypingState::new(SceneId::new("test"), 0, None, "Hello".to_string());
        assert_eq!(state.text_length(), 5);

        let state_unicode =
            TypingState::new(SceneId::new("test"), 0, None, "こんにちは".to_string());
        assert_eq!(state_unicode.text_length(), 5);
    }

    #[test]
    fn test_transition_state_update() {
        let mut state = TransitionState {
            from_scene: SceneId::new("scene1"),
            to_scene: SceneId::new("scene2"),
            kind: TransitionKind::Fade,
            progress: 0.0,
            duration: 2.0,
        };

        assert!(!state.is_complete());

        state.update(0.5);
        assert_eq!(state.progress, 0.5);
        assert!(!state.is_complete());

        state.update(1.5);
        assert_eq!(state.progress, 2.0);
        assert!(state.is_complete());

        // Should not exceed duration
        state.update(1.0);
        assert_eq!(state.progress, 2.0);
    }

    #[test]
    fn test_wait_state_new() {
        let state = WaitState::new(3.0);
        assert_eq!(state.remaining, 3.0);
    }

    #[test]
    fn test_wait_state_default() {
        let state = WaitState::default();
        assert_eq!(state.remaining, 0.0);
        assert!(state.is_complete());
    }

    #[test]
    fn test_wait_state_update() {
        let mut state = WaitState::new(2.0);

        assert!(!state.is_complete());

        let complete = state.update(1.0);
        assert!(!complete);
        assert_eq!(state.remaining, 1.0);

        let complete = state.update(1.0);
        assert!(complete);
        assert_eq!(state.remaining, 0.0);

        // Should not go negative
        let complete = state.update(1.0);
        assert!(complete);
        assert_eq!(state.remaining, 0.0);
    }

    #[test]
    fn test_choice_state_is_valid_selection() {
        use narrative_core::ChoiceOption;

        let choices = vec![
            ChoiceOption::new("Choice 1", "scene1"),
            ChoiceOption::new("Choice 2", "scene2"),
        ];

        // Valid selection
        let state = ChoiceState {
            scene_id: SceneId::new("test"),
            command_index: 0,
            choices: choices.clone(),
            selected: 0,
            confirmed: false,
        };
        assert!(state.is_valid_selection());

        // Valid selection (last index)
        let state = ChoiceState {
            scene_id: SceneId::new("test"),
            command_index: 0,
            choices: choices.clone(),
            selected: 1,
            confirmed: false,
        };
        assert!(state.is_valid_selection());

        // Invalid selection (out of bounds)
        let state = ChoiceState {
            scene_id: SceneId::new("test"),
            command_index: 0,
            choices: choices.clone(),
            selected: 2,
            confirmed: false,
        };
        assert!(!state.is_valid_selection());
    }

    #[test]
    fn test_choice_state_selected_choice() {
        use narrative_core::ChoiceOption;

        let choices = vec![
            ChoiceOption::new("Choice 1", "scene1"),
            ChoiceOption::new("Choice 2", "scene2"),
        ];

        // Valid selection
        let state = ChoiceState {
            scene_id: SceneId::new("test"),
            command_index: 0,
            choices: choices.clone(),
            selected: 0,
            confirmed: false,
        };
        assert!(state.selected_choice().is_some());
        assert_eq!(state.selected_choice().unwrap().text, "Choice 1");

        // Invalid selection
        let state = ChoiceState {
            scene_id: SceneId::new("test"),
            command_index: 0,
            choices: choices.clone(),
            selected: 5,
            confirmed: false,
        };
        assert!(state.selected_choice().is_none());
    }

    #[test]
    fn test_effect_kind_shake() {
        let effect = EffectKind::Shake { intensity: 2.5 };
        match effect {
            EffectKind::Shake { intensity } => assert_eq!(intensity, 2.5),
            _ => panic!("Expected Shake effect"),
        }
    }

    #[test]
    fn test_effect_kind_flash() {
        let color = [1.0, 0.0, 0.0, 1.0];
        let effect = EffectKind::Flash { color };
        match effect {
            EffectKind::Flash { color: c } => assert_eq!(c, color),
            _ => panic!("Expected Flash effect"),
        }
    }

    #[test]
    fn test_effect_state_new() {
        let state = EffectState::new(EffectKind::Shake { intensity: 1.5 }, 2.0);
        assert!(matches!(state.kind, EffectKind::Shake { intensity } if intensity == 1.5));
        assert_eq!(state.elapsed, 0.0);
        assert_eq!(state.duration, 2.0);
    }

    #[test]
    fn test_effect_state_update() {
        let mut state = EffectState::new(
            EffectKind::Flash {
                color: [1.0, 1.0, 1.0, 1.0],
            },
            1.0,
        );

        assert!(!state.is_complete());

        let complete = state.update(0.5);
        assert!(!complete);
        assert_eq!(state.elapsed, 0.5);
        assert_eq!(state.progress_ratio(), 0.5);

        let complete = state.update(0.5);
        assert!(complete);
        assert_eq!(state.elapsed, 1.0);
        assert_eq!(state.progress_ratio(), 1.0);

        // Should not exceed duration
        state.update(1.0);
        assert_eq!(state.elapsed, 1.0);
    }

    #[test]
    fn test_effect_state_progress_ratio() {
        // Zero duration (instant effect)
        let state = EffectState::new(EffectKind::Shake { intensity: 1.0 }, 0.0);
        assert_eq!(state.progress_ratio(), 1.0);

        // Normal progress
        let state = EffectState {
            kind: EffectKind::Shake { intensity: 1.0 },
            elapsed: 1.5,
            duration: 3.0,
        };
        assert_eq!(state.progress_ratio(), 0.5);
    }

    #[test]
    fn test_save_load_state_default() {
        let state = SaveLoadState::default();
        assert!(!state.is_save_mode); // Default to load mode
        assert_eq!(state.selected_slot, 0);
    }

    #[test]
    fn test_save_load_state_save_mode() {
        let state = SaveLoadState {
            is_save_mode: true,
            selected_slot: 0,
            current_page: 0,
            layout_mode: LayoutMode::List,
        };
        assert!(state.is_save_mode);
    }

    #[test]
    fn test_save_load_state_load_mode() {
        let state = SaveLoadState {
            is_save_mode: false,
            selected_slot: 1,
            current_page: 0,
            layout_mode: LayoutMode::List,
        };
        assert!(!state.is_save_mode);
        assert_eq!(state.selected_slot, 1);
    }
}
