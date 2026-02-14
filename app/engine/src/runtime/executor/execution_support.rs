use super::*;

impl ScenarioRuntime {
    /// Evaluate a condition using current runtime state
    ///
    /// This checks flags and variables to determine if a condition is satisfied.
    pub(super) fn evaluate_condition(&self, condition: &narrative_core::Condition) -> bool {
        let get_flag = |flag_name: &str| {
            let flag_id = narrative_core::FlagId::new(flag_name.to_string());
            self.flag_store.get(&flag_id)
        };

        let get_variable = |variable_name: &str| {
            let variable_id = narrative_core::VariableId::new(variable_name.to_string());
            self.variable_store.get(&variable_id).cloned()
        };

        condition.evaluate(&get_flag, &get_variable)
    }

    /// Apply a variable modification operation
    ///
    /// This method handles the common logic for applying variable operations,
    /// including default value handling for undefined variables and error handling.
    pub(super) fn apply_variable_modification(
        &mut self,
        variable_name: &str,
        operation: &narrative_core::VariableOperation,
    ) -> EngineResult<()> {
        use narrative_core::VariableValue;

        let var_id = VariableId::new(variable_name.to_string());

        // Get current value or use default based on operation type
        let current_value = self
            .variable_store
            .get(&var_id)
            .cloned()
            .unwrap_or_else(|| {
                // Use sensible defaults for operations on undefined variables
                match operation {
                    narrative_core::VariableOperation::Set { value } => value.clone(),
                    narrative_core::VariableOperation::Add { .. }
                    | narrative_core::VariableOperation::Subtract { .. }
                    | narrative_core::VariableOperation::Multiply { .. }
                    | narrative_core::VariableOperation::Divide { .. } => VariableValue::Int(0),
                    narrative_core::VariableOperation::AddFloat { .. }
                    | narrative_core::VariableOperation::SubtractFloat { .. }
                    | narrative_core::VariableOperation::MultiplyFloat { .. }
                    | narrative_core::VariableOperation::DivideFloat { .. } => {
                        VariableValue::Float(0.0)
                    }
                    narrative_core::VariableOperation::Append { .. } => {
                        VariableValue::String(String::new())
                    }
                    narrative_core::VariableOperation::Toggle => VariableValue::Bool(false),
                }
            });

        // Apply operation
        let new_value = operation.apply(&current_value).map_err(|e| {
            EngineError::ScenarioExecution(format!(
                "Failed to apply operation to variable '{}': {}",
                variable_name, e
            ))
        })?;

        // Store the result
        self.variable_store.set(var_id, new_value);
        Ok(())
    }

    /// Execute a command inline (used for If/Else command blocks)
    ///
    /// This executes a command without affecting the main command index.
    /// Only supports commands that can be executed inline (state-changing commands).
    pub(super) fn execute_command_inline(&mut self, command: &ScenarioCommand) -> EngineResult<()> {
        match command {
            // Flag and variable operations
            ScenarioCommand::SetFlag { flag_name, value } => {
                self.flag_store.set(FlagId::new(flag_name.clone()), *value);
                Ok(())
            }
            ScenarioCommand::SetVariable {
                variable_name,
                value,
            } => {
                self.variable_store
                    .set(VariableId::new(variable_name.clone()), value.clone());
                Ok(())
            }

            ScenarioCommand::ModifyVariable {
                variable_name,
                operation,
            } => {
                let variable_name = variable_name.clone();
                let operation = operation.clone();
                self.apply_variable_modification(&variable_name, &operation)
            }

            // Nested If commands
            ScenarioCommand::If {
                condition,
                then_commands,
                else_commands,
            } => {
                let condition_result = self.evaluate_condition(condition);
                let commands_to_execute = if condition_result {
                    then_commands
                } else {
                    else_commands
                };

                for cmd in commands_to_execute {
                    self.execute_command_inline(cmd)?;
                }
                Ok(())
            }

            // Commands that cannot be executed inline should return an error
            ScenarioCommand::JumpToScene { .. }
            | ScenarioCommand::Call { .. }
            | ScenarioCommand::Return
            | ScenarioCommand::End => Err(EngineError::ScenarioExecution(format!(
                "Command {:?} cannot be executed inside If/Else block. \
                 Only SetFlag, SetVariable, ModifyVariable, and nested If commands are allowed.",
                command
            ))),

            // Other commands are silently ignored in inline execution
            // (they would normally affect rendering/audio which doesn't make sense in a conditional)
            _ => {
                tracing::warn!(
                    "Command {:?} in If/Else block has no effect in inline execution",
                    command
                );
                Ok(())
            }
        }
    }
}
