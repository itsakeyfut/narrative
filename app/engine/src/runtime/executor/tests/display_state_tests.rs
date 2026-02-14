//! Tests for display state management (background, characters, dirty flags) and save/load

use super::*;

#[test]
fn test_save_load_display_state_background() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::ShowBackground {
        asset: AssetRef::from("bg_room"),
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Test"),
    });

    scenario.add_scene("scene1", scene);

    let mut runtime1 = ScenarioRuntime::new(scenario.clone());
    runtime1.start().unwrap();

    // Execute ShowBackground command
    runtime1.execute_current_command().unwrap();
    runtime1.advance_command();

    // Verify background is set
    assert!(runtime1.current_background.is_some());
    assert_eq!(runtime1.current_background.as_ref().unwrap().0, "bg_room");

    // Save
    let save_data = runtime1.to_save_data(1);
    assert_eq!(save_data.current_background.as_ref().unwrap(), "bg_room");

    // Load into new runtime
    let mut runtime2 = ScenarioRuntime::new(scenario);
    runtime2.from_save_data(&save_data).unwrap();

    // Verify background was restored
    assert!(runtime2.current_background.is_some());
    assert_eq!(runtime2.current_background.as_ref().unwrap().0, "bg_room");
}

#[test]
fn test_save_load_display_state_characters() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::ShowCharacter {
        character_id: "alice".to_string(),
        sprite: AssetRef::from("alice_happy"),
        position: CharacterPosition::Left,
        expression: None,
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::ShowCharacter {
        character_id: "bob".to_string(),
        sprite: AssetRef::from("bob_normal"),
        position: CharacterPosition::Right,
        expression: None,
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Test"),
    });

    scenario.add_scene("scene1", scene);

    let mut runtime1 = ScenarioRuntime::new(scenario.clone());
    runtime1.start().unwrap();

    // Execute ShowCharacter commands
    runtime1.execute_current_command().unwrap(); // Alice
    runtime1.advance_command();
    runtime1.execute_current_command().unwrap(); // Bob
    runtime1.advance_command();

    // Verify characters are displayed
    assert_eq!(runtime1.displayed_characters.len(), 2);
    assert!(runtime1.displayed_characters.contains_key("alice"));
    assert!(runtime1.displayed_characters.contains_key("bob"));

    // Save
    let save_data = runtime1.to_save_data(1);
    assert_eq!(save_data.displayed_characters.len(), 2);
    assert_eq!(
        save_data.displayed_characters.get("alice").unwrap().sprite,
        "alice_happy"
    );
    assert_eq!(
        save_data
            .displayed_characters
            .get("alice")
            .unwrap()
            .position,
        CharacterPosition::Left
    );
    assert_eq!(
        save_data.displayed_characters.get("bob").unwrap().sprite,
        "bob_normal"
    );
    assert_eq!(
        save_data.displayed_characters.get("bob").unwrap().position,
        CharacterPosition::Right
    );

    // Load into new runtime
    let mut runtime2 = ScenarioRuntime::new(scenario);
    runtime2.from_save_data(&save_data).unwrap();

    // Verify characters were restored
    assert_eq!(runtime2.displayed_characters.len(), 2);

    let alice = runtime2.displayed_characters.get("alice").unwrap();
    assert_eq!(alice.character_id, "alice");
    assert_eq!(alice.sprite.0, "alice_happy");
    assert_eq!(alice.position, CharacterPosition::Left);

    let bob = runtime2.displayed_characters.get("bob").unwrap();
    assert_eq!(bob.character_id, "bob");
    assert_eq!(bob.sprite.0, "bob_normal");
    assert_eq!(bob.position, CharacterPosition::Right);
}

#[test]
fn test_save_load_display_state_full_scene() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::ShowBackground {
        asset: AssetRef::from("bg_classroom"),
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::ShowCharacter {
        character_id: "teacher".to_string(),
        sprite: AssetRef::from("teacher_normal"),
        position: CharacterPosition::Center,
        expression: None,
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Welcome to class!"),
    });

    scenario.add_scene("scene1", scene);

    let mut runtime1 = ScenarioRuntime::new(scenario.clone());
    runtime1.start().unwrap();

    // Execute all commands to set up the scene
    runtime1.execute_current_command().unwrap(); // ShowBackground
    runtime1.advance_command();
    runtime1.execute_current_command().unwrap(); // ShowCharacter
    runtime1.advance_command();

    // Save
    let save_data = runtime1.to_save_data(1);

    // Load into new runtime
    let mut runtime2 = ScenarioRuntime::new(scenario);
    runtime2.from_save_data(&save_data).unwrap();

    // Verify complete display state was restored
    assert!(runtime2.current_background.is_some());
    assert_eq!(
        runtime2.current_background.as_ref().unwrap().0,
        "bg_classroom"
    );

    assert_eq!(runtime2.displayed_characters.len(), 1);
    let teacher = runtime2.displayed_characters.get("teacher").unwrap();
    assert_eq!(teacher.sprite.0, "teacher_normal");
    assert_eq!(teacher.position, CharacterPosition::Center);
}

#[test]
fn test_save_load_empty_display_state() {
    let scenario = create_test_scenario();
    let mut runtime1 = ScenarioRuntime::new(scenario.clone());
    runtime1.start().unwrap();

    // Don't execute any display commands - display state should be empty

    // Save
    let save_data = runtime1.to_save_data(1);
    assert!(save_data.current_background.is_none());
    assert!(save_data.displayed_characters.is_empty());

    // Load into new runtime
    let mut runtime2 = ScenarioRuntime::new(scenario);
    runtime2.from_save_data(&save_data).unwrap();

    // Verify empty display state was preserved
    assert!(runtime2.current_background.is_none());
    assert!(runtime2.displayed_characters.is_empty());
}

#[test]
fn test_background_hide_and_restore() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::ShowBackground {
        asset: AssetRef::from("bg_room"),
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::HideBackground {
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Test"),
    });

    scenario.add_scene("scene1", scene);

    let mut runtime1 = ScenarioRuntime::new(scenario.clone());
    runtime1.start().unwrap();

    // Show background
    runtime1.execute_current_command().unwrap();
    runtime1.advance_command();
    assert!(runtime1.current_background.is_some());

    // Hide background
    runtime1.execute_current_command().unwrap();
    runtime1.advance_command();
    assert!(runtime1.current_background.is_none());

    // Save
    let save_data = runtime1.to_save_data(1);
    assert!(save_data.current_background.is_none());

    // Load
    let mut runtime2 = ScenarioRuntime::new(scenario);
    runtime2.from_save_data(&save_data).unwrap();
    assert!(runtime2.current_background.is_none());
}

#[test]
fn test_dirty_flag_show_character() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::ShowCharacter {
        character_id: "alice".to_string(),
        sprite: AssetRef::from("alice_normal"),
        position: CharacterPosition::Center,
        expression: None,
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Test"),
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Initially not dirty
    assert!(!runtime.displayed_characters_changed());

    // Execute ShowCharacter
    runtime.execute_current_command().unwrap();

    // Should be dirty now
    assert!(runtime.displayed_characters_changed());

    // Second call should return false (consumed)
    assert!(!runtime.displayed_characters_changed());
}

#[test]
fn test_dirty_flag_hide_character() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::ShowCharacter {
        character_id: "bob".to_string(),
        sprite: AssetRef::from("bob_normal"),
        position: CharacterPosition::Left,
        expression: None,
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::HideCharacter {
        character_id: "bob".to_string(),
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Test"),
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Show character first
    runtime.execute_current_command().unwrap();
    runtime.displayed_characters_changed(); // Consume flag
    runtime.advance_command();

    // Hide character
    runtime.execute_current_command().unwrap();

    // Should be dirty
    assert!(runtime.displayed_characters_changed());
    assert!(!runtime.displayed_characters_changed()); // Consumed
}

#[test]
fn test_dirty_flag_move_character() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::ShowCharacter {
        character_id: "charlie".to_string(),
        sprite: AssetRef::from("charlie_normal"),
        position: CharacterPosition::Left,
        expression: None,
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::MoveCharacter {
        character_id: "charlie".to_string(),
        position: CharacterPosition::Right,
        duration: 1.0,
    });
    scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Test"),
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Show character
    runtime.execute_current_command().unwrap();
    runtime.displayed_characters_changed(); // Consume flag
    runtime.advance_command();

    // Move character
    runtime.execute_current_command().unwrap();

    // Should be dirty
    assert!(runtime.displayed_characters_changed());
    assert!(!runtime.displayed_characters_changed()); // Consumed
}

#[test]
fn test_dirty_flag_change_sprite() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::ShowCharacter {
        character_id: "dave".to_string(),
        sprite: AssetRef::from("dave_normal"),
        position: CharacterPosition::Center,
        expression: None,
        transition: Transition::instant(),
    });
    scene.add_command(ScenarioCommand::ChangeSprite {
        character_id: "dave".to_string(),
        sprite: AssetRef::from("dave_smile"),
    });
    scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Test"),
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Show character
    runtime.execute_current_command().unwrap();
    runtime.displayed_characters_changed(); // Consume flag
    runtime.advance_command();

    // Change sprite
    runtime.execute_current_command().unwrap();

    // Should be dirty
    assert!(runtime.displayed_characters_changed());
    assert!(!runtime.displayed_characters_changed()); // Consumed
}

#[test]
fn test_dirty_flag_unchanged() {
    let metadata = ScenarioMetadata::new("test", "Test");
    let mut scenario = Scenario::new(metadata, "scene1");

    let mut scene = Scene::new("scene1", "Scene 1");
    scene.add_command(ScenarioCommand::Dialogue {
        dialogue: Dialogue::narrator("Test"),
    });
    scene.add_command(ScenarioCommand::ShowBackground {
        asset: AssetRef::from("bg_room"),
        transition: Transition::instant(),
    });

    scenario.add_scene("scene1", scene);

    let mut runtime = ScenarioRuntime::new(scenario);
    runtime.start().unwrap();

    // Execute non-character commands
    runtime.execute_current_command().unwrap(); // Dialogue
    assert!(!runtime.displayed_characters_changed());

    runtime.advance_command();
    runtime.execute_current_command().unwrap(); // ShowBackground
    assert!(!runtime.displayed_characters_changed());
}
