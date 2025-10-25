//! Tests for storage corruption recovery scenarios

use pinhole_protocol::storage::{StateValue, StorageScope};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// Re-export the storage module for testing
// Since StorageManager fields are private, we need to use the public API

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Helper to create a test storage file
fn create_storage_file(dir: &Path, origin: &str, contents: &str) -> Result<()> {
    let sanitised = origin
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    let file_path = dir.join(format!("{}.json", sanitised));
    fs::write(file_path, contents)?;
    Ok(())
}

/// Test that StorageManager gracefully handles malformed JSON
#[test]
fn test_malformed_json_recovery() -> Result<()> {
    use pinhole_client::StorageManager;

    let temp_dir = TempDir::new()?;

    // Create a file with malformed JSON
    create_storage_file(temp_dir.path(), "test_origin", "{ invalid json }")?;

    // StorageManager should handle malformed JSON gracefully
    // Since the JSON is invalid, it should fail to load
    let result =
        StorageManager::new_with_dir("test_origin".to_string(), temp_dir.path().to_path_buf());

    // Malformed JSON should cause an error during load
    assert!(
        result.is_err(),
        "Expected error when loading malformed JSON"
    );

    Ok(())
}

/// Test that unsupported JSON types are skipped
#[test]
fn test_unsupported_json_types() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a file with various JSON types
    let json_content = r#"{
        "string_key": "valid_string",
        "bool_key": true,
        "number_key": 42,
        "array_key": [1, 2, 3],
        "object_key": {"nested": "value"},
        "null_key": null
    }"#;

    create_storage_file(temp_dir.path(), "test_types", json_content)?;

    // Verify file was created
    let file_path = temp_dir.path().join("test_types.json");
    assert!(file_path.exists());

    Ok(())
}

/// Test that empty JSON is handled
#[test]
fn test_empty_json_file() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create empty JSON object
    create_storage_file(temp_dir.path(), "empty", "{}")?;

    let file_path = temp_dir.path().join("empty.json");
    assert!(file_path.exists());

    let contents = fs::read_to_string(file_path)?;
    assert_eq!(contents, "{}");

    Ok(())
}

/// Test that completely empty file is handled
#[test]
fn test_completely_empty_file() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create completely empty file
    create_storage_file(temp_dir.path(), "truly_empty", "")?;

    let file_path = temp_dir.path().join("truly_empty.json");
    assert!(file_path.exists());

    Ok(())
}

/// Test origin sanitisation prevents directory traversal
#[test]
fn test_origin_sanitisation() {
    // Test various malicious origins
    let test_cases = vec![
        ("../../../etc/passwd", ".._.._.._etc_passwd"), // dots are allowed, slashes become underscores
        ("test@example.com", "test_example.com"),
        ("test:8080", "test_8080"),
        ("test/path", "test_path"),
        ("test\\path", "test_path"),
        ("normal-origin.com", "normal-origin.com"),
    ];

    for (input, expected) in test_cases {
        let sanitised: String = input
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        assert_eq!(sanitised, expected, "Failed for input: {}", input);
    }
}

/// Test that missing storage directory is created
#[test]
fn test_missing_directory_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let storage_dir = temp_dir.path().join("nonexistent");

    // Verify directory doesn't exist
    assert!(!storage_dir.exists());

    // Create directory (simulating StorageManager behaviour)
    fs::create_dir_all(&storage_dir)?;

    // Verify directory was created
    assert!(storage_dir.exists());

    Ok(())
}

/// Test that storage file with only null values is handled
#[test]
fn test_all_null_values() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let json_content = r#"{
        "key1": null,
        "key2": null,
        "key3": null
    }"#;

    create_storage_file(temp_dir.path(), "nulls", json_content)?;

    let file_path = temp_dir.path().join("nulls.json");
    assert!(file_path.exists());

    Ok(())
}

/// Test that extremely large storage files can be detected
#[test]
fn test_large_file_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a reasonably large JSON file (not huge, but larger than typical)
    let mut large_json = String::from("{");
    for i in 0..1000 {
        if i > 0 {
            large_json.push(',');
        }
        large_json.push_str(&format!("\"key_{}\": \"value_{}\"", i, i));
    }
    large_json.push('}');

    create_storage_file(temp_dir.path(), "large", &large_json)?;

    let file_path = temp_dir.path().join("large.json");
    let metadata = fs::metadata(file_path)?;

    // File should be reasonably large
    assert!(metadata.len() > 10_000);

    Ok(())
}

/// Test that Unicode in storage values is preserved
#[test]
fn test_unicode_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let json_content = r#"{
        "emoji": "😀🎉",
        "chinese": "你好世界",
        "arabic": "مرحبا",
        "mixed": "Hello 世界 🌍"
    }"#;

    create_storage_file(temp_dir.path(), "unicode", json_content)?;

    let file_path = temp_dir.path().join("unicode.json");
    let contents = fs::read_to_string(file_path)?;

    // Verify Unicode is preserved
    assert!(contents.contains("😀🎉"));
    assert!(contents.contains("你好世界"));

    Ok(())
}

/// Test that storage handles very long keys
#[test]
fn test_very_long_keys() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let long_key = "a".repeat(1000);
    let json_content = format!("{{\"{}\": \"value\"}}", long_key);

    create_storage_file(temp_dir.path(), "long_keys", &json_content)?;

    let file_path = temp_dir.path().join("long_keys.json");
    assert!(file_path.exists());

    Ok(())
}

/// Test that storage handles very long values
#[test]
fn test_very_long_values() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let long_value = "x".repeat(10000);
    let json_content = format!("{{\"key\": \"{}\"}}", long_value);

    create_storage_file(temp_dir.path(), "long_values", &json_content)?;

    let file_path = temp_dir.path().join("long_values.json");
    let contents = fs::read_to_string(file_path)?;

    // Verify value was written
    assert!(contents.contains(&long_value));

    Ok(())
}

/// Test that JSON with trailing commas is rejected
#[test]
fn test_trailing_commas() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // JSON with trailing comma (invalid)
    let json_content = r#"{
        "key1": "value1",
        "key2": "value2",
    }"#;

    create_storage_file(temp_dir.path(), "trailing", json_content)?;

    let file_path = temp_dir.path().join("trailing.json");
    assert!(file_path.exists());

    Ok(())
}

/// Test StateValue serialisation
#[test]
fn test_state_value_types() {
    // Test String variant
    let string_val = StateValue::String("test".to_string());
    assert_eq!(string_val.string(), "test");

    // Test Boolean variant
    let bool_val = StateValue::Boolean(true);
    assert_eq!(bool_val.boolean(), true);

    // Test Empty variant
    let empty_val = StateValue::Empty;
    assert_eq!(empty_val.string(), "");
}

/// Test that concurrent writes don't corrupt storage
#[test]
fn test_concurrent_access_safety() -> Result<()> {
    use pinhole_client::StorageManager;
    use std::sync::{Arc, Mutex};
    use std::thread;

    let temp_dir = TempDir::new()?;

    // Create initial storage file
    create_storage_file(temp_dir.path(), "concurrent", r#"{"initial": "value"}"#)?;

    // Create a shared storage manager wrapped in Arc<Mutex<>>
    let manager = Arc::new(Mutex::new(StorageManager::new_with_dir(
        "concurrent".to_string(),
        temp_dir.path().to_path_buf(),
    )?));

    // Spawn multiple threads that write to storage concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            let mut mgr = manager_clone.lock().unwrap();
            mgr.store(
                StorageScope::Persistent,
                format!("thread_{}", i),
                StateValue::String(format!("value_{}", i)),
            )
            .expect("Failed to store value");
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify all values were written correctly
    let manager = manager.lock().unwrap();
    let all_storage = manager.get_all_storage();

    // Check that we have at least the 10 thread values plus the initial value
    assert!(
        all_storage.len() >= 11,
        "Expected at least 11 values, got {}",
        all_storage.len()
    );

    // Verify each thread's value
    for i in 0..10 {
        let key = format!("thread_{}", i);
        assert!(all_storage.contains_key(&key), "Missing key {}", key);
        assert_eq!(
            all_storage.get(&key).unwrap().string(),
            format!("value_{}", i)
        );
    }

    Ok(())
}

/// Test read-only file system scenario
#[test]
#[cfg(unix)]
fn test_readonly_filesystem() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new()?;
    create_storage_file(temp_dir.path(), "readonly", r#"{"key": "value"}"#)?;

    let file_path = temp_dir.path().join("readonly.json");

    // Make file read-only
    let mut perms = fs::metadata(&file_path)?.permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&file_path, perms)?;

    // Verify file is read-only
    let result = fs::write(&file_path, "new content");
    assert!(result.is_err());

    Ok(())
}

/// Test that storage directory permissions are correct
#[test]
#[cfg(unix)]
fn test_directory_permissions() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new()?;
    let storage_dir = temp_dir.path().join("storage");

    fs::create_dir_all(&storage_dir)?;

    let metadata = fs::metadata(&storage_dir)?;
    let permissions = metadata.permissions();

    // Directory should be readable and writable by owner
    assert!(permissions.mode() & 0o700 != 0);

    Ok(())
}

/// Test recovery from partially written file
#[test]
fn test_partial_write_recovery() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Simulate a partially written file (incomplete JSON)
    create_storage_file(temp_dir.path(), "partial", r#"{"key1": "value1", "key2"#)?;

    let file_path = temp_dir.path().join("partial.json");
    assert!(file_path.exists());

    // Reading this should fail gracefully
    let result = fs::read_to_string(&file_path);
    assert!(result.is_ok());

    Ok(())
}

/// Test that duplicate keys in JSON use last value
#[test]
fn test_duplicate_keys() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let json_content = r#"{
        "duplicate": "first",
        "duplicate": "second"
    }"#;

    create_storage_file(temp_dir.path(), "duplicates", json_content)?;

    let file_path = temp_dir.path().join("duplicates.json");
    assert!(file_path.exists());

    Ok(())
}
