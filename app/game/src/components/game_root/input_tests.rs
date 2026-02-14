//! Tests for input handling (input.rs)

use super::element::GameRootElement;
use narrative_core::types::SceneId;
use narrative_engine::EngineConfig;
use narrative_engine::runtime::{
    AppState, ChoiceState, InGameState, TypingState, WaitingInputState,
};
use narrative_gui::framework::input::{InputEvent, KeyCode, Modifiers, MouseButton};
use narrative_gui::framework::layout::{Bounds, Point};
use std::sync::Arc;

#[test]
fn test_input_handling() {
    let config = EngineConfig::default();
    let mut root = GameRootElement::new(config);

    // Test mouse click
    let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
    let event = InputEvent::MouseDown {
        position: Point::new(50.0, 50.0),
        button: MouseButton::Left,
        modifiers: Modifiers::none(),
    };

    assert!(root.handle_event_impl(&event, bounds));
    assert!(root.clicked_last_frame);
}

#[test]
fn test_ui_hidden_toggle_in_typing_state() {
    let config = EngineConfig::default();
    let mut root = GameRootElement::new(config);

    // Setup: Transition to InGame with Typing state
    root.app_state = AppState::InGame(InGameState::Typing(TypingState {
        scene_id: SceneId::new("test_scene"),
        command_index: 0,
        speaker: None,
        text: Arc::from("Test dialogue"),
        char_index: 0,
        elapsed: 0.0,
        auto_mode: false,
        skip_mode: false,
    }));

    // Initially ui_hidden should be false
    assert!(!root.ui_hidden);

    // Simulate H key press
    let event = InputEvent::KeyDown {
        key: KeyCode::H,
        modifiers: Modifiers::none(),
    };
    let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
    root.handle_event_impl(&event, bounds);

    // ui_hidden should now be true
    assert!(root.ui_hidden);
    assert!(root.children_dirty);

    // Press H again to toggle back
    root.children_dirty = false;
    root.handle_event_impl(&event, bounds);

    // ui_hidden should now be false again
    assert!(!root.ui_hidden);
    assert!(root.children_dirty);
}

#[test]
fn test_ui_hidden_toggle_in_waiting_input_state() {
    let config = EngineConfig::default();
    let mut root = GameRootElement::new(config);

    // Setup: Transition to InGame with WaitingInput state
    root.app_state = AppState::InGame(InGameState::WaitingInput(WaitingInputState {
        scene_id: SceneId::new("test_scene"),
        command_index: 0,
        auto_wait_elapsed: 0.0,
        skip_mode: false,
    }));

    // Initially ui_hidden should be false
    assert!(!root.ui_hidden);

    // Simulate right-click
    let event = InputEvent::MouseDown {
        position: Point::new(50.0, 50.0),
        button: MouseButton::Right,
        modifiers: Modifiers::none(),
    };
    let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
    root.handle_event_impl(&event, bounds);

    // ui_hidden should now be true
    assert!(root.ui_hidden);
    assert!(root.children_dirty);

    // Right-click again to toggle back
    root.children_dirty = false;
    root.handle_event_impl(&event, bounds);

    // ui_hidden should now be false again
    assert!(!root.ui_hidden);
    assert!(root.children_dirty);
}

#[test]
fn test_ui_hidden_not_allowed_in_other_states() {
    let config = EngineConfig::default();
    let mut root = GameRootElement::new(config);

    // Setup: Transition to ShowingChoices state
    root.app_state = AppState::InGame(InGameState::ShowingChoices(ChoiceState {
        scene_id: SceneId::new("test_scene"),
        command_index: 0,
        choices: vec![],
        selected: 0,
        confirmed: false,
    }));

    // Initially ui_hidden should be false
    assert!(!root.ui_hidden);

    // Try to toggle UI with H key
    let event = InputEvent::KeyDown {
        key: KeyCode::H,
        modifiers: Modifiers::none(),
    };
    let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
    root.handle_event_impl(&event, bounds);

    // ui_hidden should still be false (not toggled)
    assert!(!root.ui_hidden);

    // Try right-click as well
    let event = InputEvent::MouseDown {
        position: Point::new(50.0, 50.0),
        button: MouseButton::Right,
        modifiers: Modifiers::none(),
    };
    root.handle_event_impl(&event, bounds);

    // ui_hidden should still be false
    assert!(!root.ui_hidden);
}
