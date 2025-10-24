use async_std::future::timeout;
use async_std::os::unix::net::UnixStream;
use async_std::task;
use async_trait::async_trait;
use pinhole::{Action, Application, Context, Document, Node, Render, Route, TextProps};
use pinhole_protocol::messages::{ClientToServerMessage, ErrorCode, ServerToClientMessage};
use pinhole_protocol::network::{receive_server_message, send_message_to_server};
use pinhole_protocol::storage::{StateMap, StateValue, StorageScope};
use std::time::Duration;

/// Simple test application
#[derive(Clone, Copy)]
struct TestApp;

impl Application for TestApp {
    fn routes(&self) -> Vec<Box<dyn Route>> {
        vec![Box::new(HelloRoute), Box::new(CounterRoute)]
    }
}

/// Simple route that returns a greeting
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
                text: "Hello from real server!".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

/// Counter route that uses storage
struct CounterRoute;

#[async_trait]
impl Route for CounterRoute {
    fn path(&self) -> &'static str {
        "/counter"
    }

    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> pinhole::Result<()> {
        if action.name == "increment" {
            let count = context
                .storage
                .get("count")
                .and_then(|v| match v {
                    StateValue::String(s) => s.parse::<i32>().ok(),
                    _ => None,
                })
                .unwrap_or(0);

            let new_count = count + 1;
            context
                .store(
                    StorageScope::Session,
                    "count".to_string(),
                    StateValue::String(new_count.to_string()),
                )
                .await?;
        }

        Ok(())
    }

    async fn render(&self, storage: &StateMap) -> Render {
        let count = storage
            .get("count")
            .and_then(|v| match v {
                StateValue::String(s) => s.parse::<i32>().ok(),
                _ => None,
            })
            .unwrap_or(0);

        Render::Document(Document {
            node: Node::Text(TextProps {
                text: format!("Count: {}", count),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

/// Test fixture that manages client-server connection
struct TestFixture {
    client: TestClient,
}

impl TestFixture {
    /// Set up a new test with server running TestApp
    fn new() -> Self {
        let (client_stream, server_stream) =
            UnixStream::pair().expect("Failed to create socket pair");
        let app = TestApp;

        // Spawn server task
        task::spawn(async move {
            let mut stream = server_stream;
            let _ = pinhole::handle_connection(app, &mut stream).await;
        });

        TestFixture {
            client: TestClient::new(client_stream),
        }
    }

    /// Assert that messages contain a single Render with expected node
    fn assert_render(messages: &[ServerToClientMessage], expected_node: Node) {
        assert_eq!(messages.len(), 1);
        let ServerToClientMessage::Render { document } = &messages[0] else {
            panic!("Expected Render message");
        };
        assert_eq!(document.node, expected_node);
    }

    /// Assert that messages contain a single Store message with expected values
    fn assert_store(
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
    fn assert_error(
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
}

/// Helper to create storage with a count value
fn count_storage(count: i32) -> StateMap {
    let mut storage = StateMap::new();
    storage.insert("count".to_string(), StateValue::String(count.to_string()));
    storage
}

/// Helper to create an action
fn simple_action(name: &str) -> Action {
    Action {
        name: name.to_string(),
        args: std::collections::HashMap::new(),
        keys: vec![],
    }
}

/// Client helper that sends requests and receives responses
struct TestClient {
    stream: UnixStream,
}

impl TestClient {
    fn new(stream: UnixStream) -> Self {
        TestClient { stream }
    }

    async fn send_load(&mut self, path: &str, storage: StateMap) -> pinhole::Result<()> {
        let request = ClientToServerMessage::Load {
            path: path.to_string(),
            storage,
        };
        send_message_to_server(&mut self.stream, request).await?;
        Ok(())
    }

    async fn send_action(
        &mut self,
        path: &str,
        action: Action,
        storage: StateMap,
    ) -> pinhole::Result<()> {
        let request = ClientToServerMessage::Action {
            path: path.to_string(),
            action,
            storage,
        };
        send_message_to_server(&mut self.stream, request).await?;
        Ok(())
    }

    async fn receive_message(&mut self) -> pinhole::Result<Option<ServerToClientMessage>> {
        timeout(
            Duration::from_secs(2),
            receive_server_message(&mut self.stream),
        )
        .await
        .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> {
            "Timeout waiting for server message".into()
        })?
        .map_err(|e| e.into())
    }

    async fn receive_all_messages(&mut self) -> pinhole::Result<Vec<ServerToClientMessage>> {
        let mut messages = Vec::new();

        // Read messages until we get a terminal message (Render, RedirectTo, or Error)
        // or until timeout
        loop {
            match self.receive_message().await {
                Ok(Some(msg)) => {
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
                Ok(None) => break,
                Err(e) => {
                    // If we already have messages and timeout, that's okay - action responses might not have terminal messages
                    if !messages.is_empty() && e.to_string().contains("Timeout") {
                        break;
                    }
                    return Err(e);
                }
            }
        }

        Ok(messages)
    }
}

#[async_std::test]
async fn test_real_client_server_basic_load() {
    let mut fixture = TestFixture::new();

    fixture
        .client
        .send_load("/hello", StateMap::new())
        .await
        .expect("Failed to send load");

    let messages = fixture
        .client
        .receive_all_messages()
        .await
        .expect("Failed to receive");

    TestFixture::assert_render(
        &messages,
        Node::Text(TextProps {
            text: "Hello from real server!".to_string(),
            classes: vec![],
        }),
    );
}

#[async_std::test]
async fn test_real_client_server_with_storage() {
    let mut fixture = TestFixture::new();

    // First load - counter should be 0
    fixture
        .client
        .send_load("/counter", StateMap::new())
        .await
        .expect("Failed to send load");

    let messages = fixture
        .client
        .receive_all_messages()
        .await
        .expect("Failed to receive");
    TestFixture::assert_render(
        &messages,
        Node::Text(TextProps {
            text: "Count: 0".to_string(),
            classes: vec![],
        }),
    );

    // Send increment action
    fixture
        .client
        .send_action("/counter", simple_action("increment"), count_storage(0))
        .await
        .expect("Failed to send action");

    let messages = fixture
        .client
        .receive_all_messages()
        .await
        .expect("Failed to receive");

    // Actions don't automatically re-render, so we just get Store message
    TestFixture::assert_store(&messages, "count", StateValue::String("1".to_string()));

    // Now send a Load request to see the updated count
    fixture
        .client
        .send_load("/counter", count_storage(1))
        .await
        .expect("Failed to send load");

    let messages = fixture
        .client
        .receive_all_messages()
        .await
        .expect("Failed to receive");

    TestFixture::assert_render(
        &messages,
        Node::Text(TextProps {
            text: "Count: 1".to_string(),
            classes: vec![],
        }),
    );
}

#[async_std::test]
async fn test_real_client_server_route_not_found() {
    let mut fixture = TestFixture::new();

    fixture
        .client
        .send_load("/nonexistent", StateMap::new())
        .await
        .expect("Failed to send load");

    let messages = fixture
        .client
        .receive_all_messages()
        .await
        .expect("Failed to receive");

    TestFixture::assert_error(&messages, ErrorCode::NotFound, "/nonexistent");
}

#[async_std::test]
async fn test_real_client_server_multiple_requests() {
    let mut fixture = TestFixture::new();

    // Send multiple requests over the same connection
    for _ in 0..3 {
        fixture
            .client
            .send_load("/hello", StateMap::new())
            .await
            .expect("Failed to send load");

        let messages = fixture
            .client
            .receive_all_messages()
            .await
            .expect("Failed to receive");

        TestFixture::assert_render(
            &messages,
            Node::Text(TextProps {
                text: "Hello from real server!".to_string(),
                classes: vec![],
            }),
        );
    }
}
