//! Tests for choice system and conditional choice filtering

use super::*;
use narrative_core::Condition;

#[test]
fn test_select_choice() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene1 = Scene::new("scene1", "Scene 1");
    let choice = Choice::new(vec![
        ChoiceOption::new("Option 1", "scene2"),
        ChoiceOption::new("Option 2", "scene3"),
    ]);
    scene1.add_command(ScenarioCommand::ShowChoice { choice });
    scenario.add_scene("scene1", scene1);

    let scene2 = Scene::new("scene2", "Scene 2");
    scenario.add_scene("scene2", scene2);

    let scene3 = Scene::new("scene3", "Scene 3");
    scenario.add_scene("scene3", scene3);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    runtime.select_choice(0).unwrap();
    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("scene2".to_string()))
    );
}

#[test]
fn test_select_invalid_choice() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene1 = Scene::new("scene1", "Scene 1");
    let choice = Choice::new(vec![ChoiceOption::new("Option 1", "scene2")]);
    scene1.add_command(ScenarioCommand::ShowChoice { choice });
    scenario.add_scene("scene1", scene1);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    let result = runtime.select_choice(5);
    assert!(result.is_err());
}

#[test]
fn test_conditional_choice_filtering() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene1 = Scene::new("scene1", "Scene 1");

    // Set has_key to true
    scene1.add_command(ScenarioCommand::SetFlag {
        flag_name: "has_key".to_string(),
        value: true,
    });

    // Show choices: one requires has_key, one doesn't
    let choice = Choice::new(vec![
        ChoiceOption::new("Use key", "scene_unlock")
            .with_condition(Condition::flag("has_key", true)),
        ChoiceOption::new("Break door", "scene_break"),
    ]);
    scene1.add_command(ScenarioCommand::ShowChoice { choice });

    let scene_unlock = Scene::new("scene_unlock", "Unlocked");
    let scene_break = Scene::new("scene_break", "Broken");

    scenario.add_scene("scene1", scene1);
    scenario.add_scene("scene_unlock", scene_unlock);
    scenario.add_scene("scene_break", scene_break);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute SetFlag
    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // Execute ShowChoice - should filter and return both options
    let result = runtime.execute_current_command().unwrap();
    if let CommandExecutionResult::ShowChoices(choices) = result {
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].text, "Use key");
        assert_eq!(choices[1].text, "Break door");
    } else {
        panic!("Expected ShowChoices result");
    }
}

#[test]
fn test_conditional_choice_filtering_exclude() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene1 = Scene::new("scene1", "Scene 1");

    // Don't set has_key, so first choice should be filtered out
    let choice = Choice::new(vec![
        ChoiceOption::new("Use key", "scene_unlock")
            .with_condition(Condition::flag("has_key", true)),
        ChoiceOption::new("Break door", "scene_break"),
    ]);
    scene1.add_command(ScenarioCommand::ShowChoice { choice });

    let scene_unlock = Scene::new("scene_unlock", "Unlocked");
    let scene_break = Scene::new("scene_break", "Broken");

    scenario.add_scene("scene1", scene1);
    scenario.add_scene("scene_unlock", scene_unlock);
    scenario.add_scene("scene_break", scene_break);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute ShowChoice - should filter and return only the second option
    let result = runtime.execute_current_command().unwrap();
    if let CommandExecutionResult::ShowChoices(choices) = result {
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].text, "Break door");
    } else {
        panic!("Expected ShowChoices result");
    }
}

#[test]
fn test_conditional_choice_no_available_choices() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene1 = Scene::new("scene1", "Scene 1");

    // All choices require has_key, which is not set
    let choice = Choice::new(vec![
        ChoiceOption::new("Option 1", "scene1").with_condition(Condition::flag("has_key", true)),
        ChoiceOption::new("Option 2", "scene2").with_condition(Condition::flag("has_key", true)),
    ]);
    scene1.add_command(ScenarioCommand::ShowChoice { choice });

    scenario.add_scene("scene1", scene1);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute ShowChoice - should error because no choices are available
    let result = runtime.execute_current_command();
    assert!(result.is_err());

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("No available choices"));
    }
}
