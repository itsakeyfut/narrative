//! Mouse button identifiers

/// Mouse button identifiers
///
/// This enum represents mouse buttons with type-safe identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Left mouse button (primary)
    Left,
    /// Right mouse button (secondary)
    Right,
    /// Middle mouse button (wheel click)
    Middle,
    /// Back button (typically on side of mouse)
    Back,
    /// Forward button (typically on side of mouse)
    Forward,
    /// Other mouse buttons
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
