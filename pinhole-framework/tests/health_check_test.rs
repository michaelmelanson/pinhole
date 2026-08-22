//! Tests for the `HealthCheck`/`HealthCheckResult` protocol messages and
//! their relationship to `ServerHealth`'s draining state.

use pinhole::{Application, Route, ServerHealth};
use pinhole_protocol::messages::{ClientToServerMessage, ServerToClientMessage};
use pinhole_protocol::network::{receive_server_message, send_message_to_server};
use tokio::net::UnixStream;

#[derive(Clone, Copy)]
struct TestApp;

impl Application for TestApp {
    fn routes(&self) -> Vec<Box<dyn Route>> {
        vec![]
    }
}

#[tokio::test]
async fn test_health_check_reports_healthy_by_default() {
    let (mut client_stream, server_stream) =
        UnixStream::pair().expect("Failed to create socket pair");
    let app = TestApp;
    let health = ServerHealth::new();

    let health_for_server = health.clone();
    tokio::spawn(async move {
        let mut stream = server_stream;
        let _ = pinhole::handle_connection(app, &mut stream, &health_for_server).await;
    });

    send_message_to_server(&mut client_stream, ClientToServerMessage::HealthCheck)
        .await
        .expect("Failed to send HealthCheck");

    let response = receive_server_message(&mut client_stream)
        .await
        .expect("Failed to receive response")
        .expect("Connection closed unexpectedly");

    assert_eq!(
        response,
        ServerToClientMessage::HealthCheckResult { healthy: true }
    );
}

#[tokio::test]
async fn test_health_check_reports_unhealthy_once_draining() {
    let (mut client_stream, server_stream) =
        UnixStream::pair().expect("Failed to create socket pair");
    let app = TestApp;
    let health = ServerHealth::new();

    let health_for_server = health.clone();
    tokio::spawn(async move {
        let mut stream = server_stream;
        let _ = pinhole::handle_connection(app, &mut stream, &health_for_server).await;
    });

    // Confirm healthy before draining starts.
    send_message_to_server(&mut client_stream, ClientToServerMessage::HealthCheck)
        .await
        .expect("Failed to send first HealthCheck");
    let first_response = receive_server_message(&mut client_stream)
        .await
        .expect("Failed to receive response")
        .expect("Connection closed unexpectedly");
    assert_eq!(
        first_response,
        ServerToClientMessage::HealthCheckResult { healthy: true }
    );

    // Simulate graceful shutdown beginning, on the same already-open connection.
    health.mark_draining();

    send_message_to_server(&mut client_stream, ClientToServerMessage::HealthCheck)
        .await
        .expect("Failed to send second HealthCheck");
    let second_response = receive_server_message(&mut client_stream)
        .await
        .expect("Failed to receive response")
        .expect("Connection closed unexpectedly");
    assert_eq!(
        second_response,
        ServerToClientMessage::HealthCheckResult { healthy: false }
    );
}
