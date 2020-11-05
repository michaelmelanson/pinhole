use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::node::Node;

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Scope {
    /// Persisted across restarts
    Persistent,

    /// Cleared when the application is restarted
    Session,

    /// Cleared on navigation
    Local,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FormValue {
    String(String),
    Boolean(bool),
}

pub type FormState = HashMap<String, FormValue>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientToServerMessage {
    Load {
        path: String,
        storage: HashMap<String, String>,
    },
    Action {
        path: String,
        action: Action,
        form_state: FormState,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerToClientMessage {
    Render {
        document: Document,
    },
    RedirectTo {
        path: String,
    },
    Store {
        scope: Scope,
        key: String,
        value: String,
    },
}

impl log::kv::ToValue for ServerToClientMessage {
    fn to_value(&self) -> log::kv::Value {
        log::kv::Value::from_debug(self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document(pub Node);

impl Document {
    pub fn empty() -> Document {
        Document(Node::Empty)
    }
}
