use directories::ProjectDirs;
use kv_log_macro as log;
use pinhole_protocol::storage::{StateMap, StateValue, StorageScope};
use serde_json;
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
        origin
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
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

            log::debug!(
                "Loaded {} persistent storage items for origin {}",
                state_map.len(),
                origin
            );
            Ok(state_map)
        } else {
            log::debug!("No persistent storage file found for origin {}", origin);
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
        fs::write(file_path, contents)?;

        log::debug!(
            "Saved {} persistent storage items for origin {}",
            json_map.len(),
            self.origin
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
            log::debug!(
                "Route changed from {:?} to {}, clearing local storage",
                self.current_route,
                new_route
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
        log::debug!("Clearing local storage");
        self.local_storage.clear();
    }

    pub fn clear_session_storage(&mut self) {
        log::debug!("Clearing session storage");
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
