//! Input handling for GameRootElement (handle_event implementation)

use super::element::GameRootElement;
use crate::components::QuickMenuElement;
use narrative_engine::runtime::{AppState, InGameState};
use narrative_gui::framework::element::Element;
use narrative_gui::framework::input::{InputEvent, KeyCode, MouseButton};
use narrative_gui::framework::layout::Bounds;

impl GameRootElement {
    pub(super) fn handle_event_impl(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        // In MainMenu state, let the TitleScreenElement handle input first
        if let AppState::MainMenu(_) = &self.app_state {
            // Forward event to children (TitleScreenElement)
            for child in &mut self.children {
                if child.handle_event(event, bounds) {
                    tracing::debug!("MainMenu: Event handled by child element");
                    return true; // Event was handled by child
                }
            }
        }

        // In ShowingChoices state, let the ChoiceMenuElement handle input first
        if let AppState::InGame(InGameState::ShowingChoices(_)) = &self.app_state {
            // Forward event to children (ChoiceMenuElement)
            for child in &mut self.children {
                if child.handle_event(event, bounds) {
                    tracing::debug!("Event handled by child element");
                    return true; // Event was handled by child
                }
            }
        }

        // In Backlog state, let the BacklogElement handle input first
        if let AppState::InGame(InGameState::Backlog(_)) = &self.app_state {
            // Forward event to children (BacklogElement)
            for child in &mut self.children {
                if child.handle_event(event, bounds) {
                    tracing::debug!("Backlog: Event handled by child element");
                    return true; // Event was handled by child
                }
            }
        }

        // In Settings state, let the SettingsMenuElement handle input first
        if let AppState::Settings(_) = &self.app_state {
            // Forward event to children (SettingsMenuElement)
            for child in &mut self.children {
                if child.handle_event(event, bounds) {
                    tracing::debug!("Settings: Event handled by child element");
                    return true; // Event was handled by child
                }
            }
            // For Settings state, don't process default GameRoot input handling
            // Only process keyboard shortcuts (Escape, F1) which are handled below
            // This prevents mouse events from being consumed by GameRoot
        }

        // In PauseMenu state, let the PauseMenuElement and ConfirmDialogElement handle input first
        if let AppState::InGame(InGameState::PauseMenu(_)) = &self.app_state {
            // Forward event to children (PauseMenuElement, ConfirmDialogElement)
            for child in &mut self.children {
                if child.handle_event(event, bounds) {
                    tracing::debug!("PauseMenu: Event handled by child element");
                    return true; // Event was handled by child
                }
            }
        }

        // In SaveLoadMenu state, let the SaveLoadMenuElement handle input first
        if let AppState::InGame(InGameState::SaveLoadMenu(_)) = &self.app_state {
            // Forward event to children (SaveLoadMenuElement)
            for child in &mut self.children {
                if child.handle_event(event, bounds) {
                    tracing::debug!("SaveLoadMenu: Event handled by child element");
                    return true; // Event was handled by child
                }
            }
        }

        // In CgGallery state, let the CgGalleryElement handle input first
        if let AppState::InGame(InGameState::CgGallery(_)) = &self.app_state {
            // Forward event to children (CgGalleryElement)
            for child in &mut self.children {
                if child.handle_event(event, bounds) {
                    tracing::debug!("CgGallery: Event handled by child element");
                    return true; // Event was handled by child
                }
            }
        }

        // In CgViewer state, let the CgViewerElement handle input first
        if let AppState::InGame(InGameState::CgViewer(_)) = &self.app_state {
            // Forward event to children (CgViewerElement)
            for child in &mut self.children {
                if child.handle_event(event, bounds) {
                    tracing::debug!("CgViewer: Event handled by child element");
                    return true; // Event was handled by child
                }
            }
        }

        // In Backlog state, let the BacklogElement handle input first
        if let AppState::InGame(InGameState::Backlog(_)) = &self.app_state {
            // Forward event to children (BacklogElement)
            for child in &mut self.children {
                if child.handle_event(event, bounds) {
                    tracing::debug!("Backlog: Event handled by child element");
                    return true; // Event was handled by child
                }
            }
        }

        // In Typing or WaitingInput state, let the QuickMenuElement handle input first
        if matches!(
            self.app_state,
            AppState::InGame(InGameState::Typing(_) | InGameState::WaitingInput(_))
        ) {
            // Forward event to children (QuickMenuElement) - process in reverse to handle quick menu first
            for child in self.children.iter_mut().rev() {
                if let Some(quick_menu) = child.as_any_mut().downcast_mut::<QuickMenuElement>()
                    && quick_menu.handle_event(event, bounds)
                {
                    tracing::debug!("Quick menu: Event handled by quick menu");
                    return true; // Event was handled by quick menu
                }
            }
        }

        // Handle input events at GameRoot level (not in Settings state)
        if !matches!(self.app_state, AppState::Settings(_))
            && let InputEvent::MouseDown { button, .. } = event
        {
            match button {
                MouseButton::Left => {
                    tracing::debug!("GameRootElement: Left mouse button pressed");
                    self.clicked_last_frame = true;
                    return true;
                }
                MouseButton::Right => {
                    // Right click - toggle UI visibility (only in Typing/WaitingInput states)
                    if matches!(
                        self.app_state,
                        AppState::InGame(InGameState::Typing(_) | InGameState::WaitingInput(_))
                    ) {
                        self.ui_hidden = !self.ui_hidden;
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true; // Force rebuild to hide/show UI
                        tracing::debug!(
                            "UI visibility toggled by right-click: {}",
                            if self.ui_hidden { "hidden" } else { "visible" }
                        );
                        return true;
                    }
                }
                _ => {}
            }
        }

        // Handle keyboard shortcuts (works in all states)
        match event {
            InputEvent::KeyDown { key, .. } => match key {
                KeyCode::Enter | KeyCode::Space => {
                    // Enter/Space key acts as click for dialogue progression
                    tracing::debug!("GameRootElement: Enter/Space key pressed (acts as click)");
                    self.clicked_last_frame = true;
                    true
                }
                KeyCode::Escape => {
                    // Escape key - open settings from main menu, or go back if already in settings
                    if matches!(self.app_state, AppState::Settings(_))
                        || matches!(self.app_state, AppState::MainMenu(_))
                    {
                        self.toggle_settings_menu();
                    } else {
                        self.pause_pressed = true;
                    }
                    true
                }
                KeyCode::F1 => {
                    // F1 key - toggle settings from anywhere (except loading)
                    if !matches!(self.app_state, AppState::Loading(_)) {
                        self.toggle_settings_menu();
                        true
                    } else {
                        false
                    }
                }
                KeyCode::A => {
                    // A key - toggle auto mode (only in game)
                    if matches!(self.app_state, AppState::InGame(_)) {
                        self.auto_mode_toggle_pressed = true;
                        true
                    } else {
                        false
                    }
                }
                KeyCode::S => {
                    // S key - toggle skip mode (only in game)
                    if matches!(self.app_state, AppState::InGame(_)) {
                        self.skip_mode_toggle_pressed = true;
                        true
                    } else {
                        false
                    }
                }
                KeyCode::B => {
                    // B key - toggle backlog (open or close)
                    if matches!(self.app_state, AppState::InGame(_)) {
                        self.backlog_pressed = true;
                        return true;
                    }
                    false
                }
                KeyCode::H => {
                    // H key - toggle UI visibility (only in Typing/WaitingInput states)
                    if matches!(
                        self.app_state,
                        AppState::InGame(InGameState::Typing(_) | InGameState::WaitingInput(_))
                    ) {
                        self.ui_hidden = !self.ui_hidden;
                        tracing::debug!("children_dirty set at line {}", line!());
                        self.children_dirty = true; // Force rebuild to hide/show UI
                        tracing::debug!(
                            "UI visibility toggled: {}",
                            if self.ui_hidden { "hidden" } else { "visible" }
                        );
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }
}
