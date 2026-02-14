//! Tests for ScenarioRuntime lifecycle (construction and initialization)

use super::*;

#[test]
fn test_scenario_runtime_new() {
    let scenario = create_test_scenario();
    let runtime = ScenarioRuntime::new(scenario.clone());

    assert_eq!(runtime.current_scene(), None);
    assert_eq!(runtime.command_index(), 0);
}

#[test]
fn test_scenario_runtime_start() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);

    runtime.start().unwrap();

    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("scene1".to_string()))
    );
    assert_eq!(runtime.command_index(), 0);
}

#[test]
fn test_scenario_runtime_start_invalid_scene() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let scenario = Scenario::new(metadata, "nonexistent");
    let mut runtime = ScenarioRuntime::new(scenario);

    let result = runtime.start();
    assert!(result.is_err());
}
