//! Tests for GameRootElement public API

use super::element::GameRootElement;
use narrative_engine::EngineConfig;
use narrative_engine::runtime::AppState;

// Constants
const LOADING_COMPLETE_THRESHOLD: f32 = 1.0;

#[test]
fn test_game_root_creation() {
    let config = EngineConfig::default();
    let root = GameRootElement::new(config);

    assert!(root.app_state.is_loading());
    assert!(root.scenario_runtime.is_none());
    assert_eq!(root.children.len(), 0);
}

#[test]
fn test_loading_state_transition() {
    let config = EngineConfig::default();
    let mut root = GameRootElement::new(config);

    // Simulate loading completion
    if let AppState::Loading(loading) = &mut root.app_state {
        loading.progress = LOADING_COMPLETE_THRESHOLD;
    }

    root.update_state(GameRootElement::FRAME_TIME);

    // Should transition to main menu
    assert!(root.app_state.is_main_menu());
}

#[test]
fn test_with_scenario_constructor() {
    use std::path::PathBuf;

    let config = EngineConfig::default();
    let custom_path = "scenarios/test.toml";

    let root = GameRootElement::with_scenario(config, custom_path);

    // Verify the scenario path was set correctly
    assert_eq!(root.config.start_scenario, PathBuf::from(custom_path));
    assert!(root.app_state.is_loading());
    assert!(root.scenario_runtime.is_none());
}

#[test]
fn test_with_scenario_pathbuf() {
    use std::path::PathBuf;

    let config = EngineConfig::default();
    let path = PathBuf::from("scenarios/custom.toml");

    let root = GameRootElement::with_scenario(config, path.clone());

    assert_eq!(root.config.start_scenario, path);
}

#[test]
fn test_load_scenario_error_handling() {
    let config = EngineConfig::default();
    let mut root = GameRootElement::new(config);

    // Try to load non-existent scenario
    let result = root.load_scenario("nonexistent/scenario.toml");

    // Should return an error, not panic
    assert!(result.is_err());
}
