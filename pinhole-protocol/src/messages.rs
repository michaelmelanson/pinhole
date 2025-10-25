use serde::{Deserialize, Serialize};

use crate::{
    action::Action,
    capabilities::CapabilitySet,
    document::Document,
    storage::{StateMap, StateValue, StorageScope},
};

/// Error codes for server-to-client error messages
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ErrorCode {
    /// 400 Bad Request - The request was malformed or invalid
    BadRequest,
    /// 404 Not Found - The requested route does not exist
    NotFound,
    /// 426 Upgrade Required - Client and server have no compatible capabilities
    UpgradeRequired,
    /// 500 Internal Server Error - An error occurred processing the request
    InternalServerError,
}

impl ErrorCode {
    /// Returns the HTTP-style status code number
    pub fn as_u16(&self) -> u16 {
        match self {
            ErrorCode::BadRequest => 400,
            ErrorCode::NotFound => 404,
            ErrorCode::UpgradeRequired => 426,
            ErrorCode::InternalServerError => 500,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ClientToServerMessage {
    /// Handshake message sent by client after connection
    ClientHello {
        capabilities: CapabilitySet,
    },
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ServerToClientMessage {
    /// Handshake response from server with negotiated capabilities
    ServerHello {
        capabilities: CapabilitySet,
    },
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
    Error {
        code: ErrorCode,
        message: String,
    },
}

impl log::kv::ToValue for ServerToClientMessage {
    fn to_value(&self) -> log::kv::Value<'_> {
        log::kv::Value::from_debug(self)
    }
}
