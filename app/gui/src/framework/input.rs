//! Input event handling

use super::layout::Point;

/// Mouse button identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Back => MouseButton::Back,
            winit::event::MouseButton::Forward => MouseButton::Forward,
            winit::event::MouseButton::Other(id) => MouseButton::Other(id),
        }
    }
}

/// Keyboard key codes (subset of commonly used keys)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Numbers
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Navigation
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,

    // Editing
    Backspace,
    Delete,
    Insert,
    Enter,
    Tab,

    // Modifiers
    Shift,
    Control,
    Alt,
    Super,

    // Special
    Escape,
    Space,

    // Media (for video editing)
    PlayPause,
    Stop,
    NextTrack,
    PrevTrack,

    // Unknown
    Unknown,
}

impl From<winit::keyboard::KeyCode> for KeyCode {
    fn from(key: winit::keyboard::KeyCode) -> Self {
        use winit::keyboard::KeyCode as WK;
        match key {
            WK::KeyA => KeyCode::A,
            WK::KeyB => KeyCode::B,
            WK::KeyC => KeyCode::C,
            WK::KeyD => KeyCode::D,
            WK::KeyE => KeyCode::E,
            WK::KeyF => KeyCode::F,
            WK::KeyG => KeyCode::G,
            WK::KeyH => KeyCode::H,
            WK::KeyI => KeyCode::I,
            WK::KeyJ => KeyCode::J,
            WK::KeyK => KeyCode::K,
            WK::KeyL => KeyCode::L,
            WK::KeyM => KeyCode::M,
            WK::KeyN => KeyCode::N,
            WK::KeyO => KeyCode::O,
            WK::KeyP => KeyCode::P,
            WK::KeyQ => KeyCode::Q,
            WK::KeyR => KeyCode::R,
            WK::KeyS => KeyCode::S,
            WK::KeyT => KeyCode::T,
            WK::KeyU => KeyCode::U,
            WK::KeyV => KeyCode::V,
            WK::KeyW => KeyCode::W,
            WK::KeyX => KeyCode::X,
            WK::KeyY => KeyCode::Y,
            WK::KeyZ => KeyCode::Z,
            WK::Digit0 => KeyCode::Key0,
            WK::Digit1 => KeyCode::Key1,
            WK::Digit2 => KeyCode::Key2,
            WK::Digit3 => KeyCode::Key3,
            WK::Digit4 => KeyCode::Key4,
            WK::Digit5 => KeyCode::Key5,
            WK::Digit6 => KeyCode::Key6,
            WK::Digit7 => KeyCode::Key7,
            WK::Digit8 => KeyCode::Key8,
            WK::Digit9 => KeyCode::Key9,
            WK::F1 => KeyCode::F1,
            WK::F2 => KeyCode::F2,
            WK::F3 => KeyCode::F3,
            WK::F4 => KeyCode::F4,
            WK::F5 => KeyCode::F5,
            WK::F6 => KeyCode::F6,
            WK::F7 => KeyCode::F7,
            WK::F8 => KeyCode::F8,
            WK::F9 => KeyCode::F9,
            WK::F10 => KeyCode::F10,
            WK::F11 => KeyCode::F11,
            WK::F12 => KeyCode::F12,
            WK::ArrowUp => KeyCode::Up,
            WK::ArrowDown => KeyCode::Down,
            WK::ArrowLeft => KeyCode::Left,
            WK::ArrowRight => KeyCode::Right,
            WK::Home => KeyCode::Home,
            WK::End => KeyCode::End,
            WK::PageUp => KeyCode::PageUp,
            WK::PageDown => KeyCode::PageDown,
            WK::Backspace => KeyCode::Backspace,
            WK::Delete => KeyCode::Delete,
            WK::Insert => KeyCode::Insert,
            WK::Enter => KeyCode::Enter,
            WK::Tab => KeyCode::Tab,
            WK::ShiftLeft | WK::ShiftRight => KeyCode::Shift,
            WK::ControlLeft | WK::ControlRight => KeyCode::Control,
            WK::AltLeft | WK::AltRight => KeyCode::Alt,
            WK::SuperLeft | WK::SuperRight => KeyCode::Super,
            WK::Escape => KeyCode::Escape,
            WK::Space => KeyCode::Space,
            WK::MediaPlayPause => KeyCode::PlayPause,
            WK::MediaStop => KeyCode::Stop,
            WK::MediaTrackNext => KeyCode::NextTrack,
            WK::MediaTrackPrevious => KeyCode::PrevTrack,
            _ => KeyCode::Unknown,
        }
    }
}

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

/// Input events from the windowing system
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Mouse cursor moved
    MouseMove {
        position: Point,
        modifiers: Modifiers,
    },

    /// Mouse button pressed
    MouseDown {
        button: MouseButton,
        position: Point,
        modifiers: Modifiers,
    },

    /// Mouse button released
    MouseUp {
        button: MouseButton,
        position: Point,
        modifiers: Modifiers,
    },

    /// Mouse wheel scrolled
    MouseScroll {
        delta: Point,
        position: Point,
        modifiers: Modifiers,
    },

    /// Key pressed
    KeyDown { key: KeyCode, modifiers: Modifiers },

    /// Key released
    KeyUp { key: KeyCode, modifiers: Modifiers },

    /// Character input (for text input)
    CharInput { character: char },

    /// Window focused
    Focus,

    /// Window lost focus
    Blur,

    /// File dropped onto window
    DroppedFile { path: std::path::PathBuf },

    /// File drag hovering over window
    HoveredFile { path: std::path::PathBuf },

    /// File drag cancelled
    HoveredFileCancelled,
}

/// Tracks the current input state
#[derive(Debug, Default)]
pub struct InputState {
    pub mouse_position: Point,
    pub modifiers: Modifiers,
    pressed_mouse_buttons: std::collections::HashSet<MouseButton>,
    pressed_keys: std::collections::HashSet<KeyCode>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn handle_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::MouseMove {
                position,
                modifiers,
            } => {
                self.mouse_position = *position;
                self.modifiers = *modifiers;
            }
            InputEvent::MouseDown {
                button,
                position,
                modifiers,
            } => {
                self.pressed_mouse_buttons.insert(*button);
                self.mouse_position = *position;
                self.modifiers = *modifiers;
            }
            InputEvent::MouseUp {
                button,
                position,
                modifiers,
            } => {
                self.pressed_mouse_buttons.remove(button);
                self.mouse_position = *position;
                self.modifiers = *modifiers;
            }
            InputEvent::MouseScroll { modifiers, .. } => {
                self.modifiers = *modifiers;
            }
            InputEvent::KeyDown { key, modifiers } => {
                self.pressed_keys.insert(*key);
                self.modifiers = *modifiers;
            }
            InputEvent::KeyUp { key, modifiers } => {
                self.pressed_keys.remove(key);
                self.modifiers = *modifiers;
            }
            InputEvent::Blur => {
                // Clear all pressed state on blur
                self.pressed_mouse_buttons.clear();
                self.pressed_keys.clear();
            }
            _ => {}
        }
    }
}
