//! Common test utilities shared across integration tests

use pinhole::Node;
use pinhole_protocol::messages::{ErrorCode, ServerToClientMessage};
use pinhole_protocol::storage::{StateValue, StorageScope};

/// Assert that messages contain a single Render with expected node
pub fn assert_render(messages: &[ServerToClientMessage], expected_node: Node) {
    assert_eq!(messages.len(), 1);
    let ServerToClientMessage::Render { document } = &messages[0] else {
        panic!("Expected Render message");
    };
    assert_eq!(document.node, expected_node);
}

/// Assert that messages contain a single Store message with expected values
#[allow(dead_code)]
pub fn assert_store(
    messages: &[ServerToClientMessage],
    expected_key: &str,
    expected_value: StateValue,
) {
    assert_eq!(messages.len(), 1);
    let ServerToClientMessage::Store { scope, key, value } = &messages[0] else {
        panic!("Expected Store message, got: {:?}", messages[0]);
    };
    assert_eq!(*scope, StorageScope::Session);
    assert_eq!(key, expected_key);
    assert_eq!(*value, expected_value);
}

/// Assert that messages contain a single Error with expected code
#[allow(dead_code)]
pub fn assert_error(
    messages: &[ServerToClientMessage],
    expected_code: ErrorCode,
    contains_text: &str,
) {
    assert_eq!(messages.len(), 1);
    let ServerToClientMessage::Error { code, message } = &messages[0] else {
        panic!("Expected Error message");
    };
    assert_eq!(*code, expected_code);
    assert!(message.contains(contains_text));
}

/// Assert that messages contain a single RedirectTo with expected path
#[allow(dead_code)]
pub fn assert_redirect(messages: &[ServerToClientMessage], expected_path: &str) {
    assert_eq!(messages.len(), 1);
    let ServerToClientMessage::RedirectTo { path } = &messages[0] else {
        panic!("Expected RedirectTo message");
    };
    assert_eq!(path, expected_path);
}
