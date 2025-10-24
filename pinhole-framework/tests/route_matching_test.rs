//! Route Matching Tests
//!
//! These tests verify the route matching system including:
//! - Exact path matching
//! - Root path handling
//! - Multiple routes selection
//! - Non-existent paths (404 errors)
//! - Case sensitivity
//! - Edge cases (empty paths, trailing slashes, similar paths)
//! - Both Load and Action request types

#[cfg(test)]
mod common;

use async_trait::async_trait;
use common::{
    connect_test_client, receive_message, send_load, send_simple_action, start_test_server,
};
use pinhole::{Action, Application, Context, Document, Node, Render, Route, TextProps};
use pinhole_protocol::messages::{ErrorCode, ServerToClientMessage};
use pinhole_protocol::storage::StateMap;

// Test application with multiple routes
#[derive(Clone, Copy)]
struct RouteTestApp;

impl Application for RouteTestApp {
    fn routes(&self) -> Vec<Box<dyn Route>> {
        vec![
            Box::new(RootRoute),
            Box::new(UserRoute),
            Box::new(UsersRoute),
            Box::new(AboutRoute),
            Box::new(ContactRoute),
            Box::new(ApiV1Route),
        ]
    }
}

// Root route "/"
struct RootRoute;

#[async_trait]
impl Route for RootRoute {
    fn path(&self) -> &'static str {
        "/"
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
                text: "root".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

// User route "/user"
struct UserRoute;

#[async_trait]
impl Route for UserRoute {
    fn path(&self) -> &'static str {
        "/user"
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
                text: "user".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

// Users route "/users" (similar to "/user")
struct UsersRoute;

#[async_trait]
impl Route for UsersRoute {
    fn path(&self) -> &'static str {
        "/users"
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
                text: "users".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

// About route "/about"
struct AboutRoute;

#[async_trait]
impl Route for AboutRoute {
    fn path(&self) -> &'static str {
        "/about"
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
                text: "about".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

// Contact route "/contact"
struct ContactRoute;

#[async_trait]
impl Route for ContactRoute {
    fn path(&self) -> &'static str {
        "/contact"
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
                text: "contact".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

// API v1 route "/api/v1"
struct ApiV1Route;

#[async_trait]
impl Route for ApiV1Route {
    fn path(&self) -> &'static str {
        "/api/v1"
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
                text: "api-v1".to_string(),
                classes: vec![],
            }),
            stylesheet: Default::default(),
        })
    }
}

#[tokio::test]
async fn test_exact_path_match_root() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_load(&mut client, "/", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Render { document } => {
            assert_eq!(
                document.node,
                Node::Text(TextProps {
                    text: "root".to_string(),
                    classes: vec![],
                })
            );
        }
        _ => panic!("Expected Render message, got {:?}", message),
    }
}

#[tokio::test]
async fn test_exact_path_match_simple() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_load(&mut client, "/about", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Render { document } => {
            assert_eq!(
                document.node,
                Node::Text(TextProps {
                    text: "about".to_string(),
                    classes: vec![],
                })
            );
        }
        _ => panic!("Expected Render message"),
    }
}

#[tokio::test]
async fn test_exact_path_match_nested() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_load(&mut client, "/api/v1", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Render { document } => {
            assert_eq!(
                document.node,
                Node::Text(TextProps {
                    text: "api-v1".to_string(),
                    classes: vec![],
                })
            );
        }
        _ => panic!("Expected Render message"),
    }
}

#[tokio::test]
async fn test_similar_paths_user_vs_users() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    // Test /user
    send_load(&mut client, "/user", StateMap::new())
        .await
        .expect("Failed to send");
    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Render { document } => {
            assert_eq!(
                document.node,
                Node::Text(TextProps {
                    text: "user".to_string(),
                    classes: vec![],
                })
            );
        }
        _ => panic!("Expected Render message"),
    }

    // Test /users
    send_load(&mut client, "/users", StateMap::new())
        .await
        .expect("Failed to send");
    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Render { document } => {
            assert_eq!(
                document.node,
                Node::Text(TextProps {
                    text: "users".to_string(),
                    classes: vec![],
                })
            );
        }
        _ => panic!("Expected Render message"),
    }
}

#[tokio::test]
async fn test_non_existent_path_load() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_load(&mut client, "/nonexistent", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, message } => {
            assert_eq!(code, ErrorCode::NotFound);
            assert!(message.contains("/nonexistent"));
            assert!(message.contains("Route not found"));
        }
        _ => panic!("Expected Error message, got {:?}", message),
    }
}

#[tokio::test]
async fn test_non_existent_path_action() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    send_simple_action(&mut client, "/nonexistent", "test")
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, message } => {
            assert_eq!(code, ErrorCode::NotFound);
            assert!(message.contains("/nonexistent"));
            assert!(message.contains("Route not found"));
        }
        _ => panic!("Expected Error message"),
    }
}

#[tokio::test]
async fn test_case_sensitivity() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    // Route is "/about", so "/About" should not match
    send_load(&mut client, "/About", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, .. } => {
            assert_eq!(code, ErrorCode::NotFound);
        }
        _ => panic!("Expected Error message for case mismatch"),
    }
}

#[tokio::test]
async fn test_trailing_slash_mismatch() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    // Route is "/about", so "/about/" should not match
    send_load(&mut client, "/about/", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, .. } => {
            assert_eq!(code, ErrorCode::NotFound);
        }
        _ => panic!("Expected Error message for trailing slash"),
    }
}

#[tokio::test]
async fn test_empty_path() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    // Empty path should not match root "/"
    send_load(&mut client, "", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, .. } => {
            assert_eq!(code, ErrorCode::NotFound);
        }
        _ => panic!("Expected Error message for empty path"),
    }
}

#[tokio::test]
async fn test_path_with_query_string() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    // Query strings are not stripped, so this won't match "/about"
    send_load(&mut client, "/about?foo=bar", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, .. } => {
            assert_eq!(code, ErrorCode::NotFound);
        }
        _ => panic!("Expected Error message for path with query string"),
    }
}

#[tokio::test]
async fn test_path_with_fragment() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    // Fragments are not stripped, so this won't match "/about"
    send_load(&mut client, "/about#section", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, .. } => {
            assert_eq!(code, ErrorCode::NotFound);
        }
        _ => panic!("Expected Error message for path with fragment"),
    }
}

#[tokio::test]
async fn test_multiple_routes_load_all() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    let test_cases = vec![
        ("/", "root"),
        ("/user", "user"),
        ("/users", "users"),
        ("/about", "about"),
        ("/contact", "contact"),
        ("/api/v1", "api-v1"),
    ];

    for (path, expected_text) in test_cases {
        send_load(&mut client, path, StateMap::new())
            .await
            .expect("Failed to send");
        let message = receive_message(&mut client)
            .await
            .expect("Failed to receive");
        match message {
            ServerToClientMessage::Render { document } => {
                assert_eq!(
                    document.node,
                    Node::Text(TextProps {
                        text: expected_text.to_string(),
                        classes: vec![],
                    }),
                    "Path {} should render {}",
                    path,
                    expected_text
                );
            }
            _ => panic!("Expected Render message for path {}", path),
        }
    }
}

#[tokio::test]
async fn test_action_uses_same_route_matching() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    // Action on existing route should succeed (no error)
    send_simple_action(&mut client, "/about", "test")
        .await
        .expect("Failed to send");

    // Since the route's action method does nothing, we won't get a response message
    // But we shouldn't get an error either. Let's try sending a load after to verify server is still working
    send_load(&mut client, "/about", StateMap::new())
        .await
        .expect("Failed to send");
    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Render { .. } => {
            // Success - server processed the action without error
        }
        _ => panic!("Server should still be responsive after action"),
    }
}

#[tokio::test]
async fn test_partial_path_no_match() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    // "/api" should not match "/api/v1"
    send_load(&mut client, "/api", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, .. } => {
            assert_eq!(code, ErrorCode::NotFound);
        }
        _ => panic!("Expected Error message for partial path"),
    }
}

#[tokio::test]
async fn test_extended_path_no_match() {
    let socket_path = start_test_server(RouteTestApp);
    let mut client = connect_test_client(&socket_path).await;

    // "/api/v1/users" should not match "/api/v1"
    send_load(&mut client, "/api/v1/users", StateMap::new())
        .await
        .expect("Failed to send");

    let message = receive_message(&mut client)
        .await
        .expect("Failed to receive");
    match message {
        ServerToClientMessage::Error { code, .. } => {
            assert_eq!(code, ErrorCode::NotFound);
        }
        _ => panic!("Expected Error message for extended path"),
    }
}
