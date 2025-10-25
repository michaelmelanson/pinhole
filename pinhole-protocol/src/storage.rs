use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StorageScope {
    /// Persisted across restarts
    Persistent,

    /// Cleared when the application is restarted
    Session,

    /// Cleared on navigation
    Local,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StateValue {
    Empty,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<StateValue>),
    Object(HashMap<String, StateValue>),
}

impl StateValue {
    pub fn is_empty(&self) -> bool {
        matches!(self, StateValue::Empty)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, StateValue::Null)
    }

    pub fn boolean(&self) -> bool {
        match self {
            StateValue::Boolean(b) => *b,
            _ => false,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            StateValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn number(&self) -> f64 {
        match self {
            StateValue::Number(n) => *n,
            _ => 0.0,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            StateValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn string(&self) -> &str {
        match self {
            StateValue::String(s) => s,
            _ => "",
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            StateValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn array(&self) -> &[StateValue] {
        match self {
            StateValue::Array(arr) => arr,
            _ => &[],
        }
    }

    pub fn as_array(&self) -> Option<&Vec<StateValue>> {
        match self {
            StateValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<StateValue>> {
        match self {
            StateValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn object(&self) -> &HashMap<String, StateValue> {
        match self {
            StateValue::Object(obj) => obj,
            _ => {
                static EMPTY: std::sync::OnceLock<HashMap<String, StateValue>> =
                    std::sync::OnceLock::new();
                EMPTY.get_or_init(HashMap::new)
            }
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, StateValue>> {
        match self {
            StateValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, StateValue>> {
        match self {
            StateValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

impl From<bool> for StateValue {
    fn from(value: bool) -> Self {
        StateValue::Boolean(value)
    }
}

impl From<f64> for StateValue {
    fn from(value: f64) -> Self {
        StateValue::Number(value)
    }
}

impl From<i32> for StateValue {
    fn from(value: i32) -> Self {
        StateValue::Number(value as f64)
    }
}

impl From<i64> for StateValue {
    fn from(value: i64) -> Self {
        StateValue::Number(value as f64)
    }
}

impl From<u32> for StateValue {
    fn from(value: u32) -> Self {
        StateValue::Number(value as f64)
    }
}

impl From<u64> for StateValue {
    fn from(value: u64) -> Self {
        StateValue::Number(value as f64)
    }
}

impl From<&str> for StateValue {
    fn from(value: &str) -> Self {
        StateValue::String(value.to_string())
    }
}

impl From<String> for StateValue {
    fn from(value: String) -> Self {
        StateValue::String(value)
    }
}

impl From<Vec<StateValue>> for StateValue {
    fn from(value: Vec<StateValue>) -> Self {
        StateValue::Array(value)
    }
}

impl From<HashMap<String, StateValue>> for StateValue {
    fn from(value: HashMap<String, StateValue>) -> Self {
        StateValue::Object(value)
    }
}

pub type StateMap = HashMap<String, StateValue>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_and_null() {
        let empty = StateValue::Empty;
        let null = StateValue::Null;

        assert!(empty.is_empty());
        assert!(!empty.is_null());
        assert!(!null.is_empty());
        assert!(null.is_null());
    }

    #[test]
    fn test_boolean_accessors() {
        let val = StateValue::Boolean(true);
        assert_eq!(val.boolean(), true);
        assert_eq!(val.as_boolean(), Some(true));

        let val = StateValue::String("test".to_string());
        assert_eq!(val.boolean(), false);
        assert_eq!(val.as_boolean(), None);
    }

    #[test]
    fn test_number_accessors() {
        let val = StateValue::Number(42.5);
        assert_eq!(val.number(), 42.5);
        assert_eq!(val.as_number(), Some(42.5));

        let val = StateValue::String("test".to_string());
        assert_eq!(val.number(), 0.0);
        assert_eq!(val.as_number(), None);
    }

    #[test]
    fn test_string_accessors() {
        let val = StateValue::String("hello".to_string());
        assert_eq!(val.string(), "hello");
        assert_eq!(val.as_string(), Some("hello"));

        let val = StateValue::Number(42.0);
        assert_eq!(val.string(), "");
        assert_eq!(val.as_string(), None);
    }

    #[test]
    fn test_array_accessors() {
        let arr = vec![
            StateValue::Number(1.0),
            StateValue::Number(2.0),
            StateValue::Number(3.0),
        ];
        let val = StateValue::Array(arr.clone());

        assert_eq!(val.array().len(), 3);
        assert_eq!(val.as_array(), Some(&arr));

        let val = StateValue::String("test".to_string());
        assert_eq!(val.array().len(), 0);
        assert_eq!(val.as_array(), None);
    }

    #[test]
    fn test_object_accessors() {
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), StateValue::String("value".to_string()));
        let val = StateValue::Object(obj.clone());

        assert_eq!(val.object().len(), 1);
        assert_eq!(val.as_object(), Some(&obj));

        let val = StateValue::String("test".to_string());
        assert_eq!(val.object().len(), 0);
        assert_eq!(val.as_object(), None);
    }

    #[test]
    fn test_from_conversions() {
        assert_eq!(StateValue::from(true), StateValue::Boolean(true));
        assert_eq!(StateValue::from(42.5), StateValue::Number(42.5));
        assert_eq!(StateValue::from(42i32), StateValue::Number(42.0));
        assert_eq!(StateValue::from(42i64), StateValue::Number(42.0));
        assert_eq!(StateValue::from(42u32), StateValue::Number(42.0));
        assert_eq!(StateValue::from(42u64), StateValue::Number(42.0));
        assert_eq!(
            StateValue::from("test"),
            StateValue::String("test".to_string())
        );
        assert_eq!(
            StateValue::from("test".to_string()),
            StateValue::String("test".to_string())
        );
    }

    #[test]
    fn test_nested_structures() {
        let mut inner_obj = HashMap::new();
        inner_obj.insert(
            "nested".to_string(),
            StateValue::String("value".to_string()),
        );

        let arr = vec![
            StateValue::Number(1.0),
            StateValue::Object(inner_obj),
            StateValue::Array(vec![StateValue::Boolean(true)]),
        ];

        let mut outer_obj = HashMap::new();
        outer_obj.insert("array".to_string(), StateValue::Array(arr));

        let val = StateValue::Object(outer_obj);

        // Access nested structure
        if let Some(obj) = val.as_object() {
            if let Some(StateValue::Array(arr)) = obj.get("array") {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], StateValue::Number(1.0));

                if let StateValue::Object(inner) = &arr[1] {
                    assert_eq!(
                        inner.get("nested"),
                        Some(&StateValue::String("value".to_string()))
                    );
                }
            }
        }
    }

    #[test]
    fn test_serialization() {
        let mut obj = HashMap::new();
        obj.insert("bool".to_string(), StateValue::Boolean(true));
        obj.insert("num".to_string(), StateValue::Number(42.0));
        obj.insert("str".to_string(), StateValue::String("test".to_string()));
        obj.insert("null".to_string(), StateValue::Null);
        obj.insert(
            "arr".to_string(),
            StateValue::Array(vec![StateValue::Number(1.0), StateValue::Number(2.0)]),
        );

        let val = StateValue::Object(obj);

        // Test CBOR serialization roundtrip
        let encoded = serde_cbor::to_vec(&val).unwrap();
        let decoded: StateValue = serde_cbor::from_slice(&encoded).unwrap();

        assert_eq!(val, decoded);
    }
}
