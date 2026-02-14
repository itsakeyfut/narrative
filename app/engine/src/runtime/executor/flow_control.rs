use super::*;

impl ScenarioRuntime {
    /// Advance to the next command in the current scene
    ///
    /// Returns `true` if advanced successfully, `false` if at the end of the scene
    pub fn advance_command(&mut self) -> bool {
        // Early return if no current scene
        if self.current_scene.is_none() {
            return false;
        }

        // Get current scene data
        let Some(scene) = self.get_current_scene_data() else {
            return false;
        };

        // Check if we can advance to next command
        if self.command_index < scene.command_count() {
            self.command_index = self.command_index.saturating_add(1);
            true
        } else {
            false
        }
    }

    /// Get the current scene data
    pub fn get_current_scene_data(&self) -> Option<&Scene> {
        self.current_scene
            .as_ref()
            .and_then(|id| self.scenario.scenes.get(id.as_str()))
    }

    /// Get the current command
    pub fn get_current_command(&self) -> Option<&ScenarioCommand> {
        self.get_current_scene_data()
            .and_then(|scene| scene.commands.get(self.command_index))
    }

    /// Handle choice selection
    ///
    /// # Arguments
    /// * `choice_index` - The index of the selected choice
    ///
    /// # Returns
    /// Returns (exit_transition, entry_transition) for the scene change
    ///
    /// # Errors
    /// Returns an error if the choice index is invalid
    pub fn select_choice(
        &mut self,
        choice_index: usize,
    ) -> EngineResult<(Option<Transition>, Option<Transition>)> {
        // Get current command and validate it's a choice
        let command = self
            .get_current_command()
            .ok_or_else(|| {
                EngineError::ScenarioExecution("No command at current position".to_string())
            })?
            .clone();

        if let ScenarioCommand::ShowChoice { choice } = command {
            let selected_option = choice.options.get(choice_index).ok_or_else(|| {
                EngineError::ScenarioExecution(format!(
                    "Invalid choice index: {} (max: {})",
                    choice_index,
                    choice.options.len()
                ))
            })?;

            // Set flags associated with this choice
            for flag_name in &selected_option.flags_to_set {
                self.flag_store.set(FlagId::new(flag_name.clone()), true);
            }

            // Jump to the next scene and return transitions
            let (exit_transition, entry_transition) =
                self.jump_to_scene(&SceneId::new(selected_option.next_scene.clone()))?;

            Ok((exit_transition, entry_transition))
        } else {
            Err(EngineError::ScenarioExecution(
                "Current command is not a choice".to_string(),
            ))
        }
    }
}
