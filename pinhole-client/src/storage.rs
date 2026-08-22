use directories::ProjectDirs;

use pinhole_protocol::storage::{StateMap, StateValue, StateVersion, StorageScope, VersionMap};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// A stored value together with the version it was written at, so
/// `apply_remote_store` can tell a fresh write from a stale/conflicting one.
#[derive(Clone, Debug)]
struct VersionedEntry {
    value: StateValue,
    version: StateVersion,
}

fn state_value_to_json(value: &StateValue) -> serde_json::Value {
    match value {
        StateValue::Empty => serde_json::Value::Null,
        StateValue::Null => serde_json::Value::Null,
        StateValue::Boolean(b) => serde_json::Value::Bool(*b),
        StateValue::Number(n) => serde_json::Value::Number(
            serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        StateValue::String(s) => serde_json::Value::String(s.clone()),
        StateValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(state_value_to_json).collect())
        }
        StateValue::Object(obj) => {
            let map = obj
                .iter()
                .map(|(k, v)| (k.clone(), state_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}

fn json_to_state_value(value: &serde_json::Value) -> StateValue {
    match value {
        serde_json::Value::Null => StateValue::Null,
        serde_json::Value::Bool(b) => StateValue::Boolean(*b),
        serde_json::Value::Number(n) => StateValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => StateValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            StateValue::Array(arr.iter().map(json_to_state_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let map = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_state_value(v)))
                .collect();
            StateValue::Object(map)
        }
    }
}

pub struct StorageManager {
    persistent_storage: HashMap<String, VersionedEntry>,
    session_storage: HashMap<String, VersionedEntry>,
    local_storage: HashMap<String, VersionedEntry>,
    current_route: Option<String>,
    storage_dir: PathBuf,
    origin: String,
}

impl StorageManager {
    pub fn new(origin: String) -> Result<Self> {
        let storage_dir = Self::get_storage_dir()?;
        Self::new_with_dir(origin, storage_dir)
    }

    /// Create a new StorageManager with a custom storage directory
    ///
    /// This is primarily intended for testing, allowing tests to specify
    /// a temporary directory rather than using the system data directory.
    pub fn new_with_dir(origin: String, storage_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&storage_dir)?;
        let loaded = Self::load_persistent_storage(&storage_dir, &origin)?;
        // Loaded-from-disk values have no network-observed version yet;
        // start them at 1 so a subsequent server write (version >= 1) is
        // treated as a legitimate update rather than a stale one.
        let persistent_storage = loaded
            .into_iter()
            .map(|(key, value)| (key, VersionedEntry { value, version: 1 }))
            .collect();
        Ok(StorageManager {
            persistent_storage,
            session_storage: HashMap::new(),
            local_storage: HashMap::new(),
            current_route: None,
            storage_dir,
            origin,
        })
    }

    fn get_storage_dir() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("net", "michaelmelanson", "pinhole") {
            Ok(proj_dirs.data_dir().to_path_buf())
        } else {
            Err("Could not determine platform storage directory".into())
        }
    }

    fn get_persistent_file_path(&self) -> PathBuf {
        self.storage_dir
            .join(format!("{}.json", self.sanitize_origin(&self.origin)))
    }

    fn sanitize_origin(&self, origin: &str) -> String {
        // Sanitise to alphanumeric + dots + hyphens
        let sanitised: String = origin
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        // Add cryptographic hash suffix to prevent collisions between different origins
        // that sanitise to the same string (e.g., "test@example.com" and "test:example.com")
        // Full SHA-256 (256 bits) is used because origins come from untrusted network sources
        // and we need collision resistance against intentional attacks
        let mut hasher = Sha256::new();
        hasher.update(origin.as_bytes());
        let hash = hasher.finalize();

        // Use full 32-byte SHA-256 hash (64 hex characters)
        // This provides cryptographic collision resistance
        let hash_hex = hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        format!("{}-{}", sanitised, hash_hex)
    }

    fn load_persistent_storage(storage_dir: &Path, origin: &str) -> Result<StateMap> {
        let file_path = storage_dir.join(format!(
            "{}.json",
            origin
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                })
                .collect::<String>()
        ));

        if file_path.exists() {
            let contents = fs::read_to_string(file_path)?;
            let json_map: HashMap<String, serde_json::Value> = serde_json::from_str(&contents)?;

            let mut state_map = HashMap::new();
            for (key, value) in json_map {
                let state_value = json_to_state_value(&value);
                state_map.insert(key, state_value);
            }

            tracing::debug!(
                items = state_map.len(),
                origin = %origin,
                "Loaded persistent storage"
            );
            Ok(state_map)
        } else {
            tracing::debug!(origin = %origin, "No persistent storage file found");
            Ok(HashMap::new())
        }
    }

    fn save_persistent_storage(&self) -> Result<()> {
        let file_path = self.get_persistent_file_path();

        let mut json_map = HashMap::new();
        for (key, entry) in &self.persistent_storage {
            let json_value = state_value_to_json(&entry.value);
            json_map.insert(key.clone(), json_value);
        }

        let contents = serde_json::to_string_pretty(&json_map)?;

        // Atomic write: write to temp file, then rename
        // This prevents corruption if the process crashes mid-write
        let temp_path = file_path.with_extension("tmp");
        fs::write(&temp_path, contents)?;
        fs::rename(&temp_path, &file_path)?;

        tracing::debug!(
            items = json_map.len(),
            origin = %self.origin,
            "Saved persistent storage"
        );
        Ok(())
    }

    /// Store a value locally, always bumping its version. Use this for
    /// client-originated writes (e.g. form field state); for writes coming
    /// from the server's `Store` message, use `apply_remote_store` instead
    /// so stale/conflicting writes can be detected and dropped.
    #[allow(dead_code)]
    pub fn store(&mut self, scope: StorageScope, key: String, value: StateValue) -> Result<()> {
        let map = self.map_for_scope_mut(&scope);
        let version = map.get(&key).map_or(1, |entry| entry.version + 1);
        map.insert(key, VersionedEntry { value, version });

        if scope == StorageScope::Persistent {
            self.save_persistent_storage()?;
        }
        Ok(())
    }

    /// Apply a `Store` write received from the server. The write is only
    /// applied if `version` is strictly greater than the version already
    /// known for `key` (0 if unknown); otherwise it's a stale or
    /// conflicting write racing an already-applied one, and is dropped.
    pub fn apply_remote_store(
        &mut self,
        scope: StorageScope,
        key: String,
        value: StateValue,
        version: StateVersion,
    ) -> Result<()> {
        let map = self.map_for_scope_mut(&scope);
        let current_version = map.get(&key).map_or(0, |entry| entry.version);

        if version <= current_version {
            tracing::warn!(
                key = %key,
                incoming_version = version,
                current_version,
                "Dropping stale Store: conflicting write detected"
            );
            return Ok(());
        }

        map.insert(key.clone(), VersionedEntry { value, version });

        if scope == StorageScope::Persistent {
            self.save_persistent_storage()?;
        }
        Ok(())
    }

    fn map_for_scope_mut(&mut self, scope: &StorageScope) -> &mut HashMap<String, VersionedEntry> {
        match scope {
            StorageScope::Persistent => &mut self.persistent_storage,
            StorageScope::Session => &mut self.session_storage,
            StorageScope::Local => &mut self.local_storage,
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, scope: StorageScope, key: &str) -> Option<&StateValue> {
        let map = match scope {
            StorageScope::Persistent => &self.persistent_storage,
            StorageScope::Session => &self.session_storage,
            StorageScope::Local => &self.local_storage,
        };
        map.get(key).map(|entry| &entry.value)
    }

    pub fn navigate_to(&mut self, new_route: String) {
        if self.current_route.as_ref() != Some(&new_route) {
            tracing::debug!(
                from = ?self.current_route,
                to = %new_route,
                "Route changed, clearing local storage"
            );
            self.local_storage.clear();
            self.current_route = Some(new_route);
        }
    }

    pub fn get_all_storage(&self) -> StateMap {
        let mut combined = HashMap::new();

        // Order matters: persistent -> session -> local (local wins on conflicts)
        for map in [
            &self.persistent_storage,
            &self.session_storage,
            &self.local_storage,
        ] {
            combined.extend(map.iter().map(|(k, e)| (k.clone(), e.value.clone())));
        }

        combined
    }

    /// Per-key versions mirroring `get_all_storage`, sent alongside it in
    /// `Load`/`Action` requests so the server can tell the client which
    /// prior version each `Store` write supersedes.
    pub fn get_all_versions(&self) -> VersionMap {
        let mut combined = HashMap::new();

        // Same precedence as get_all_storage: persistent -> session -> local
        for map in [
            &self.persistent_storage,
            &self.session_storage,
            &self.local_storage,
        ] {
            combined.extend(map.iter().map(|(k, e)| (k.clone(), e.version)));
        }

        combined
    }

    pub fn clear_local_storage(&mut self) {
        tracing::trace!("Clearing local storage");
        self.local_storage.clear();
    }

    #[allow(dead_code)]
    pub fn clear_all_storage(&mut self) -> Result<()> {
        self.persistent_storage.clear();
        self.session_storage.clear();
        self.local_storage.clear();
        self.save_persistent_storage()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_scopes() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut manager = StorageManager {
            persistent_storage: HashMap::new(),
            session_storage: HashMap::new(),
            local_storage: HashMap::new(),
            current_route: None,
            storage_dir: temp_dir.path().to_path_buf(),
            origin: "test".to_string(),
        };

        // Test storing in different scopes
        manager.store(
            StorageScope::Persistent,
            "p_key".to_string(),
            "p_value".into(),
        )?;
        manager.store(StorageScope::Session, "s_key".to_string(), "s_value".into())?;
        manager.store(StorageScope::Local, "l_key".to_string(), "l_value".into())?;

        // Test retrieval
        assert_eq!(
            manager
                .get(StorageScope::Persistent, "p_key")
                .unwrap()
                .string(),
            "p_value"
        );
        assert_eq!(
            manager
                .get(StorageScope::Session, "s_key")
                .unwrap()
                .string(),
            "s_value"
        );
        assert_eq!(
            manager.get(StorageScope::Local, "l_key").unwrap().string(),
            "l_value"
        );

        // Test route navigation clears local storage
        manager.navigate_to("route1".to_string());
        manager.store(StorageScope::Local, "l_key".to_string(), "l_value".into())?;
        assert_eq!(
            manager.get(StorageScope::Local, "l_key").unwrap().string(),
            "l_value"
        );

        manager.navigate_to("route2".to_string());
        assert!(manager.get(StorageScope::Local, "l_key").is_none());

        Ok(())
    }

    fn empty_manager(temp_dir: &TempDir) -> StorageManager {
        StorageManager {
            persistent_storage: HashMap::new(),
            session_storage: HashMap::new(),
            local_storage: HashMap::new(),
            current_route: None,
            storage_dir: temp_dir.path().to_path_buf(),
            origin: "test".to_string(),
        }
    }

    #[test]
    fn test_store_bumps_version_on_each_write() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut manager = empty_manager(&temp_dir);

        manager.store(StorageScope::Session, "key".to_string(), "a".into())?;
        assert_eq!(manager.get_all_versions().get("key"), Some(&1));

        manager.store(StorageScope::Session, "key".to_string(), "b".into())?;
        assert_eq!(manager.get_all_versions().get("key"), Some(&2));

        Ok(())
    }

    /// Regression test for the double-action race: if a user fires two
    /// actions before the first `Store` reply arrives, both are built from
    /// the same stale storage snapshot and the server computes the same
    /// target version for both replies. The second `Store` to arrive must
    /// be dropped rather than silently reapplied over the first.
    #[test]
    fn test_apply_remote_store_drops_stale_duplicate_write() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut manager = empty_manager(&temp_dir);

        // First Store reply: done=true at version 1.
        manager.apply_remote_store(StorageScope::Session, "done".to_string(), true.into(), 1)?;
        assert!(manager
            .get(StorageScope::Session, "done")
            .unwrap()
            .boolean());

        // Second Store reply racing the first: computed from the same
        // stale pre-image, so it also claims version 1. Must be dropped,
        // not applied.
        manager.apply_remote_store(StorageScope::Session, "done".to_string(), false.into(), 1)?;
        assert!(
            manager
                .get(StorageScope::Session, "done")
                .unwrap()
                .boolean(),
            "stale write with a non-newer version must not overwrite the current value"
        );

        Ok(())
    }

    #[test]
    fn test_apply_remote_store_accepts_strictly_greater_version() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut manager = empty_manager(&temp_dir);

        manager.apply_remote_store(StorageScope::Session, "done".to_string(), true.into(), 1)?;
        manager.apply_remote_store(StorageScope::Session, "done".to_string(), false.into(), 2)?;

        assert!(!manager
            .get(StorageScope::Session, "done")
            .unwrap()
            .boolean());

        Ok(())
    }
}
