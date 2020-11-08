use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub args: HashMap<String, String>,
    pub keys: Vec<String>,
}

impl Action {
    pub fn named(name: impl ToString, keys: Vec<String>) -> Action {
        Action::new(name, HashMap::default(), keys)
    }

    pub fn new(name: impl ToString, args: HashMap<String, String>, keys: Vec<String>) -> Action {
        Action {
            name: name.to_string(),
            args,
            keys,
        }
    }
}

impl log::kv::ToValue for Action {
    fn to_value(&self) -> log::kv::Value {
        log::kv::Value::from_debug(self)
    }
}
