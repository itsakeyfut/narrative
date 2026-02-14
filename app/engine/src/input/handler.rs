//! Input handler and state management

use super::{KeyCode, MouseButton};
use std::collections::HashSet;

/// Modifier key state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl Modifiers {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.super_key
    }
}

impl From<winit::keyboard::ModifiersState> for Modifiers {
    fn from(state: winit::keyboard::ModifiersState) -> Self {
        Self {
            shift: state.shift_key(),
            ctrl: state.control_key(),
            alt: state.alt_key(),
            super_key: state.super_key(),
        }
    }
}

/// Input state
///
/// Tracks the current state of keyboard and mouse input.
/// Provides both low-level input queries and high-level game actions.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    // Keyboard state
    pressed_keys: HashSet<KeyCode>,
    just_pressed_keys: HashSet<KeyCode>,
    just_released_keys: HashSet<KeyCode>,

    // Mouse state
    mouse_position: (f32, f32),
    pressed_mouse_buttons: HashSet<MouseButton>,
    just_pressed_mouse_buttons: HashSet<MouseButton>,
    just_released_mouse_buttons: HashSet<MouseButton>,

    // Modifiers
    modifiers: Modifiers,

    // High-level game state
    pub choice_hover_index: Option<usize>,
}

impl InputState {
    /// Create a new input state
    pub fn new() -> Self {
        Self::default()
    }

    // ========================================================================
    // Low-level keyboard queries
    // ========================================================================

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.just_released_keys.contains(&key)
    }

    // ========================================================================
    // Low-level mouse queries
    // ========================================================================

    /// Get mouse position
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    /// Check if a mouse button is pressed
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    /// Check if a mouse button was just pressed this frame
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed_mouse_buttons.contains(&button)
    }

    /// Check if a mouse button was just released this frame
    pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.just_released_mouse_buttons.contains(&button)
    }

    // ========================================================================
    // High-level game actions (as used in runtime design)
    // ========================================================================

    /// Check if the confirm action was triggered (left click, space, or enter)
    pub fn clicked(&self) -> bool {
        self.is_mouse_button_just_pressed(MouseButton::Left)
            || self.is_key_just_pressed(KeyCode::Space)
            || self.is_key_just_pressed(KeyCode::Enter)
    }

    /// Check if the pause button was pressed (Escape)
    pub fn pause_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::Escape)
    }

    /// Check if the confirm button was pressed (Enter or Space)
    pub fn confirm_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::Enter) || self.is_key_just_pressed(KeyCode::Space)
    }

    /// Check if the up key was pressed (for navigation)
    pub fn up_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::Up)
    }

    /// Check if the down key was pressed (for navigation)
    pub fn down_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::Down)
    }

    /// Check if the left key was pressed (for navigation)
    pub fn left_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::Left)
    }

    /// Check if the right key was pressed (for navigation)
    pub fn right_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::Right)
    }

    /// Check if auto mode toggle was pressed (A key)
    pub fn auto_mode_toggle_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::A)
    }

    /// Check if skip mode toggle was pressed (S key)
    pub fn skip_mode_toggle_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::S)
    }

    /// Get modifiers state
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    // ========================================================================
    // State modification (for InputHandler)
    // ========================================================================

    /// Clear frame-specific state (just_pressed, just_released)
    pub(super) fn clear_frame_state(&mut self) {
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.just_pressed_mouse_buttons.clear();
        self.just_released_mouse_buttons.clear();
    }

    /// Press a key
    pub(super) fn press_key(&mut self, key: KeyCode) {
        if key != KeyCode::Unknown && self.pressed_keys.insert(key) {
            self.just_pressed_keys.insert(key);
        }
    }

    /// Release a key
    pub(super) fn release_key(&mut self, key: KeyCode) {
        if key != KeyCode::Unknown && self.pressed_keys.remove(&key) {
            self.just_released_keys.insert(key);
        }
    }

    /// Set mouse position
    pub(super) fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
    }

    /// Press mouse button
    pub(super) fn press_mouse_button(&mut self, button: MouseButton) {
        if self.pressed_mouse_buttons.insert(button) {
            self.just_pressed_mouse_buttons.insert(button);
        }
    }

    /// Release mouse button
    pub(super) fn release_mouse_button(&mut self, button: MouseButton) {
        if self.pressed_mouse_buttons.remove(&button) {
            self.just_released_mouse_buttons.insert(button);
        }
    }

    /// Set modifiers state
    pub(super) fn set_modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    /// Set choice hover index
    pub fn set_choice_hover_index(&mut self, index: Option<usize>) {
        self.choice_hover_index = index;
    }
}

/// Input handler
///
/// Manages input state and provides winit event processing.
pub struct InputHandler {
    state: InputState,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self {
            state: InputState::new(),
        }
    }

    /// Get the current input state
    pub fn state(&self) -> &InputState {
        &self.state
    }

    /// Get mutable input state
    pub fn state_mut(&mut self) -> &mut InputState {
        &mut self.state
    }

    /// Update input state (call at the start of each frame)
    ///
    /// This clears frame-specific state like just_pressed and just_released.
    pub fn update(&mut self) {
        self.state.clear_frame_state();
    }

    /// Process a winit keyboard event
    pub fn process_keyboard_event(
        &mut self,
        key: winit::keyboard::KeyCode,
        state: winit::event::ElementState,
    ) {
        let key_code = KeyCode::from(key);
        match state {
            winit::event::ElementState::Pressed => {
                self.state.press_key(key_code);
            }
            winit::event::ElementState::Released => {
                self.state.release_key(key_code);
            }
        }
    }

    /// Process a winit mouse button event
    pub fn process_mouse_button_event(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        let mouse_button = MouseButton::from(button);
        match state {
            winit::event::ElementState::Pressed => {
                self.state.press_mouse_button(mouse_button);
            }
            winit::event::ElementState::Released => {
                self.state.release_mouse_button(mouse_button);
            }
        }
    }

    /// Process a winit mouse motion event
    pub fn process_mouse_motion(&mut self, x: f64, y: f64) {
        self.state.set_mouse_position(x as f32, y as f32);
    }

    /// Process modifiers change
    pub fn process_modifiers(&mut self, modifiers: winit::keyboard::ModifiersState) {
        self.state.set_modifiers(Modifiers::from(modifiers));
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_press() {
        let mut state = InputState::new();

        assert!(!state.is_key_pressed(KeyCode::Space));

        state.press_key(KeyCode::Space);
        assert!(state.is_key_pressed(KeyCode::Space));
        assert!(state.is_key_just_pressed(KeyCode::Space));

        state.clear_frame_state();
        assert!(state.is_key_pressed(KeyCode::Space));
        assert!(!state.is_key_just_pressed(KeyCode::Space));

        state.release_key(KeyCode::Space);
        assert!(!state.is_key_pressed(KeyCode::Space));
        assert!(state.is_key_just_released(KeyCode::Space));
    }

    #[test]
    fn test_multiple_keys() {
        let mut state = InputState::new();

        state.press_key(KeyCode::A);
        state.press_key(KeyCode::S);
        state.press_key(KeyCode::D);

        assert!(state.is_key_pressed(KeyCode::A));
        assert!(state.is_key_pressed(KeyCode::S));
        assert!(state.is_key_pressed(KeyCode::D));
        assert!(!state.is_key_pressed(KeyCode::W));
    }

    #[test]
    fn test_mouse_position() {
        let mut state = InputState::new();

        assert_eq!(state.mouse_position(), (0.0, 0.0));

        state.set_mouse_position(100.5, 200.5);
        assert_eq!(state.mouse_position(), (100.5, 200.5));

        state.set_mouse_position(-50.0, -100.0);
        assert_eq!(state.mouse_position(), (-50.0, -100.0));
    }

    #[test]
    fn test_mouse_buttons() {
        let mut state = InputState::new();

        assert!(!state.is_mouse_button_pressed(MouseButton::Left));
        assert!(!state.is_mouse_button_just_pressed(MouseButton::Left));

        state.press_mouse_button(MouseButton::Left);
        assert!(state.is_mouse_button_pressed(MouseButton::Left));
        assert!(state.is_mouse_button_just_pressed(MouseButton::Left));

        state.clear_frame_state();
        assert!(state.is_mouse_button_pressed(MouseButton::Left));
        assert!(!state.is_mouse_button_just_pressed(MouseButton::Left));

        state.release_mouse_button(MouseButton::Left);
        assert!(!state.is_mouse_button_pressed(MouseButton::Left));
        assert!(state.is_mouse_button_just_released(MouseButton::Left));
    }

    #[test]
    fn test_input_handler_update() {
        let mut handler = InputHandler::new();

        handler.state_mut().press_key(KeyCode::Space);
        assert!(handler.state().is_key_just_pressed(KeyCode::Space));

        handler.update();
        assert!(!handler.state().is_key_just_pressed(KeyCode::Space));
        assert!(handler.state().is_key_pressed(KeyCode::Space));
    }

    #[test]
    fn test_key_hold() {
        let mut state = InputState::new();

        state.press_key(KeyCode::Space);
        assert!(state.is_key_just_pressed(KeyCode::Space));

        state.clear_frame_state();
        assert!(!state.is_key_just_pressed(KeyCode::Space));
        assert!(state.is_key_pressed(KeyCode::Space));

        state.clear_frame_state();
        assert!(state.is_key_pressed(KeyCode::Space)); // Still held
    }

    #[test]
    fn test_key_double_press() {
        let mut state = InputState::new();

        state.press_key(KeyCode::Space);
        assert!(state.is_key_just_pressed(KeyCode::Space));

        // Pressing again while already pressed shouldn't add to just_pressed
        state.press_key(KeyCode::Space);
        state.clear_frame_state();
        state.press_key(KeyCode::Space);
        // Key is still pressed, so just_pressed won't be triggered
        assert!(state.is_key_pressed(KeyCode::Space));
    }

    #[test]
    fn test_input_state_default() {
        let state = InputState::default();

        assert_eq!(state.mouse_position(), (0.0, 0.0));
        assert!(!state.is_key_pressed(KeyCode::A));
        assert!(!state.is_mouse_button_pressed(MouseButton::Left));
    }

    #[test]
    fn test_input_handler_default() {
        let handler = InputHandler::default();

        assert_eq!(handler.state().mouse_position(), (0.0, 0.0));
    }

    #[test]
    fn test_multiple_mouse_buttons() {
        let mut state = InputState::new();

        state.press_mouse_button(MouseButton::Left);
        state.press_mouse_button(MouseButton::Right);
        state.press_mouse_button(MouseButton::Middle);

        assert!(state.is_mouse_button_pressed(MouseButton::Left));
        assert!(state.is_mouse_button_pressed(MouseButton::Right));
        assert!(state.is_mouse_button_pressed(MouseButton::Middle));

        state.release_mouse_button(MouseButton::Right);
        assert!(state.is_mouse_button_pressed(MouseButton::Left));
        assert!(!state.is_mouse_button_pressed(MouseButton::Right));
        assert!(state.is_mouse_button_pressed(MouseButton::Middle));
    }

    #[test]
    fn test_key_release_without_press() {
        let mut state = InputState::new();

        state.release_key(KeyCode::Space);
        assert!(!state.is_key_pressed(KeyCode::Space));
        assert!(!state.is_key_just_released(KeyCode::Space));
    }

    #[test]
    fn test_high_level_clicked() {
        let mut state = InputState::new();

        // Test mouse click
        state.press_mouse_button(MouseButton::Left);
        assert!(state.clicked());

        state.clear_frame_state();
        assert!(!state.clicked());

        // Test space key
        state.press_key(KeyCode::Space);
        assert!(state.clicked());

        state.clear_frame_state();
        assert!(!state.clicked());

        // Test enter key
        state.press_key(KeyCode::Enter);
        assert!(state.clicked());
    }

    #[test]
    fn test_high_level_navigation() {
        let mut state = InputState::new();

        state.press_key(KeyCode::Up);
        assert!(state.up_pressed());
        assert!(!state.down_pressed());

        state.clear_frame_state();
        state.press_key(KeyCode::Down);
        assert!(state.down_pressed());
        assert!(!state.up_pressed());

        state.clear_frame_state();
        state.press_key(KeyCode::Left);
        assert!(state.left_pressed());

        state.clear_frame_state();
        state.press_key(KeyCode::Right);
        assert!(state.right_pressed());
    }

    #[test]
    fn test_high_level_pause() {
        let mut state = InputState::new();

        state.press_key(KeyCode::Escape);
        assert!(state.pause_pressed());

        state.clear_frame_state();
        assert!(!state.pause_pressed());
    }

    #[test]
    fn test_high_level_confirm() {
        let mut state = InputState::new();

        state.press_key(KeyCode::Enter);
        assert!(state.confirm_pressed());

        state.clear_frame_state();
        state.press_key(KeyCode::Space);
        assert!(state.confirm_pressed());
    }

    #[test]
    fn test_choice_hover_index() {
        let mut state = InputState::new();

        assert_eq!(state.choice_hover_index, None);

        state.set_choice_hover_index(Some(2));
        assert_eq!(state.choice_hover_index, Some(2));

        state.set_choice_hover_index(None);
        assert_eq!(state.choice_hover_index, None);
    }

    #[test]
    fn test_modifiers() {
        let mut state = InputState::new();

        assert!(state.modifiers().is_empty());

        let mods = Modifiers {
            shift: true,
            ctrl: false,
            alt: false,
            super_key: false,
        };
        state.set_modifiers(mods);
        assert!(state.modifiers().shift);
        assert!(!state.modifiers().is_empty());
    }

    #[test]
    fn test_unknown_key_ignored() {
        let mut state = InputState::new();

        state.press_key(KeyCode::Unknown);
        assert!(!state.is_key_pressed(KeyCode::Unknown));
        assert!(!state.is_key_just_pressed(KeyCode::Unknown));
    }

    #[test]
    fn test_mouse_button_just_pressed_cleared() {
        let mut state = InputState::new();

        state.press_mouse_button(MouseButton::Left);
        assert!(state.is_mouse_button_just_pressed(MouseButton::Left));

        state.clear_frame_state();
        assert!(!state.is_mouse_button_just_pressed(MouseButton::Left));
        assert!(state.is_mouse_button_pressed(MouseButton::Left));
    }
}
