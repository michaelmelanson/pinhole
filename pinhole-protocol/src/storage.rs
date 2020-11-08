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
    fn to_value(&self) -> log::kv::Value {
        log::kv::Value::from_debug(self)
    }
}

pub type StateMap = HashMap<String, StateValue>;
