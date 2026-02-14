use crate::scenario::VariableValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during variable operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum VariableError {
    /// Type mismatch - operation cannot be applied to this variable type
    #[error("Operation {operation} cannot be applied to value type {value_type}")]
    TypeMismatch {
        operation: String,
        value_type: String,
    },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,
}

/// Variable operation for modifying variables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum VariableOperation {
    /// Set variable to a value
    Set { value: VariableValue },

    /// Add to integer variable
    Add { value: i64 },

    /// Subtract from integer variable
    Subtract { value: i64 },

    /// Multiply integer variable
    Multiply { value: i64 },

    /// Divide integer variable
    Divide { value: i64 },

    /// Add to floating-point variable
    AddFloat { value: f64 },

    /// Subtract from floating-point variable
    SubtractFloat { value: f64 },

    /// Multiply floating-point variable
    MultiplyFloat { value: f64 },

    /// Divide floating-point variable
    DivideFloat { value: f64 },

    /// Append to string variable
    Append { text: String },

    /// Toggle boolean variable
    Toggle,
}

impl VariableOperation {
    /// Apply this operation to a variable value
    pub fn apply(&self, current: &VariableValue) -> Result<VariableValue, VariableError> {
        match (self, current) {
            (Self::Set { value }, _) => Ok(value.clone()),

            (Self::Add { value }, VariableValue::Int(n)) => {
                Ok(VariableValue::Int(n.saturating_add(*value)))
            }

            (Self::Subtract { value }, VariableValue::Int(n)) => {
                Ok(VariableValue::Int(n.saturating_sub(*value)))
            }

            (Self::Multiply { value }, VariableValue::Int(n)) => {
                Ok(VariableValue::Int(n.saturating_mul(*value)))
            }

            (Self::Divide { value }, VariableValue::Int(n)) => {
                if *value == 0 {
                    Err(VariableError::DivisionByZero)
                } else {
                    Ok(VariableValue::Int(n / value))
                }
            }

            (Self::AddFloat { value }, VariableValue::Float(f)) => {
                Ok(VariableValue::Float(f + value))
            }

            (Self::SubtractFloat { value }, VariableValue::Float(f)) => {
                Ok(VariableValue::Float(f - value))
            }

            (Self::MultiplyFloat { value }, VariableValue::Float(f)) => {
                Ok(VariableValue::Float(f * value))
            }

            (Self::DivideFloat { value }, VariableValue::Float(f)) => {
                if *value == 0.0 {
                    Err(VariableError::DivisionByZero)
                } else {
                    Ok(VariableValue::Float(f / value))
                }
            }

            (Self::Append { text }, VariableValue::String(s)) => {
                let mut result = s.clone();
                result.push_str(text);
                Ok(VariableValue::String(result))
            }

            (Self::Toggle, VariableValue::Bool(b)) => Ok(VariableValue::Bool(!b)),

            _ => Err(VariableError::TypeMismatch {
                operation: format!("{:?}", self),
                value_type: format!("{:?}", current),
            }),
        }
    }
}

/// Variable store entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    /// Variable name
    pub name: String,
    /// Current value
    pub value: VariableValue,
}

impl Variable {
    /// Create a new variable
    pub fn new(name: impl Into<String>, value: VariableValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Apply an operation to this variable
    pub fn apply_operation(&mut self, operation: &VariableOperation) -> Result<(), VariableError> {
        self.value = operation.apply(&self.value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_operation_set() {
        let op = VariableOperation::Set {
            value: VariableValue::Int(42),
        };
        let result = op.apply(&VariableValue::Int(0)).unwrap();
        assert_eq!(result, VariableValue::Int(42));
    }

    #[test]
    fn test_variable_operation_add() {
        let op = VariableOperation::Add { value: 10 };
        let result = op.apply(&VariableValue::Int(5)).unwrap();
        assert_eq!(result, VariableValue::Int(15));
    }

    #[test]
    fn test_variable_operation_add_saturating() {
        let op = VariableOperation::Add { value: i64::MAX };
        let result = op.apply(&VariableValue::Int(100)).unwrap();
        assert_eq!(result, VariableValue::Int(i64::MAX));
    }

    #[test]
    fn test_variable_operation_subtract() {
        let op = VariableOperation::Subtract { value: 3 };
        let result = op.apply(&VariableValue::Int(10)).unwrap();
        assert_eq!(result, VariableValue::Int(7));
    }

    #[test]
    fn test_variable_operation_subtract_saturating() {
        let op = VariableOperation::Subtract { value: 200 };
        let result = op.apply(&VariableValue::Int(i64::MIN + 150)).unwrap();
        // Should saturate at i64::MIN
        assert_eq!(result, VariableValue::Int(i64::MIN));
    }

    #[test]
    fn test_variable_operation_multiply() {
        let op = VariableOperation::Multiply { value: 3 };
        let result = op.apply(&VariableValue::Int(5)).unwrap();
        assert_eq!(result, VariableValue::Int(15));
    }

    #[test]
    fn test_variable_operation_divide() {
        let op = VariableOperation::Divide { value: 2 };
        let result = op.apply(&VariableValue::Int(10)).unwrap();
        assert_eq!(result, VariableValue::Int(5));
    }

    #[test]
    fn test_variable_operation_divide_by_zero() {
        let op = VariableOperation::Divide { value: 0 };
        let result = op.apply(&VariableValue::Int(10));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), VariableError::DivisionByZero);
    }

    #[test]
    fn test_variable_operation_append() {
        let op = VariableOperation::Append {
            text: " world".to_string(),
        };
        let result = op
            .apply(&VariableValue::String("hello".to_string()))
            .unwrap();
        assert_eq!(result, VariableValue::String("hello world".to_string()));
    }

    #[test]
    fn test_variable_operation_toggle() {
        let op = VariableOperation::Toggle;
        let result = op.apply(&VariableValue::Bool(true)).unwrap();
        assert_eq!(result, VariableValue::Bool(false));

        let result = op.apply(&VariableValue::Bool(false)).unwrap();
        assert_eq!(result, VariableValue::Bool(true));
    }

    #[test]
    fn test_variable_operation_type_mismatch_add() {
        let op = VariableOperation::Add { value: 5 };
        let result = op.apply(&VariableValue::String("hello".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_operation_type_mismatch_append() {
        let op = VariableOperation::Append {
            text: "test".to_string(),
        };
        let result = op.apply(&VariableValue::Int(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_operation_type_mismatch_toggle() {
        let op = VariableOperation::Toggle;
        let result = op.apply(&VariableValue::Int(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_operation_serialization() {
        let op = VariableOperation::Add { value: 10 };
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: VariableOperation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(op, deserialized);
    }

    #[test]
    fn test_variable_new() {
        let var = Variable::new("score", VariableValue::Int(100));
        assert_eq!(var.name, "score");
        assert_eq!(var.value, VariableValue::Int(100));
    }

    #[test]
    fn test_variable_apply_operation_success() {
        let mut var = Variable::new("count", VariableValue::Int(5));
        let op = VariableOperation::Add { value: 3 };
        var.apply_operation(&op).unwrap();
        assert_eq!(var.value, VariableValue::Int(8));
    }

    #[test]
    fn test_variable_apply_operation_failure() {
        let mut var = Variable::new("count", VariableValue::Int(5));
        let op = VariableOperation::Divide { value: 0 };
        let result = var.apply_operation(&op);
        assert!(result.is_err());
        // Value should remain unchanged on error
        assert_eq!(var.value, VariableValue::Int(5));
    }

    #[test]
    fn test_variable_serialization() {
        let var = Variable::new("test_var", VariableValue::Bool(true));
        let serialized = serde_json::to_string(&var).unwrap();
        let deserialized: Variable = serde_json::from_str(&serialized).unwrap();
        assert_eq!(var, deserialized);
    }

    #[test]
    fn test_variable_multiple_operations() {
        let mut var = Variable::new("value", VariableValue::Int(10));

        var.apply_operation(&VariableOperation::Add { value: 5 })
            .unwrap();
        assert_eq!(var.value, VariableValue::Int(15));

        var.apply_operation(&VariableOperation::Multiply { value: 2 })
            .unwrap();
        assert_eq!(var.value, VariableValue::Int(30));

        var.apply_operation(&VariableOperation::Subtract { value: 10 })
            .unwrap();
        assert_eq!(var.value, VariableValue::Int(20));
    }

    #[test]
    fn test_variable_operation_add_float() {
        let op = VariableOperation::AddFloat { value: 3.5 };
        let result = op.apply(&VariableValue::Float(10.2)).unwrap();
        assert_eq!(result, VariableValue::Float(13.7));
    }

    #[test]
    fn test_variable_operation_subtract_float() {
        let op = VariableOperation::SubtractFloat { value: 2.5 };
        let result = op.apply(&VariableValue::Float(10.0)).unwrap();
        assert_eq!(result, VariableValue::Float(7.5));
    }

    #[test]
    fn test_variable_operation_multiply_float() {
        let op = VariableOperation::MultiplyFloat { value: 2.0 };
        let result = op.apply(&VariableValue::Float(5.5)).unwrap();
        assert_eq!(result, VariableValue::Float(11.0));
    }

    #[test]
    fn test_variable_operation_divide_float() {
        let op = VariableOperation::DivideFloat { value: 2.0 };
        let result = op.apply(&VariableValue::Float(10.0)).unwrap();
        assert_eq!(result, VariableValue::Float(5.0));
    }

    #[test]
    fn test_variable_operation_divide_float_by_zero() {
        let op = VariableOperation::DivideFloat { value: 0.0 };
        let result = op.apply(&VariableValue::Float(10.0));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), VariableError::DivisionByZero);
    }

    #[test]
    fn test_variable_operation_float_type_mismatch() {
        let op = VariableOperation::AddFloat { value: 5.5 };
        let result = op.apply(&VariableValue::Int(10));
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_multiple_float_operations() {
        let mut var = Variable::new("value", VariableValue::Float(10.0));

        var.apply_operation(&VariableOperation::AddFloat { value: 5.5 })
            .unwrap();
        assert_eq!(var.value, VariableValue::Float(15.5));

        var.apply_operation(&VariableOperation::MultiplyFloat { value: 2.0 })
            .unwrap();
        assert_eq!(var.value, VariableValue::Float(31.0));

        var.apply_operation(&VariableOperation::SubtractFloat { value: 6.0 })
            .unwrap();
        assert_eq!(var.value, VariableValue::Float(25.0));
    }
}
