use pinhole_protocol::{
    action::Action,
    messages::{ClientToServerMessage, ErrorCode, ServerToClientMessage},
    storage::{StateMap, StateValue, StorageScope},
};
use std::collections::HashMap;

/// Helper to test CBOR serialization against expected byte sequences
fn assert_cbor_encoding<T>(value: &T, expected_bytes: &[u8])
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let actual_bytes = serde_cbor::to_vec(value).unwrap();
    assert_eq!(actual_bytes, expected_bytes);

    let deserialized: T = serde_cbor::from_slice(expected_bytes).unwrap();
    assert_eq!(*value, deserialized);
}

#[test]
fn test_state_value_empty_cbor() {
    let value = StateValue::Empty;
    let expected_bytes = &[101, 69, 109, 112, 116, 121];
    assert_cbor_encoding(&value, expected_bytes);
}

#[test]
fn test_state_value_boolean_true_cbor() {
    let value = StateValue::Boolean(true);
    let expected_bytes = &[161, 103, 66, 111, 111, 108, 101, 97, 110, 245];
    assert_cbor_encoding(&value, expected_bytes);
}

#[test]
fn test_state_value_boolean_false_cbor() {
    let value = StateValue::Boolean(false);
    let expected_bytes = &[161, 103, 66, 111, 111, 108, 101, 97, 110, 244];
    assert_cbor_encoding(&value, expected_bytes);
}

#[test]
fn test_state_value_string_cbor() {
    let value = StateValue::String("test".to_string());
    let expected_bytes = &[
        161, 102, 83, 116, 114, 105, 110, 103, 100, 116, 101, 115, 116,
    ];
    assert_cbor_encoding(&value, expected_bytes);
}

#[test]
fn test_storage_scope_persistent_cbor() {
    let value = StorageScope::Persistent;
    let expected_bytes = &[106, 80, 101, 114, 115, 105, 115, 116, 101, 110, 116];
    assert_cbor_encoding(&value, expected_bytes);
}

#[test]
fn test_storage_scope_session_cbor() {
    let value = StorageScope::Session;
    let expected_bytes = &[103, 83, 101, 115, 115, 105, 111, 110];
    assert_cbor_encoding(&value, expected_bytes);
}

#[test]
fn test_storage_scope_local_cbor() {
    let value = StorageScope::Local;
    let expected_bytes = &[101, 76, 111, 99, 97, 108];
    assert_cbor_encoding(&value, expected_bytes);
}

#[test]
fn test_server_redirect_message_cbor() {
    let message = ServerToClientMessage::RedirectTo {
        path: "/login".to_string(),
    };
    let expected_bytes = &[
        161, 106, 82, 101, 100, 105, 114, 101, 99, 116, 84, 111, 161, 100, 112, 97, 116, 104, 102,
        47, 108, 111, 103, 105, 110,
    ];
    assert_cbor_encoding(&message, expected_bytes);
}

#[test]
fn test_server_error_message_cbor() {
    let message = ServerToClientMessage::Error {
        code: ErrorCode::NotFound,
        message: "Not found".to_string(),
    };
    let expected_bytes = &[
        161, 101, 69, 114, 114, 111, 114, 162, 100, 99, 111, 100, 101, 104, 78, 111, 116, 70, 111,
        117, 110, 100, 103, 109, 101, 115, 115, 97, 103, 101, 105, 78, 111, 116, 32, 102, 111, 117,
        110, 100,
    ];
    assert_cbor_encoding(&message, expected_bytes);
}

#[test]
fn test_server_store_message_cbor() {
    let message = ServerToClientMessage::Store {
        scope: StorageScope::Session,
        key: "user_id".to_string(),
        value: StateValue::String("12345".to_string()),
    };
    let expected_bytes = &[
        161, 101, 83, 116, 111, 114, 101, 163, 101, 115, 99, 111, 112, 101, 103, 83, 101, 115, 115,
        105, 111, 110, 99, 107, 101, 121, 103, 117, 115, 101, 114, 95, 105, 100, 101, 118, 97, 108,
        117, 101, 161, 102, 83, 116, 114, 105, 110, 103, 101, 49, 50, 51, 52, 53,
    ];
    assert_cbor_encoding(&message, expected_bytes);
}

#[test]
fn test_client_load_message_cbor() {
    let mut storage = StateMap::new();
    storage.insert("key1".to_string(), StateValue::String("value1".to_string()));

    let message = ClientToServerMessage::Load {
        path: "/test".to_string(),
        storage,
    };
    let expected_bytes = &[
        161, 100, 76, 111, 97, 100, 162, 100, 112, 97, 116, 104, 101, 47, 116, 101, 115, 116, 103,
        115, 116, 111, 114, 97, 103, 101, 161, 100, 107, 101, 121, 49, 161, 102, 83, 116, 114, 105,
        110, 103, 102, 118, 97, 108, 117, 101, 49,
    ];
    assert_cbor_encoding(&message, expected_bytes);
}

#[test]
fn test_client_action_message_cbor() {
    let action = Action::new("submit", HashMap::new(), vec![]);
    let mut storage = StateMap::new();
    storage.insert("field".to_string(), StateValue::String("data".to_string()));

    let message = ClientToServerMessage::Action {
        path: "/form".to_string(),
        action,
        storage,
    };
    let expected_bytes = &[
        161, 102, 65, 99, 116, 105, 111, 110, 163, 100, 112, 97, 116, 104, 101, 47, 102, 111, 114,
        109, 102, 97, 99, 116, 105, 111, 110, 163, 100, 110, 97, 109, 101, 102, 115, 117, 98, 109,
        105, 116, 100, 97, 114, 103, 115, 160, 100, 107, 101, 121, 115, 128, 103, 115, 116, 111,
        114, 97, 103, 101, 161, 101, 102, 105, 101, 108, 100, 161, 102, 83, 116, 114, 105, 110,
        103, 100, 100, 97, 116, 97,
    ];
    assert_cbor_encoding(&message, expected_bytes);
}

#[test]
fn test_error_code_values() {
    assert_eq!(ErrorCode::BadRequest.as_u16(), 400);
    assert_eq!(ErrorCode::NotFound.as_u16(), 404);
    assert_eq!(ErrorCode::InternalServerError.as_u16(), 500);
}
