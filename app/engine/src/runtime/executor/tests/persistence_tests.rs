//! Tests for save/load functionality

use super::*;
use crate::save::SaveData;

#[test]
fn test_to_save_data() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Set some flags
    runtime
        .flags_mut()
        .set(FlagId::new("completed_intro"), true);
    runtime.flags_mut().set(FlagId::new("saw_ending_a"), false);

    // Set some variables
    runtime
        .variables_mut()
        .set(VariableId::new("score"), VariableValue::Int(100));

    // Create save data
    let save_data = runtime.to_save_data(1);

    // Verify save data
    assert_eq!(save_data.slot, 1);
    assert_eq!(save_data.current_scene, SceneId::new("scene1".to_string()));
    assert_eq!(save_data.command_index, 0);
    assert_eq!(save_data.flags.get("completed_intro"), Some(&true));
    assert_eq!(save_data.flags.get("saw_ending_a"), Some(&false));
    assert_eq!(save_data.variables.get("score"), Some(&100));
}

#[test]
fn test_from_save_data() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario.clone());

    // Create a save data with specific state
    let mut save_data = SaveData::new(1);
    save_data.current_scene = SceneId::new("scene2".to_string());
    save_data.command_index = 1;
    save_data.flags.insert("flag_loaded".to_string(), true);
    save_data.variables.insert("hp".to_string(), 50);
    save_data
        .read_history
        .mark_read(SceneId::new("scene1".to_string()), 0);

    // Load the save data
    runtime.from_save_data(&save_data).unwrap();

    // Verify runtime state was restored
    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("scene2".to_string()))
    );
    assert_eq!(runtime.command_index(), 1);

    // Verify flags were loaded
    assert!(runtime.flags().is_set(&FlagId::new("flag_loaded")));

    // Verify variables were loaded
    assert_eq!(
        runtime.variables().get(&VariableId::new("hp")),
        Some(&VariableValue::Int(50))
    );

    // Verify read history was loaded
    assert!(
        runtime
            .read_history()
            .is_read(&SceneId::new("scene1".to_string()), 0)
    );
}

#[test]
fn test_save_load_roundtrip() {
    let scenario = create_test_scenario();
    let mut runtime1 = ScenarioRuntime::new(scenario.clone());
    runtime1.start().unwrap();

    // Set up some state
    runtime1.advance_command();
    runtime1.flags_mut().set(FlagId::new("checkpoint_1"), true);
    runtime1
        .variables_mut()
        .set(VariableId::new("gold"), VariableValue::Int(999));

    // Save to SaveData
    let save_data = runtime1.to_save_data(5);

    // Create a new runtime and load the save data
    let mut runtime2 = ScenarioRuntime::new(scenario);
    runtime2.from_save_data(&save_data).unwrap();

    // Verify all state was preserved
    assert_eq!(runtime1.current_scene(), runtime2.current_scene());
    assert_eq!(runtime1.command_index(), runtime2.command_index());
    assert_eq!(
        runtime1.flags().is_set(&FlagId::new("checkpoint_1")),
        runtime2.flags().is_set(&FlagId::new("checkpoint_1"))
    );
    assert_eq!(
        runtime1.variables().get(&VariableId::new("gold")),
        runtime2.variables().get(&VariableId::new("gold"))
    );
}

#[test]
fn test_from_save_data_invalid_scene() {
    let scenario = create_test_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);

    // Create a save data with a non-existent scene
    let mut save_data = SaveData::new(1);
    save_data.current_scene = SceneId::new("nonexistent_scene".to_string());

    // Loading should fail
    let result = runtime.from_save_data(&save_data);
    assert!(result.is_err());

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("non-existent scene"));
        assert!(error_msg.contains("nonexistent_scene"));
    }
}

#[test]
fn test_from_save_data_with_scene_stack() {
    let scenario = create_call_return_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);

    // Create a save data with a scene stack
    let mut save_data = SaveData::new(1);
    save_data.current_scene = SceneId::new("subroutine".to_string());
    save_data.command_index = 0;
    save_data
        .scene_stack
        .push((SceneId::new("main".to_string()), 2));

    // Load the save data
    runtime.from_save_data(&save_data).unwrap();

    // Verify scene stack was restored
    assert_eq!(runtime.scene_stack.len(), 1);
    assert_eq!(runtime.scene_stack[0].0, SceneId::new("main".to_string()));
    assert_eq!(runtime.scene_stack[0].1, 2);
}
