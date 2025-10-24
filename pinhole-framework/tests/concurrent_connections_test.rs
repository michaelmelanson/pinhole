//! Concurrent connection tests
//!
//! These tests verify that the server can handle multiple simultaneous client connections.

#[cfg(test)]
mod common;

use async_std::os::unix::net::UnixListener;
use async_std::prelude::*;
use async_std::task;
use async_trait::async_trait;
use common::assert_render;
use pinhole::{Action, Application, Context, Document, Node, Render, Route, TextProps};
use pinhole_protocol::messages::{ClientToServerMessage, ServerToClientMessage};
use pinhole_protocol::network::{receive_server_message, send_message_to_server};
use pinhole_protocol::storage::{StateMap, StateValue};
use std::time::Duration;
use tempfile::NamedTempFile;

// Test application
#[derive(Copy, Clone)]
struct ConcurrentTestApp;

impl Application for ConcurrentTestApp {
    fn routes(&self) -> Vec<Box<dyn Route>> {
        vec![Box::new(HelloRoute), Box::new(EchoRoute)]
    }
}

struct HelloRoute;

#[async_trait]
impl Route for HelloRoute {
    fn path(&self) -> &'static str {
        "/hello"
    }

    async fn action<'a>(
        &self,
        _action: &Action,
        _context: &mut Context<'a>,
    ) -> pinhole::Result<()> {
        Ok(())
    }

    async fn render(&self, _storage: &StateMap) -> Render {
        Render::Document(Document {
            node: Node::Text(TextProps {
                text: "Hello".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

struct EchoRoute;

#[async_trait]
impl Route for EchoRoute {
    fn path(&self) -> &'static str {
        "/echo"
    }

    async fn action<'a>(
        &self,
        _action: &Action,
        _context: &mut Context<'a>,
    ) -> pinhole::Result<()> {
        Ok(())
    }

    async fn render(&self, storage: &StateMap) -> Render {
        // Echo back the "client_id" value from storage
        let text = if let Some(StateValue::String(id)) = storage.get("client_id") {
            format!("Echo: {}", id)
        } else {
            "No client_id".to_string()
        };

        Render::Document(Document {
            node: Node::Text(TextProps {
                text,
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

/// Helper to connect a client and send a request
async fn send_request_and_receive(
    socket_path: &str,
    path: &str,
    storage: StateMap,
) -> pinhole::Result<Vec<ServerToClientMessage>> {
    let mut stream = async_std::os::unix::net::UnixStream::connect(socket_path).await?;

    let request = ClientToServerMessage::Load {
        path: path.to_string(),
        storage,
    };

    send_message_to_server(&mut stream, request).await?;

    let mut messages = Vec::new();
    loop {
        match async_std::future::timeout(
            Duration::from_secs(2),
            receive_server_message(&mut stream),
        )
        .await
        {
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
            Err(_) => return Err("Timeout waiting for response".into()),
        }
    }

    Ok(messages)
}

#[async_std::test]
async fn test_multiple_concurrent_connections() {
    // Create unique temporary socket path
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let socket_path = temp_file.path().with_extension("sock");
    drop(temp_file); // Delete the temp file so we can use the path for a socket

    let listener = UnixListener::bind(&socket_path)
        .await
        .expect("Failed to bind socket");

    let app = ConcurrentTestApp;

    // Spawn server that accepts multiple connections
    let server_task = task::spawn(async move {
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            if let Ok(mut stream) = stream {
                task::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });

    // Give server time to start
    task::sleep(Duration::from_millis(50)).await;

    // Spawn multiple concurrent clients
    let num_clients = 5;
    let mut client_tasks = Vec::new();

    for i in 0..num_clients {
        let socket_path_clone = socket_path.clone();
        let task = task::spawn(async move {
            send_request_and_receive(
                socket_path_clone.to_str().unwrap(),
                "/hello",
                StateMap::new(),
            )
            .await
            .expect(&format!("Client {} failed", i))
        });
        client_tasks.push(task);
    }

    // Wait for all clients to complete
    for task in client_tasks {
        let messages = task.await;
        assert_render(
            &messages,
            Node::Text(TextProps {
                text: "Hello".to_string(),
                classes: vec![],
            }),
        );
    }

    // Clean up
    drop(server_task);
    let _ = std::fs::remove_file(&socket_path);
}

#[async_std::test]
async fn test_concurrent_requests_to_shared_state() {
    // Create unique temporary socket path
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let socket_path = temp_file.path().with_extension("sock");
    drop(temp_file); // Delete the temp file so we can use the path for a socket

    let listener = UnixListener::bind(&socket_path)
        .await
        .expect("Failed to bind socket");

    let app = ConcurrentTestApp;

    // Spawn server that accepts multiple connections
    let server_task = task::spawn(async move {
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            if let Ok(mut stream) = stream {
                task::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });

    // Give server time to start
    task::sleep(Duration::from_millis(50)).await;

    // Spawn multiple concurrent clients, each with their own client_id
    let num_requests = 10;
    let mut client_tasks = Vec::new();

    for i in 0..num_requests {
        let socket_path_clone = socket_path.clone();
        let task = task::spawn(async move {
            let mut storage = StateMap::new();
            storage.insert(
                "client_id".to_string(),
                StateValue::String(format!("client-{}", i)),
            );

            let messages =
                send_request_and_receive(socket_path_clone.to_str().unwrap(), "/echo", storage)
                    .await
                    .expect(&format!("Client {} failed", i));

            (i, messages)
        });
        client_tasks.push(task);
    }

    // Wait for all clients to complete and verify each got their own ID back
    let mut received_ids = Vec::new();
    for task in client_tasks {
        let (client_num, messages) = task.await;
        assert_eq!(messages.len(), 1);

        let ServerToClientMessage::Render { document } = &messages[0] else {
            panic!("Expected Render message");
        };

        let Node::Text(props) = &document.node else {
            panic!("Expected Text node");
        };

        // Should get back "Echo: client-N"
        let expected = format!("Echo: client-{}", client_num);
        assert_eq!(props.text, expected);
        received_ids.push(client_num);
    }

    // All requests should have completed
    assert_eq!(received_ids.len(), num_requests);

    // All IDs should be unique
    received_ids.sort();
    let expected: Vec<usize> = (0..num_requests).collect();
    assert_eq!(received_ids, expected);

    // Clean up
    drop(server_task);
    let _ = std::fs::remove_file(&socket_path);
}

#[async_std::test]
async fn test_interleaved_requests() {
    // Create unique temporary socket path
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let socket_path = temp_file.path().with_extension("sock");
    drop(temp_file); // Delete the temp file so we can use the path for a socket

    let listener = UnixListener::bind(&socket_path)
        .await
        .expect("Failed to bind socket");

    let app = ConcurrentTestApp;

    // Spawn server
    let server_task = task::spawn(async move {
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            if let Ok(mut stream) = stream {
                task::spawn(async move {
                    let _ = pinhole::handle_connection(app, &mut stream).await;
                });
            }
        }
    });

    // Give server time to start
    task::sleep(Duration::from_millis(50)).await;

    // Create two clients that will send multiple requests
    let socket_path1 = socket_path.clone();
    let socket_path2 = socket_path.clone();

    let client1 = task::spawn(async move {
        let mut storage = StateMap::new();
        storage.insert(
            "client_id".to_string(),
            StateValue::String("client-A".to_string()),
        );

        // Send 3 requests to /echo
        for i in 0..3 {
            let messages =
                send_request_and_receive(socket_path1.to_str().unwrap(), "/echo", storage.clone())
                    .await
                    .expect(&format!("Client 1 request {} failed", i));

            assert_eq!(messages.len(), 1);

            let ServerToClientMessage::Render { document } = &messages[0] else {
                panic!("Expected Render message");
            };

            let Node::Text(props) = &document.node else {
                panic!("Expected Text node");
            };

            assert_eq!(props.text, "Echo: client-A");
        }
    });

    let client2 = task::spawn(async move {
        let mut storage = StateMap::new();
        storage.insert(
            "client_id".to_string(),
            StateValue::String("client-B".to_string()),
        );

        // Send 3 requests to /echo
        for i in 0..3 {
            let messages =
                send_request_and_receive(socket_path2.to_str().unwrap(), "/echo", storage.clone())
                    .await
                    .expect(&format!("Client 2 request {} failed", i));

            assert_eq!(messages.len(), 1);

            let ServerToClientMessage::Render { document } = &messages[0] else {
                panic!("Expected Render message");
            };

            let Node::Text(props) = &document.node else {
                panic!("Expected Text node");
            };

            assert_eq!(props.text, "Echo: client-B");
        }
    });

    // Wait for both clients to complete
    client1.await;
    client2.await;

    // Clean up
    drop(server_task);
    let _ = std::fs::remove_file(&socket_path);
}
