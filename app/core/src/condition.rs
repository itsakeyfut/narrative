use crate::scenario::VariableValue;
use serde::{Deserialize, Serialize};

/// Comparison operator for conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompareOp {
    /// Equal to
    Equal,
    /// Not equal to
    NotEqual,
    /// Greater than
    GreaterThan,
    /// Less than
    LessThan,
    /// Greater than or equal to
    GreaterOrEqual,
    /// Less than or equal to
    LessOrEqual,
}

impl CompareOp {
    /// Tolerance for floating-point comparisons
    /// Using 1e-10 provides reasonable precision for game logic
    const FLOAT_TOLERANCE: f64 = 1e-10;

    /// Check if the comparison is true
    pub fn compare(&self, left: &VariableValue, right: &VariableValue) -> bool {
        use VariableValue::*;
        match (left, right) {
            (Bool(a), Bool(b)) => self.compare_bool(*a, *b),
            (Int(a), Int(b)) => self.compare_int(*a, *b),
            (Float(a), Float(b)) => self.compare_float(*a, *b),
            (String(a), String(b)) => self.compare_string(a, b),
            // Type mismatch - only equality checks work
            _ => matches!(self, Self::NotEqual),
        }
    }

    fn compare_bool(&self, a: bool, b: bool) -> bool {
        match self {
            Self::Equal => a == b,
            Self::NotEqual => a != b,
            _ => false,
        }
    }

    fn compare_int(&self, a: i64, b: i64) -> bool {
        match self {
            Self::Equal => a == b,
            Self::NotEqual => a != b,
            Self::GreaterThan => a > b,
            Self::LessThan => a < b,
            Self::GreaterOrEqual => a >= b,
            Self::LessOrEqual => a <= b,
        }
    }

    fn compare_float(&self, a: f64, b: f64) -> bool {
        let diff = (a - b).abs();
        match self {
            Self::Equal => diff < Self::FLOAT_TOLERANCE,
            Self::NotEqual => diff >= Self::FLOAT_TOLERANCE,
            Self::GreaterThan => a > b + Self::FLOAT_TOLERANCE,
            Self::LessThan => a < b - Self::FLOAT_TOLERANCE,
            Self::GreaterOrEqual => a >= b - Self::FLOAT_TOLERANCE,
            Self::LessOrEqual => a <= b + Self::FLOAT_TOLERANCE,
        }
    }

    fn compare_string(&self, a: &str, b: &str) -> bool {
        match self {
            Self::Equal => a == b,
            Self::NotEqual => a != b,
            Self::GreaterThan => a > b,
            Self::LessThan => a < b,
            Self::GreaterOrEqual => a >= b,
            Self::LessOrEqual => a <= b,
        }
    }
}

/// Condition for branching logic
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Condition {
    /// Flag condition (check if flag is true/false)
    Flag {
        flag_name: String,
        #[serde(default = "default_true")]
        expected: bool,
    },

    /// Variable comparison
    Variable {
        variable_name: String,
        op: CompareOp,
        value: VariableValue,
    },

    /// AND combination of conditions
    And { conditions: Vec<Condition> },

    /// OR combination of conditions
    Or { conditions: Vec<Condition> },

    /// NOT (invert condition)
    Not { condition: Box<Condition> },

    /// Always true
    True,

    /// Always false
    False,
}

impl Condition {
    /// Create a flag condition
    pub fn flag(flag_name: impl Into<String>, expected: bool) -> Self {
        Self::Flag {
            flag_name: flag_name.into(),
            expected,
        }
    }

    /// Create a variable comparison condition
    pub fn variable(variable_name: impl Into<String>, op: CompareOp, value: VariableValue) -> Self {
        Self::Variable {
            variable_name: variable_name.into(),
            op,
            value,
        }
    }

    /// Create an AND condition
    pub fn and(conditions: Vec<Condition>) -> Self {
        Self::And { conditions }
    }

    /// Create an OR condition
    pub fn or(conditions: Vec<Condition>) -> Self {
        Self::Or { conditions }
    }

    /// Create a NOT condition (inverts the given condition)
    pub fn negate(condition: Condition) -> Self {
        Self::Not {
            condition: Box::new(condition),
        }
    }

    /// Evaluate this condition using provided lookups
    ///
    /// # Arguments
    /// * `get_flag` - Function to check if a flag is set
    /// * `get_variable` - Function to get a variable value
    ///
    /// # Returns
    /// `true` if the condition is satisfied, `false` otherwise
    pub fn evaluate(
        &self,
        get_flag: &dyn Fn(&str) -> bool,
        get_variable: &dyn Fn(&str) -> Option<VariableValue>,
    ) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
            Self::Flag {
                flag_name,
                expected,
            } => get_flag(flag_name) == *expected,
            Self::Variable {
                variable_name,
                op,
                value,
            } => {
                if let Some(var_value) = get_variable(variable_name) {
                    op.compare(&var_value, value)
                } else {
                    // Variable doesn't exist - treat as default value based on type
                    let default_value = match value {
                        VariableValue::Bool(_) => VariableValue::Bool(false),
                        VariableValue::Int(_) => VariableValue::Int(0),
                        VariableValue::Float(_) => VariableValue::Float(0.0),
                        VariableValue::String(_) => VariableValue::String(String::new()),
                    };
                    op.compare(&default_value, value)
                }
            }
            Self::And { conditions } => conditions
                .iter()
                .all(|cond| cond.evaluate(get_flag, get_variable)),
            Self::Or { conditions } => conditions
                .iter()
                .any(|cond| cond.evaluate(get_flag, get_variable)),
            Self::Not { condition } => !condition.evaluate(get_flag, get_variable),
        }
    }
}

// Helper function for default true value
fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_op_equal_int() {
        let op = CompareOp::Equal;
        assert!(op.compare(&VariableValue::Int(5), &VariableValue::Int(5)));
        assert!(!op.compare(&VariableValue::Int(5), &VariableValue::Int(6)));
    }

    #[test]
    fn test_compare_op_not_equal_int() {
        let op = CompareOp::NotEqual;
        assert!(op.compare(&VariableValue::Int(5), &VariableValue::Int(6)));
        assert!(!op.compare(&VariableValue::Int(5), &VariableValue::Int(5)));
    }

    #[test]
    fn test_compare_op_greater_than_int() {
        let op = CompareOp::GreaterThan;
        assert!(op.compare(&VariableValue::Int(10), &VariableValue::Int(5)));
        assert!(!op.compare(&VariableValue::Int(5), &VariableValue::Int(10)));
        assert!(!op.compare(&VariableValue::Int(5), &VariableValue::Int(5)));
    }

    #[test]
    fn test_compare_op_less_than_int() {
        let op = CompareOp::LessThan;
        assert!(op.compare(&VariableValue::Int(5), &VariableValue::Int(10)));
        assert!(!op.compare(&VariableValue::Int(10), &VariableValue::Int(5)));
    }

    #[test]
    fn test_compare_op_greater_or_equal_int() {
        let op = CompareOp::GreaterOrEqual;
        assert!(op.compare(&VariableValue::Int(10), &VariableValue::Int(5)));
        assert!(op.compare(&VariableValue::Int(5), &VariableValue::Int(5)));
        assert!(!op.compare(&VariableValue::Int(5), &VariableValue::Int(10)));
    }

    #[test]
    fn test_compare_op_less_or_equal_int() {
        let op = CompareOp::LessOrEqual;
        assert!(op.compare(&VariableValue::Int(5), &VariableValue::Int(10)));
        assert!(op.compare(&VariableValue::Int(5), &VariableValue::Int(5)));
        assert!(!op.compare(&VariableValue::Int(10), &VariableValue::Int(5)));
    }

    #[test]
    fn test_compare_op_bool() {
        let op = CompareOp::Equal;
        assert!(op.compare(&VariableValue::Bool(true), &VariableValue::Bool(true)));
        assert!(!op.compare(&VariableValue::Bool(true), &VariableValue::Bool(false)));
    }

    #[test]
    fn test_compare_op_float() {
        let op = CompareOp::Equal;
        assert!(op.compare(&VariableValue::Float(5.0), &VariableValue::Float(5.0)));
        assert!(!op.compare(&VariableValue::Float(5.0), &VariableValue::Float(6.0)));
    }

    #[test]
    fn test_compare_op_string() {
        let op = CompareOp::Equal;
        assert!(op.compare(
            &VariableValue::String("hello".to_string()),
            &VariableValue::String("hello".to_string())
        ));
        assert!(!op.compare(
            &VariableValue::String("hello".to_string()),
            &VariableValue::String("world".to_string())
        ));
    }

    #[test]
    fn test_compare_op_type_mismatch() {
        let op = CompareOp::Equal;
        assert!(!op.compare(
            &VariableValue::Int(5),
            &VariableValue::String("5".to_string())
        ));

        let op = CompareOp::NotEqual;
        assert!(op.compare(&VariableValue::Int(5), &VariableValue::Bool(true)));
    }

    #[test]
    fn test_condition_flag() {
        let cond = Condition::flag("has_item", true);
        if let Condition::Flag {
            flag_name,
            expected,
        } = cond
        {
            assert_eq!(flag_name, "has_item");
            assert_eq!(expected, true);
        } else {
            panic!("Expected Flag condition");
        }
    }

    #[test]
    fn test_condition_variable() {
        let cond = Condition::variable("score", CompareOp::GreaterThan, VariableValue::Int(100));
        if let Condition::Variable {
            variable_name,
            op,
            value,
        } = cond
        {
            assert_eq!(variable_name, "score");
            assert_eq!(op, CompareOp::GreaterThan);
            assert_eq!(value, VariableValue::Int(100));
        } else {
            panic!("Expected Variable condition");
        }
    }

    #[test]
    fn test_condition_and() {
        let cond1 = Condition::flag("flag1", true);
        let cond2 = Condition::flag("flag2", true);
        let and_cond = Condition::and(vec![cond1, cond2]);

        if let Condition::And { conditions } = and_cond {
            assert_eq!(conditions.len(), 2);
        } else {
            panic!("Expected And condition");
        }
    }

    #[test]
    fn test_condition_or() {
        let cond1 = Condition::flag("flag1", true);
        let cond2 = Condition::flag("flag2", true);
        let or_cond = Condition::or(vec![cond1, cond2]);

        if let Condition::Or { conditions } = or_cond {
            assert_eq!(conditions.len(), 2);
        } else {
            panic!("Expected Or condition");
        }
    }

    #[test]
    fn test_condition_negate() {
        let cond = Condition::flag("flag", true);
        let not_cond = Condition::negate(cond);

        if let Condition::Not { condition } = not_cond {
            if let Condition::Flag {
                flag_name,
                expected,
            } = *condition
            {
                assert_eq!(flag_name, "flag");
                assert_eq!(expected, true);
            } else {
                panic!("Expected Flag condition inside Not");
            }
        } else {
            panic!("Expected Not condition");
        }
    }

    #[test]
    fn test_condition_true_false() {
        let _ = Condition::True;
        let _ = Condition::False;
    }

    #[test]
    fn test_condition_serialization() {
        let cond = Condition::flag("test_flag", true);
        let serialized = serde_json::to_string(&cond).unwrap();
        let deserialized: Condition = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cond, deserialized);
    }

    #[test]
    fn test_condition_complex_serialization() {
        let cond1 = Condition::flag("flag1", true);
        let cond2 = Condition::variable("score", CompareOp::GreaterThan, VariableValue::Int(50));
        let and_cond = Condition::and(vec![cond1, cond2]);

        let serialized = serde_json::to_string(&and_cond).unwrap();
        let deserialized: Condition = serde_json::from_str(&serialized).unwrap();
        assert_eq!(and_cond, deserialized);
    }

    // =============================================================================
    // Condition Evaluation Tests
    // =============================================================================

    #[test]
    fn test_evaluate_true_false() {
        let get_flag = |_: &str| false;
        let get_var = |_: &str| None;

        assert!(Condition::True.evaluate(&get_flag, &get_var));
        assert!(!Condition::False.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_flag_true() {
        let get_flag = |name: &str| name == "has_key";
        let get_var = |_: &str| None;

        let cond = Condition::flag("has_key", true);
        assert!(cond.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_flag_false() {
        let get_flag = |name: &str| name == "has_key";
        let get_var = |_: &str| None;

        let cond = Condition::flag("completed", true);
        assert!(!cond.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_flag_expected_false() {
        let get_flag = |name: &str| name == "has_key";
        let get_var = |_: &str| None;

        // Expects the flag to be false, but it's true
        let cond = Condition::flag("has_key", false);
        assert!(!cond.evaluate(&get_flag, &get_var));

        // Expects the flag to be false, and it is
        let cond2 = Condition::flag("completed", false);
        assert!(cond2.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_variable_equal() {
        let get_flag = |_: &str| false;
        let get_var = |name: &str| {
            if name == "score" {
                Some(VariableValue::Int(100))
            } else {
                None
            }
        };

        let cond = Condition::variable("score", CompareOp::Equal, VariableValue::Int(100));
        assert!(cond.evaluate(&get_flag, &get_var));

        let cond2 = Condition::variable("score", CompareOp::Equal, VariableValue::Int(50));
        assert!(!cond2.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_variable_greater_than() {
        let get_flag = |_: &str| false;
        let get_var = |name: &str| {
            if name == "score" {
                Some(VariableValue::Int(100))
            } else {
                None
            }
        };

        let cond = Condition::variable("score", CompareOp::GreaterThan, VariableValue::Int(50));
        assert!(cond.evaluate(&get_flag, &get_var));

        let cond2 = Condition::variable("score", CompareOp::GreaterThan, VariableValue::Int(150));
        assert!(!cond2.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_variable_not_found() {
        let get_flag = |_: &str| false;
        let get_var = |_: &str| None;

        // Variable doesn't exist, defaults to 0
        let cond = Condition::variable("score", CompareOp::Equal, VariableValue::Int(0));
        assert!(cond.evaluate(&get_flag, &get_var));

        let cond2 = Condition::variable("score", CompareOp::Equal, VariableValue::Int(100));
        assert!(!cond2.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_and_all_true() {
        let get_flag = |name: &str| name == "flag1" || name == "flag2";
        let get_var = |_: &str| None;

        let cond = Condition::and(vec![
            Condition::flag("flag1", true),
            Condition::flag("flag2", true),
        ]);
        assert!(cond.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_and_some_false() {
        let get_flag = |name: &str| name == "flag1";
        let get_var = |_: &str| None;

        let cond = Condition::and(vec![
            Condition::flag("flag1", true),
            Condition::flag("flag2", true),
        ]);
        assert!(!cond.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_or_some_true() {
        let get_flag = |name: &str| name == "flag1";
        let get_var = |_: &str| None;

        let cond = Condition::or(vec![
            Condition::flag("flag1", true),
            Condition::flag("flag2", true),
        ]);
        assert!(cond.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_or_all_false() {
        let get_flag = |_: &str| false;
        let get_var = |_: &str| None;

        let cond = Condition::or(vec![
            Condition::flag("flag1", true),
            Condition::flag("flag2", true),
        ]);
        assert!(!cond.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_not() {
        let get_flag = |name: &str| name == "has_key";
        let get_var = |_: &str| None;

        let cond = Condition::negate(Condition::flag("has_key", true));
        assert!(!cond.evaluate(&get_flag, &get_var));

        let cond2 = Condition::negate(Condition::flag("completed", true));
        assert!(cond2.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_complex_condition() {
        let get_flag = |name: &str| name == "has_key" || name == "door_unlocked";
        let get_var = |name: &str| {
            if name == "score" {
                Some(VariableValue::Int(150))
            } else {
                None
            }
        };

        // (has_key AND score > 100) OR door_unlocked
        let cond = Condition::or(vec![
            Condition::and(vec![
                Condition::flag("has_key", true),
                Condition::variable("score", CompareOp::GreaterThan, VariableValue::Int(100)),
            ]),
            Condition::flag("door_unlocked", true),
        ]);

        assert!(cond.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_complex_condition_with_not() {
        let get_flag = |name: &str| name == "has_key";
        let get_var = |name: &str| {
            if name == "hp" {
                Some(VariableValue::Int(50))
            } else {
                None
            }
        };

        // has_key AND NOT(hp <= 0)
        let cond = Condition::and(vec![
            Condition::flag("has_key", true),
            Condition::negate(Condition::variable(
                "hp",
                CompareOp::LessOrEqual,
                VariableValue::Int(0),
            )),
        ]);

        assert!(cond.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_string_variable() {
        let get_flag = |_: &str| false;
        let get_var = |name: &str| {
            if name == "player_name" {
                Some(VariableValue::String("Alice".to_string()))
            } else {
                None
            }
        };

        let cond = Condition::variable(
            "player_name",
            CompareOp::Equal,
            VariableValue::String("Alice".to_string()),
        );
        assert!(cond.evaluate(&get_flag, &get_var));

        let cond2 = Condition::variable(
            "player_name",
            CompareOp::Equal,
            VariableValue::String("Bob".to_string()),
        );
        assert!(!cond2.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_float_variable() {
        let get_flag = |_: &str| false;
        let get_var = |name: &str| {
            if name == "health_percent" {
                Some(VariableValue::Float(0.75))
            } else {
                None
            }
        };

        let cond = Condition::variable(
            "health_percent",
            CompareOp::GreaterThan,
            VariableValue::Float(0.5),
        );
        assert!(cond.evaluate(&get_flag, &get_var));

        let cond2 = Condition::variable(
            "health_percent",
            CompareOp::LessThan,
            VariableValue::Float(0.5),
        );
        assert!(!cond2.evaluate(&get_flag, &get_var));
    }

    #[test]
    fn test_evaluate_bool_variable() {
        let get_flag = |_: &str| false;
        let get_var = |name: &str| {
            if name == "is_daytime" {
                Some(VariableValue::Bool(true))
            } else {
                None
            }
        };

        let cond = Condition::variable("is_daytime", CompareOp::Equal, VariableValue::Bool(true));
        assert!(cond.evaluate(&get_flag, &get_var));

        let cond2 = Condition::variable(
            "is_daytime",
            CompareOp::NotEqual,
            VariableValue::Bool(false),
        );
        assert!(cond2.evaluate(&get_flag, &get_var));
    }
}
