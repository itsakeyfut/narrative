//! State management for GameRootElement

use super::element::GameRootElement;
use crate::components::{PauseMenuAction, PauseMenuElement, TitleScreenAction, TitleScreenElement};
use narrative_core::config::UserSettings;
use narrative_engine::runtime::{
    AppState, InGameState, LayoutMode, MainMenuState, SaveLoadState, ScenarioRuntime,
};
use std::sync::Arc;

// Constants
const LOADING_DURATION: f32 = 1.0;
const LOADING_COMPLETE_THRESHOLD: f32 = 1.0;

impl GameRootElement {
    /// Update game state (called every frame from tick())
    pub(super) fn update_state(&mut self, delta: f32) {
        // Save old state discriminant for dirty flag detection
        let old_state_discriminant = std::mem::discriminant(&self.app_state);
        let was_in_game = matches!(self.app_state, AppState::InGame(_));
        let was_main_menu = matches!(self.app_state, AppState::MainMenu(_));
        let was_typing_or_waiting = matches!(
            self.app_state,
            AppState::InGame(InGameState::Typing(_) | InGameState::WaitingInput(_))
        );

        // Match and extract values to avoid borrowing conflicts
        match &self.app_state {
            AppState::Loading(_) => {
                let mut should_transition = false;
                if let AppState::Loading(loading) = &mut self.app_state {
                    loading.progress += delta / LOADING_DURATION;
                    loading.set_progress(loading.progress);
                    if loading.progress >= LOADING_COMPLETE_THRESHOLD {
                        should_transition = true;
                    }
                }
                if should_transition {
                    tracing::info!("Loading complete, transitioning to main menu");
                    self.app_state = AppState::MainMenu(MainMenuState::default());
                }
            }
            AppState::MainMenu(_) => {
                self.update_main_menu_state();
            }
            AppState::InGame(_) => {
                // Extract state data to avoid borrow conflicts
                self.update_in_game_state_wrapper(delta);
            }
            AppState::Settings(_) => {
                // Settings menu handling would go here
            }
        }

        // Mark children as dirty if state changed
        let new_state_discriminant = std::mem::discriminant(&self.app_state);
        if old_state_discriminant != new_state_discriminant {
            tracing::debug!(
                "update_state(): State discriminant changed, setting children_dirty=true"
            );
            tracing::debug!("children_dirty set at line {}", line!());
            self.children_dirty = true;
        }

        // Handle BGM transitions based on state changes
        let is_in_game = matches!(self.app_state, AppState::InGame(_));
        let is_main_menu = matches!(self.app_state, AppState::MainMenu(_));

        // Start title BGM when transitioning to MainMenu
        if !was_main_menu && is_main_menu {
            self.start_title_bgm();
        }

        // Start game BGM when transitioning to InGame state
        if !was_in_game && is_in_game {
            self.start_bgm();
        }

        // Reset UI hidden flag when leaving Typing/WaitingInput states
        let is_typing_or_waiting = matches!(
            self.app_state,
            AppState::InGame(InGameState::Typing(_) | InGameState::WaitingInput(_))
        );
        if was_typing_or_waiting && !is_typing_or_waiting && self.ui_hidden {
            self.ui_hidden = false;
            tracing::debug!("children_dirty set at line {}", line!());
            self.children_dirty = true;
            tracing::debug!("UI visibility restored (left Typing/WaitingInput state)");
        }
    }

    /// Update main menu state
    pub(super) fn update_main_menu_state(&mut self) {
        // Check if title screen has a confirmed action
        let confirmed_action = self.children.iter().find_map(|child| {
            child
                .as_any()
                .downcast_ref::<TitleScreenElement>()
                .and_then(|title_screen| title_screen.confirmed_action())
        });

        if let Some(action) = confirmed_action {
            tracing::debug!("Title screen action confirmed: {:?}", action);

            // Reset the confirmation to prevent repeated processing
            for child in &mut self.children {
                if let Some(title_screen) = child.as_any_mut().downcast_mut::<TitleScreenElement>()
                {
                    title_screen.reset_confirmation();
                    break;
                }
            }

            match action {
                TitleScreenAction::NewGame => {
                    self.start_new_game();
                }
                TitleScreenAction::Continue => {
                    // TODO: Implement continue from last save
                    tracing::warn!("Continue not yet implemented, starting new game");
                    self.start_new_game();
                }
                TitleScreenAction::Load => {
                    // Transition to Save/Load menu in load mode
                    tracing::debug!("Opening load menu from title screen");

                    // Load user settings to get layout preference
                    let layout_mode = UserSettings::load("assets/config/settings.ron")
                        .map(|s| match s.display.save_menu_layout {
                            narrative_core::config::SaveMenuLayoutMode::List => LayoutMode::List,
                            narrative_core::config::SaveMenuLayoutMode::Grid => LayoutMode::Grid,
                        })
                        .inspect_err(|e| {
                            tracing::warn!(
                                "Failed to load user settings, using default layout (List): {}",
                                e
                            );
                        })
                        .unwrap_or(LayoutMode::List);

                    self.app_state = AppState::InGame(InGameState::SaveLoadMenu(SaveLoadState {
                        is_save_mode: false,
                        selected_slot: 0,
                        current_page: 0,
                        layout_mode,
                    }));
                    tracing::debug!("children_dirty set at line {}", line!());
                    self.children_dirty = true;
                }
                TitleScreenAction::CgGallery => {
                    // Transition to CG Gallery
                    tracing::debug!("Opening CG Gallery from title screen");
                    let total_cgs = self.cg_registry.total_count();
                    self.app_state = AppState::InGame(InGameState::CgGallery(
                        narrative_engine::runtime::CgGalleryState::new(total_cgs),
                    ));
                    tracing::debug!("children_dirty set at line {}", line!());
                    self.children_dirty = true;
                }
                TitleScreenAction::Settings => {
                    // Transition to settings menu
                    tracing::debug!("Opening settings from title screen");
                    self.previous_app_state = Some(Box::new(self.app_state.clone()));
                    self.app_state = AppState::Settings(Default::default());
                    tracing::debug!("children_dirty set at line {}", line!());
                    self.children_dirty = true;
                }
                TitleScreenAction::Exit => {
                    tracing::info!("Exit requested - closing application");
                    self.window_operations
                        .push(narrative_gui::framework::element::WindowOperation::Close);
                }
            }
        }
    }

    /// Update pause menu state
    pub(super) fn update_pause_menu_state(&mut self) {
        // Check if pause menu has a confirmed action
        let confirmed_action = self.children.iter().find_map(|child| {
            child
                .as_any()
                .downcast_ref::<PauseMenuElement>()
                .and_then(|pause_menu| pause_menu.confirmed_action())
        });

        if let Some(action) = confirmed_action {
            tracing::debug!("Pause menu action confirmed: {:?}", action);

            // Reset the confirmation to prevent repeated processing
            for child in &mut self.children {
                if let Some(pause_menu) = child.as_any_mut().downcast_mut::<PauseMenuElement>() {
                    pause_menu.reset_confirmation();
                    break;
                }
            }

            match action {
                PauseMenuAction::Resume => {
                    // Resume game - restore previous state
                    if let Some(prev_state) = self.previous_in_game_state.take()
                        && let AppState::InGame(in_game_state) = &mut self.app_state
                    {
                        *in_game_state = *prev_state;
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                    }
                }
                PauseMenuAction::Save => {
                    // Transition to Save/Load menu in save mode
                    tracing::debug!("Opening save menu from pause menu");

                    // Load user settings to get layout preference
                    let layout_mode = UserSettings::load("assets/config/settings.ron")
                        .map(|s| match s.display.save_menu_layout {
                            narrative_core::config::SaveMenuLayoutMode::List => LayoutMode::List,
                            narrative_core::config::SaveMenuLayoutMode::Grid => LayoutMode::Grid,
                        })
                        .inspect_err(|e| {
                            tracing::warn!(
                                "Failed to load user settings, using default layout (List): {}",
                                e
                            );
                        })
                        .unwrap_or(LayoutMode::List);

                    if let AppState::InGame(in_game_state) = &mut self.app_state {
                        // Save current pause menu state so we can return to it
                        self.previous_in_game_state = Some(Box::new(in_game_state.clone()));

                        *in_game_state = InGameState::SaveLoadMenu(SaveLoadState {
                            is_save_mode: true,
                            selected_slot: 0,
                            current_page: 0,
                            layout_mode,
                        });
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                    }
                }
                PauseMenuAction::Load => {
                    // Transition to Save/Load menu in load mode
                    tracing::debug!("Opening load menu from pause menu");

                    // Load user settings to get layout preference
                    let layout_mode = UserSettings::load("assets/config/settings.ron")
                        .map(|s| match s.display.save_menu_layout {
                            narrative_core::config::SaveMenuLayoutMode::List => LayoutMode::List,
                            narrative_core::config::SaveMenuLayoutMode::Grid => LayoutMode::Grid,
                        })
                        .inspect_err(|e| {
                            tracing::warn!(
                                "Failed to load user settings, using default layout (List): {}",
                                e
                            );
                        })
                        .unwrap_or(LayoutMode::List);

                    if let AppState::InGame(in_game_state) = &mut self.app_state {
                        // Save current pause menu state so we can return to it
                        self.previous_in_game_state = Some(Box::new(in_game_state.clone()));

                        *in_game_state = InGameState::SaveLoadMenu(SaveLoadState {
                            is_save_mode: false,
                            selected_slot: 0,
                            current_page: 0,
                            layout_mode,
                        });
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                    }
                }
                PauseMenuAction::Settings => {
                    // Open settings from pause menu
                    tracing::debug!("Opening settings from pause menu");
                    self.previous_app_state = Some(Box::new(self.app_state.clone()));
                    self.app_state = AppState::Settings(Default::default());
                    tracing::debug!("children_dirty set at line {}", line!());
                    self.children_dirty = true;
                }
                PauseMenuAction::Title => {
                    // Show confirmation dialog
                    tracing::debug!("Showing confirmation dialog for return to title");
                    self.showing_title_confirm = true;
                    tracing::debug!("children_dirty set at line {}", line!());
                    self.children_dirty = true;
                }
            }
        }
    }

    /// Start a new game
    pub(super) fn start_new_game(&mut self) {
        if self.scenario_runtime.is_some() {
            tracing::debug!("Scenario already loaded, starting new game will reset it");
        }

        tracing::info!(
            "Starting new game: {}",
            self.config.start_scenario.display()
        );
        match ScenarioRuntime::from_toml(&self.config.start_scenario) {
            Ok(mut runtime) => {
                // Set unlock data for CG tracking
                runtime.set_unlock_data(Arc::clone(&self.unlock_data));

                if let Err(e) = runtime.start() {
                    tracing::error!("Failed to start scenario: {}", e);
                    tracing::warn!("Staying in MainMenu due to scenario start failure");
                    return;
                }

                // Execute commands until we reach a waiting state
                let mut audio = self.audio_manager.lock().unwrap_or_else(|e| {
                    tracing::warn!("AudioManager mutex poisoned, recovering: {}", e);
                    e.into_inner()
                });
                if let Some(initial_state) = Self::execute_and_transition(&mut runtime, &mut audio)
                {
                    self.scenario_runtime = Some(runtime);
                    self.app_state = AppState::InGame(initial_state);
                    tracing::debug!("children_dirty set at line {}", line!());
                    self.children_dirty = true;
                    tracing::debug!("Scenario started successfully");
                } else {
                    tracing::error!("Failed to create initial state from command");
                    tracing::warn!("Staying in MainMenu - scenario has no valid initial command");
                }
            }
            Err(e) => {
                tracing::error!(
                    "Failed to load scenario file '{}': {}",
                    self.config.start_scenario.display(),
                    e
                );
                tracing::warn!("Staying in MainMenu - please check scenario file path");
            }
        }
    }

    /// Toggle settings menu (shared logic for F1 and ESC keys)
    pub(super) fn toggle_settings_menu(&mut self) {
        if matches!(self.app_state, AppState::Settings(_)) {
            // Exiting settings - restore previous state
            if let Some(prev_state) = self.previous_app_state.take() {
                tracing::debug!("Exiting settings, restoring previous state");
                self.app_state = *prev_state;
            } else {
                // Fallback if no previous state saved
                tracing::warn!("No previous state to restore, going to main menu");
                self.app_state =
                    AppState::MainMenu(narrative_engine::runtime::MainMenuState::default());
            }
        } else {
            // Entering settings - save current state
            tracing::debug!("Opening settings and saving current state");
            self.previous_app_state = Some(Box::new(self.app_state.clone()));
            self.app_state =
                AppState::Settings(narrative_engine::runtime::SettingsState::default());
        }
        tracing::debug!("children_dirty set at line {}", line!());
        self.children_dirty = true;
    }
}
