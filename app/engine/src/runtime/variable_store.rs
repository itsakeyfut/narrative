//! Variable storage and management

use narrative_core::{VariableId, VariableValue};
use std::collections::HashMap;

/// Storage for variables
#[derive(Debug, Clone, Default)]
pub struct VariableStore {
    variables: HashMap<VariableId, VariableValue>,
}

impl VariableStore {
    /// Create a new empty variable store
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable value
    pub fn set(&mut self, variable: VariableId, value: VariableValue) {
        self.variables.insert(variable, value);
    }

    /// Get a variable value
    pub fn get(&self, variable: &VariableId) -> Option<&VariableValue> {
        self.variables.get(variable)
    }

    /// Remove a variable
    pub fn remove(&mut self, variable: &VariableId) -> Option<VariableValue> {
        self.variables.remove(variable)
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Convert variables to save data format (HashMap<String, i64>)
    ///
    /// Note: Only integer variables are saved. Other types are ignored.
    pub fn to_save_format(&self) -> HashMap<String, i64> {
        self.variables
            .iter()
            .filter_map(|(var_id, value)| {
                if let VariableValue::Int(n) = value {
                    Some((var_id.name().to_string(), *n))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Load variables from save data format (HashMap<String, i64>)
    pub fn from_save_format(data: &HashMap<String, i64>) -> Self {
        let variables = data
            .iter()
            .map(|(name, value)| (VariableId::new(name.clone()), VariableValue::Int(*value)))
            .collect();

        Self { variables }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_operations() {
        let mut store = VariableStore::new();
        let var = VariableId::new("test_var");

        assert!(store.get(&var).is_none());

        store.set(var.clone(), VariableValue::Int(42));
        assert_eq!(store.get(&var), Some(&VariableValue::Int(42)));

        store.remove(&var);
        assert!(store.get(&var).is_none());
    }

    #[test]
    fn test_variable_types() {
        let mut store = VariableStore::new();

        let bool_var = VariableId::new("bool_var");
        let int_var = VariableId::new("int_var");
        let float_var = VariableId::new("float_var");
        let string_var = VariableId::new("string_var");

        store.set(bool_var.clone(), VariableValue::Bool(true));
        store.set(int_var.clone(), VariableValue::Int(123));
        store.set(float_var.clone(), VariableValue::Float(45.67));
        store.set(
            string_var.clone(),
            VariableValue::String("hello".to_string()),
        );

        assert_eq!(store.get(&bool_var), Some(&VariableValue::Bool(true)));
        assert_eq!(store.get(&int_var), Some(&VariableValue::Int(123)));
        assert_eq!(store.get(&float_var), Some(&VariableValue::Float(45.67)));
        assert_eq!(
            store.get(&string_var),
            Some(&VariableValue::String("hello".to_string()))
        );
    }

    #[test]
    fn test_variable_overwrite() {
        let mut store = VariableStore::new();
        let var = VariableId::new("var");

        store.set(var.clone(), VariableValue::Int(10));
        assert_eq!(store.get(&var), Some(&VariableValue::Int(10)));

        store.set(var.clone(), VariableValue::Int(20));
        assert_eq!(store.get(&var), Some(&VariableValue::Int(20)));

        // Can change type
        store.set(var.clone(), VariableValue::String("changed".to_string()));
        assert_eq!(
            store.get(&var),
            Some(&VariableValue::String("changed".to_string()))
        );
    }

    #[test]
    fn test_variable_remove() {
        let mut store = VariableStore::new();
        let var = VariableId::new("var");

        store.set(var.clone(), VariableValue::Int(42));
        assert_eq!(store.remove(&var), Some(VariableValue::Int(42)));
        assert!(store.get(&var).is_none());

        // Removing non-existent variable
        assert_eq!(store.remove(&var), None);
    }

    #[test]
    fn test_variable_clear() {
        let mut store = VariableStore::new();

        store.set(VariableId::new("var1"), VariableValue::Int(1));
        store.set(VariableId::new("var2"), VariableValue::Int(2));
        store.set(VariableId::new("var3"), VariableValue::Int(3));

        store.clear();

        assert!(store.get(&VariableId::new("var1")).is_none());
        assert!(store.get(&VariableId::new("var2")).is_none());
        assert!(store.get(&VariableId::new("var3")).is_none());
    }

    #[test]
    fn test_variable_clone() {
        let mut store1 = VariableStore::new();
        let var = VariableId::new("var");

        store1.set(var.clone(), VariableValue::Int(99));

        let store2 = store1.clone();
        assert_eq!(store2.get(&var), Some(&VariableValue::Int(99)));
    }

    #[test]
    fn test_variable_store_default() {
        let store = VariableStore::default();
        let var = VariableId::new("any_var");

        assert!(store.get(&var).is_none());
    }

    #[test]
    fn test_many_variables() {
        let mut store = VariableStore::new();

        for i in 0..100 {
            let var = VariableId::new(format!("var_{}", i));
            store.set(var, VariableValue::Int(i as i64));
        }

        for i in 0..100 {
            let var = VariableId::new(format!("var_{}", i));
            assert_eq!(store.get(&var), Some(&VariableValue::Int(i as i64)));
        }
    }

    #[test]
    fn test_negative_numbers() {
        let mut store = VariableStore::new();
        let var = VariableId::new("negative");

        store.set(var.clone(), VariableValue::Int(-42));
        assert_eq!(store.get(&var), Some(&VariableValue::Int(-42)));

        store.set(var.clone(), VariableValue::Float(-3.14));
        assert_eq!(store.get(&var), Some(&VariableValue::Float(-3.14)));
    }

    #[test]
    fn test_empty_string() {
        let mut store = VariableStore::new();
        let var = VariableId::new("empty");

        store.set(var.clone(), VariableValue::String(String::new()));
        assert_eq!(store.get(&var), Some(&VariableValue::String(String::new())));
    }
}
