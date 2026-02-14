use crate::condition::Condition;
use serde::{Deserialize, Serialize};

/// Choice option in a branching scenario
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChoiceOption {
    /// Option text displayed to the player
    pub text: String,
    /// Scene ID to jump to if this option is selected
    pub next_scene: String,
    /// Optional conditions that must be met for this option to be available
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
    /// Flags to set when this option is selected
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flags_to_set: Vec<String>,
}

impl ChoiceOption {
    /// Create a new choice option
    pub fn new(text: impl Into<String>, next_scene: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            next_scene: next_scene.into(),
            conditions: Vec::new(),
            flags_to_set: Vec::new(),
        }
    }

    /// Add a condition to this choice
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Add a flag to set when selected
    pub fn with_flag(mut self, flag: impl Into<String>) -> Self {
        self.flags_to_set.push(flag.into());
        self
    }

    /// Check if this choice is available based on conditions
    pub fn is_available(&self, check_condition: impl Fn(&Condition) -> bool) -> bool {
        self.conditions.iter().all(check_condition)
    }
}

/// Choice group presented to the player
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    /// Optional prompt text before the choices
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Available choice options
    pub options: Vec<ChoiceOption>,
}

impl Choice {
    /// Create a new choice group
    pub fn new(options: Vec<ChoiceOption>) -> Self {
        Self {
            prompt: None,
            options,
        }
    }

    /// Create a choice with a prompt
    pub fn with_prompt(prompt: impl Into<String>, options: Vec<ChoiceOption>) -> Self {
        Self {
            prompt: Some(prompt.into()),
            options,
        }
    }

    /// Add an option to this choice
    pub fn add_option(&mut self, option: ChoiceOption) {
        self.options.push(option);
    }

    /// Get available options based on conditions
    pub fn available_options(
        &self,
        check_condition: impl Fn(&Condition) -> bool,
    ) -> Vec<&ChoiceOption> {
        self.options
            .iter()
            .filter(|opt| opt.is_available(&check_condition))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choice_option_new() {
        let option = ChoiceOption::new("Go left", "scene_left");
        assert_eq!(option.text, "Go left");
        assert_eq!(option.next_scene, "scene_left");
        assert!(option.conditions.is_empty());
        assert!(option.flags_to_set.is_empty());
    }

    #[test]
    fn test_choice_option_with_condition() {
        let condition = Condition::flag("has_key", true);
        let option = ChoiceOption::new("Open door", "scene_door").with_condition(condition.clone());
        assert_eq!(option.conditions.len(), 1);
        assert_eq!(option.conditions[0], condition);
    }

    #[test]
    fn test_choice_option_with_flag() {
        let option = ChoiceOption::new("Take item", "scene_item").with_flag("item_taken");
        assert_eq!(option.flags_to_set.len(), 1);
        assert_eq!(option.flags_to_set[0], "item_taken");
    }

    #[test]
    fn test_choice_option_builder_chain() {
        let condition1 = Condition::flag("flag1", true);
        let condition2 = Condition::flag("flag2", false);

        let option = ChoiceOption::new("Complex choice", "scene_complex")
            .with_condition(condition1)
            .with_condition(condition2)
            .with_flag("choice_made")
            .with_flag("points_earned");

        assert_eq!(option.conditions.len(), 2);
        assert_eq!(option.flags_to_set.len(), 2);
    }

    #[test]
    fn test_choice_option_is_available_no_conditions() {
        let option = ChoiceOption::new("Simple choice", "scene_simple");
        let is_available = option.is_available(|_| true);
        assert!(is_available);
    }

    #[test]
    fn test_choice_option_is_available_all_true() {
        let option = ChoiceOption::new("Choice", "scene_6")
            .with_condition(Condition::flag("flag1", true))
            .with_condition(Condition::flag("flag2", true));

        let is_available = option.is_available(|_| true);
        assert!(is_available);
    }

    #[test]
    fn test_choice_option_is_available_some_false() {
        let option = ChoiceOption::new("Choice", "scene_7")
            .with_condition(Condition::flag("flag1", true))
            .with_condition(Condition::flag("flag2", true));

        // If any condition returns false, the option is not available
        let is_available = option.is_available(|_| false);
        assert!(!is_available);
    }

    #[test]
    fn test_choice_option_serialization() {
        let option = ChoiceOption::new("Test choice", "scene_test");
        let serialized = serde_json::to_string(&option).unwrap();
        let deserialized: ChoiceOption = serde_json::from_str(&serialized).unwrap();
        assert_eq!(option, deserialized);
    }

    #[test]
    fn test_choice_new() {
        let option1 = ChoiceOption::new("Option 1", "scene_1");
        let option2 = ChoiceOption::new("Option 2", "scene_2");
        let choice = Choice::new(vec![option1, option2]);

        assert_eq!(choice.prompt, None);
        assert_eq!(choice.options.len(), 2);
    }

    #[test]
    fn test_choice_with_prompt() {
        let option = ChoiceOption::new("Yes", "scene_yes");
        let choice = Choice::with_prompt("What will you do?", vec![option]);

        assert_eq!(choice.prompt, Some("What will you do?".to_string()));
        assert_eq!(choice.options.len(), 1);
    }

    #[test]
    fn test_choice_add_option() {
        let mut choice = Choice::new(vec![]);
        assert_eq!(choice.options.len(), 0);

        let option = ChoiceOption::new("New option", "scene_new");
        choice.add_option(option);

        assert_eq!(choice.options.len(), 1);
    }

    #[test]
    fn test_choice_available_options_all_available() {
        let option1 = ChoiceOption::new("Option 1", "scene_opt1");
        let option2 = ChoiceOption::new("Option 2", "scene_opt2");
        let choice = Choice::new(vec![option1, option2]);

        let available = choice.available_options(|_| true);
        assert_eq!(available.len(), 2);
    }

    #[test]
    fn test_choice_available_options_some_unavailable() {
        let option1 = ChoiceOption::new("Always available", "scene_available");
        let option2 = ChoiceOption::new("Conditional", "scene_conditional")
            .with_condition(Condition::flag("special", true));
        let choice = Choice::new(vec![option1, option2]);

        let available = choice.available_options(|_| false);
        assert_eq!(available.len(), 1); // Only option1 is available
        assert_eq!(available[0].text, "Always available");
    }

    #[test]
    fn test_choice_available_options_none_available() {
        let option1 = ChoiceOption::new("Conditional 1", "scene_cond1")
            .with_condition(Condition::flag("flag1", true));
        let option2 = ChoiceOption::new("Conditional 2", "scene_cond2")
            .with_condition(Condition::flag("flag2", true));
        let choice = Choice::new(vec![option1, option2]);

        let available = choice.available_options(|_| false);
        assert_eq!(available.len(), 0);
    }

    #[test]
    fn test_choice_serialization() {
        let option = ChoiceOption::new("Test", "scene_test");
        let choice = Choice::with_prompt("Test prompt", vec![option]);

        let serialized = serde_json::to_string(&choice).unwrap();
        let deserialized: Choice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(choice, deserialized);
    }
}
