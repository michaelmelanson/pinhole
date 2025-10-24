//! Comprehensive Action Dispatch Tests
//!
//! These tests verify the action dispatching system including:
//! - Action argument handling
//! - Action keys and state capture
//! - Multiple actions on same route
//! - Storage modifications via context.store()
//! - Redirects from actions via context.redirect()
//! - Error handling
//! - Edge cases and invalid inputs

use async_trait::async_trait;
use pinhole::{Action, Application, Context, Document, Node, Render, Route, TextProps};
use pinhole_protocol::messages::{ClientToServerMessage, ErrorCode, ServerToClientMessage};
use pinhole_protocol::network::{receive_server_message, send_message_to_server};
use pinhole_protocol::storage::{StateMap, StateValue, StorageScope};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::net::{UnixListener, UnixStream};

// Test application
#[derive(Clone, Copy)]
struct ActionTestApp;

impl Application for ActionTestApp {
    fn routes(&self) -> Vec<Box<dyn Route>> {
        vec![
            Box::new(ArgumentsRoute),
            Box::new(KeysRoute),
            Box::new(MultiActionRoute),
            Box::new(StorageRoute),
            Box::new(RedirectRoute),
            Box::new(ErrorRoute),
            Box::new(ComplexDataRoute),
        ]
    }
}

/// Route that tests action with arguments
struct ArgumentsRoute;

#[async_trait]
impl Route for ArgumentsRoute {
    fn path(&self) -> &'static str {
        "/arguments"
    }

    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> pinhole::Result<()> {
        match action.name.as_str() {
            "echo_args" => {
                // Store each argument back to verify they were received
                for (key, value) in &action.args {
                    context
                        .store(
                            StorageScope::Session,
                            key.clone(),
                            StateValue::String(value.clone()),
                        )
                        .await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn render(&self, _storage: &StateMap) -> Render {
        Render::Document(Document {
            node: Node::Text(TextProps {
                text: "Arguments test".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

/// Route that tests action with keys (field capture)
struct KeysRoute;

#[async_trait]
impl Route for KeysRoute {
    fn path(&self) -> &'static str {
        "/keys"
    }

    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> pinhole::Result<()> {
        match action.name.as_str() {
            "submit" => {
                // Echo back the captured fields to verify they were received
                for key in &action.keys {
                    if let Some(value) = context.storage.get(key) {
                        context
                            .store(
                                StorageScope::Session,
                                format!("captured_{}", key),
                                value.clone(),
                            )
                            .await?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn render(&self, _storage: &StateMap) -> Render {
        Render::Document(Document {
            node: Node::Text(TextProps {
                text: "Keys test".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

/// Route that tests multiple different actions
struct MultiActionRoute;

#[async_trait]
impl Route for MultiActionRoute {
    fn path(&self) -> &'static str {
        "/multi"
    }

    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> pinhole::Result<()> {
        match action.name.as_str() {
            "increment" => {
                let count = context
                    .storage
                    .get("count")
                    .and_then(|v| v.string().parse::<i32>().ok())
                    .unwrap_or(0);
                context
                    .store(
                        StorageScope::Session,
                        "count",
                        StateValue::String((count + 1).to_string()),
                    )
                    .await?;
            }
            "decrement" => {
                let count = context
                    .storage
                    .get("count")
                    .and_then(|v| v.string().parse::<i32>().ok())
                    .unwrap_or(0);
                context
                    .store(
                        StorageScope::Session,
                        "count",
                        StateValue::String((count - 1).to_string()),
                    )
                    .await?;
            }
            "reset" => {
                context
                    .store(
                        StorageScope::Session,
                        "count",
                        StateValue::String("0".to_string()),
                    )
                    .await?;
            }
            "unknown" => {
                // Store a marker to show unknown action was handled
                context
                    .store(
                        StorageScope::Session,
                        "unknown_handled",
                        StateValue::Boolean(true),
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn render(&self, _storage: &StateMap) -> Render {
        Render::Document(Document {
            node: Node::Text(TextProps {
                text: "Multi-action test".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

/// Route that tests storage modifications
struct StorageRoute;

#[async_trait]
impl Route for StorageRoute {
    fn path(&self) -> &'static str {
        "/storage"
    }

    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> pinhole::Result<()> {
        match action.name.as_str() {
            "store_session" => {
                context
                    .store(StorageScope::Session, "session_key", "session_value")
                    .await?;
            }
            "store_persistent" => {
                context
                    .store(
                        StorageScope::Persistent,
                        "persistent_key",
                        "persistent_value",
                    )
                    .await?;
            }
            "store_multiple" => {
                context
                    .store(StorageScope::Session, "key1", "value1")
                    .await?;
                context
                    .store(StorageScope::Session, "key2", "value2")
                    .await?;
                context
                    .store(StorageScope::Persistent, "key3", "value3")
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn render(&self, _storage: &StateMap) -> Render {
        Render::Document(Document {
            node: Node::Text(TextProps {
                text: "Storage test".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

/// Route that tests redirects
struct RedirectRoute;

#[async_trait]
impl Route for RedirectRoute {
    fn path(&self) -> &'static str {
        "/redirect"
    }

    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> pinhole::Result<()> {
        match action.name.as_str() {
            "go_home" => {
                context.redirect("/").await?;
            }
            "go_to_path" => {
                if let Some(path) = action.args.get("path") {
                    context.redirect(path).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn render(&self, _storage: &StateMap) -> Render {
        Render::Document(Document {
            node: Node::Text(TextProps {
                text: "Redirect test".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

/// Route that tests error handling
struct ErrorRoute;

#[async_trait]
impl Route for ErrorRoute {
    fn path(&self) -> &'static str {
        "/error"
    }

    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> pinhole::Result<()> {
        match action.name.as_str() {
            "trigger_error" => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Intentional test error",
                )));
            }
            "conditional_error" => {
                if let Some(should_error) = action.args.get("error") {
                    if should_error == "true" {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Conditional error triggered",
                        )));
                    }
                }
                // Success case - store a marker
                context
                    .store(StorageScope::Session, "success", StateValue::Boolean(true))
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn render(&self, _storage: &StateMap) -> Render {
        Render::Document(Document {
            node: Node::Text(TextProps {
                text: "Error test".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

/// Route that tests complex data types
struct ComplexDataRoute;

#[async_trait]
impl Route for ComplexDataRoute {
    fn path(&self) -> &'static str {
        "/complex"
    }

    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> pinhole::Result<()> {
        match action.name.as_str() {
            "store_boolean" => {
                if let Some(value_str) = action.args.get("value") {
                    let value = value_str == "true";
                    context
                        .store(
                            StorageScope::Session,
                            "bool_value",
                            StateValue::Boolean(value),
                        )
                        .await?;
                }
            }
            "store_empty" => {
                context
                    .store(StorageScope::Session, "empty_value", StateValue::Empty)
                    .await?;
            }
            "read_from_storage" => {
                // Echo back what we read from storage
                for key in &action.keys {
                    if let Some(value) = context.storage.get(key) {
                        context
                            .store(
                                StorageScope::Session,
                                format!("read_{}", key),
                                value.clone(),
                            )
                            .await?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn render(&self, _storage: &StateMap) -> Render {
        Render::Document(Document {
            node: Node::Text(TextProps {
                text: "Complex data test".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

// Helper to create test server and client
async fn setup_test_server() -> (UnixListener, String) {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let socket_path = temp_file.path().with_extension("sock");
    drop(temp_file);

    let listener = UnixListener::bind(&socket_path).expect("Failed to bind socket");
    let socket_path_str = socket_path.to_string_lossy().to_string();

    (listener, socket_path_str)
}

async fn connect_client(socket_path: &str) -> UnixStream {
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
    panic!("Failed to connect to server")
}

async fn send_action(
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

async fn receive_message(
    stream: &mut UnixStream,
) -> Result<ServerToClientMessage, Box<dyn std::error::Error>> {
    match receive_server_message(stream).await? {
        Some(msg) => Ok(msg),
        None => Err("Connection closed".into()),
    }
}

#[tokio::test]
async fn test_action_with_single_argument() {
    let (listener, socket_path) = setup_test_server().await;

    // Start server
    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    // Send action with single argument
    let mut args = HashMap::new();
    args.insert("test_key".to_string(), "test_value".to_string());
    let action = Action::new("echo_args", args, vec![]);

    send_action(&mut client, "/arguments", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive message");

    // Should get Store message echoing back our argument
    match message {
        ServerToClientMessage::Store { scope, key, value } => {
            assert_eq!(scope, StorageScope::Session);
            assert_eq!(key, "test_key");
            assert_eq!(value, StateValue::String("test_value".to_string()));
        }
        _ => panic!("Expected Store message, got {:?}", message),
    }
}

#[tokio::test]
async fn test_action_with_multiple_arguments() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    // Send action with multiple arguments
    let mut args = HashMap::new();
    args.insert("key1".to_string(), "value1".to_string());
    args.insert("key2".to_string(), "value2".to_string());
    let action = Action::new("echo_args", args, vec![]);

    send_action(&mut client, "/arguments", action, StateMap::new())
        .await
        .expect("Failed to send action");

    // Receive all store messages
    let mut received_keys = Vec::new();
    for _ in 0..2 {
        let message = receive_message(&mut client)
            .await
            .expect("Failed to receive message");
        match message {
            ServerToClientMessage::Store { key, .. } => {
                received_keys.push(key);
            }
            _ => panic!("Expected Store message"),
        }
    }

    assert!(received_keys.contains(&"key1".to_string()));
    assert!(received_keys.contains(&"key2".to_string()));
}

#[tokio::test]
async fn test_action_with_keys_capturing_storage() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    // Create storage with field values
    let mut storage = StateMap::new();
    storage.insert(
        "email".to_string(),
        StateValue::String("test@example.com".to_string()),
    );
    storage.insert(
        "password".to_string(),
        StateValue::String("secret123".to_string()),
    );

    // Send action with keys to capture from storage
    let action = Action::new(
        "submit",
        HashMap::new(),
        vec!["email".to_string(), "password".to_string()],
    );

    send_action(&mut client, "/keys", action, storage)
        .await
        .expect("Failed to send action");

    // Receive captured fields
    for _ in 0..2 {
        let message = receive_message(&mut client)
            .await
            .expect("Failed to receive message");
        match message {
            ServerToClientMessage::Store { key, value, .. } => {
                if key == "captured_email" {
                    assert_eq!(value, StateValue::String("test@example.com".to_string()));
                } else if key == "captured_password" {
                    assert_eq!(value, StateValue::String("secret123".to_string()));
                }
            }
            _ => panic!("Expected Store message"),
        }
    }
}

#[tokio::test]
async fn test_multiple_actions_on_same_route() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    // Test increment action
    let action = Action::new("increment", HashMap::new(), vec![]);
    let mut storage = StateMap::new();
    storage.insert("count".to_string(), StateValue::String("0".to_string()));

    send_action(&mut client, "/multi", action, storage.clone())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Store { key, value, .. } => {
            assert_eq!(key, "count");
            assert_eq!(value, StateValue::String("1".to_string()));
        }
        _ => panic!("Expected Store message"),
    }

    // Test decrement action
    storage.insert("count".to_string(), StateValue::String("5".to_string()));
    let action = Action::new("decrement", HashMap::new(), vec![]);

    send_action(&mut client, "/multi", action, storage)
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Store { key, value, .. } => {
            assert_eq!(key, "count");
            assert_eq!(value, StateValue::String("4".to_string()));
        }
        _ => panic!("Expected Store message"),
    }

    // Test reset action
    let action = Action::new("reset", HashMap::new(), vec![]);
    send_action(&mut client, "/multi", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Store { key, value, .. } => {
            assert_eq!(key, "count");
            assert_eq!(value, StateValue::String("0".to_string()));
        }
        _ => panic!("Expected Store message"),
    }
}

#[tokio::test]
async fn test_action_with_storage_scopes() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    // Test session storage
    let action = Action::new("store_session", HashMap::new(), vec![]);
    send_action(&mut client, "/storage", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Store { scope, key, value } => {
            assert_eq!(scope, StorageScope::Session);
            assert_eq!(key, "session_key");
            assert_eq!(value, StateValue::String("session_value".to_string()));
        }
        _ => panic!("Expected Store message"),
    }

    // Test persistent storage
    let action = Action::new("store_persistent", HashMap::new(), vec![]);
    send_action(&mut client, "/storage", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Store { scope, key, value } => {
            assert_eq!(scope, StorageScope::Persistent);
            assert_eq!(key, "persistent_key");
            assert_eq!(value, StateValue::String("persistent_value".to_string()));
        }
        _ => panic!("Expected Store message"),
    }
}

#[tokio::test]
async fn test_action_with_redirect() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    let action = Action::new("go_home", HashMap::new(), vec![]);
    send_action(&mut client, "/redirect", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::RedirectTo { path } => {
            assert_eq!(path, "/");
        }
        _ => panic!("Expected RedirectTo message, got {:?}", message),
    }
}

#[tokio::test]
async fn test_action_with_dynamic_redirect_path() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    let mut args = HashMap::new();
    args.insert("path".to_string(), "/custom/path".to_string());
    let action = Action::new("go_to_path", args, vec![]);

    send_action(&mut client, "/redirect", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::RedirectTo { path } => {
            assert_eq!(path, "/custom/path");
        }
        _ => panic!("Expected RedirectTo message"),
    }
}

#[tokio::test]
async fn test_action_error_handling() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    let action = Action::new("trigger_error", HashMap::new(), vec![]);
    send_action(&mut client, "/error", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, message } => {
            assert_eq!(code, ErrorCode::InternalServerError);
            assert!(message.contains("Intentional test error"));
        }
        _ => panic!("Expected Error message, got {:?}", message),
    }
}

#[tokio::test]
async fn test_action_conditional_error() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    // Test error case
    let mut args = HashMap::new();
    args.insert("error".to_string(), "true".to_string());
    let action = Action::new("conditional_error", args, vec![]);

    send_action(&mut client, "/error", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, .. } => {
            assert_eq!(code, ErrorCode::InternalServerError);
        }
        _ => panic!("Expected Error message"),
    }

    // Test success case
    let mut args = HashMap::new();
    args.insert("error".to_string(), "false".to_string());
    let action = Action::new("conditional_error", args, vec![]);

    send_action(&mut client, "/error", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Store { key, value, .. } => {
            assert_eq!(key, "success");
            assert_eq!(value, StateValue::Boolean(true));
        }
        _ => panic!("Expected Store message"),
    }
}

#[tokio::test]
async fn test_action_with_boolean_storage() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    let mut args = HashMap::new();
    args.insert("value".to_string(), "true".to_string());
    let action = Action::new("store_boolean", args, vec![]);

    send_action(&mut client, "/complex", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Store { key, value, .. } => {
            assert_eq!(key, "bool_value");
            assert_eq!(value, StateValue::Boolean(true));
        }
        _ => panic!("Expected Store message"),
    }
}

#[tokio::test]
async fn test_action_with_empty_storage_value() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    let action = Action::new("store_empty", HashMap::new(), vec![]);
    send_action(&mut client, "/complex", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Store { key, value, .. } => {
            assert_eq!(key, "empty_value");
            assert_eq!(value, StateValue::Empty);
        }
        _ => panic!("Expected Store message"),
    }
}

#[tokio::test]
async fn test_action_route_not_found() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    let action = Action::new("any_action", HashMap::new(), vec![]);
    send_action(&mut client, "/nonexistent", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, message } => {
            assert_eq!(code, ErrorCode::NotFound);
            assert!(message.contains("/nonexistent"));
        }
        _ => panic!("Expected Error message"),
    }
}

#[tokio::test]
async fn test_action_with_empty_arguments() {
    let (listener, socket_path) = setup_test_server().await;

    let _server = tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let app = ActionTestApp;
                tokio::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });


    let mut client = connect_client(&socket_path).await;

    // Action with empty args should work fine - increment just uses default
    let action = Action::new("increment", HashMap::new(), vec![]);
    send_action(&mut client, "/multi", action, StateMap::new())
        .await
        .expect("Failed to send action");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Store { key, value, .. } => {
            assert_eq!(key, "count");
            assert_eq!(value, StateValue::String("1".to_string())); // 0 + 1
        }
        _ => panic!("Expected Store message"),
    }
}
