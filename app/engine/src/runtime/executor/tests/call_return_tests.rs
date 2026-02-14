//! Tests for Call/Return scene stack management

use super::*;

#[test]
fn test_execute_call_command() {
    let scenario = create_call_return_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute first dialogue
    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // Execute Call command
    let result = runtime.execute_current_command().unwrap();
    assert!(matches!(
        result,
        CommandExecutionResult::SceneChanged { .. }
    ));

    // Should be in subroutine scene
    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("subroutine".to_string()))
    );
    assert_eq!(runtime.command_index(), 0);

    // Stack should have one entry: (main, 2)
    assert_eq!(runtime.scene_stack.len(), 1);
    assert_eq!(runtime.scene_stack[0].0, SceneId::new("main".to_string()));
    assert_eq!(runtime.scene_stack[0].1, 2); // Next command after Call
}

#[test]
fn test_execute_return_command() {
    let scenario = create_call_return_scenario();
    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute to Call command
    runtime.execute_current_command().unwrap();
    runtime.advance_command();
    runtime.execute_current_command().unwrap(); // Execute Call

    // Now in subroutine
    runtime.execute_current_command().unwrap(); // Execute dialogue
    runtime.advance_command();

    // Execute Return command
    let result = runtime.execute_current_command().unwrap();
    assert!(matches!(
        result,
        CommandExecutionResult::SceneChanged { .. }
    ));

    // Should be back in main scene at command index 2
    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("main".to_string()))
    );
    assert_eq!(runtime.command_index(), 2);

    // Stack should be empty
    assert_eq!(runtime.scene_stack.len(), 0);
}

#[test]
fn test_call_with_invalid_return_scene() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "main");

    let mut main_scene = Scene::new("main", "Main");
    main_scene.add_command(ScenarioCommand::Call {
        scene_id: "sub".to_string(),
        return_scene: "nonexistent".to_string(), // Invalid return scene
    });

    let mut sub_scene = Scene::new("sub", "Sub");
    sub_scene.add_command(ScenarioCommand::End);

    scenario.add_scene("main", main_scene);
    scenario.add_scene("sub", sub_scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute Call with invalid return_scene should fail
    let result = runtime.execute_current_command();
    assert!(result.is_err());

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("Return scene"));
        assert!(error_msg.contains("nonexistent"));
    }
}

#[test]
fn test_return_on_empty_stack() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "main");

    let mut main_scene = Scene::new("main", "Main");
    main_scene.add_command(ScenarioCommand::Return); // Return without Call

    scenario.add_scene("main", main_scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute Return on empty stack should fail
    let result = runtime.execute_current_command();
    assert!(result.is_err());

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("scene_stack is empty"));
    }
}

#[test]
fn test_nested_call_sequence() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "a");

    // Scene A: Call B
    let mut scene_a = Scene::new("a", "A");
    scene_a.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("A"),
    });
    scene_a.add_command(ScenarioCommand::Call {
        scene_id: "b".to_string(),
        return_scene: "a".to_string(),
    });
    scene_a.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("A after B"),
    });
    scene_a.add_command(ScenarioCommand::End);

    // Scene B: Call C
    let mut scene_b = Scene::new("b", "B");
    scene_b.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("B"),
    });
    scene_b.add_command(ScenarioCommand::Call {
        scene_id: "c".to_string(),
        return_scene: "b".to_string(),
    });
    scene_b.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("B after C"),
    });
    scene_b.add_command(ScenarioCommand::Return);

    // Scene C: Just return
    let mut scene_c = Scene::new("c", "C");
    scene_c.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("C"),
    });
    scene_c.add_command(ScenarioCommand::Return);

    scenario.add_scene("a", scene_a);
    scenario.add_scene("b", scene_b);
    scenario.add_scene("c", scene_c);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // A: Execute dialogue
    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // A: Call B
    runtime.execute_current_command().unwrap();
    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("b".to_string()))
    );
    assert_eq!(runtime.scene_stack.len(), 1);

    // B: Execute dialogue
    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // B: Call C
    runtime.execute_current_command().unwrap();
    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("c".to_string()))
    );
    assert_eq!(runtime.scene_stack.len(), 2);

    // C: Execute dialogue
    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // C: Return to B
    runtime.execute_current_command().unwrap();
    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("b".to_string()))
    );
    assert_eq!(runtime.command_index(), 2); // After Call in B
    assert_eq!(runtime.scene_stack.len(), 1);

    // B: Execute dialogue after C
    runtime.execute_current_command().unwrap();
    runtime.advance_command();

    // B: Return to A
    runtime.execute_current_command().unwrap();
    assert_eq!(
        runtime.current_scene(),
        Some(&SceneId::new("a".to_string()))
    );
    assert_eq!(runtime.command_index(), 2); // After Call in A
    assert_eq!(runtime.scene_stack.len(), 0);
}

#[test]
fn test_call_stack_depth_limit() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "recursive");

    // Create a recursive scene that calls itself
    let mut recursive_scene = Scene::new("recursive", "Recursive");
    recursive_scene.add_command(ScenarioCommand::Call {
        scene_id: "recursive".to_string(),
        return_scene: "recursive".to_string(),
    });

    scenario.add_scene("recursive", recursive_scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute Call until we hit the depth limit
    // Each Call jumps to the same scene, so we keep executing the same command
    for i in 0..=100 {
        let result = runtime.execute_current_command();
        if result.is_err() {
            // Should fail at depth 100
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("Call stack depth limit exceeded"));
            assert!(error_msg.contains("100"));
            assert_eq!(i, 100); // Should fail on the 101st attempt (100 is the limit)
            return;
        }
        // After successful Call, we're at the start of the same scene again
    }

    panic!("Expected stack depth limit error");
}

#[test]
fn test_call_with_invalid_target_scene() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "main");

    let mut main_scene = Scene::new("main", "Main");
    main_scene.add_command(ScenarioCommand::Call {
        scene_id: "nonexistent".to_string(), // Invalid target scene
        return_scene: "main".to_string(),
    });

    scenario.add_scene("main", main_scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute Call should fail (jump_to_scene will fail)
    let result = runtime.execute_current_command();
    assert!(result.is_err());

    // The error should be about scene not found
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("Scene") && error_msg.contains("nonexistent"));
    }
}
