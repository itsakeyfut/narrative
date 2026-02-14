//! Tests for ScenarioRuntime
//!
//! This module contains integration tests for the scenario runtime executor.
//! Tests are organized by functionality.

use super::*;
use narrative_core::{Choice, ChoiceOption, Dialogue, ScenarioMetadata, VariableValue};

/// Helper function to create a basic test scenario with two scenes
pub(super) fn create_test_scenario() -> Scenario {
    let metadata = ScenarioMetadata::new("test", "Test Scenario");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene1 = Scene::new("scene1", "Scene 1");
    scene1.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Test dialogue"),
    });
    scene1.add_command(ScenarioCommand::SetFlag {
        flag_name: "test_flag".to_string(),
        value: true,
    });
    scene1.add_command(ScenarioCommand::JumpToScene {
        scene_id: "scene2".to_string(),
    });

    let mut scene2 = Scene::new("scene2", "Scene 2");
    scene2.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Scene 2 dialogue"),
    });
    scene2.add_command(ScenarioCommand::End);

    scenario.add_scene("scene1", scene1);
    scenario.add_scene("scene2", scene2);

    scenario
}

/// Helper function to create a scenario with Call/Return commands
pub(super) fn create_call_return_scenario() -> Scenario {
    let metadata = ScenarioMetadata::new("test_call", "Test Call/Return");
    let mut scenario = Scenario::new(metadata, "main");

    // Main scene: Call subroutine and continue
    let mut main_scene = Scene::new("main", "Main Scene");
    main_scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Before call"),
    });
    main_scene.add_command(ScenarioCommand::Call {
        scene_id: "subroutine".to_string(),
        return_scene: "main".to_string(),
    });
    main_scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("After return"),
    });
    main_scene.add_command(ScenarioCommand::End);

    // Subroutine scene
    let mut subroutine = Scene::new("subroutine", "Subroutine");
    subroutine.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("In subroutine"),
    });
    subroutine.add_command(ScenarioCommand::Return);

    // Nested subroutine scene
    let mut nested = Scene::new("nested", "Nested Subroutine");
    nested.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("In nested"),
    });
    nested.add_command(ScenarioCommand::Return);

    scenario.add_scene("main", main_scene);
    scenario.add_scene("subroutine", subroutine);
    scenario.add_scene("nested", nested);

    scenario
}

mod call_return_tests;
mod choice_tests;
mod command_execution_tests;
mod conditional_tests;
mod display_state_tests;
mod flow_control_tests;
mod lifecycle_tests;
mod persistence_tests;
mod variable_tests;
