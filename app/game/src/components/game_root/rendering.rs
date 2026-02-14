//! Rendering logic for GameRootElement (Element trait implementation)

use super::element::GameRootElement;
use crate::components::SettingsMenuElement;
use narrative_engine::runtime::{AppState, InGameState};
use narrative_gui::framework::element::{
    Element, ElementId, LayoutContext, PaintContext, WindowOperation,
};
use narrative_gui::framework::layout::Bounds;
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

impl GameRootElement {
    /// Render dissolve transition overlay effect
    ///
    /// Creates a pixelated block pattern that masks the new background progressively
    fn render_dissolve_overlay(
        &self,
        cx: &mut narrative_gui::framework::element::PaintContext,
        progress: f32,
    ) {
        // Grid size for dissolve blocks (larger = chunkier effect)
        const BLOCK_SIZE: f32 = 20.0;

        let width = cx.bounds.size.width;
        let height = cx.bounds.size.height;

        // Calculate number of blocks
        let cols = (width / BLOCK_SIZE).ceil() as i32;
        let rows = (height / BLOCK_SIZE).ceil() as i32;

        // Draw old background blocks that haven't dissolved yet
        if let Some(old_bg_id) = self.previous_background_texture_id {
            // Use a pseudo-random but deterministic pattern
            // We want the same blocks to dissolve in the same order
            for row in 0..rows {
                for col in 0..cols {
                    // Create a deterministic "random" value based on position
                    // Using a simple hash-like function for determinism
                    let seed = ((row * 73) + (col * 151)) % 256;
                    let threshold = (seed as f32) / 256.0;

                    // If this block's threshold is greater than progress, draw old background block
                    // (i.e., mask the new background to reveal old background underneath)
                    if threshold > progress {
                        let block_x = cx.bounds.origin.x + (col as f32 * BLOCK_SIZE);
                        let block_y = cx.bounds.origin.y + (row as f32 * BLOCK_SIZE);

                        let block_bounds = narrative_gui::Bounds {
                            origin: narrative_gui::Point::new(block_x, block_y),
                            size: narrative_gui::Size::new(BLOCK_SIZE, BLOCK_SIZE),
                        };

                        // Draw old background texture for this block
                        cx.draw_texture(old_bg_id, block_bounds, 1.0);
                    }
                }
            }
        }
    }
    /// Calculate bounds that fit the given texture size within the container bounds
    /// while preserving aspect ratio. The result is centered and letterboxed/pillarboxed as needed.
    fn calculate_aspect_ratio_fit(
        &self,
        container: narrative_gui::Bounds,
        texture_width: f32,
        texture_height: f32,
    ) -> narrative_gui::Bounds {
        let container_width = container.size.width;
        let container_height = container.size.height;

        // Guard against zero or invalid dimensions
        if container_width <= 0.0
            || container_height <= 0.0
            || texture_width <= 0.0
            || texture_height <= 0.0
        {
            tracing::warn!(
                "Invalid dimensions for aspect ratio calculation: container=({}, {}), texture=({}, {}). Returning original bounds.",
                container_width,
                container_height,
                texture_width,
                texture_height
            );
            return container;
        }

        // Calculate aspect ratios
        let container_aspect = container_width / container_height;
        let texture_aspect = texture_width / texture_height;

        // Calculate fitted size
        let (fitted_width, fitted_height) = if texture_aspect > container_aspect {
            // Texture is wider - fit to width
            (container_width, container_width / texture_aspect)
        } else {
            // Texture is taller - fit to height
            (container_height * texture_aspect, container_height)
        };

        // Center the fitted bounds
        let x = container.origin.x + (container_width - fitted_width) / 2.0;
        let y = container.origin.y + (container_height - fitted_height) / 2.0;

        narrative_gui::Bounds {
            origin: narrative_gui::Point::new(x, y),
            size: narrative_gui::Size::new(fitted_width, fitted_height),
        }
    }
}

impl Element for GameRootElement {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout_node(&self) -> Option<NodeId> {
        self.layout_node
    }

    fn set_layout_node(&mut self, node: NodeId) {
        self.layout_node = Some(node);
    }

    fn layout(&mut self, cx: &mut LayoutContext) -> taffy::Style {
        use taffy::prelude::*;

        // Update window size from available size (if valid)
        if cx.available_size.width > 0.0 && cx.available_size.height > 0.0 {
            self.window_size = (cx.available_size.width, cx.available_size.height);
            tracing::debug!(
                "GameRoot layout: updated window_size = ({}, {})",
                self.window_size.0,
                self.window_size.1
            );
        } else {
            tracing::debug!(
                "GameRoot layout: available_size is (0, 0), keeping current window_size = ({}, {})",
                self.window_size.0,
                self.window_size.1
            );
        }

        // Fill the entire window
        taffy::Style {
            size: taffy::geometry::Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        use narrative_core::types::transition::{SlideDirection, TransitionKind, WipeDirection};

        // Check if we're in a special transition that needs custom rendering
        let transition_kind =
            if let AppState::InGame(InGameState::Transition(transition)) = &self.app_state {
                Some((transition.kind, transition.progress_ratio()))
            } else {
                None
            };

        match transition_kind {
            // Crossfade: blend old and new backgrounds and CGs
            Some((TransitionKind::Crossfade, progress)) => {
                // Draw old background at full opacity
                if let Some(old_bg_id) = self.previous_background_texture_id {
                    cx.draw_texture(old_bg_id, cx.bounds, 1.0);
                }

                // Draw new background on top with increasing opacity
                if let Some(new_bg_id) = self.current_background_texture_id {
                    cx.draw_texture(new_bg_id, cx.bounds, progress);
                }

                // Draw old CG at full opacity (with aspect ratio preserved)
                if let Some(old_cg_id) = self.previous_cg_texture_id {
                    if let Some((width, height)) = self.previous_cg_texture_size {
                        let cg_bounds =
                            self.calculate_aspect_ratio_fit(cx.bounds, width as f32, height as f32);
                        cx.draw_texture(old_cg_id, cg_bounds, 1.0);
                    } else {
                        cx.draw_texture(old_cg_id, cx.bounds, 1.0);
                    }
                }

                // Draw new CG on top with increasing opacity (with aspect ratio preserved)
                if let Some(new_cg_id) = self.current_cg_texture_id {
                    if let Some((width, height)) = self.current_cg_texture_size {
                        let cg_bounds =
                            self.calculate_aspect_ratio_fit(cx.bounds, width as f32, height as f32);
                        cx.draw_texture(new_cg_id, cg_bounds, progress);
                    } else {
                        cx.draw_texture(new_cg_id, cx.bounds, progress);
                    }
                }
            }

            // Slide transitions: move backgrounds in/out
            Some((TransitionKind::Slide(direction), progress)) => {
                let width = cx.bounds.size.width;
                let height = cx.bounds.size.height;

                // Calculate positions based on slide direction
                let (old_offset, new_offset) = match direction {
                    SlideDirection::Left => (
                        narrative_gui::Point::new(-width * progress, 0.0),
                        narrative_gui::Point::new(width * (1.0 - progress), 0.0),
                    ),
                    SlideDirection::Right => (
                        narrative_gui::Point::new(width * progress, 0.0),
                        narrative_gui::Point::new(-width * (1.0 - progress), 0.0),
                    ),
                    SlideDirection::Up => (
                        narrative_gui::Point::new(0.0, -height * progress),
                        narrative_gui::Point::new(0.0, height * (1.0 - progress)),
                    ),
                    SlideDirection::Down => (
                        narrative_gui::Point::new(0.0, height * progress),
                        narrative_gui::Point::new(0.0, -height * (1.0 - progress)),
                    ),
                };

                // Draw old background sliding out
                if let Some(old_bg_id) = self.previous_background_texture_id {
                    let old_bounds = narrative_gui::Bounds {
                        origin: narrative_gui::Point::new(
                            cx.bounds.origin.x + old_offset.x,
                            cx.bounds.origin.y + old_offset.y,
                        ),
                        size: cx.bounds.size,
                    };
                    cx.draw_texture(old_bg_id, old_bounds, 1.0);
                }

                // Draw new background sliding in
                if let Some(new_bg_id) = self.current_background_texture_id {
                    let new_bounds = narrative_gui::Bounds {
                        origin: narrative_gui::Point::new(
                            cx.bounds.origin.x + new_offset.x,
                            cx.bounds.origin.y + new_offset.y,
                        ),
                        size: cx.bounds.size,
                    };
                    cx.draw_texture(new_bg_id, new_bounds, 1.0);
                }

                // Draw old CG sliding out (with aspect ratio preserved)
                if let Some(old_cg_id) = self.previous_cg_texture_id {
                    if let Some((width, height)) = self.previous_cg_texture_size {
                        let cg_fitted_bounds =
                            self.calculate_aspect_ratio_fit(cx.bounds, width as f32, height as f32);
                        let old_cg_bounds = narrative_gui::Bounds {
                            origin: narrative_gui::Point::new(
                                cg_fitted_bounds.origin.x + old_offset.x,
                                cg_fitted_bounds.origin.y + old_offset.y,
                            ),
                            size: cg_fitted_bounds.size,
                        };
                        cx.draw_texture(old_cg_id, old_cg_bounds, 1.0);
                    } else {
                        let old_cg_bounds = narrative_gui::Bounds {
                            origin: narrative_gui::Point::new(
                                cx.bounds.origin.x + old_offset.x,
                                cx.bounds.origin.y + old_offset.y,
                            ),
                            size: cx.bounds.size,
                        };
                        cx.draw_texture(old_cg_id, old_cg_bounds, 1.0);
                    }
                }

                // Draw new CG sliding in (with aspect ratio preserved)
                if let Some(new_cg_id) = self.current_cg_texture_id {
                    if let Some((width, height)) = self.current_cg_texture_size {
                        let cg_fitted_bounds =
                            self.calculate_aspect_ratio_fit(cx.bounds, width as f32, height as f32);
                        let new_cg_bounds = narrative_gui::Bounds {
                            origin: narrative_gui::Point::new(
                                cg_fitted_bounds.origin.x + new_offset.x,
                                cg_fitted_bounds.origin.y + new_offset.y,
                            ),
                            size: cg_fitted_bounds.size,
                        };
                        cx.draw_texture(new_cg_id, new_cg_bounds, 1.0);
                    } else {
                        let new_cg_bounds = narrative_gui::Bounds {
                            origin: narrative_gui::Point::new(
                                cx.bounds.origin.x + new_offset.x,
                                cx.bounds.origin.y + new_offset.y,
                            ),
                            size: cx.bounds.size,
                        };
                        cx.draw_texture(new_cg_id, new_cg_bounds, 1.0);
                    }
                }
            }

            // Wipe transitions: reveal new background progressively
            Some((TransitionKind::Wipe(direction), progress)) => {
                let width = cx.bounds.size.width;
                let height = cx.bounds.size.height;

                // Draw old background at full size
                if let Some(old_bg_id) = self.previous_background_texture_id {
                    cx.draw_texture(old_bg_id, cx.bounds, 1.0);
                }

                // Calculate new background bounds based on wipe direction
                let new_bounds = match direction {
                    WipeDirection::Left => narrative_gui::Bounds {
                        origin: narrative_gui::Point::new(
                            cx.bounds.origin.x + width * (1.0 - progress),
                            cx.bounds.origin.y,
                        ),
                        size: narrative_gui::Size::new(width * progress, height),
                    },
                    WipeDirection::Right => narrative_gui::Bounds {
                        origin: cx.bounds.origin,
                        size: narrative_gui::Size::new(width * progress, height),
                    },
                    WipeDirection::Up => narrative_gui::Bounds {
                        origin: narrative_gui::Point::new(
                            cx.bounds.origin.x,
                            cx.bounds.origin.y + height * (1.0 - progress),
                        ),
                        size: narrative_gui::Size::new(width, height * progress),
                    },
                    WipeDirection::Down => narrative_gui::Bounds {
                        origin: cx.bounds.origin,
                        size: narrative_gui::Size::new(width, height * progress),
                    },
                };

                // Draw new background with wipe effect
                if let Some(new_bg_id) = self.current_background_texture_id {
                    cx.draw_texture(new_bg_id, new_bounds, 1.0);
                }

                // Draw old CG at full opacity (with aspect ratio preserved)
                // Note: CG uses crossfade during wipe transitions since partial texture
                // rendering with aspect ratio preservation is complex
                if let Some(old_cg_id) = self.previous_cg_texture_id {
                    if let Some((width, height)) = self.previous_cg_texture_size {
                        let cg_bounds =
                            self.calculate_aspect_ratio_fit(cx.bounds, width as f32, height as f32);
                        cx.draw_texture(old_cg_id, cg_bounds, 1.0 - progress);
                    } else {
                        cx.draw_texture(old_cg_id, cx.bounds, 1.0 - progress);
                    }
                }

                // Draw new CG with increasing opacity (with aspect ratio preserved)
                if let Some(new_cg_id) = self.current_cg_texture_id {
                    if let Some((width, height)) = self.current_cg_texture_size {
                        let cg_bounds =
                            self.calculate_aspect_ratio_fit(cx.bounds, width as f32, height as f32);
                        cx.draw_texture(new_cg_id, cg_bounds, progress);
                    } else {
                        cx.draw_texture(new_cg_id, cx.bounds, progress);
                    }
                }
            }

            // Dissolve transition: draw both backgrounds, mask with blocks in overlay
            Some((TransitionKind::Dissolve, _progress)) => {
                // Draw old background first
                if let Some(old_bg_id) = self.previous_background_texture_id {
                    cx.draw_texture(old_bg_id, cx.bounds, 1.0);
                }

                // Draw new background on top
                if let Some(new_bg_id) = self.current_background_texture_id {
                    cx.draw_texture(new_bg_id, cx.bounds, 1.0);
                }

                // Draw old CG first (with aspect ratio preserved)
                if let Some(old_cg_id) = self.previous_cg_texture_id {
                    if let Some((width, height)) = self.previous_cg_texture_size {
                        let cg_bounds =
                            self.calculate_aspect_ratio_fit(cx.bounds, width as f32, height as f32);
                        cx.draw_texture(old_cg_id, cg_bounds, 1.0);
                    } else {
                        cx.draw_texture(old_cg_id, cx.bounds, 1.0);
                    }
                }

                // Draw new CG on top (with aspect ratio preserved)
                if let Some(new_cg_id) = self.current_cg_texture_id {
                    if let Some((width, height)) = self.current_cg_texture_size {
                        let cg_bounds =
                            self.calculate_aspect_ratio_fit(cx.bounds, width as f32, height as f32);
                        cx.draw_texture(new_cg_id, cg_bounds, 1.0);
                    } else {
                        cx.draw_texture(new_cg_id, cx.bounds, 1.0);
                    }
                }

                // The dissolve pattern mask will be applied in paint_overlay()
            }

            // Normal rendering or other transition types
            _ => {
                // Normal rendering: Draw current background texture if loaded, otherwise use solid color
                // current_background_texture_id is dynamically updated when background changes
                if let Some(bg_texture_id) = self.current_background_texture_id {
                    // Draw background image to fill the entire window
                    cx.draw_texture(bg_texture_id, cx.bounds, 1.0);
                } else {
                    // Fallback: Draw background with a visible color
                    // Shown when: (1) background not loaded, (2) HideBackground command, (3) texture load failed
                    let bg_color = narrative_gui::Color::new(0.1, 0.15, 0.2, 1.0); // Dark blue-gray
                    cx.fill_rect(cx.bounds, bg_color);
                }
            }
        }

        // Draw CG (event graphics) if present
        // CG is drawn above background but below characters
        // Aspect ratio is preserved by fitting the texture within the window bounds
        if let Some(cg_texture_id) = self.current_cg_texture_id {
            if let Some((width, height)) = self.current_cg_texture_size {
                let cg_bounds =
                    self.calculate_aspect_ratio_fit(cx.bounds, width as f32, height as f32);
                cx.draw_texture(cg_texture_id, cg_bounds, 1.0);
            } else {
                // Fallback: draw fullscreen if size is unknown
                cx.draw_texture(cg_texture_id, cx.bounds, 1.0);
            }
        }

        // Character sprites are now managed by CharacterSpriteElement children
        // (removed fixed character rendering)

        // Draw debug visual indicators (only in debug builds)
        // NOTE: These indicators are always visible in debug builds to help with development.
        // For a cleaner view during testing, use --release build or implement a runtime toggle.
        // TODO: Consider adding a feature flag (e.g., "dev-ui-debug") or runtime toggle
        // to disable debug overlays without switching to release build.
        #[cfg(debug_assertions)]
        {
            use narrative_gui::Size;

            // Draw colored indicator based on state
            let (indicator_color, y_pos) = match &self.app_state {
                AppState::Loading(_) => (narrative_gui::Color::new(1.0, 1.0, 0.0, 1.0), 100.0), // Yellow
                AppState::MainMenu(_) => (narrative_gui::Color::new(0.0, 1.0, 1.0, 1.0), 200.0), // Cyan
                AppState::InGame(state) => match state {
                    InGameState::Typing(_) => {
                        (narrative_gui::Color::new(0.0, 1.0, 0.0, 1.0), 300.0)
                    } // Green
                    InGameState::WaitingInput(_) => {
                        (narrative_gui::Color::new(1.0, 0.5, 0.0, 1.0), 400.0)
                    } // Orange
                    InGameState::ShowingChoices(_) => {
                        (narrative_gui::Color::new(1.0, 0.0, 1.0, 1.0), 500.0)
                    } // Magenta
                    _ => (narrative_gui::Color::new(1.0, 1.0, 1.0, 1.0), 350.0), // White
                },
                _ => (narrative_gui::Color::new(0.5, 0.5, 0.5, 1.0), 250.0), // Gray
            };

            let indicator_bounds = narrative_gui::Bounds {
                origin: narrative_gui::Point::new(50.0, y_pos),
                size: Size::new(200.0, 50.0),
            };
            cx.fill_rect(indicator_bounds, indicator_color);

            // Draw a second indicator for children count
            let children_indicator = narrative_gui::Bounds {
                origin: narrative_gui::Point::new(300.0, 100.0),
                size: Size::new(50.0 * self.children.len() as f32, 50.0),
            };
            let children_color = narrative_gui::Color::new(0.0, 0.8, 0.8, 1.0);
            cx.fill_rect(children_indicator, children_color);

            // Try to draw text (might not work if text rendering isn't ready)
            let debug_text = format!("State: {:?}", std::mem::discriminant(&self.app_state));
            let debug_color = narrative_gui::Color::new(1.0, 1.0, 1.0, 1.0);
            cx.draw_text(
                &debug_text,
                narrative_gui::Point::new(50.0, 50.0),
                debug_color,
                24.0,
            );
        }
    }

    fn handle_event(
        &mut self,
        event: &narrative_gui::framework::input::InputEvent,
        bounds: Bounds,
    ) -> bool {
        self.handle_event_impl(event, bounds)
    }

    fn children(&self) -> &[Box<dyn Element>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Element>] {
        &mut self.children
    }

    fn tick(&mut self, delta: Duration) -> bool {
        // Track if we need to repaint/relayout
        let mut needs_update = false;

        // Use provided delta or fall back to assumed frame time
        let frame_time = if delta.as_millis() > 0 {
            delta.as_secs_f32()
        } else {
            Self::FRAME_TIME
        };

        // Track play time when in-game (excluding menus)
        if matches!(
            self.app_state,
            AppState::InGame(
                InGameState::Typing(_)
                    | InGameState::WaitingInput(_)
                    | InGameState::ShowingChoices(_)
                    | InGameState::Transition(_)
                    | InGameState::PlayingEffect(_)
                    | InGameState::Waiting(_)
            )
        ) {
            // Accumulate fractional seconds for accurate time tracking
            self.play_time_accumulator += frame_time;
            if self.play_time_accumulator >= 1.0 {
                let whole_seconds = self.play_time_accumulator as u64;
                self.total_play_time_secs = self.total_play_time_secs.saturating_add(whole_seconds);
                self.play_time_accumulator -= whole_seconds as f32;
            }
        }

        // Update game state
        self.update_state(frame_time);

        // Detect background changes (InGame state only)
        if matches!(self.app_state, AppState::InGame(_)) {
            let bg_changed = self.update_background_if_changed();
            if bg_changed {
                tracing::debug!("tick(): Background changed, setting children_dirty=true");
                tracing::debug!("children_dirty set at line {}", line!());
                self.children_dirty = true;
                needs_update = true;
            }
        }

        // Detect CG changes (InGame state only)
        if matches!(self.app_state, AppState::InGame(_)) {
            let cg_changed = self.update_cg_if_changed();
            if cg_changed {
                tracing::debug!("tick(): CG changed, setting children_dirty=true");
                tracing::debug!("children_dirty set at line {}", line!());
                self.children_dirty = true;
                needs_update = true;
            }
        }

        // Rebuild children only if state changed
        if self.children_dirty {
            tracing::debug!("tick(): Rebuilding children (children_dirty=true)");
            self.rebuild_children();
            self.children_dirty = false;
            needs_update = true; // Children changed, need relayout
        } else {
            tracing::trace!(
                "tick(): Not rebuilding children (children_dirty=false), children count: {}",
                self.children.len()
            );
        }

        // Check if any child needs update (e.g., typewriter effect, animations)
        for child in &mut self.children {
            if child.tick(delta) {
                needs_update = true;
            }
        }

        // Handle settings menu interactions
        if matches!(self.app_state, AppState::Settings(_)) {
            // Find settings menu in children
            for child in &mut self.children {
                if let Some(settings_menu) =
                    child.as_any_mut().downcast_mut::<SettingsMenuElement>()
                {
                    // Check if settings changed and save them
                    if let Some(user_settings) = settings_menu.take_settings_if_changed() {
                        tracing::debug!(
                            "Settings changed, saving: text_speed = {:?}, fullscreen = {}",
                            user_settings.text.speed,
                            user_settings.display.fullscreen
                        );

                        // Save user settings to RON file
                        match user_settings.save("assets/config/settings.ron") {
                            Ok(_) => {
                                tracing::info!(
                                    "Settings saved successfully to assets/config/settings.ron"
                                );
                            }
                            Err(e) => {
                                tracing::error!("Failed to save settings: {}", e);
                            }
                        }

                        // Update engine config from user settings (for compatibility)
                        self.config.audio.master_volume = user_settings.audio.master_volume;
                        self.config.audio.music_volume = user_settings.audio.bgm_volume;
                        self.config.audio.sound_volume = user_settings.audio.se_volume;
                        self.config.audio.voice_volume = user_settings.audio.voice_volume;
                        self.config.window.fullscreen = user_settings.display.fullscreen;

                        needs_update = true;
                    }

                    // Check for window operations (e.g., resolution change)
                    let ops = settings_menu.take_window_operations();
                    if !ops.is_empty() {
                        tracing::debug!("Settings menu requested {} window operations", ops.len());
                        self.window_operations.extend(ops);
                    }

                    // Check if back button was pressed
                    if settings_menu.take_back_pressed() {
                        // Restore previous state like F1/ESC do
                        if let Some(prev_state) = self.previous_app_state.take() {
                            tracing::debug!(
                                "Settings back button pressed, restoring previous state"
                            );
                            self.app_state = *prev_state;
                        } else {
                            tracing::warn!("No previous state to restore, going to main menu");
                            self.app_state = AppState::MainMenu(
                                narrative_engine::runtime::MainMenuState::default(),
                            );
                        }
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true;
                        needs_update = true;
                    }
                }
            }
        }

        // Reset frame-specific input flags
        if self.clicked_last_frame {
            tracing::trace!("Resetting clicked_last_frame");
        }
        self.clicked_last_frame = false;
        self.pause_pressed = false;
        self.auto_mode_toggle_pressed = false;
        self.skip_mode_toggle_pressed = false;
        self.backlog_pressed = false;

        // Only repaint/relayout if something actually changed
        needs_update
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn take_window_operations(&mut self) -> Vec<WindowOperation> {
        std::mem::take(&mut self.window_operations)
    }

    fn paint_overlay(&self, cx: &mut PaintContext) {
        // Render transition overlay if in Transition state
        if let AppState::InGame(InGameState::Transition(transition)) = &self.app_state {
            use narrative_core::types::TransitionKind;

            // Calculate transition progress ratio (0.0 to 1.0)
            let progress = transition.progress_ratio();

            // Render based on transition kind
            match transition.kind {
                TransitionKind::Fade => {
                    // Fade to/from black
                    // For fade-in: start with alpha=1.0 (fully black), end with alpha=0.0 (transparent)
                    // This creates a fade-in effect from black
                    let alpha = 1.0 - progress;
                    let fade_color = narrative_gui::Color::new(0.0, 0.0, 0.0, alpha);
                    cx.fill_rect(cx.bounds, fade_color);
                }
                TransitionKind::FadeWhite => {
                    // Fade to/from white
                    // For fade-in: start with alpha=1.0 (fully white), end with alpha=0.0 (transparent)
                    // This creates a fade-in effect from white
                    let alpha = 1.0 - progress;
                    let fade_color = narrative_gui::Color::new(1.0, 1.0, 1.0, alpha);
                    cx.fill_rect(cx.bounds, fade_color);
                }
                TransitionKind::Crossfade | TransitionKind::Slide(_) | TransitionKind::Wipe(_) => {
                    // These transitions are handled entirely in paint()
                    // No overlay needed
                }
                TransitionKind::Dissolve => {
                    // Dissolve effect: create a pixelated/block transition effect
                    // Show old background through a decreasing grid pattern
                    self.render_dissolve_overlay(cx, progress);
                }
                TransitionKind::None => {
                    // No transition
                }
            }
        }
    }

    fn load_pending_background_texture(
        &mut self,
        renderer: &mut narrative_gui::framework::renderer::Renderer,
    ) -> bool {
        // Delegate to the method in textures.rs
        // Note: This calls the inherent impl method, not recursively
        GameRootElement::load_pending_background_texture(self, renderer)
    }
}
