//! Command execution and transition logic for GameRootElement

use super::element::GameRootElement;
use narrative_core::config::UserSettings;
use narrative_core::{ScenarioCommand, Speaker};
use narrative_engine::AudioManager;
use narrative_engine::runtime::{
    AppState, ChoiceState, CommandExecutionResult, InGameState, MainMenuState, ScenarioRuntime,
    TypingState, WaitState,
};
use narrative_gui::framework::animation::AnimationContext;
use std::sync::Arc;

impl GameRootElement {
    /// Advance after waiting input state
    pub(super) fn advance_after_waiting_input(&mut self) {
        tracing::debug!("advance_after_waiting_input called");
        let Some(runtime) = self.scenario_runtime.as_mut() else {
            tracing::warn!("No runtime available");
            return;
        };

        // Advance to next command
        if runtime.advance_command() {
            tracing::debug!("Successfully advanced to next command");
            // Successfully advanced, execute new command
            let new_state = {
                let mut audio = self.audio_manager.lock().unwrap_or_else(|e| {
                    tracing::warn!("AudioManager mutex poisoned, recovering: {}", e);
                    e.into_inner()
                });
                Self::execute_and_transition(runtime, &mut audio)
            }; // audio lock is dropped here

            if let Some(new_state) = new_state {
                tracing::debug!(
                    "Transitioning to new state: {:?}",
                    std::mem::discriminant(&new_state)
                );

                // If transitioning to TransitionState, immediately schedule background load
                if matches!(new_state, InGameState::Transition(_)) {
                    self.update_background_if_changed();
                }

                if let Some(in_game_state) = self.app_state.in_game_state_mut() {
                    *in_game_state = new_state;
                    tracing::debug!("children_dirty set at line {}", line!());
                    self.children_dirty = true;
                }
            } else {
                // No next state after command execution
                tracing::warn!("No next state after command execution");
                if runtime.is_ended() {
                    tracing::debug!("Scenario ended");
                    self.app_state = AppState::MainMenu(MainMenuState::default());
                    tracing::debug!("children_dirty set at line {}", line!());
                    self.children_dirty = true;
                } else {
                    // Unexpected: not ended but no next state
                    tracing::error!("Runtime not ended but no next state available");
                    self.app_state = AppState::MainMenu(MainMenuState::default());
                    tracing::debug!("children_dirty set at line {}", line!());
                    self.children_dirty = true;
                }
            }
        } else {
            // At end of scene/scenario
            if runtime.is_ended() {
                tracing::debug!("Scenario ended");
                self.app_state = AppState::MainMenu(MainMenuState::default());
                tracing::debug!("children_dirty set at line {}", line!());
                self.children_dirty = true;
            }
        }
    }

    /// Create InGameState from the current command in the runtime
    pub(super) fn create_state_from_command(runtime: &ScenarioRuntime) -> Option<InGameState> {
        let command = runtime.get_current_command()?;
        let scene_id = runtime.current_scene()?.clone();
        let command_index = runtime.command_index();

        tracing::debug!(
            "create_state_from_command: scene={:?}, command_index={}, command={:?}",
            scene_id,
            command_index,
            std::mem::discriminant(command)
        );

        match command {
            ScenarioCommand::Dialogue { dialogue } => {
                // Convert Speaker enum to Option<String>
                let speaker = match &dialogue.speaker {
                    Speaker::Character(name) => Some(name.clone()),
                    Speaker::Narrator | Speaker::System => None,
                };

                Some(InGameState::Typing(TypingState {
                    scene_id,
                    command_index,
                    speaker,
                    text: Arc::from(dialogue.text.clone()),
                    char_index: 0,
                    elapsed: 0.0,
                    auto_mode: false,
                    skip_mode: false,
                }))
            }

            ScenarioCommand::ShowChoice { choice } => {
                tracing::debug!("ShowChoice command - {} options", choice.options.len());
                Some(InGameState::ShowingChoices(ChoiceState {
                    scene_id,
                    command_index,
                    choices: choice.options.clone(),
                    selected: 0,
                    confirmed: false,
                }))
            }

            ScenarioCommand::Wait { duration } => {
                Some(InGameState::Waiting(WaitState::new(*duration)))
            }

            // Other commands don't create waiting states, they execute immediately
            _ => None,
        }
    }

    /// Execute current command and transition to next state
    pub(super) fn execute_and_transition(
        runtime: &mut ScenarioRuntime,
        audio_manager: &mut AudioManager,
    ) -> Option<InGameState> {
        tracing::debug!("execute_and_transition called");

        // Loop to execute commands until we reach a waiting state
        loop {
            // Handle audio commands before executing
            if let Some(command) = runtime.get_current_command() {
                match command {
                    ScenarioCommand::PlaySe { asset, volume } => {
                        tracing::debug!("Playing SE: {}", asset.path());
                        if let Err(e) = audio_manager.play_se(asset.path(), *volume) {
                            tracing::error!("Failed to play SE '{}': {}", asset.path(), e);
                        }
                    }
                    ScenarioCommand::PlayBgm {
                        asset,
                        volume,
                        fade_in,
                    } => {
                        tracing::debug!("Playing BGM: {}", asset.path());
                        let fade_duration = if *fade_in > 0.0 {
                            Some(*fade_in as f64)
                        } else {
                            None
                        };
                        if let Err(e) =
                            audio_manager.play_bgm(asset.path(), true, fade_duration, *volume)
                        {
                            tracing::error!("Failed to play BGM '{}': {}", asset.path(), e);
                        }
                    }
                    ScenarioCommand::StopBgm { fade_out } => {
                        tracing::debug!("Stopping BGM");
                        let fade_duration = if *fade_out > 0.0 {
                            Some(*fade_out as f64)
                        } else {
                            None
                        };
                        if let Err(e) = audio_manager.stop_bgm(fade_duration) {
                            tracing::error!("Failed to stop BGM: {}", e);
                        }
                    }
                    _ => {}
                }
            }

            // Execute current command
            let result = match runtime.execute_current_command() {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Command execution failed: {}", e);
                    return None;
                }
            };

            tracing::debug!(
                "Command execution result: {:?}",
                std::mem::discriminant(&result)
            );

            match result {
                CommandExecutionResult::Continue => {
                    // Advance to next command
                    if !runtime.advance_command() {
                        // End of scene
                        tracing::warn!("Reached end of scene with no waiting state");
                        return None;
                    }

                    // Try to create state from new command
                    if let Some(state) = Self::create_state_from_command(runtime) {
                        // Add dialogue to backlog when creating Typing state from Dialogue command
                        if let Some(command) = runtime.get_current_command()
                            && let ScenarioCommand::Dialogue { dialogue } = command
                            && let Some(scene_id) = runtime.current_scene()
                        {
                            let command_index = runtime.command_index();
                            runtime.add_to_backlog(
                                scene_id.clone(),
                                command_index,
                                dialogue.speaker.clone(),
                                dialogue.text.clone(),
                            );
                        }
                        return Some(state);
                    }
                    // If no state was created, loop to execute the next command
                    tracing::debug!("No waiting state from command, continuing to next command");
                    continue;
                }

                CommandExecutionResult::SceneChanged {
                    exit_transition,
                    entry_transition,
                } => {
                    // TODO: Handle exit transitions properly
                    if let Some(exit) = exit_transition {
                        tracing::debug!("Exit transition: {:?} ({:.1}s)", exit.kind, exit.duration);
                    }

                    // If there's an entry transition, create a TransitionState
                    if let Some(entry) = entry_transition {
                        tracing::debug!(
                            "Entry transition: {:?} ({:.1}s)",
                            entry.kind,
                            entry.duration
                        );

                        // Get current scene for transition state
                        let to_scene = runtime.current_scene()?.clone();
                        // For now, use the same scene as from_scene (we can improve this later)
                        let from_scene = to_scene.clone();

                        return Some(InGameState::Transition(
                            narrative_engine::runtime::TransitionState {
                                from_scene,
                                to_scene,
                                kind: entry.kind,
                                progress: 0.0,
                                duration: entry.duration,
                            },
                        ));
                    }

                    // No entry transition, scene changed, try to create state from first command of new scene
                    if let Some(state) = Self::create_state_from_command(runtime) {
                        // Add dialogue to backlog when creating Typing state from Dialogue command
                        if let Some(command) = runtime.get_current_command()
                            && let ScenarioCommand::Dialogue { dialogue } = command
                            && let Some(scene_id) = runtime.current_scene()
                        {
                            let command_index = runtime.command_index();
                            runtime.add_to_backlog(
                                scene_id.clone(),
                                command_index,
                                dialogue.speaker.clone(),
                                dialogue.text.clone(),
                            );
                        }
                        return Some(state);
                    }
                    // If no waiting state, continue executing commands
                    tracing::debug!(
                        "SceneChanged but no waiting state from first command, continuing"
                    );
                    continue;
                }

                CommandExecutionResult::ShowChoices(choices) => {
                    let scene_id = runtime.current_scene()?.clone();
                    let command_index = runtime.command_index();

                    return Some(InGameState::ShowingChoices(ChoiceState {
                        scene_id,
                        command_index,
                        choices,
                        selected: 0,
                        confirmed: false,
                    }));
                }

                CommandExecutionResult::Wait(duration) => {
                    return Some(InGameState::Waiting(WaitState::new(duration)));
                }

                CommandExecutionResult::End => {
                    tracing::debug!("Scenario ended");
                    return None;
                }
            }
        }
    }

    /// Get current animation context from settings
    ///
    /// Loads user settings from assets/config/settings.ron and creates an AnimationContext.
    /// Falls back to default context if settings cannot be loaded.
    pub(super) fn animation_context(&self) -> AnimationContext {
        match UserSettings::load("assets/config/settings.ron") {
            Ok(settings) => AnimationContext::from_enabled_and_speed(
                settings.animation.enabled,
                settings.animation.speed,
            ),
            Err(e) => {
                tracing::warn!("Failed to load settings for animation context: {}", e);
                AnimationContext::default()
            }
        }
    }
}
