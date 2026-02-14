use super::*;

impl ScenarioRuntime {
    /// Execute the current command
    ///
    /// This executes the command at the current position and returns
    /// a result indicating whether execution should continue.
    ///
    /// # Returns
    /// * `Ok(CommandExecutionResult)` - Result of command execution
    /// * `Err(EngineError)` - If command execution failed
    pub fn execute_current_command(&mut self) -> EngineResult<CommandExecutionResult> {
        // Get the current command reference (no clone needed)
        let command = self.get_current_command().ok_or_else(|| {
            EngineError::ScenarioExecution("No command at current position".to_string())
        })?;

        // Execute command based on type
        match command {
            // Dialogue - just returns Continue, actual display is handled by the game loop
            ScenarioCommand::Dialogue { .. } => Ok(CommandExecutionResult::Continue),

            // Background commands
            ScenarioCommand::ShowBackground { asset, .. } => {
                tracing::info!("ShowBackground: asset={}", asset.0);
                self.current_background = Some(asset.clone());
                Ok(CommandExecutionResult::Continue)
            }
            ScenarioCommand::HideBackground { .. } => {
                tracing::info!("HideBackground");
                self.current_background = None;
                Ok(CommandExecutionResult::Continue)
            }

            // CG commands
            ScenarioCommand::ShowCG { asset, .. } => {
                tracing::info!("ShowCG: asset={}", asset.0);
                let asset_clone = asset.clone();
                let asset_path = asset.0.clone();
                self.current_cg = Some(asset_clone);

                // Track CG unlock
                if let Some(unlock_data_arc) = &self.unlock_data {
                    if let Some(cg_id) =
                        narrative_core::CgRegistry::extract_cg_id_from_path(&asset_path)
                    {
                        match unlock_data_arc.lock() {
                            Ok(mut data) => {
                                if data.unlock_cg(&cg_id) {
                                    tracing::info!("CG unlocked: {}", cg_id);
                                    // Save unlock data to file
                                    if let Err(e) = data.save_default() {
                                        tracing::warn!("Failed to save unlock data: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to lock unlock_data: {}", e);
                            }
                        }
                    } else {
                        tracing::debug!("Could not extract CG ID from path: {}", asset_path);
                    }
                }

                Ok(CommandExecutionResult::Continue)
            }
            ScenarioCommand::HideCG { .. } => {
                tracing::info!("HideCG");
                self.current_cg = None;
                Ok(CommandExecutionResult::Continue)
            }

            // Character commands
            ScenarioCommand::ShowCharacter {
                character_id,
                sprite,
                position,
                expression: _,
                transition,
            } => {
                tracing::info!(
                    "ShowCharacter: id={}, sprite={}, position={:?}, transition={:?}",
                    character_id,
                    sprite.0,
                    position,
                    transition
                );

                // Store character display information
                self.displayed_characters.insert(
                    character_id.to_string(),
                    DisplayedCharacter {
                        character_id: character_id.to_string(),
                        sprite: sprite.clone(),
                        position: *position,
                        transition: *transition,
                    },
                );
                self.displayed_characters_dirty = true;

                Ok(CommandExecutionResult::Continue)
            }
            ScenarioCommand::HideCharacter { character_id, .. } => {
                tracing::info!("HideCharacter: id={}", character_id);
                // Clone needed to release immutable borrow from get_current_command()
                let char_id = character_id.to_string();
                self.displayed_characters.remove(&char_id);
                self.displayed_characters_dirty = true;
                Ok(CommandExecutionResult::Continue)
            }
            ScenarioCommand::MoveCharacter {
                character_id,
                position,
                duration,
            } => {
                // Clone needed to release immutable borrow from get_current_command()
                let char_id = character_id.to_string();
                let target_position = *position;

                tracing::info!(
                    "MoveCharacter: id={}, position={:?}, duration={}",
                    char_id,
                    target_position,
                    duration
                );

                // Update the position in displayed_characters
                if let Some(character) = self.displayed_characters.get_mut(&char_id) {
                    character.position = target_position;
                    self.displayed_characters_dirty = true;
                    // Note: The actual animation is handled by the app layer (CharacterSpriteElement)
                    // This just updates the target position in the runtime state
                } else {
                    tracing::warn!(
                        "MoveCharacter: Character '{}' not currently displayed, ignoring",
                        char_id
                    );
                }

                // Non-blocking execution: immediately continue to next command
                // The animation will run asynchronously in the rendering layer
                Ok(CommandExecutionResult::Continue)
            }
            ScenarioCommand::ChangeExpression { .. } => Ok(CommandExecutionResult::Continue),
            ScenarioCommand::ChangeSprite {
                character_id,
                sprite,
            } => {
                // Clone needed to release immutable borrow from get_current_command()
                let char_id = character_id.to_string();
                let new_sprite = sprite.clone();

                tracing::info!("ChangeSprite: id={}, new_sprite={}", char_id, new_sprite.0);

                // Update the sprite in displayed_characters
                if let Some(character) = self.displayed_characters.get_mut(&char_id) {
                    character.sprite = new_sprite;
                    self.displayed_characters_dirty = true;
                    tracing::debug!(
                        "Updated sprite for character '{}' to '{}'",
                        char_id,
                        character.sprite.0
                    );
                } else {
                    tracing::warn!(
                        "ChangeSprite: Character '{}' not currently displayed, ignoring",
                        char_id
                    );
                }

                Ok(CommandExecutionResult::Continue)
            }

            // Audio commands
            ScenarioCommand::PlayBgm { .. } => Ok(CommandExecutionResult::Continue),
            ScenarioCommand::StopBgm { .. } => Ok(CommandExecutionResult::Continue),
            ScenarioCommand::PlaySe { .. } => Ok(CommandExecutionResult::Continue),
            ScenarioCommand::PlayVoice { .. } => Ok(CommandExecutionResult::Continue),

            // Choice - returns the choices for the game loop to display
            ScenarioCommand::ShowChoice { choice } => {
                // Filter choices based on conditions
                let available_choices: Vec<ChoiceOption> = choice
                    .options
                    .iter()
                    .filter(|option| {
                        // Check if all conditions for this option are satisfied
                        option
                            .conditions
                            .iter()
                            .all(|cond| self.evaluate_condition(cond))
                    })
                    .cloned()
                    .collect();

                // If no choices are available after filtering, this is an error
                if available_choices.is_empty() {
                    return Err(EngineError::ScenarioExecution(
                        "No available choices after condition filtering. \
                         At least one choice must be available."
                            .to_string(),
                    ));
                }

                Ok(CommandExecutionResult::ShowChoices(available_choices))
            }

            // Jump to another scene
            ScenarioCommand::JumpToScene { scene_id } => {
                let (exit_transition, entry_transition) =
                    self.jump_to_scene(&SceneId::new(scene_id.clone()))?;
                Ok(CommandExecutionResult::SceneChanged {
                    exit_transition,
                    entry_transition,
                })
            }

            // Flag operations
            ScenarioCommand::SetFlag { flag_name, value } => {
                self.flag_store.set(FlagId::new(flag_name.clone()), *value);
                Ok(CommandExecutionResult::Continue)
            }

            // Variable operations
            ScenarioCommand::SetVariable {
                variable_name,
                value,
            } => {
                self.variable_store
                    .set(VariableId::new(variable_name.clone()), value.clone());
                Ok(CommandExecutionResult::Continue)
            }

            ScenarioCommand::ModifyVariable {
                variable_name,
                operation,
            } => {
                let variable_name = variable_name.clone();
                let operation = operation.clone();
                self.apply_variable_modification(&variable_name, &operation)?;
                Ok(CommandExecutionResult::Continue)
            }

            // Wait command
            ScenarioCommand::Wait { duration } => Ok(CommandExecutionResult::Wait(*duration)),

            // Call command: Push to stack and jump to target scene
            ScenarioCommand::Call {
                scene_id,
                return_scene,
            } => {
                // Clone values to avoid borrow issues
                let scene_id = scene_id.clone();
                let return_scene = return_scene.clone();

                // Check stack depth limit
                if self.scene_stack.len() >= MAX_CALL_STACK_DEPTH {
                    return Err(EngineError::ScenarioExecution(format!(
                        "Call stack depth limit exceeded: maximum depth is {}. \
                         This may indicate infinite recursion in your scenario.",
                        MAX_CALL_STACK_DEPTH
                    )));
                }

                // Validate return_scene exists
                if !self.scenario.scenes.contains_key(return_scene.as_str()) {
                    return Err(EngineError::ScenarioExecution(format!(
                        "Return scene '{}' not found in Call command",
                        return_scene
                    )));
                }

                // Save current position (next command after Call)
                if self.current_scene.is_some() {
                    let next_index = self.command_index.saturating_add(1);
                    let return_scene_id = SceneId::new(return_scene);
                    self.scene_stack.push((return_scene_id, next_index));
                }

                // Jump to target scene
                let (exit_transition, entry_transition) =
                    self.jump_to_scene(&SceneId::new(scene_id))?;
                Ok(CommandExecutionResult::SceneChanged {
                    exit_transition,
                    entry_transition,
                })
            }

            // Return command: Pop from stack and return
            ScenarioCommand::Return => {
                // Pop return destination
                let (return_scene, command_index) = self.scene_stack.pop().ok_or_else(|| {
                    EngineError::ScenarioExecution(
                        "Return command executed but scene_stack is empty. \
                         Make sure Call command was used to enter this subroutine."
                            .to_string(),
                    )
                })?;

                // Jump to return scene
                let (exit_transition, entry_transition) = self.jump_to_scene(&return_scene)?;

                // Restore saved command position
                self.command_index = command_index;

                Ok(CommandExecutionResult::SceneChanged {
                    exit_transition,
                    entry_transition,
                })
            }

            // Conditional branching
            ScenarioCommand::If {
                condition,
                then_commands,
                else_commands,
            } => {
                // Evaluate the condition
                let condition_result = self.evaluate_condition(condition);

                // Choose which commands to execute based on condition and clone them
                // We need to clone to avoid borrowing issues
                let commands_to_execute = if condition_result {
                    then_commands.clone()
                } else {
                    else_commands.clone()
                };

                // Execute all commands in the chosen branch
                // Note: These are executed inline, not as a scene jump
                for cmd in &commands_to_execute {
                    // Recursively execute each command
                    // We need to be careful not to advance the command index here
                    // since these are inline commands
                    self.execute_command_inline(cmd)?;
                }

                Ok(CommandExecutionResult::Continue)
            }

            // End scenario
            ScenarioCommand::End => Ok(CommandExecutionResult::End),
            // TODO: Implement additional commands for future phases
            // - Camera: Camera control commands (zoom, pan, shake)
            // - Effect: Visual effects (flash, fade, etc.)
        }
    }
}
