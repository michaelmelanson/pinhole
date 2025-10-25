//! Common test utilities shared across integration tests

use pinhole::{Action, Application, Node};
use pinhole_protocol::messages::{ClientToServerMessage, ErrorCode, ServerToClientMessage};
use pinhole_protocol::network::{receive_server_message, send_message_to_server};
use pinhole_protocol::storage::{StateMap, StateValue, StorageScope};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::net::{UnixListener, UnixStream};
use tokio::time::timeout;

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
/// Automatically performs capability negotiation handshake.
pub async fn connect_test_client(socket_path: &str) -> UnixStream {
    // Retry connection with backoff to handle server startup race
    for i in 0..10 {
        if let Ok(mut stream) = UnixStream::connect(socket_path).await {
            // Perform capability negotiation
            let capabilities = pinhole_protocol::supported_capabilities();
            send_message_to_server(
                &mut stream,
                ClientToServerMessage::ClientHello { capabilities },
            )
            .await
            .expect("Failed to send ClientHello");

            // Wait for ServerHello
            match receive_server_message(&mut stream).await {
                Ok(Some(ServerToClientMessage::ServerHello { .. })) => {
                    // Negotiation successful
                    return stream;
                }
                Ok(Some(ServerToClientMessage::Error { code, message })) => {
                    panic!("Capability negotiation failed: {:?} - {}", code, message);
                }
                Ok(Some(msg)) => {
                    panic!("Expected ServerHello, got: {:?}", msg);
                }
                Ok(None) => {
                    panic!("Connection closed during handshake");
                }
                Err(e) => {
                    panic!("Network error during handshake: {:?}", e);
                }
            }
        }
        tokio::task::yield_now().await;
        if i > 5 {
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
    }
    panic!("Failed to connect to test server at {}", socket_path)
}

/// Send a Load request to the server
pub async fn send_load(
    stream: &mut UnixStream,
    path: &str,
    storage: StateMap,
) -> Result<(), Box<dyn std::error::Error>> {
    send_message_to_server(
        stream,
        ClientToServerMessage::Load {
            path: path.to_string(),
            storage,
        },
    )
    .await
    .map_err(|e| e.into())
}

/// Send an Action request to the server
pub async fn send_action(
    stream: &mut UnixStream,
    path: &str,
    action: Action,
    storage: StateMap,
) -> Result<(), Box<dyn std::error::Error>> {
    send_message_to_server(
        stream,
        ClientToServerMessage::Action {
            path: path.to_string(),
            action,
            storage,
        },
    )
    .await
    .map_err(|e| e.into())
}

/// Send a simple Action request with just a name
pub async fn send_simple_action(
    stream: &mut UnixStream,
    path: &str,
    action_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    send_action(
        stream,
        path,
        Action::new(action_name, HashMap::new(), vec![]),
        StateMap::new(),
    )
    .await
}

/// Receive a message from the server
pub async fn receive_message(
    stream: &mut UnixStream,
) -> Result<ServerToClientMessage, Box<dyn std::error::Error>> {
    match receive_server_message(stream).await? {
        Some(msg) => Ok(msg),
        None => Err("Connection closed".into()),
    }
}

/// Receive all messages until a terminal message (Render, RedirectTo, or Error)
pub async fn receive_all_messages(
    stream: &mut UnixStream,
) -> Result<Vec<ServerToClientMessage>, Box<dyn std::error::Error>> {
    let mut messages = Vec::new();

    loop {
        let result = timeout(Duration::from_secs(2), receive_server_message(stream)).await;

        match result {
            Ok(Ok(Some(msg))) => {
                let is_terminal = matches!(
                    msg,
                    ServerToClientMessage::Render { .. }
                        | ServerToClientMessage::RedirectTo { .. }
                        | ServerToClientMessage::Error { .. }
                );
                messages.push(msg);
                if is_terminal {
                    break;
                }
            }
            Ok(Ok(None)) => break,
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => {
                // Timeout - if we have messages and no terminal, that's okay for actions
                if !messages.is_empty() {
                    break;
                }
                return Err("Timeout waiting for server message".into());
            }
        }
    }

    Ok(messages)
}
