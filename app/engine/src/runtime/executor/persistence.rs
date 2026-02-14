use super::*;

impl ScenarioRuntime {
    /// Create a SaveData from the current runtime state
    ///
    /// This captures the current runtime state for persistence.
    /// Note: This does not include the timestamp and play_time, which should be set by the caller.
    pub fn to_save_data(&self, slot: usize) -> crate::save::SaveData {
        use crate::save::{SAVE_VERSION, SaveData, SavedCharacterDisplay};

        // Convert displayed characters to save format
        let displayed_characters: HashMap<String, SavedCharacterDisplay> = self
            .displayed_characters
            .iter()
            .map(|(id, char_display)| {
                (
                    id.clone(),
                    SavedCharacterDisplay {
                        character_id: char_display.character_id.clone(),
                        sprite: char_display.sprite.0.to_string(),
                        position: char_display.position,
                    },
                )
            })
            .collect();

        SaveData {
            version: SAVE_VERSION,
            slot,
            timestamp: 0,      // Caller should set this
            play_time_secs: 0, // Caller should set this
            current_scene: self
                .current_scene
                .clone()
                .unwrap_or_else(|| SceneId::new("")),
            command_index: self.command_index,
            flags: self.flag_store.to_save_format(),
            variables: self.variable_store.to_save_format(),
            read_scenes: vec![], // Deprecated field
            read_history: self.read_history.clone(),
            scene_stack: self.scene_stack.clone(),
            current_background: self.current_background.as_ref().map(|bg| bg.0.to_string()),
            current_cg: self.current_cg.as_ref().map(|cg| cg.0.to_string()),
            displayed_characters,
            thumbnail_path: None, // Thumbnail will be added later during save
        }
    }

    /// Load runtime state from SaveData
    ///
    /// This restores the runtime state from saved data.
    /// The scenario must already be loaded before calling this.
    pub fn from_save_data(&mut self, save_data: &crate::save::SaveData) -> EngineResult<()> {
        // Validate that the save data's current scene exists
        if !save_data.current_scene.as_str().is_empty()
            && !self
                .scenario
                .scenes
                .contains_key(save_data.current_scene.as_str())
        {
            return Err(EngineError::ScenarioExecution(format!(
                "Save data references non-existent scene: {}",
                save_data.current_scene.as_str()
            )));
        }

        // Restore runtime state
        self.current_scene = if save_data.current_scene.as_str().is_empty() {
            None
        } else {
            Some(save_data.current_scene.clone())
        };
        self.command_index = save_data.command_index;

        // Restore flags
        self.flag_store = FlagStore::from_save_format(&save_data.flags);

        // Restore variables
        self.variable_store = VariableStore::from_save_format(&save_data.variables);

        // Restore read history
        self.read_history = save_data.read_history.clone();

        // Restore scene stack
        self.scene_stack = save_data.scene_stack.clone();

        // Restore display state: background
        self.current_background = save_data
            .current_background
            .as_ref()
            .map(|bg| AssetRef::from(bg.clone()));

        // Restore display state: CG (event graphics)
        self.current_cg = save_data
            .current_cg
            .as_ref()
            .map(|cg| AssetRef::from(cg.clone()));

        // Restore display state: displayed characters
        self.displayed_characters = save_data
            .displayed_characters
            .iter()
            .map(|(id, saved_char)| {
                (
                    id.clone(),
                    DisplayedCharacter {
                        character_id: saved_char.character_id.clone(),
                        sprite: AssetRef::from(saved_char.sprite.clone()),
                        position: saved_char.position,
                        transition: Transition::instant(), // Use instant transition on load
                    },
                )
            })
            .collect();

        Ok(())
    }
}
