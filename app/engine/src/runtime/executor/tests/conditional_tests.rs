//! Tests for conditional logic and If commands

use super::*;
use narrative_core::{CompareOp, Condition};

#[test]
fn test_if_command_then_branch() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");

    // Set a flag first
    scene.add_command(ScenarioCommand::SetFlag {
        flag_name: "has_item".to_string(),
        value: true,
    });

    // If has_item is true, set another flag
    scene.add_command(ScenarioCommand::If {
        condition: Condition::flag("has_item", true),
        then_commands: vec![ScenarioCommand::SetFlag {
            flag_name: "unlocked".to_string(),
            value: true,
        }],
        else_commands: vec![],
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute SetFlag command
    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // Execute If command
    runtime.execute_current_command().unwrap();

    // Check that the then branch was executed
    assert!(runtime.flags().is_set(&FlagId::new("unlocked")));
}

#[test]
fn test_if_command_else_branch() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");

    // If has_item is true (but it's not), set unlocked, else set locked
    scene.add_command(ScenarioCommand::If {
        condition: Condition::flag("has_item", true),
        then_commands: vec![ScenarioCommand::SetFlag {
            flag_name: "unlocked".to_string(),
            value: true,
        }],
        else_commands: vec![ScenarioCommand::SetFlag {
            flag_name: "locked".to_string(),
            value: true,
        }],
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute If command
    runtime.execute_current_command().unwrap();

    // Check that the else branch was executed
    assert!(!runtime.flags().is_set(&FlagId::new("unlocked")));
    assert!(runtime.flags().is_set(&FlagId::new("locked")));
}

#[test]
fn test_if_command_with_variable_condition() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");

    // Set score to 150
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "score".to_string(),
        value: VariableValue::Int(150),
    });

    // If score > 100, set high_score flag
    scene.add_command(ScenarioCommand::If {
        condition: Condition::variable("score", CompareOp::GreaterThan, VariableValue::Int(100)),
        then_commands: vec![ScenarioCommand::SetFlag {
            flag_name: "high_score".to_string(),
            value: true,
        }],
        else_commands: vec![],
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute SetVariable
    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // Execute If command
    runtime.execute_current_command().unwrap();

    // Check that high_score was set
    assert!(runtime.flags().is_set(&FlagId::new("high_score")));
}

#[test]
fn test_if_command_nested() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");

    scene.add_command(ScenarioCommand::SetFlag {
        flag_name: "outer".to_string(),
        value: true,
    });
    scene.add_command(ScenarioCommand::SetFlag {
        flag_name: "inner".to_string(),
        value: true,
    });

    // Nested If: if outer, then if inner, set result
    scene.add_command(ScenarioCommand::If {
        condition: Condition::flag("outer", true),
        then_commands: vec![ScenarioCommand::If {
            condition: Condition::flag("inner", true),
            then_commands: vec![ScenarioCommand::SetFlag {
                flag_name: "result".to_string(),
                value: true,
            }],
            else_commands: vec![],
        }],
        else_commands: vec![],
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();
    runtime.execute_current_command().unwrap();
    runtime.advance_command();
    runtime.execute_current_command().unwrap();

    assert!(runtime.flags().is_set(&FlagId::new("result")));
}

#[test]
fn test_if_command_complex_condition() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");

    scene.add_command(ScenarioCommand::SetFlag {
        flag_name: "has_key".to_string(),
        value: true,
    });
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "level".to_string(),
        value: VariableValue::Int(5),
    });

    // If (has_key AND level >= 5), unlock
    scene.add_command(ScenarioCommand::If {
        condition: Condition::and(vec![
            Condition::flag("has_key", true),
            Condition::variable("level", CompareOp::GreaterOrEqual, VariableValue::Int(5)),
        ]),
        then_commands: vec![ScenarioCommand::SetFlag {
            flag_name: "door_unlocked".to_string(),
            value: true,
        }],
        else_commands: vec![],
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();
    runtime.execute_current_command().unwrap();
    runtime.advance_command();
    runtime.execute_current_command().unwrap();

    assert!(runtime.flags().is_set(&FlagId::new("door_unlocked")));
}
