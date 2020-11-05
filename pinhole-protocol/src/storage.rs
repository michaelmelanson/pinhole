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
