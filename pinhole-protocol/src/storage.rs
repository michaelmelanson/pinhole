use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StorageScope {
    /// Persisted across restarts
    Persistent,

    /// Cleared when the application is restarted
    Session,

    /// Cleared on navigation
    Local,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StateValue {
    Empty,
    String(String),
    Boolean(bool),
}

impl StateValue {
    pub fn boolean(&self) -> bool {
        match self {
            StateValue::Boolean(b) => *b,
            _ => false,
        }
    }

    pub fn string(&self) -> &str {
        match self {
            StateValue::String(s) => s,
            _ => "",
        }
    }
}

impl log::kv::ToValue for StateValue {
    fn to_value(&self) -> log::kv::Value<'_> {
        log::kv::Value::from_debug(self)
    }
}

impl From<bool> for StateValue {
    fn from(value: bool) -> Self {
        StateValue::Boolean(value)
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

pub type StateMap = HashMap<String, StateValue>;
