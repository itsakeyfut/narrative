//! Tests for basic command execution

use super::*;

#[test]
fn test_execute_dialogue_command() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    let result = runtime.execute_current_command().unwrap();
    assert_eq!(result, CommandExecutionResult::Continue);
}

#[test]
fn test_execute_set_flag_command() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Advance to SetFlag command
    runtime.advance_command();

    let flag_id = FlagId::new("test_flag");
    assert!(!runtime.flags().is_set(&flag_id));

    let result = runtime.execute_current_command().unwrap();
    assert_eq!(result, CommandExecutionResult::Continue);
    assert!(runtime.flags().is_set(&flag_id));
}

#[test]
fn test_execute_jump_command() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Advance to JumpToScene command
    runtime.advance_command();
    runtime.advance_command();

    let result = runtime.execute_current_command().unwrap();
    assert!(matches!(
        result,
        CommandExecutionResult::SceneChanged { .. }
    ));
    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("scene2".to_string()))
    );
    assert_eq!(runtime.command_index(), 0);
}

#[test]
fn test_jump_to_scene() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime
        .jump_to_scene(&SceneId::new("scene2".to_string()))
        .unwrap();

    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("scene2".to_string()))
    );
    assert_eq!(runtime.command_index(), 0);
}

#[test]
fn test_jump_to_invalid_scene() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    let result = runtime.jump_to_scene(&SceneId::new("nonexistent".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_execute_wait_command() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::Wait { duration: 2.5 });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    let result = runtime.execute_current_command().unwrap();
    assert_eq!(result, CommandExecutionResult::Wait(2.5));
}

#[test]
fn test_execute_end_command() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Jump to scene2 which has End command
    runtime
        .jump_to_scene(&SceneId::new("scene2".to_string()))
        .unwrap();
    runtime.advance_command();

    let result = runtime.execute_current_command().unwrap();
    assert_eq!(result, CommandExecutionResult::End);
}
