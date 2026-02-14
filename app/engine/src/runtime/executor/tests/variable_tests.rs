//! Tests for variable operations

use super::*;
use narrative_core::VariableOperation;

#[test]
fn test_variable_operations() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "score".to_string(),
        value: VariableValue::Int(100),
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    let var_id = VariableId::new("score");
    assert!(runtime.variables().get(&var_id).is_none());

    runtime.execute_current_command().unwrap();

    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Int(100))
    );
}

#[test]
fn test_modify_variable_add() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    // Set initial value
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "score".to_string(),
        value: VariableValue::Int(10),
    });
    // Add 5
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "score".to_string(),
        operation: VariableOperation::Add { value: 5 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    let var_id = VariableId::new("score");

    // Execute SetVariable
    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Int(10))
    );

    // Execute ModifyVariable (Add)
    runtime.execute_current_command().unwrap();

    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Int(15))
    );
}

#[test]
fn test_modify_variable_subtract() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "health".to_string(),
        value: VariableValue::Int(100),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "health".to_string(),
        operation: VariableOperation::Subtract { value: 25 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("health");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Int(75))
    );
}

#[test]
fn test_modify_variable_multiply() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "multiplier".to_string(),
        value: VariableValue::Int(7),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "multiplier".to_string(),
        operation: VariableOperation::Multiply { value: 3 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("multiplier");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Int(21))
    );
}

#[test]
fn test_modify_variable_divide() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "value".to_string(),
        value: VariableValue::Int(100),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "value".to_string(),
        operation: VariableOperation::Divide { value: 4 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("value");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Int(25))
    );
}

#[test]
fn test_modify_variable_divide_by_zero() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "value".to_string(),
        value: VariableValue::Int(100),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "value".to_string(),
        operation: VariableOperation::Divide { value: 0 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // Division by zero should return an error
    let result = runtime.execute_current_command();
    assert!(result.is_err());
}

#[test]
fn test_modify_variable_append() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "message".to_string(),
        value: VariableValue::String("Hello".to_string()),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "message".to_string(),
        operation: VariableOperation::Append {
            text: " World".to_string(),
        },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("message");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::String("Hello World".to_string()))
    );
}

#[test]
fn test_modify_variable_toggle() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "enabled".to_string(),
        value: VariableValue::Bool(true),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "enabled".to_string(),
        operation: VariableOperation::Toggle,
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("enabled");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Bool(false))
    );
}

#[test]
fn test_modify_variable_add_float() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "temperature".to_string(),
        value: VariableValue::Float(20.5),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "temperature".to_string(),
        operation: VariableOperation::AddFloat { value: 3.2 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("temperature");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Float(23.7))
    );
}

#[test]
fn test_modify_variable_subtract_float() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "balance".to_string(),
        value: VariableValue::Float(100.75),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "balance".to_string(),
        operation: VariableOperation::SubtractFloat { value: 25.5 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("balance");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Float(75.25))
    );
}

#[test]
fn test_modify_variable_multiply_float() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "multiplier".to_string(),
        value: VariableValue::Float(2.5),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "multiplier".to_string(),
        operation: VariableOperation::MultiplyFloat { value: 4.0 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("multiplier");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Float(10.0))
    );
}

#[test]
fn test_modify_variable_divide_float() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "value".to_string(),
        value: VariableValue::Float(100.0),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "value".to_string(),
        operation: VariableOperation::DivideFloat { value: 4.0 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("value");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Float(25.0))
    );
}

#[test]
fn test_modify_variable_divide_float_by_zero() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "value".to_string(),
        value: VariableValue::Float(100.0),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "value".to_string(),
        operation: VariableOperation::DivideFloat { value: 0.0 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // Division by zero should return an error
    let result = runtime.execute_current_command();
    assert!(result.is_err());
}

#[test]
fn test_modify_variable_float_undefined_defaults_to_zero() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    // Add to undefined float variable (should default to 0.0)
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "undefined_float".to_string(),
        operation: VariableOperation::AddFloat { value: 5.5 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("undefined_float");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Float(5.5))
    );
}

#[test]
fn test_modify_variable_float_type_mismatch() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "int_var".to_string(),
        value: VariableValue::Int(42),
    });
    // Try to add float to an integer variable (should fail)
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "int_var".to_string(),
        operation: VariableOperation::AddFloat { value: 5.5 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // Type mismatch should return an error
    let result = runtime.execute_current_command();
    assert!(result.is_err());
}

#[test]
fn test_modify_variable_saturating_add() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "big".to_string(),
        value: VariableValue::Int(i64::MAX - 10),
    });
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "big".to_string(),
        operation: VariableOperation::Add { value: 100 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("big");
    // Should saturate at i64::MAX
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Int(i64::MAX))
    );
}

#[test]
fn test_modify_variable_type_mismatch() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "text".to_string(),
        value: VariableValue::String("hello".to_string()),
    });
    // Try to add a number to a string (should fail)
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "text".to_string(),
        operation: VariableOperation::Add { value: 10 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // Type mismatch should return an error
    let result = runtime.execute_current_command();
    assert!(result.is_err());
}

#[test]
fn test_modify_variable_undefined_defaults_to_zero() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    // Add to undefined variable (should default to 0)
    scene.add_command(ScenarioCommand::ModifyVariable {
        variable_name: "undefined_var".to_string(),
        operation: VariableOperation::Add { value: 5 },
    });
    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("undefined_var");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Int(5))
    );
}

#[test]
fn test_modify_variable_in_if_block() {
    use narrative_core::{CompareOp, Condition};

    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");

    // Set score to 100
    scene.add_command(ScenarioCommand::SetVariable {
        variable_name: "score".to_string(),
        value: VariableValue::Int(100),
    });

    // If score >= 50, add 10 to score
    scene.add_command(ScenarioCommand::If {
        condition: Condition::variable("score", CompareOp::GreaterOrEqual, VariableValue::Int(50)),
        then_commands: vec![ScenarioCommand::ModifyVariable {
            variable_name: "score".to_string(),
            operation: VariableOperation::Add { value: 10 },
        }],
        else_commands: vec![],
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute SetVariable
    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // Execute If command (should add 10 to score)
    runtime.execute_current_command().unwrap();

    let var_id = VariableId::new("score");
    assert_eq!(
        runtime.variables().get(&var_id),
        Some(&VariableValue::Int(110))
    );
}
