use serde::{Deserialize, Serialize};

use crate::{
    action::Action,
    document::Document,
    storage::{StateMap, StateValue, StorageScope},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientToServerMessage {
    Load {
        path: String,
        storage: StateMap,
    },
    Action {
        path: String,
        action: Action,
        storage: StateMap,
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
        scope: StorageScope,
        key: String,
        value: StateValue,
    },
}

impl log::kv::ToValue for ServerToClientMessage {
    fn to_value(&self) -> log::kv::Value {
        log::kv::Value::from_debug(self)
    }
}
