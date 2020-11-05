use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{action::Action, document::Document, form_state::FormState, storage::StorageScope};

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
        scope: StorageScope,
        key: String,
        value: String,
    },
}

impl log::kv::ToValue for ServerToClientMessage {
    fn to_value(&self) -> log::kv::Value {
        log::kv::Value::from_debug(self)
    }
}
