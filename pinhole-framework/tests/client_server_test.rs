#[cfg(test)]
mod common;

use async_trait::async_trait;
use common::{
    assert_error, assert_redirect, assert_render, assert_store, connect_test_client,
    receive_all_messages, send_action, send_load, start_test_server,
};
use pinhole::{
    Action, Application, ButtonProps, ContainerProps, Context, Document, Node, Params, Render,
    Route, TextProps,
};
use pinhole_protocol::messages::ErrorCode;
use pinhole_protocol::storage::{StateMap, StateValue, StorageScope};
use pinhole_protocol::stylesheet::Direction;

/// Simple test application
#[derive(Clone, Copy)]
struct TestApp;

impl Application for TestApp {
    fn routes(&self) -> Vec<Box<dyn Route>> {
        vec![
            Box::new(HelloRoute),
            Box::new(CounterRoute),
            Box::new(RedirectRoute),
            Box::new(ErrorRoute),
            Box::new(ButtonRoute),
        ]
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
        _params: &Params,
        _context: &mut Context<'a>,
    ) -> pinhole::Result<()> {
        Ok(())
    }

    async fn render(&self, _params: &Params, _storage: &StateMap) -> Render {
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

    async fn action<'a>(
        &self,
        action: &Action,
        _params: &Params,
        context: &mut Context<'a>,
    ) -> pinhole::Result<()> {
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

    async fn render(&self, _params: &Params, storage: &StateMap) -> Render {
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

/// Route that redirects to another route
struct RedirectRoute;

#[async_trait]
impl Route for RedirectRoute {
    fn path(&self) -> &'static str {
        "/redirect"
    }

    async fn action<'a>(
        &self,
        _action: &Action,
        _params: &Params,
        _context: &mut Context<'a>,
    ) -> pinhole::Result<()> {
        Ok(())
    }

    async fn render(&self, _params: &Params, _storage: &StateMap) -> Render {
        Render::RedirectTo("/hello".to_string())
    }
}

/// Route that throws an error
struct ErrorRoute;

#[async_trait]
impl Route for ErrorRoute {
    fn path(&self) -> &'static str {
        "/error"
    }

    async fn action<'a>(
        &self,
        _action: &Action,
        _params: &Params,
        _context: &mut Context<'a>,
    ) -> pinhole::Result<()> {
        Err("Intentional error from action".into())
    }

    async fn render(&self, _params: &Params, _storage: &StateMap) -> Render {
        // Return a document that will work, but the route is meant to test error handling
        // We'll test render errors by making the action fail instead
        Render::Document(Document {
            node: Node::Text(TextProps {
                text: "This shouldn't be reached".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

/// Route that returns a button node
struct ButtonRoute;

#[async_trait]
impl Route for ButtonRoute {
    fn path(&self) -> &'static str {
        "/button"
    }

    async fn action<'a>(
        &self,
        _action: &Action,
        _params: &Params,
        _context: &mut Context<'a>,
    ) -> pinhole::Result<()> {
        Ok(())
    }

    async fn render(&self, _params: &Params, _storage: &StateMap) -> Render {
        Render::Document(Document {
            node: Node::Container(ContainerProps {
                direction: Direction::Vertical,
                children: vec![Node::Button(ButtonProps {
                    label: "Click me!".to_string(),
                    on_click: simple_action("click"),
                    classes: vec![],
                })],
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
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

#[tokio::test]
async fn test_real_client_server_basic_load() {
    let socket_path = start_test_server(TestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_load(&mut client, "/hello", StateMap::new())
        .await
        .expect("Failed to send load");

    let messages = receive_all_messages(&mut client)
        .await
        .expect("Failed to receive");

    assert_render(
        &messages,
        Node::Text(TextProps {
            text: "Hello from real server!".to_string(),
            classes: vec![],
        }),
    );
}

#[tokio::test]
async fn test_real_client_server_with_storage() {
    let socket_path = start_test_server(TestApp);
    let mut client = connect_test_client(&socket_path).await;

    // First load - counter should be 0
    send_load(&mut client, "/counter", StateMap::new())
        .await
        .expect("Failed to send load");

    let messages = receive_all_messages(&mut client)
        .await
        .expect("Failed to receive");
    assert_render(
        &messages,
        Node::Text(TextProps {
            text: "Count: 0".to_string(),
            classes: vec![],
        }),
    );

    // Send increment action
    send_action(
        &mut client,
        "/counter",
        simple_action("increment"),
        count_storage(0),
    )
    .await
    .expect("Failed to send action");

    let messages = receive_all_messages(&mut client)
        .await
        .expect("Failed to receive");

    // Actions don't automatically re-render, so we just get Store message
    assert_store(&messages, "count", StateValue::String("1".to_string()));

    // Now send a Load request to see the updated count
    send_load(&mut client, "/counter", count_storage(1))
        .await
        .expect("Failed to send load");

    let messages = receive_all_messages(&mut client)
        .await
        .expect("Failed to receive");

    assert_render(
        &messages,
        Node::Text(TextProps {
            text: "Count: 1".to_string(),
            classes: vec![],
        }),
    );
}

#[tokio::test]
async fn test_real_client_server_route_not_found() {
    let socket_path = start_test_server(TestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_load(&mut client, "/nonexistent", StateMap::new())
        .await
        .expect("Failed to send load");

    let messages = receive_all_messages(&mut client)
        .await
        .expect("Failed to receive");

    assert_error(&messages, ErrorCode::NotFound, "/nonexistent");
}

#[tokio::test]
async fn test_real_client_server_multiple_requests() {
    let socket_path = start_test_server(TestApp);
    let mut client = connect_test_client(&socket_path).await;

    // Send multiple requests over the same connection
    for _ in 0..3 {
        send_load(&mut client, "/hello", StateMap::new())
            .await
            .expect("Failed to send load");

        let messages = receive_all_messages(&mut client)
            .await
            .expect("Failed to receive");

        assert_render(
            &messages,
            Node::Text(TextProps {
                text: "Hello from real server!".to_string(),
                classes: vec![],
            }),
        );
    }
}

#[tokio::test]
async fn test_redirect_response() {
    let socket_path = start_test_server(TestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_load(&mut client, "/redirect", StateMap::new())
        .await
        .expect("Failed to send load");

    let messages = receive_all_messages(&mut client)
        .await
        .expect("Failed to receive");

    assert_redirect(&messages, "/hello");
}

#[tokio::test]
async fn test_action_route_not_found() {
    let socket_path = start_test_server(TestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_action(
        &mut client,
        "/nonexistent",
        simple_action("test"),
        StateMap::new(),
    )
    .await
    .expect("Failed to send action");

    let messages = receive_all_messages(&mut client)
        .await
        .expect("Failed to receive");

    assert_error(&messages, ErrorCode::NotFound, "/nonexistent");
}

#[tokio::test]
async fn test_internal_error_from_action() {
    let socket_path = start_test_server(TestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_action(
        &mut client,
        "/error",
        simple_action("test"),
        StateMap::new(),
    )
    .await
    .expect("Failed to send action");

    let messages = receive_all_messages(&mut client)
        .await
        .expect("Failed to receive");

    assert_error(
        &messages,
        ErrorCode::InternalServerError,
        "Intentional error",
    );
}

#[tokio::test]
async fn test_button_and_container_nodes() {
    let socket_path = start_test_server(TestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_load(&mut client, "/button", StateMap::new())
        .await
        .expect("Failed to send load");

    let messages = receive_all_messages(&mut client)
        .await
        .expect("Failed to receive");

    assert_render(
        &messages,
        Node::Container(ContainerProps {
            direction: Direction::Vertical,
            children: vec![Node::Button(ButtonProps {
                label: "Click me!".to_string(),
                on_click: simple_action("click"),
                classes: vec![],
            })],
            classes: vec![],
        }),
    );
}
