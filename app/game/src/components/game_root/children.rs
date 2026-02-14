//! UI children building logic for GameRootElement

use super::element::GameRootElement;
use crate::components::{
    BacklogElement, CgGalleryElement, CgViewerElement, CharacterSpriteElement, ChoiceMenuElement,
    ConfirmDialogElement, DialogueBoxElement, PauseMenuElement, QuickMenuElement,
    SaveLoadMenuElement, SettingsMenuElement, TitleScreenElement,
};
use narrative_core::config::DialogueBoxConfig;
use narrative_core::{AssetRef, UnlockData};
use narrative_engine::runtime::{AppState, InGameState};
use std::sync::Arc;

impl GameRootElement {
    /// Rebuild children elements based on current state
    pub(super) fn rebuild_children(&mut self) {
        self.children.clear();

        // Get animation context from settings (loaded once per rebuild)
        let anim_ctx = self.animation_context();

        // NOTE: Debug logging is controlled by RUST_LOG environment variable at runtime
        // Set RUST_LOG=narrative_app=debug to see these logs, or RUST_LOG=info to hide them
        tracing::debug!(
            "Rebuilding children for state: {:?}",
            std::mem::discriminant(&self.app_state)
        );

        match &self.app_state {
            AppState::Loading(_loading) => {
                // TODO: Add loading screen UI (Phase 1.5 or later)
                tracing::debug!("Loading state - no UI");
            }
            AppState::MainMenu(menu) => {
                tracing::debug!("MainMenu state - showing title screen");
                let title_screen =
                    TitleScreenElement::new(menu.has_continue).with_animation_context(anim_ctx);
                self.children.push(Box::new(title_screen));
            }
            AppState::InGame(in_game_state) => {
                tracing::debug!(
                    "InGame state variant: {:?}",
                    std::mem::discriminant(in_game_state)
                );

                // Add character sprites from runtime state
                if let Some(runtime) = &mut self.scenario_runtime {
                    use narrative_core::TransitionKind;

                    // Debug: Log all displayed characters
                    let displayed = runtime.displayed_characters();
                    tracing::debug!("displayed_characters count: {}", displayed.len());
                    for (key, char_info) in displayed.iter() {
                        tracing::debug!(
                            "  Character key='{}', id='{}', sprite='{}', position={:?}, transition={:?}",
                            key,
                            char_info.character_id,
                            char_info.sprite.0,
                            char_info.position,
                            char_info.transition
                        );
                    }

                    // Queue textures for loading if characters changed (using dirty flag)
                    if runtime.displayed_characters_changed() {
                        for char_info in runtime.displayed_characters().values() {
                            if self
                                .character_texture_cache
                                .get(&char_info.sprite)
                                .is_none()
                            {
                                self.pending_character_textures.push((
                                    char_info.character_id.clone(),
                                    char_info.sprite.clone(),
                                ));
                            }
                        }
                    }

                    for char_info in runtime.displayed_characters().values() {
                        // Use window size if available, otherwise use default
                        let (win_width, win_height) =
                            if self.window_size.0 > 0.0 && self.window_size.1 > 0.0 {
                                self.window_size
                            } else {
                                (1280.0, 720.0) // Fallback to default
                            };

                        tracing::debug!(
                            "Creating character '{}' with position: {:?}, window_size: ({}, {})",
                            char_info.character_id,
                            char_info.position,
                            win_width,
                            win_height
                        );

                        let mut sprite = CharacterSpriteElement::new(
                            &char_info.character_id,
                            "",
                            char_info.position,
                        )
                        .with_animation_context(anim_ctx)
                        .with_window_size(win_width, win_height);

                        // Apply sprite offset and scale from character definition
                        if let Some(char_def) = runtime
                            .scenario()
                            .characters
                            .iter()
                            .find(|c| c.id == char_info.character_id)
                        {
                            if let Some((offset_x, offset_y)) = char_def.sprite_offset {
                                sprite = sprite.with_sprite_offset(offset_x, offset_y);
                                tracing::debug!(
                                    "Applied sprite offset ({}, {}) to character '{}'",
                                    offset_x,
                                    offset_y,
                                    char_info.character_id
                                );
                            }
                            if let Some(scale) = char_def.sprite_scale {
                                sprite = sprite.with_sprite_scale(scale);
                                tracing::debug!(
                                    "Applied sprite scale {} to character '{}'",
                                    scale,
                                    char_info.character_id
                                );
                            }
                        }

                        // Apply transitions only if character state changed
                        let character_changed = self
                            .last_seen_characters
                            .get(&char_info.character_id)
                            .map(|(last_sprite, last_pos)| {
                                last_sprite != &char_info.sprite || last_pos != &char_info.position
                            })
                            .unwrap_or(true); // New character, apply transition

                        if character_changed {
                            match char_info.transition.kind {
                                TransitionKind::Fade => {
                                    sprite.fade_in(char_info.transition);
                                }
                                TransitionKind::Crossfade => {
                                    // Crossfade requires texture IDs, handled separately
                                }
                                TransitionKind::Slide(direction) => {
                                    sprite.slide_in(char_info.transition, direction);
                                }
                                _ => {}
                            }
                        }

                        // Set texture from cache or fallback to development texture
                        // TODO(layered-sprites): Add support for layered sprite rendering
                        // Current implementation only supports single texture per character.
                        // For layered sprites, need to render multiple textures with blending.
                        if let Some(handle) = self.character_texture_cache.get(&char_info.sprite) {
                            let texture_id = handle.id();
                            sprite = sprite.with_texture(texture_id);
                            tracing::debug!(
                                "Rendering character '{}' at {:?} with sprite '{}' (texture_id: {})",
                                char_info.character_id,
                                char_info.position,
                                char_info.sprite.0,
                                texture_id
                            );
                        } else if let Some(texture_id) = self.character_texture_id {
                            // Fallback to development texture if available
                            sprite = sprite.with_texture(texture_id);
                            tracing::warn!(
                                "Using fallback texture for character '{}' sprite '{}' (texture_id: {})",
                                char_info.character_id,
                                char_info.sprite.0,
                                texture_id
                            );
                        } else {
                            tracing::warn!(
                                "No texture ID available for character '{}' (sprite: '{}')",
                                char_info.character_id,
                                char_info.sprite.0
                            );
                        }

                        // Apply animation from current dialogue if the character is the speaker
                        if let Some(command) = runtime.get_current_command() {
                            if let narrative_core::ScenarioCommand::Dialogue { dialogue } = command {
                                if let narrative_core::Speaker::Character(speaker_id) = &dialogue.speaker {
                                    if speaker_id == &char_info.character_id {
                                        if let Some(ref animation) = dialogue.animation {
                                            sprite.start_animation(animation.clone());
                                            tracing::info!(
                                                "Started animation for '{}': {:?}",
                                                char_info.character_id,
                                                animation
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        self.children.push(Box::new(sprite));
                    }

                    // Update last seen characters for transition optimization
                    self.last_seen_characters.clear();
                    for (id, char_info) in runtime.displayed_characters() {
                        self.last_seen_characters
                            .insert(id.clone(), (char_info.sprite.clone(), char_info.position));
                    }
                }

                // Inline rebuild logic to avoid borrow checker issues
                match in_game_state {
                    InGameState::Typing(typing) => {
                        tracing::debug!(
                            "Typing state - creating DialogueBox (char_index: {}/{})",
                            typing.char_index,
                            typing.text.chars().count()
                        );
                        // Create dialogue box with typewriter effect
                        // Use default DialogueBoxConfig
                        let mut dialogue_box =
                            DialogueBoxElement::new(DialogueBoxConfig::default())
                                .with_animation_context(anim_ctx);

                        if let Some(speaker) = &typing.speaker {
                            dialogue_box.set_speaker(Some(Arc::from(speaker.as_str())));
                        }

                        dialogue_box.set_text(typing.text.clone());
                        dialogue_box.set_visible_chars(typing.char_index);
                        dialogue_box.set_auto_mode_enabled(self.config.gameplay.auto_mode_enabled);
                        dialogue_box.set_skip_mode_enabled(
                            self.config.gameplay.skip_mode_enabled,
                            self.config.gameplay.skip_mode,
                        );

                        // Add quick menu
                        let mut quick_menu =
                            QuickMenuElement::new().with_animation_context(anim_ctx);
                        quick_menu.set_skip_active(self.config.gameplay.skip_mode_enabled);
                        quick_menu.set_auto_active(self.config.gameplay.auto_mode_enabled);

                        // Only add UI elements if not hidden
                        if !self.ui_hidden {
                            self.children.push(Box::new(dialogue_box));
                            self.children.push(Box::new(quick_menu));
                        }
                    }
                    InGameState::WaitingInput(_waiting) => {
                        tracing::debug!(
                            "WaitingInput state - showing full dialogue with click indicator"
                        );
                        // Show dialogue box with full text and click indicator
                        if let Some(runtime) = &self.scenario_runtime
                            && let Some(command) = runtime.get_current_command()
                            && let narrative_core::ScenarioCommand::Dialogue { dialogue } = command
                        {
                            // Use default DialogueBoxConfig
                            let mut dialogue_box =
                                DialogueBoxElement::new(DialogueBoxConfig::default())
                                    .with_animation_context(anim_ctx);

                            if let narrative_core::Speaker::Character(name) = &dialogue.speaker {
                                dialogue_box.set_speaker(Some(Arc::from(name.as_str())));
                            }

                            dialogue_box.set_text(Arc::from(dialogue.text.clone()));
                            dialogue_box.set_visible_chars(dialogue.text.chars().count());
                            dialogue_box.set_text_complete(true);
                            dialogue_box
                                .set_auto_mode_enabled(self.config.gameplay.auto_mode_enabled);
                            dialogue_box.set_skip_mode_enabled(
                                self.config.gameplay.skip_mode_enabled,
                                self.config.gameplay.skip_mode,
                            );

                            // Add quick menu
                            let mut quick_menu =
                                QuickMenuElement::new().with_animation_context(anim_ctx);
                            quick_menu.set_skip_active(self.config.gameplay.skip_mode_enabled);
                            quick_menu.set_auto_active(self.config.gameplay.auto_mode_enabled);

                            // Only add UI elements if not hidden
                            if !self.ui_hidden {
                                self.children.push(Box::new(dialogue_box));
                                self.children.push(Box::new(quick_menu));
                            }
                        }
                    }
                    InGameState::ShowingChoices(choice_state) => {
                        tracing::debug!(
                            "ShowingChoices state - {} choices, selected: {}",
                            choice_state.choices.len(),
                            choice_state.selected
                        );
                        for (i, choice) in choice_state.choices.iter().enumerate() {
                            tracing::debug!("  Choice {}: {}", i, choice.text);
                        }
                        // Create choice menu with current choices
                        let mut choice_menu = ChoiceMenuElement::new(
                            choice_state
                                .choices
                                .iter()
                                .map(|s| s.text.as_str())
                                .collect(),
                        )
                        .with_animation_context(anim_ctx);
                        choice_menu.set_selected_index(choice_state.selected);
                        self.children.push(Box::new(choice_menu));
                    }
                    InGameState::Transition(_transition) => {
                        // TODO: Add transition effects (Phase 1.5 or later)
                    }
                    InGameState::PlayingEffect(_effect) => {
                        // TODO: Add visual effects (Phase 1.5 or later)
                    }
                    InGameState::Waiting(_wait) => {
                        // Wait state typically doesn't show UI
                    }
                    InGameState::PauseMenu(_pause) => {
                        // If showing confirmation dialog, only show the dialog
                        if self.showing_title_confirm {
                            tracing::debug!("Showing confirmation dialog for return to title");
                            let confirm_dialog = ConfirmDialogElement::new(
                                "Return to title screen? Unsaved progress will be lost.",
                            )
                            .with_animation_context(anim_ctx);
                            self.children.push(Box::new(confirm_dialog));
                        } else {
                            // Show pause menu normally
                            tracing::debug!("PauseMenu state - showing pause menu");
                            let pause_menu =
                                PauseMenuElement::new().with_animation_context(anim_ctx);
                            self.children.push(Box::new(pause_menu));
                        }
                    }
                    InGameState::SaveLoadMenu(save_load_state) => {
                        // Show save/load menu
                        tracing::debug!(
                            "SaveLoadMenu state - showing save/load menu (save_mode: {})",
                            save_load_state.is_save_mode
                        );
                        let save_load_menu = SaveLoadMenuElement::new(
                            Arc::clone(&self.save_manager),
                            save_load_state.is_save_mode,
                            save_load_state.layout_mode,
                        )
                        .with_animation_context(anim_ctx);
                        self.children.push(Box::new(save_load_menu));
                    }
                    InGameState::Backlog(_backlog) => {
                        // Show backlog UI
                        if let Some(runtime) = &self.scenario_runtime {
                            // Get backlog entries (newest first)
                            let entries: Vec<_> =
                                runtime.backlog().entries_reversed().cloned().collect();

                            let backlog_element =
                                BacklogElement::new(entries).with_animation_context(anim_ctx);
                            self.children.push(Box::new(backlog_element));
                        }
                    }
                    InGameState::CgGallery(cg_gallery_state) => {
                        // Create CG gallery UI element
                        tracing::debug!("CgGallery state - creating CG gallery");
                        let unlock_data_arc = self
                            .unlock_data
                            .lock()
                            .map(|data| Arc::new((*data).clone()))
                            .unwrap_or_else(|_| Arc::new(UnlockData::new()));

                        let gallery = CgGalleryElement::new(
                            cg_gallery_state.clone(),
                            Arc::clone(&self.cg_registry),
                            unlock_data_arc,
                            self.cg_thumbnail_cache.clone(),
                        )
                        .with_animation_context(anim_ctx);

                        self.children.push(Box::new(gallery));
                    }
                    InGameState::CgViewer(cg_viewer_state) => {
                        // Create CG viewer UI element
                        tracing::debug!("CgViewer state - creating CG viewer");

                        // Get texture ID and size for the current CG (will be loaded in next frame if not cached)
                        let (cg_texture_id, cg_texture_size) = if let Some(cg_meta) =
                            self.cg_registry.get(&cg_viewer_state.cg_id)
                        {
                            // Get the appropriate asset path based on current variation
                            let asset_path = if cg_viewer_state.variation_index == 0 {
                                &cg_meta.asset_path
                            } else {
                                cg_meta
                                    .variations
                                    .get(cg_viewer_state.variation_index.saturating_sub(1))
                                    .map(|v| &v.asset_path)
                                    .unwrap_or(&cg_meta.asset_path)
                            };

                            let asset_ref = AssetRef::new(asset_path.to_string());

                            // Check if texture is already cached
                            if let Some(&(texture_id, size)) = self.cg_texture_cache.get(&asset_ref)
                            {
                                (Some(texture_id), Some(size))
                            } else {
                                // Mark for loading in next frame
                                self.pending_cg = Some(asset_ref);
                                (None, None)
                            }
                        } else {
                            tracing::warn!("CG not found in registry: {}", cg_viewer_state.cg_id);
                            (None, None)
                        };

                        let viewer = CgViewerElement::new(
                            cg_viewer_state.clone(),
                            Arc::clone(&self.cg_registry),
                            cg_texture_id,
                            cg_texture_size,
                        )
                        .with_animation_context(anim_ctx);

                        self.children.push(Box::new(viewer));
                    }
                }
            }
            AppState::Settings(_settings) => {
                tracing::debug!("Settings state - creating settings menu");

                // Load user settings from RON file, or create from current config
                let user_settings =
                    narrative_core::config::UserSettings::load("assets/config/settings.ron")
                        .unwrap_or_else(|e| {
                            tracing::debug!("Failed to load settings.ron, using defaults: {}", e);
                            // Create default settings from current engine config
                            let mut settings = narrative_core::config::UserSettings::default();
                            settings.audio.master_volume = self.config.audio.master_volume;
                            settings.audio.bgm_volume = self.config.audio.music_volume;
                            settings.audio.se_volume = self.config.audio.sound_volume;
                            settings.audio.voice_volume = self.config.audio.voice_volume;
                            settings.display.fullscreen = self.config.window.fullscreen;
                            settings.display.resolution =
                                (self.config.window.width, self.config.window.height);
                            settings
                        });

                let settings_menu =
                    SettingsMenuElement::new(user_settings, Arc::clone(&self.audio_manager))
                        .with_animation_context(anim_ctx);

                self.children.push(Box::new(settings_menu));
            }
        }
    }
}
