//! Concurrent connection tests
//!
//! These tests verify that the server can handle multiple simultaneous client connections.
//! They use a global counter, so they must be run with --test-threads=1 to avoid interference.
//!
//! Run with: cargo test --test concurrent_connections_test -- --test-threads=1

use async_std::os::unix::net::UnixListener;
use async_std::prelude::*;
use async_std::task;
use async_trait::async_trait;
use pinhole::{Action, Application, Context, Document, Node, Render, Route, TextProps};
use pinhole_protocol::messages::{ClientToServerMessage, ServerToClientMessage};
use pinhole_protocol::network::{receive_server_message, send_message_to_server};
use pinhole_protocol::storage::StateMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

// Static counter for testing concurrent access
// Note: This means these tests must run serially (--test-threads=1)
static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn reset_counter() {
    GLOBAL_COUNTER.store(0, Ordering::SeqCst);
}

fn get_counter() -> usize {
    GLOBAL_COUNTER.load(Ordering::SeqCst)
}

// Test application
#[derive(Copy, Clone)]
struct ConcurrentTestApp;

impl Application for ConcurrentTestApp {
    fn routes(&self) -> Vec<Box<dyn Route>> {
        vec![Box::new(HelloRoute), Box::new(CounterRoute)]
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

// Route that increments a counter on each request (for testing concurrent access)
struct CounterRoute;

#[async_trait]
impl Route for CounterRoute {
    fn path(&self) -> &'static str {
        "/counter"
    }

    async fn action<'a>(
        &self,
        _action: &Action,
        _context: &mut Context<'a>,
    ) -> pinhole::Result<()> {
        Ok(())
    }

    async fn render(&self, _storage: &StateMap) -> Render {
        let count = GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst);
        Render::Document(Document {
            node: Node::Text(TextProps {
                text: format!("Request #{}", count),
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
) -> pinhole::Result<Vec<ServerToClientMessage>> {
    let mut stream = async_std::os::unix::net::UnixStream::connect(socket_path).await?;

    let request = ClientToServerMessage::Load {
        path: path.to_string(),
        storage: StateMap::new(),
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
    let socket_path = "/tmp/pinhole_test_concurrent.sock";

    // Clean up any existing socket
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)
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
        let socket_path = socket_path.to_string();
        let task = task::spawn(async move {
            send_request_and_receive(&socket_path, "/hello")
                .await
                .expect(&format!("Client {} failed", i))
        });
        client_tasks.push(task);
    }

    // Wait for all clients to complete
    for task in client_tasks {
        let messages = task.await;
        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0], ServerToClientMessage::Render { .. }));
    }

    // Clean up
    drop(server_task);
    let _ = std::fs::remove_file(socket_path);
}

#[async_std::test]
async fn test_concurrent_requests_to_shared_state() {
    reset_counter();

    let socket_path = "/tmp/pinhole_test_concurrent_state.sock";

    // Clean up any existing socket
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)
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

    // Spawn multiple concurrent clients hitting the counter route
    let num_requests = 10;
    let mut client_tasks = Vec::new();

    for i in 0..num_requests {
        let socket_path = socket_path.to_string();
        let task = task::spawn(async move {
            send_request_and_receive(&socket_path, "/counter")
                .await
                .expect(&format!("Client {} failed", i))
        });
        client_tasks.push(task);
    }

    // Wait for all clients to complete
    let mut received_counts = Vec::new();
    for task in client_tasks {
        let messages = task.await;
        assert_eq!(messages.len(), 1);

        if let ServerToClientMessage::Render { document } = &messages[0] {
            if let Node::Text(props) = &document.node {
                // Extract the count from "Request #N"
                if let Some(count_str) = props.text.strip_prefix("Request #") {
                    if let Ok(count) = count_str.parse::<usize>() {
                        received_counts.push(count);
                    }
                }
            }
        }
    }

    // All requests should have completed
    assert_eq!(received_counts.len(), num_requests);

    // The counter should have been incremented exactly num_requests times
    // (the counts should be 0..num_requests-1 in some order)
    received_counts.sort();
    let expected: Vec<usize> = (0..num_requests).collect();
    assert_eq!(
        received_counts, expected,
        "Counter should have incremented atomically"
    );

    // Verify the final count
    assert_eq!(get_counter(), num_requests);

    // Clean up
    drop(server_task);
    let _ = std::fs::remove_file(socket_path);
}

#[async_std::test]
async fn test_interleaved_requests() {
    reset_counter();

    let socket_path = "/tmp/pinhole_test_interleaved.sock";

    // Clean up any existing socket
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)
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
    let socket_path1 = socket_path.to_string();
    let socket_path2 = socket_path.to_string();

    let client1 = task::spawn(async move {
        for _ in 0..3 {
            send_request_and_receive(&socket_path1, "/hello")
                .await
                .expect("Client 1 failed");
        }
    });

    let client2 = task::spawn(async move {
        for _ in 0..3 {
            send_request_and_receive(&socket_path2, "/counter")
                .await
                .expect("Client 2 failed");
        }
    });

    // Wait for both clients to complete
    client1.await;
    client2.await;

    // Verify counter was incremented 3 times by client2
    assert_eq!(get_counter(), 3);

    // Clean up
    drop(server_task);
    let _ = std::fs::remove_file(socket_path);
}
