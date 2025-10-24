//! Common test utilities shared across integration tests

use pinhole::{Application, Node};
use pinhole_protocol::messages::{ErrorCode, ServerToClientMessage};
use pinhole_protocol::storage::{StateValue, StorageScope};
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::net::{UnixListener, UnixStream};

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

/// Start a test server with the given application
///
/// Returns the socket path for clients to connect to. The server runs in the background
/// and will accept connections until dropped.
pub fn start_test_server<A: Application + 'static>(app: A) -> String {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let socket_path = temp_file.path().with_extension("sock");
    drop(temp_file);

    let listener = UnixListener::bind(&socket_path).expect("Failed to bind socket");
    let socket_path_str = socket_path.to_string_lossy().to_string();

    // Spawn server task
    tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });

    socket_path_str
}

/// Connect to a test server with retry logic
///
/// Retries connection with backoff to handle server startup race conditions.
pub async fn connect_test_client(socket_path: &str) -> UnixStream {
    // Retry connection with backoff to handle server startup race
    for i in 0..10 {
        if let Ok(stream) = UnixStream::connect(socket_path).await {
            return stream;
        }
        tokio::task::yield_now().await;
        if i > 5 {
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
    }
    panic!("Failed to connect to test server at {}", socket_path)
}
