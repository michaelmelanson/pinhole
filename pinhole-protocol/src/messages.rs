use std::fmt;

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
    /// 401 Unauthorized - Authentication is required or has failed
    Unauthorized,
    /// 403 Forbidden - The client does not have permission for this request
    Forbidden,
    /// 404 Not Found - The requested route does not exist
    NotFound,
    /// 426 Upgrade Required - Client and server have no compatible capabilities
    UpgradeRequired,
    /// 429 Too Many Requests - The client should back off and retry later
    TooManyRequests,
    /// 500 Internal Server Error - An error occurred processing the request
    InternalServerError,
}

impl ErrorCode {
    /// Returns the HTTP-style status code number
    pub fn as_u16(&self) -> u16 {
        match self {
            ErrorCode::BadRequest => 400,
            ErrorCode::Unauthorized => 401,
            ErrorCode::Forbidden => 403,
            ErrorCode::NotFound => 404,
            ErrorCode::UpgradeRequired => 426,
            ErrorCode::TooManyRequests => 429,
            ErrorCode::InternalServerError => 500,
        }
    }

    /// Returns the standard HTTP reason phrase for this status code
    pub fn reason_phrase(&self) -> &'static str {
        match self {
            ErrorCode::BadRequest => "Bad Request",
            ErrorCode::Unauthorized => "Unauthorized",
            ErrorCode::Forbidden => "Forbidden",
            ErrorCode::NotFound => "Not Found",
            ErrorCode::UpgradeRequired => "Upgrade Required",
            ErrorCode::TooManyRequests => "Too Many Requests",
            ErrorCode::InternalServerError => "Internal Server Error",
        }
    }

    /// Whether a client can reasonably expect retrying the same request to succeed.
    ///
    /// Only `TooManyRequests` signals a transient condition the client caused and
    /// controls (request rate). The other 4xx codes are client errors that won't
    /// resolve by resending the same request, and `InternalServerError` here
    /// reflects an unclassified handler failure rather than a known-transient state.
    pub fn is_retryable(&self) -> bool {
        matches!(self, ErrorCode::TooManyRequests)
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.as_u16(), self.reason_phrase())
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
