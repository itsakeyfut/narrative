//! Tests for flow control (scene navigation, command advancement, etc.)

use super::*;

#[test]
fn test_get_current_scene_data() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    let scene = runtime.get_current_scene_data();
    assert!(scene.is_some());
    assert_eq!(scene.unwrap().id, "scene1");
}

#[test]
fn test_get_current_command() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    let command = runtime.get_current_command();
    assert!(command.is_some());
    assert!(matches!(command, Some(ScenarioCommand::Dialogue { .. })));
}

#[test]
fn test_advance_command() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    assert_eq!(runtime.command_index(), 0);

    let advanced = runtime.advance_command();
    assert!(advanced);
    assert_eq!(runtime.command_index(), 1);

    runtime.advance_command();
    assert_eq!(runtime.command_index(), 2);
}

#[test]
fn test_is_ended() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    assert!(!runtime.is_ended());

    // Jump to scene2 and go to End command
    runtime
        .jump_to_scene(&SceneId::new("scene2".to_string()))
        .unwrap();
    assert!(!runtime.is_ended());

    runtime.advance_command();
    assert!(runtime.is_ended());
}

#[test]
fn test_dialogue_level_read_history_tracking() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);

    let scene1_id = SceneId::new("scene1");
    let scene2_id = SceneId::new("scene2");

    // Before marking, dialogue should not be read
    assert!(!runtime.read_history().is_read(&scene1_id, 0));
    assert!(!runtime.read_history().is_read(&scene1_id, 1));
    assert!(!runtime.read_history().is_read(&scene2_id, 0));

    // Mark dialogue at scene1, command_index=0 as read
    runtime.read_history_mut().mark_read(scene1_id.clone(), 0);
    assert!(runtime.read_history().is_read(&scene1_id, 0));
    assert!(!runtime.read_history().is_read(&scene1_id, 1)); // Different command_index
    assert!(!runtime.read_history().is_read(&scene2_id, 0)); // Different scene

    // Mark dialogue at scene1, command_index=1 as read
    runtime.read_history_mut().mark_read(scene1_id.clone(), 1);
    assert!(runtime.read_history().is_read(&scene1_id, 0));
    assert!(runtime.read_history().is_read(&scene1_id, 1));
    assert!(!runtime.read_history().is_read(&scene2_id, 0));

    // Mark dialogue at scene2, command_index=0 as read
    runtime.read_history_mut().mark_read(scene2_id.clone(), 0);
    assert!(runtime.read_history().is_read(&scene1_id, 0));
    assert!(runtime.read_history().is_read(&scene1_id, 1));
    assert!(runtime.read_history().is_read(&scene2_id, 0));
}
