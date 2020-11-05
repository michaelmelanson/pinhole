use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub args: HashMap<String, String>,
}

impl Action {
    pub fn named(name: impl ToString) -> Action {
        Action::new(name, HashMap::default())
    }

    pub fn new(name: impl ToString, args: HashMap<String, String>) -> Action {
        Action {
            name: name.to_string(),
            args,
        }
    }
}

impl log::kv::ToValue for Action {
    fn to_value(&self) -> log::kv::Value {
        log::kv::Value::from_debug(self)
    }
}
