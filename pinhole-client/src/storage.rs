use directories::ProjectDirs;

use pinhole_protocol::storage::{StateMap, StateValue, StorageScope};
use serde_json;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, path::PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct StorageManager {
    persistent_storage: StateMap,
    session_storage: StateMap,
    local_storage: StateMap,
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
        let persistent_storage = Self::load_persistent_storage(&storage_dir, &origin)?;
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

    fn load_persistent_storage(storage_dir: &PathBuf, origin: &str) -> Result<StateMap> {
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
                let state_value = match value {
                    serde_json::Value::String(s) => StateValue::String(s),
                    serde_json::Value::Bool(b) => StateValue::Boolean(b),
                    _ => continue, // Skip unsupported types
                };
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
        for (key, value) in &self.persistent_storage {
            let json_value = match value {
                StateValue::Empty => serde_json::Value::Null,
                StateValue::String(s) => serde_json::Value::String(s.clone()),
                StateValue::Boolean(b) => serde_json::Value::Bool(*b),
            };
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

    pub fn store(&mut self, scope: StorageScope, key: String, value: StateValue) -> Result<()> {
        match scope {
            StorageScope::Persistent => {
                self.persistent_storage.insert(key, value);
                self.save_persistent_storage()?;
            }
            StorageScope::Session => {
                self.session_storage.insert(key, value);
            }
            StorageScope::Local => {
                self.local_storage.insert(key, value);
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get(&self, scope: StorageScope, key: &str) -> Option<&StateValue> {
        match scope {
            StorageScope::Persistent => self.persistent_storage.get(key),
            StorageScope::Session => self.session_storage.get(key),
            StorageScope::Local => self.local_storage.get(key),
        }
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
        combined.extend(self.persistent_storage.clone());
        combined.extend(self.session_storage.clone());
        combined.extend(self.local_storage.clone());

        combined
    }

    pub fn clear_local_storage(&mut self) {
        tracing::trace!("Clearing local storage");
        self.local_storage.clear();
    }

    pub fn clear_session_storage(&mut self) {
        tracing::debug!("Clearing session storage");
        self.session_storage.clear();
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
}
