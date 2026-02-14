use super::*;

impl ScenarioRuntime {
    /// Check if the scenario has ended
    pub fn is_ended(&self) -> bool {
        if let Some(command) = self.get_current_command() {
            matches!(command, ScenarioCommand::End)
        } else {
            // If there's no current command, we're at the end
            true
        }
    }

    /// Get the current scene ID
    pub fn current_scene(&self) -> Option<&SceneId> {
        self.current_scene.as_ref()
    }

    /// Get the current background asset
    pub fn current_background(&self) -> Option<&AssetRef> {
        self.current_background.as_ref()
    }

    /// Get the current CG (event graphics) asset
    pub fn current_cg(&self) -> Option<&AssetRef> {
        self.current_cg.as_ref()
    }

    /// Set the unlock data reference
    pub fn set_unlock_data(&mut self, unlock_data: Arc<Mutex<UnlockData>>) {
        self.unlock_data = Some(unlock_data);
    }

    /// Get the current command index
    pub fn command_index(&self) -> usize {
        self.command_index
    }

    /// Get reference to the scenario
    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }

    /// Get reference to flag store
    pub fn flags(&self) -> &FlagStore {
        &self.flag_store
    }

    /// Get mutable reference to flag store
    pub fn flags_mut(&mut self) -> &mut FlagStore {
        &mut self.flag_store
    }

    /// Get reference to variable store
    pub fn variables(&self) -> &VariableStore {
        &self.variable_store
    }

    /// Get mutable reference to variable store
    pub fn variables_mut(&mut self) -> &mut VariableStore {
        &mut self.variable_store
    }

    /// Get reference to read history
    pub fn read_history(&self) -> &ReadHistory {
        &self.read_history
    }

    /// Get mutable reference to read history
    pub fn read_history_mut(&mut self) -> &mut ReadHistory {
        &mut self.read_history
    }

    /// Get reference to backlog
    pub fn backlog(&self) -> &Backlog {
        &self.backlog
    }

    /// Get mutable reference to backlog
    pub fn backlog_mut(&mut self) -> &mut Backlog {
        &mut self.backlog
    }

    /// Add a dialogue to the backlog
    ///
    /// This should be called when a dialogue is displayed to the player.
    pub fn add_to_backlog(
        &mut self,
        scene_id: SceneId,
        command_index: usize,
        speaker: narrative_core::Speaker,
        text: impl Into<String>,
    ) {
        let entry = BacklogEntry::new(scene_id, command_index, speaker, text);
        self.backlog.add_entry(entry);
    }
}
