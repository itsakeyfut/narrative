//! In-game state update logic for GameRootElement

use super::element::GameRootElement;
use crate::components::{
    BacklogElement, CgGalleryAction, CgGalleryElement, CgViewerAction, CgViewerElement,
    ChoiceMenuElement, ConfirmDialogElement, DialogueBoxElement, QuickMenuAction, QuickMenuElement,
    SaveLoadMenuAction, SaveLoadMenuElement,
};
use narrative_core::ScenarioCommand;
use narrative_engine::runtime::{
    AppState, InGameState, MainMenuState, ScenarioRuntime, WaitingInputState,
};
use std::sync::Arc;

impl GameRootElement {
    pub(super) fn update_in_game_state_wrapper(&mut self, delta: f32) {
        // Check if runtime exists (only required for gameplay states, not menus)
        if self.scenario_runtime.is_none()
            && let AppState::InGame(in_game_state) = &self.app_state
            && !matches!(
                in_game_state,
                InGameState::SaveLoadMenu(_)
                    | InGameState::PauseMenu(_)
                    | InGameState::Backlog(_)
                    | InGameState::CgGallery(_)
                    | InGameState::CgViewer(_)
            )
        {
            tracing::error!("InGame state without runtime!");
            return;
        }

        // Handle quick menu actions FIRST (before checking flags)
        // This ensures that actions set flags in the same frame they will be processed
        for child in &mut self.children {
            if let Some(quick_menu) = child.as_any_mut().downcast_mut::<QuickMenuElement>()
                && let Some(action) = quick_menu.pending_action()
            {
                match action {
                    QuickMenuAction::ToggleSkip => {
                        self.skip_mode_toggle_pressed = true;
                    }
                    QuickMenuAction::ToggleAuto => {
                        self.auto_mode_toggle_pressed = true;
                    }
                    QuickMenuAction::OpenBacklog => {
                        self.backlog_pressed = true;
                    }
                    QuickMenuAction::QuickSave => {
                        // Quick save to slot 0
                        if let Some(runtime) = &self.scenario_runtime {
                            let mut save_data = runtime.to_save_data(0);

                            // Set timestamp and play time
                            save_data.timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or_else(|e| {
                                    tracing::error!(
                                        "Failed to get system time for quick save: {:?}",
                                        e
                                    );
                                    // Fallback: use 0 (will be logged as error above)
                                    0
                                });
                            save_data.play_time_secs = self.total_play_time_secs;

                            // Save to file
                            match self.save_manager.lock() {
                                Ok(manager) => match manager.save(0, &save_data) {
                                    Ok(_) => {
                                        tracing::info!("Quick save successful (slot 0)");
                                    }
                                    Err(e) => {
                                        tracing::error!("Quick save failed: {:?}", e);
                                    }
                                },
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to lock save_manager for quick save: {:?}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    QuickMenuAction::OpenMenu => {
                        self.pause_pressed = true;
                    }
                }
                quick_menu.clear_pending_action();
            }
        }

        // Handle auto mode toggle
        if self.auto_mode_toggle_pressed {
            self.config.gameplay.auto_mode_enabled = !self.config.gameplay.auto_mode_enabled;
            tracing::info!(
                "Auto mode toggled: enabled={}",
                self.config.gameplay.auto_mode_enabled
            );
            tracing::debug!("children_dirty set at line {}", line!());
            self.children_dirty = true;
        }

        // Handle skip mode toggle
        if self.skip_mode_toggle_pressed {
            self.config.gameplay.skip_mode_enabled = !self.config.gameplay.skip_mode_enabled;
            tracing::info!(
                "Skip mode toggled: enabled={}, mode={:?}",
                self.config.gameplay.skip_mode_enabled,
                self.config.gameplay.skip_mode
            );
            tracing::debug!("children_dirty set at line {}", line!());
            self.children_dirty = true;
        }

        if let AppState::InGame(in_game_state) = &mut self.app_state {
            match in_game_state {
                InGameState::Typing(typing) => {
                    // Handle pause key
                    if self.pause_pressed {
                        self.previous_in_game_state = Some(Box::new(in_game_state.clone()));
                        *in_game_state = InGameState::PauseMenu(Default::default());
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                        return;
                    }

                    // Handle backlog key
                    if self.backlog_pressed {
                        self.previous_in_game_state = Some(Box::new(in_game_state.clone()));
                        *in_game_state = InGameState::Backlog(Default::default());
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                        return;
                    }

                    // Check if skip mode should be active for this dialogue
                    typing.skip_mode = self.config.gameplay.skip_mode_enabled
                        && self.config.gameplay.skip_mode.is_enabled()
                        && {
                            if self.config.gameplay.skip_mode.allows_unread() {
                                // Skip all mode - always skip
                                true
                            } else if self.config.gameplay.skip_mode.requires_read() {
                                // Skip read-only mode - check if this dialogue has been read
                                if let Some(runtime) = &self.scenario_runtime {
                                    runtime
                                        .read_history()
                                        .is_read(&typing.scene_id, typing.command_index)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };

                    // Inline typewriter logic to avoid borrow checker issues
                    typing.elapsed += delta;

                    let text_len = typing.text.chars().count();
                    let old_char_index = typing.char_index;

                    // Calculate character delay from text speed
                    let char_delay = if typing.skip_mode {
                        // In skip mode, show text instantly
                        0.0
                    } else if self.config.gameplay.text_speed > 0.0 {
                        1.0 / self.config.gameplay.text_speed
                    } else {
                        0.0
                    };

                    // Progress typewriter
                    while typing.elapsed >= char_delay && typing.char_index < text_len {
                        typing.char_index = typing.char_index.saturating_add(1);
                        typing.elapsed -= char_delay;
                    }

                    // In skip mode, immediately show all text
                    if typing.skip_mode && typing.char_index < text_len {
                        typing.char_index = text_len;
                    }

                    // Handle input - skip to end
                    if self.clicked_last_frame && typing.char_index < text_len {
                        typing.char_index = text_len;
                    }

                    // Update DialogueBoxElement's visible_chars if char_index changed
                    // This avoids rebuilding all children (which would restart character sprite transitions)
                    if typing.char_index != old_char_index {
                        for child in &mut self.children {
                            if let Some(dialogue_box) =
                                child.as_any_mut().downcast_mut::<DialogueBoxElement>()
                            {
                                dialogue_box.set_visible_chars(typing.char_index);
                                break;
                            }
                        }
                    }

                    // Check if we should transition
                    let should_transition =
                        (typing.skip_mode || !typing.auto_mode || self.clicked_last_frame)
                            && typing.char_index >= text_len;

                    // Transition to WaitingInput if needed
                    if should_transition {
                        let scene_id = typing.scene_id.clone();
                        let command_index = typing.command_index;
                        *in_game_state = InGameState::WaitingInput(WaitingInputState {
                            scene_id,
                            command_index,
                            auto_wait_elapsed: 0.0,
                            skip_mode: typing.skip_mode,
                        });
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                    }
                }
                InGameState::WaitingInput(waiting) => {
                    // Handle pause key
                    if self.pause_pressed {
                        self.previous_in_game_state = Some(Box::new(in_game_state.clone()));
                        *in_game_state = InGameState::PauseMenu(Default::default());
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                        return;
                    }

                    // Handle backlog key
                    if self.backlog_pressed {
                        self.previous_in_game_state = Some(Box::new(in_game_state.clone()));
                        *in_game_state = InGameState::Backlog(Default::default());
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                        return;
                    }

                    // Mark dialogue as read
                    if let Some(runtime) = &mut self.scenario_runtime {
                        runtime
                            .read_history_mut()
                            .mark_read(waiting.scene_id.clone(), waiting.command_index);
                    }

                    // In skip mode, auto-advance immediately
                    if waiting.skip_mode {
                        tracing::debug!("Skip mode active, auto-advancing immediately");
                        self.advance_after_waiting_input();
                        return;
                    }

                    // Update auto-advance timer
                    if self.config.gameplay.auto_mode_enabled {
                        waiting.auto_wait_elapsed += delta;

                        // Calculate wait duration based on auto_advance_speed
                        let wait_duration = self.config.gameplay.auto_advance_speed;

                        // Check if we should auto-advance
                        // Note: voice waiting is not implemented yet (voice player is stub)
                        if waiting.auto_wait_elapsed >= wait_duration {
                            tracing::debug!(
                                "Auto-advancing after {:.2}s (wait_duration={:.2}s)",
                                waiting.auto_wait_elapsed,
                                wait_duration
                            );
                            self.advance_after_waiting_input();
                            return;
                        }
                    }

                    // Handle manual click to advance
                    // Note: Manual click works even in auto mode (intentional behavior)
                    if self.clicked_last_frame {
                        tracing::debug!(
                            "WaitingInput: clicked_last_frame=true, calling advance_after_waiting_input"
                        );
                        self.advance_after_waiting_input();
                    } else {
                        tracing::trace!("WaitingInput: clicked_last_frame=false");
                    }
                }
                InGameState::ShowingChoices(choice_state) => {
                    // Handle backlog key
                    if self.backlog_pressed {
                        self.previous_in_game_state = Some(Box::new(in_game_state.clone()));
                        *in_game_state = InGameState::Backlog(Default::default());
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                        return;
                    }

                    // Stop skip mode at choices if configured
                    if self.config.gameplay.skip_stop_at_choices
                        && self.config.gameplay.skip_mode_enabled
                    {
                        self.config.gameplay.skip_mode_enabled = false;
                        tracing::debug!("Skip mode stopped at choice");
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                    }

                    // Check if ChoiceMenuElement has confirmed a choice
                    let mut choice_confirmed = false;
                    let mut selected_index = choice_state.selected;

                    // Find the ChoiceMenuElement from children (not necessarily the first one)
                    for child in self.children.iter_mut() {
                        if let Some(choice_menu) =
                            child.as_any_mut().downcast_mut::<ChoiceMenuElement>()
                        {
                            // Update selection from menu (in case user used arrow keys on the element)
                            selected_index = choice_menu.selected_index();
                            choice_state.selected = selected_index;

                            // Check if choice was confirmed
                            if choice_menu.is_choice_confirmed() && !choice_state.confirmed {
                                choice_confirmed = true;
                                choice_menu.reset_confirmation();
                            }
                            break; // Found the choice menu, no need to continue
                        }
                    }

                    // Execute choice if confirmed (choice_confirmed already includes !confirmed check)
                    if choice_confirmed {
                        tracing::debug!("Executing choice: index={}", selected_index);
                        // Execute choice in runtime
                        if let Some(runtime) = self.scenario_runtime.as_mut() {
                            // select_choice() jumps to the next scene and returns transitions
                            let (exit_transition, entry_transition) =
                                match runtime.select_choice(selected_index) {
                                    Ok(transitions) => transitions,
                                    Err(e) => {
                                        tracing::error!("Failed to select choice: {}", e);
                                        return;
                                    }
                                };

                            // Log transitions
                            if let Some(exit) = exit_transition {
                                tracing::debug!(
                                    "Exit transition: {:?} ({:.1}s)",
                                    exit.kind,
                                    exit.duration
                                );
                            }
                            if let Some(entry) = &entry_transition {
                                tracing::debug!(
                                    "Entry transition: {:?} ({:.1}s)",
                                    entry.kind,
                                    entry.duration
                                );
                            }

                            // Mark as confirmed to prevent re-execution
                            choice_state.confirmed = true;

                            // If there's an entry transition, create a TransitionState
                            if let Some(entry) = entry_transition {
                                // Execute ShowBackground command BEFORE creating transition
                                // This ensures the background is changed before transition starts
                                if let Some(ScenarioCommand::ShowBackground { .. }) =
                                    runtime.get_current_command()
                                {
                                    tracing::debug!("Executing ShowBackground before transition");
                                    if let Err(e) = runtime.execute_current_command() {
                                        tracing::error!("Failed to execute ShowBackground: {}", e);
                                    }
                                    runtime.advance_command();
                                }

                                // Get current scene for transition
                                if let Some(to_scene) = runtime.current_scene() {
                                    let to_scene = to_scene.clone();
                                    let from_scene = to_scene.clone();

                                    *in_game_state = InGameState::Transition(
                                        narrative_engine::runtime::TransitionState {
                                            from_scene,
                                            to_scene,
                                            kind: entry.kind,
                                            progress: 0.0,
                                            duration: entry.duration,
                                        },
                                    );

                                    // Now detect background change and schedule load
                                    // The background has already changed, so this will schedule the load
                                    self.update_background_if_changed();
                                    tracing::debug!("children_dirty set at line {}", line!());
                                    self.children_dirty = true;
                                } else {
                                    tracing::error!(
                                        "Failed to create transition: no current scene"
                                    );
                                    // Fall back to executing next command without transition
                                    let mut audio = self.audio_manager.lock().unwrap_or_else(|e| {
                                        tracing::warn!(
                                            "AudioManager mutex poisoned, recovering: {}",
                                            e
                                        );
                                        e.into_inner()
                                    });
                                    if let Some(new_state) =
                                        Self::execute_and_transition(runtime, &mut audio)
                                    {
                                        *in_game_state = new_state;
                                        tracing::debug!("children_dirty set at line {}", line!());
                                        self.children_dirty = true;
                                    }
                                }
                            } else {
                                // No entry transition, execute the first command of the new scene
                                let mut audio = self.audio_manager.lock().unwrap_or_else(|e| {
                                    tracing::warn!(
                                        "AudioManager mutex poisoned, recovering: {}",
                                        e
                                    );
                                    e.into_inner()
                                });
                                if let Some(new_state) =
                                    Self::execute_and_transition(runtime, &mut audio)
                                {
                                    tracing::debug!("Choice confirmed, transitioning to new state");
                                    *in_game_state = new_state;
                                    tracing::debug!("children_dirty set at line {}", line!());
                                    self.children_dirty = true;
                                } else {
                                    tracing::error!("Failed to create state after choice");
                                }
                            }
                        }
                    }
                }
                InGameState::Transition(transition) => {
                    transition.update(delta);
                    if transition.is_complete()
                        && let Some(runtime) = self.scenario_runtime.as_mut()
                    {
                        let mut audio = self.audio_manager.lock().unwrap_or_else(|e| {
                            tracing::warn!("AudioManager mutex poisoned, recovering: {}", e);
                            e.into_inner()
                        });
                        if let Some(new_state) = Self::execute_and_transition(runtime, &mut audio) {
                            *in_game_state = new_state;
                            // Clear previous background and CG after transition completes
                            self.previous_background_texture_id = None;
                            self.previous_cg_texture_id = None;
                            self.previous_cg_texture_size = None;
                            tracing::debug!("children_dirty set at line {}", line!());
                            self.children_dirty = true;
                        } else {
                            tracing::debug!("Scenario ended after transition");
                            self.app_state = AppState::MainMenu(MainMenuState::default());
                            // Clear previous background and CG when scenario ends
                            self.previous_background_texture_id = None;
                            self.previous_cg_texture_id = None;
                            self.previous_cg_texture_size = None;
                            tracing::debug!("children_dirty set at line {}", line!());
                            self.children_dirty = true;
                        }
                    }
                }
                InGameState::PlayingEffect(effect) => {
                    if effect.update(delta)
                        && let Some(runtime) = self.scenario_runtime.as_mut()
                    {
                        let mut audio = self.audio_manager.lock().unwrap_or_else(|e| {
                            tracing::warn!("AudioManager mutex poisoned, recovering: {}", e);
                            e.into_inner()
                        });
                        if let Some(new_state) = Self::execute_and_transition(runtime, &mut audio) {
                            *in_game_state = new_state;
                            tracing::debug!("children_dirty set at line {}", line!());
                            self.children_dirty = true;
                        } else {
                            tracing::debug!("Scenario ended after effect");
                            self.app_state = AppState::MainMenu(MainMenuState::default());
                            tracing::debug!("children_dirty set at line {}", line!());
                            self.children_dirty = true;
                        }
                    }
                }
                InGameState::Waiting(wait) => {
                    if wait.update(delta)
                        && let Some(runtime) = self.scenario_runtime.as_mut()
                    {
                        // Wait completed, advance to next command
                        runtime.advance_command();

                        let mut audio = self.audio_manager.lock().unwrap_or_else(|e| {
                            tracing::warn!("AudioManager mutex poisoned, recovering: {}", e);
                            e.into_inner()
                        });
                        if let Some(new_state) = Self::execute_and_transition(runtime, &mut audio) {
                            *in_game_state = new_state;
                            tracing::debug!("children_dirty set at line {}", line!());
                            self.children_dirty = true;
                        } else {
                            tracing::debug!("Scenario ended after wait");
                            self.app_state = AppState::MainMenu(MainMenuState::default());
                            tracing::debug!("children_dirty set at line {}", line!());
                            self.children_dirty = true;
                        }
                    }
                }
                InGameState::PauseMenu(_) => {
                    // Check if confirmation dialog is being shown
                    if self.showing_title_confirm {
                        // Handle confirmation dialog response
                        let mut confirmed = false;
                        let mut cancelled = false;

                        for child in &self.children {
                            if let Some(dialog) =
                                child.as_any().downcast_ref::<ConfirmDialogElement>()
                            {
                                if dialog.is_confirmed() {
                                    confirmed = true;
                                } else if dialog.is_cancelled() {
                                    cancelled = true;
                                }
                                break;
                            }
                        }

                        if confirmed {
                            // User confirmed - return to title
                            tracing::debug!("Returning to title screen from pause menu");
                            self.app_state = AppState::MainMenu(MainMenuState::default());
                            self.showing_title_confirm = false;
                            self.previous_in_game_state = None;
                            // Note: bgm_started flag will be reset by start_title_bgm() in update_state()
                            tracing::debug!("children_dirty set at line {}", line!());
                            self.children_dirty = true;
                        } else if cancelled {
                            // User cancelled - go back to pause menu
                            tracing::debug!("Cancelled return to title");
                            self.showing_title_confirm = false;
                            tracing::debug!("children_dirty set at line {}", line!());
                            self.children_dirty = true;
                        }
                    } else {
                        // Check if ESC was pressed to resume
                        if self.pause_pressed {
                            // Restore previous in-game state
                            if let Some(prev_state) = self.previous_in_game_state.take() {
                                *in_game_state = *prev_state;
                                tracing::debug!("children_dirty set at line {}", line!());
                                self.children_dirty = true;
                            }
                        } else {
                            // Check if pause menu has a confirmed action
                            self.update_pause_menu_state();
                        }
                    }
                }
                InGameState::SaveLoadMenu(_save_load_state) => {
                    // Check if SaveLoadMenuElement has a confirmed action
                    let confirmed_action = self.children.iter().find_map(|child| {
                        child
                            .as_any()
                            .downcast_ref::<SaveLoadMenuElement>()
                            .and_then(|menu| menu.confirmed_action())
                    });

                    if let Some(action) = confirmed_action {
                        tracing::debug!("SaveLoadMenu action confirmed: {:?}", action);

                        // Reset the confirmation to prevent repeated processing
                        for child in &mut self.children {
                            if let Some(menu) =
                                child.as_any_mut().downcast_mut::<SaveLoadMenuElement>()
                            {
                                menu.reset_confirmation();
                                break;
                            }
                        }

                        match action {
                            SaveLoadMenuAction::SaveToSlot(slot) => {
                                tracing::debug!("Saving to slot {}", slot);

                                // Perform save operation
                                let save_result = if let Some(runtime) = &self.scenario_runtime {
                                    // Get save data from runtime
                                    let mut save_data = runtime.to_save_data(slot);

                                    // Set timestamp and play time
                                    save_data.timestamp = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    save_data.play_time_secs = self.total_play_time_secs;

                                    // Save to file
                                    match self.save_manager.lock() {
                                        Ok(manager) => manager.save(slot, &save_data),
                                        Err(e) => {
                                            tracing::error!("Failed to lock save_manager: {:?}", e);
                                            Err(narrative_core::EngineError::Other(
                                                "Failed to access save system".to_string(),
                                            ))
                                        }
                                    }
                                } else {
                                    Err(narrative_core::EngineError::Other(
                                        "No scenario runtime available".to_string(),
                                    ))
                                };

                                // Handle result
                                match save_result {
                                    Ok(_) => {
                                        tracing::info!("Successfully saved to slot {}", slot);
                                        // Return to previous state
                                        if let Some(prev_state) = self.previous_in_game_state.take()
                                        {
                                            *in_game_state = *prev_state;
                                            tracing::debug!(
                                                "children_dirty set at line {}",
                                                line!()
                                            );
                                            self.children_dirty = true;
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to save to slot {}: {}", slot, e);
                                        // TODO: Show error dialog to user
                                        // For now, still go back
                                        if let Some(prev_state) = self.previous_in_game_state.take()
                                        {
                                            *in_game_state = *prev_state;
                                            tracing::debug!(
                                                "children_dirty set at line {}",
                                                line!()
                                            );
                                            self.children_dirty = true;
                                        }
                                    }
                                }
                            }
                            SaveLoadMenuAction::LoadFromSlot(slot) => {
                                tracing::debug!("Loading from slot {}", slot);

                                // Perform load operation
                                let load_result = match self.save_manager.lock() {
                                    Ok(manager) => manager.load(slot),
                                    Err(e) => {
                                        tracing::error!("Failed to lock save_manager: {:?}", e);
                                        Err(narrative_core::EngineError::Other(
                                            "SaveManager lock poisoned".to_string(),
                                        ))
                                    }
                                };

                                match load_result {
                                    Ok(save_data) => {
                                        tracing::info!("Successfully loaded from slot {}", slot);

                                        // Update play time
                                        self.total_play_time_secs = save_data.play_time_secs;

                                        // Create or get runtime
                                        let runtime = if let Some(runtime) =
                                            &mut self.scenario_runtime
                                        {
                                            runtime
                                        } else {
                                            // Create new runtime from scenario file (e.g., when loading from title screen)
                                            tracing::debug!(
                                                "Creating new scenario runtime for load"
                                            );
                                            match ScenarioRuntime::from_toml(
                                                &self.config.start_scenario,
                                            ) {
                                                Ok(new_runtime) => {
                                                    self.scenario_runtime = Some(new_runtime);
                                                }
                                                Err(e) => {
                                                    tracing::error!(
                                                        "Failed to create scenario runtime: {}",
                                                        e
                                                    );
                                                    self.app_state = AppState::MainMenu(
                                                        MainMenuState::default(),
                                                    );
                                                    tracing::debug!(
                                                        "children_dirty set at line {}",
                                                        line!()
                                                    );
                                                    self.children_dirty = true;
                                                    return;
                                                }
                                            }
                                            // scenario_runtime is now guaranteed to be Some
                                            if let Some(runtime) = &mut self.scenario_runtime {
                                                runtime
                                            } else {
                                                // This should never happen since we just set it above
                                                tracing::error!(
                                                    "Critical error: scenario_runtime is None after creation"
                                                );
                                                self.app_state =
                                                    AppState::MainMenu(MainMenuState::default());
                                                tracing::debug!(
                                                    "children_dirty set at line {}",
                                                    line!()
                                                );
                                                self.children_dirty = true;
                                                return;
                                            }
                                        };

                                        // Set unlock data for CG tracking
                                        runtime.set_unlock_data(Arc::clone(&self.unlock_data));

                                        // Restore runtime state from save data
                                        match runtime.from_save_data(&save_data) {
                                            Ok(_) => {
                                                tracing::debug!("Runtime state restored");
                                                // Transition to gameplay - use the restored scene/index from runtime
                                                let current_scene = runtime
                                                    .current_scene()
                                                    .cloned()
                                                    .unwrap_or_else(|| {
                                                        narrative_core::SceneId::new("")
                                                    });
                                                let command_index = runtime.command_index();

                                                *in_game_state =
                                                    InGameState::WaitingInput(WaitingInputState {
                                                        scene_id: current_scene,
                                                        command_index,
                                                        auto_wait_elapsed: 0.0,
                                                        skip_mode: false,
                                                    });
                                                tracing::debug!(
                                                    "children_dirty set at line {}",
                                                    line!()
                                                );
                                                self.children_dirty = true;
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                    "Failed to restore runtime state: {}",
                                                    e
                                                );
                                                // TODO: Show error dialog
                                                self.app_state =
                                                    AppState::MainMenu(MainMenuState::default());
                                                tracing::debug!(
                                                    "children_dirty set at line {}",
                                                    line!()
                                                );
                                                self.children_dirty = true;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to load from slot {}: {}", slot, e);
                                        // TODO: Show error dialog to user
                                        // Go back to main menu
                                        self.app_state =
                                            AppState::MainMenu(MainMenuState::default());
                                        tracing::debug!("children_dirty set at line {}", line!());
                                        self.children_dirty = true;
                                    }
                                }
                            }
                            SaveLoadMenuAction::DeleteSlot(slot) => {
                                tracing::debug!("Deleting slot {}", slot);

                                // Perform delete operation
                                let delete_result = match self.save_manager.lock() {
                                    Ok(manager) => manager.delete_slot(slot),
                                    Err(e) => {
                                        tracing::error!("Failed to lock save_manager: {:?}", e);
                                        Err(narrative_core::EngineError::Other(
                                            "SaveManager lock poisoned".to_string(),
                                        ))
                                    }
                                };

                                match delete_result {
                                    Ok(_) => {
                                        tracing::info!("Successfully deleted slot {}", slot);
                                        // Refresh the save/load menu by marking as dirty
                                        // The menu will reload slot information on next rebuild
                                        tracing::debug!("children_dirty set at line {}", line!());
                                        self.children_dirty = true;
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to delete slot {}: {}", slot, e);
                                        // TODO: Show error dialog to user
                                    }
                                }
                            }
                            SaveLoadMenuAction::Back => {
                                // Return to previous state if available, otherwise go to title
                                if let Some(prev_state) = self.previous_in_game_state.take() {
                                    // Restore previous state (PauseMenu, or original game state)
                                    *in_game_state = *prev_state;
                                    tracing::debug!("children_dirty set at line {}", line!());
                                    self.children_dirty = true;
                                } else {
                                    // No previous state (came from title screen), go to main menu
                                    self.app_state = AppState::MainMenu(MainMenuState::default());
                                    tracing::debug!("children_dirty set at line {}", line!());
                                    self.children_dirty = true;
                                }
                            }
                            SaveLoadMenuAction::NextPage
                            | SaveLoadMenuAction::PrevPage
                            | SaveLoadMenuAction::ToggleLayout => {
                                // These are handled internally by SaveLoadMenuElement
                            }
                        }
                    }
                }
                InGameState::Backlog(_backlog) => {
                    // Handle backlog close via 'B' key or Escape
                    if self.backlog_pressed || self.pause_pressed {
                        // Restore previous in-game state
                        if let Some(prev_state) = self.previous_in_game_state.take() {
                            *in_game_state = *prev_state;
                            tracing::debug!("children_dirty set at line {}", line!());
                            self.children_dirty = true;
                        } else {
                            // Fallback: Return to waiting input state if no previous state
                            tracing::warn!(
                                "No previous state to restore from Backlog, falling back to WaitingInput"
                            );
                            if let Some(runtime) = &self.scenario_runtime
                                && let Some(scene_id) = runtime.current_scene()
                            {
                                *in_game_state = InGameState::WaitingInput(WaitingInputState {
                                    scene_id: scene_id.clone(),
                                    command_index: runtime.command_index(),
                                    auto_wait_elapsed: 0.0,
                                    skip_mode: false,
                                });
                                tracing::debug!("children_dirty set at line {}", line!());
                                self.children_dirty = true;
                            }
                        }
                    }

                    // Check if BacklogElement requested close
                    if let Some(backlog_element) = self.children.first()
                        && let Some(backlog) =
                            backlog_element.as_any().downcast_ref::<BacklogElement>()
                        && backlog.is_close_requested()
                    {
                        // Restore previous in-game state
                        if let Some(prev_state) = self.previous_in_game_state.take() {
                            *in_game_state = *prev_state;
                            tracing::debug!("children_dirty set at line {}", line!());
                            self.children_dirty = true;
                        } else {
                            // Fallback: Return to waiting input state if no previous state
                            tracing::warn!(
                                "No previous state to restore from Backlog, falling back to WaitingInput"
                            );
                            if let Some(runtime) = &self.scenario_runtime
                                && let Some(scene_id) = runtime.current_scene()
                            {
                                *in_game_state = InGameState::WaitingInput(WaitingInputState {
                                    scene_id: scene_id.clone(),
                                    command_index: runtime.command_index(),
                                    auto_wait_elapsed: 0.0,
                                    skip_mode: false,
                                });
                                tracing::debug!("children_dirty set at line {}", line!());
                                self.children_dirty = true;
                            }
                        }
                    }
                }
                InGameState::CgGallery(_cg_gallery_state) => {
                    // Check if CgGalleryElement has a confirmed action
                    let confirmed_action = self.children.iter().find_map(|child| {
                        child
                            .as_any()
                            .downcast_ref::<CgGalleryElement>()
                            .and_then(|gallery| gallery.confirmed_action())
                    });

                    if let Some(action) = confirmed_action {
                        // Reset confirmation
                        for child in &mut self.children {
                            if let Some(gallery) =
                                child.as_any_mut().downcast_mut::<CgGalleryElement>()
                            {
                                gallery.reset_confirmation();
                                break;
                            }
                        }

                        match action {
                            CgGalleryAction::Back => {
                                tracing::debug!("Returning to main menu from CG Gallery");
                                self.app_state = AppState::MainMenu(MainMenuState::default());
                                tracing::debug!("children_dirty set at line {}", line!());
                                self.children_dirty = true;
                            }
                            CgGalleryAction::ViewCg(cg_index) => {
                                tracing::debug!("Attempting to view CG at index {}", cg_index);
                                let sorted_cgs = self.cg_registry.get_all_sorted();
                                if let Some(cg) = sorted_cgs.get(cg_index) {
                                    // Check if unlocked
                                    let is_unlocked = self
                                        .unlock_data
                                        .lock()
                                        .map(|data| data.is_cg_unlocked(&cg.id))
                                        .unwrap_or(false);

                                    if is_unlocked {
                                        tracing::debug!("Opening CG viewer for: {}", cg.id);
                                        self.previous_in_game_state =
                                            Some(Box::new(in_game_state.clone()));
                                        *in_game_state = InGameState::CgViewer(
                                            narrative_engine::runtime::CgViewerState::new(
                                                cg.id.clone(),
                                                cg.total_image_count(),
                                            ),
                                        );
                                        tracing::debug!("children_dirty set at line {}", line!());
                                        self.children_dirty = true;
                                    } else {
                                        tracing::warn!("Attempted to view locked CG: {}", cg.id);
                                    }
                                } else {
                                    tracing::warn!("Invalid CG index: {}", cg_index);
                                }
                            }
                        }
                    }
                }
                InGameState::CgViewer(cg_viewer_state) => {
                    // Check if variation changed (need to reload texture)
                    let viewer_variation = self.children.iter().find_map(|child| {
                        child
                            .as_any()
                            .downcast_ref::<CgViewerElement>()
                            .map(|viewer| viewer.get_variation_index())
                    });

                    if let Some(viewer_var) = viewer_variation
                        && viewer_var != cg_viewer_state.variation_index
                    {
                        tracing::debug!(
                            "Variation changed: {} -> {}",
                            cg_viewer_state.variation_index,
                            viewer_var
                        );
                        // Update state and trigger rebuild to load new texture
                        cg_viewer_state.variation_index = viewer_var;
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                    }

                    // Check if CgViewerElement has a confirmed action
                    let confirmed_action = self.children.iter().find_map(|child| {
                        child
                            .as_any()
                            .downcast_ref::<CgViewerElement>()
                            .and_then(|viewer| viewer.confirmed_action())
                    });

                    if let Some(action) = confirmed_action {
                        // Reset confirmation
                        for child in &mut self.children {
                            if let Some(viewer) =
                                child.as_any_mut().downcast_mut::<CgViewerElement>()
                            {
                                viewer.reset_confirmation();
                                break;
                            }
                        }

                        match action {
                            CgViewerAction::Close => {
                                tracing::debug!("Closing CG viewer");
                                if let Some(prev) = self.previous_in_game_state.take() {
                                    *in_game_state = *prev;
                                } else {
                                    // Fallback: Return to CG gallery
                                    tracing::warn!("No previous state, returning to CG Gallery");
                                    let total_cgs = self.cg_registry.total_count();
                                    *in_game_state = InGameState::CgGallery(
                                        narrative_engine::runtime::CgGalleryState::new(total_cgs),
                                    );
                                }
                                tracing::debug!("children_dirty set at line {}", line!());
                                self.children_dirty = true;
                            }
                        }
                    }
                }
            }
        }
    }
}
