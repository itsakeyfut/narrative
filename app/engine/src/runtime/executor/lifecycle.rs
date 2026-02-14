use super::*;

impl ScenarioRuntime {
    /// Create a new scenario runtime from a scenario
    pub fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            current_scene: None,
            command_index: 0,
            flag_store: FlagStore::default(),
            variable_store: VariableStore::default(),
            read_history: ReadHistory::default(),
            backlog: Backlog::new(),
            scene_stack: Vec::new(),
            displayed_characters: HashMap::new(),
            displayed_characters_dirty: false,
            current_background: None,
            current_cg: None,
            unlock_data: None,
        }
    }

    /// Load a scenario from a TOML file using AssetLoader
    ///
    /// # Arguments
    /// * `path` - Path to the TOML scenario file (absolute or relative)
    ///
    /// # Example
    /// ```no_run
    /// use narrative_engine::runtime::ScenarioRuntime;
    ///
    /// let runtime = ScenarioRuntime::from_toml("assets/scenarios/chapter_01.toml")?;
    /// # Ok::<(), narrative_engine::error::EngineError>(())
    /// ```
    pub fn from_toml<P: AsRef<Path>>(path: P) -> EngineResult<Self> {
        // Use empty base_path since caller typically provides absolute path
        let mut loader = AssetLoader::new("");
        let scenario = loader.load_scenario(path)?.clone();
        Ok(Self::new(scenario))
    }

    /// Start the scenario from the initial scene
    ///
    /// This sets the current scene to the start scene defined in the scenario
    /// and marks it as read in the history.
    pub fn start(&mut self) -> EngineResult<()> {
        let start_scene_id = self.scenario.start_scene.clone();

        // Validate that start scene exists
        if !self.scenario.scenes.contains_key(&start_scene_id) {
            return Err(EngineError::ScenarioExecution(format!(
                "Start scene '{}' not found in scenario",
                start_scene_id
            )));
        }

        let scene_id = SceneId::new(start_scene_id);
        self.current_scene = Some(scene_id.clone());
        self.command_index = 0;

        Ok(())
    }

    /// Jump to a specific scene
    ///
    /// # Arguments
    /// * `scene_id` - The ID of the scene to jump to
    ///
    /// # Returns
    /// Returns (exit_transition, entry_transition) for the scene change
    ///
    /// # Errors
    /// Returns an error if the scene doesn't exist
    pub fn jump_to_scene(
        &mut self,
        scene_id: &SceneId,
    ) -> EngineResult<(Option<Transition>, Option<Transition>)> {
        // Validate scene exists
        if !self.scenario.scenes.contains_key(scene_id.as_str()) {
            return Err(EngineError::ScenarioExecution(format!(
                "Scene '{}' not found",
                scene_id.as_str()
            )));
        }

        // Get exit transition from current scene
        let exit_transition = self
            .current_scene
            .as_ref()
            .and_then(|current_id| self.scenario.scenes.get(current_id.as_str()))
            .and_then(|scene| scene.exit_transition);

        // Get entry transition from new scene
        let entry_transition = self
            .scenario
            .scenes
            .get(scene_id.as_str())
            .and_then(|scene| scene.entry_transition);

        self.current_scene = Some(scene_id.clone());
        self.command_index = 0;

        Ok((exit_transition, entry_transition))
    }
}
