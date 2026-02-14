//! Input handling module
//!
//! This module provides input handling for keyboard, mouse, and gamepad.
//!
//! # Input Architecture
//!
//! The input system consists of three main components:
//!
//! - `InputHandler`: Manages input state and processes winit events
//! - `InputState`: Tracks current input state (keyboard, mouse, modifiers)
//! - `KeyCode`/`MouseButton`: Type-safe key and button identifiers
//!
//! # High-Level Game Actions
//!
//! `InputState` provides high-level game action queries that abstract
//! over multiple input methods:
//!
//! - `clicked()`: Left mouse button, Space, or Enter
//! - `pause_pressed()`: Escape key
//! - `confirm_pressed()`: Enter or Space
//! - `up_pressed()`, `down_pressed()`: Arrow key navigation
//! - `choice_hover_index`: Mouse hover over choice options
//!
//! These are used by the runtime state machine (see `docs/design/engine/runtime.md`).
//!
//! # Example
//!
//! ```rust
//! use narrative_engine::input::{InputHandler, KeyCode, MouseButton};
//!
//! let mut input = InputHandler::new();
//!
//! // In your event loop:
//! // input.process_keyboard_event(key, state);
//! // input.process_mouse_button_event(button, state);
//!
//! // At the start of each frame:
//! input.update();
//!
//! // Query input state:
//! if input.state().clicked() {
//!     println!("Clicked!");
//! }
//!
//! if input.state().pause_pressed() {
//!     println!("Pause pressed!");
//! }
//! ```

mod handler;
mod key;
mod mouse;

pub use handler::{InputHandler, InputState, Modifiers};
pub use key::KeyCode;
pub use mouse::MouseButton;
