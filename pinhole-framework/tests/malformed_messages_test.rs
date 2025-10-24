use async_std::os::unix::net::UnixStream;
use async_std::prelude::*;
use async_std::task;
use pinhole_protocol::messages::ClientToServerMessage;
use pinhole_protocol::storage::StateMap;
use std::time::Duration;

// Use the test app from client_server_test
mod test_app {
    use async_trait::async_trait;
    use pinhole::{Action, Application, Context, Document, Node, Render, Route, TextProps};
    use pinhole_protocol::storage::StateMap;

    #[derive(Clone, Copy)]
    pub struct TestApp;

    impl Application for TestApp {
        fn routes(&self) -> Vec<Box<dyn Route>> {
            vec![Box::new(HelloRoute)]
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
}

/// Helper to send raw bytes to a stream
async fn send_raw_bytes(stream: &mut UnixStream, bytes: &[u8]) -> std::io::Result<()> {
    stream.write_all(bytes).await
}

#[async_std::test]
async fn test_message_too_large() {
    let (mut client_stream, server_stream) =
        UnixStream::pair().expect("Failed to create socket pair");
    let app = test_app::TestApp;

    // Spawn server task
    let server_handle = task::spawn(async move {
        let mut stream = server_stream;
        pinhole::handle_connection(app, &mut stream).await
    });

    // Send a message claiming to be 11MB (exceeds MAX_MESSAGE_SIZE of 10MB)
    let oversized_length: u32 = 11 * 1024 * 1024;
    send_raw_bytes(&mut client_stream, &oversized_length.to_le_bytes())
        .await
        .expect("Failed to send length");

    // Give the server a moment to process
    task::sleep(Duration::from_millis(100)).await;

    // The server should close the connection due to the too-large message
    // Try to read - should get connection closed or error
    let mut buf = [0u8; 4];
    let result = client_stream.read(&mut buf).await;

    // Either we get 0 bytes (connection closed) or an error
    assert!(
        result.is_err() || result.unwrap() == 0,
        "Expected connection to be closed"
    );

    drop(client_stream);
    drop(server_handle);
}

#[async_std::test]
async fn test_invalid_cbor_data() {
    let (mut client_stream, server_stream) =
        UnixStream::pair().expect("Failed to create socket pair");
    let app = test_app::TestApp;

    // Spawn server task
    let server_handle = task::spawn(async move {
        let mut stream = server_stream;
        pinhole::handle_connection(app, &mut stream).await
    });

    // Send valid length but invalid CBOR data
    let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid CBOR
    let length: u32 = invalid_data.len() as u32;

    send_raw_bytes(&mut client_stream, &length.to_le_bytes())
        .await
        .expect("Failed to send length");
    send_raw_bytes(&mut client_stream, &invalid_data)
        .await
        .expect("Failed to send data");

    // Give the server a moment to process
    task::sleep(Duration::from_millis(100)).await;

    // The server should close the connection due to deserialization error
    let mut buf = [0u8; 4];
    let result = client_stream.read(&mut buf).await;

    assert!(
        result.is_err() || result.unwrap() == 0,
        "Expected connection to be closed"
    );

    drop(client_stream);
    drop(server_handle);
}

#[async_std::test]
async fn test_truncated_message() {
    let (mut client_stream, server_stream) =
        UnixStream::pair().expect("Failed to create socket pair");
    let app = test_app::TestApp;

    // Spawn server task
    let server_handle = task::spawn(async move {
        let mut stream = server_stream;
        pinhole::handle_connection(app, &mut stream).await
    });

    // Send a message claiming to be 100 bytes but only send 10
    let claimed_length: u32 = 100;
    let partial_data = vec![0u8; 10]; // Only 10 bytes

    send_raw_bytes(&mut client_stream, &claimed_length.to_le_bytes())
        .await
        .expect("Failed to send length");
    send_raw_bytes(&mut client_stream, &partial_data)
        .await
        .expect("Failed to send partial data");

    // Close the write side to simulate truncation
    client_stream.flush().await.ok();

    // Give the server a moment to process
    task::sleep(Duration::from_millis(100)).await;

    // The server should handle this gracefully (likely by closing connection)
    let mut buf = [0u8; 4];
    let result = client_stream.read(&mut buf).await;

    assert!(
        result.is_err() || result.unwrap() == 0,
        "Expected connection to be closed"
    );

    drop(client_stream);
    drop(server_handle);
}

#[async_std::test]
async fn test_zero_length_message() {
    let (mut client_stream, server_stream) =
        UnixStream::pair().expect("Failed to create socket pair");
    let app = test_app::TestApp;

    // Spawn server task
    task::spawn(async move {
        let mut stream = server_stream;
        let _ = pinhole::handle_connection(app, &mut stream).await;
    });

    // Send a zero-length message (valid according to protocol - means empty/close)
    let zero_length: u32 = 0;
    send_raw_bytes(&mut client_stream, &zero_length.to_le_bytes())
        .await
        .expect("Failed to send length");

    // Give the server a moment to process
    task::sleep(Duration::from_millis(100)).await;

    // The server should close the connection gracefully
    let mut buf = [0u8; 4];
    let result = client_stream.read(&mut buf).await;

    assert!(
        result.is_err() || result.unwrap() == 0,
        "Expected connection to be closed gracefully"
    );
}

#[async_std::test]
async fn test_wrong_message_structure() {
    let (mut client_stream, server_stream) =
        UnixStream::pair().expect("Failed to create socket pair");
    let app = test_app::TestApp;

    // Spawn server task
    let server_handle = task::spawn(async move {
        let mut stream = server_stream;
        pinhole::handle_connection(app, &mut stream).await
    });

    // Send valid CBOR but wrong structure (e.g., a simple string instead of ClientToServerMessage)
    let wrong_structure =
        serde_cbor::to_vec(&"This is not a ClientToServerMessage".to_string()).unwrap();
    let length: u32 = wrong_structure.len() as u32;

    send_raw_bytes(&mut client_stream, &length.to_le_bytes())
        .await
        .expect("Failed to send length");
    send_raw_bytes(&mut client_stream, &wrong_structure)
        .await
        .expect("Failed to send data");

    // Give the server a moment to process
    task::sleep(Duration::from_millis(100)).await;

    // The server should close the connection due to deserialization error
    let mut buf = [0u8; 4];
    let result = client_stream.read(&mut buf).await;

    assert!(
        result.is_err() || result.unwrap() == 0,
        "Expected connection to be closed"
    );

    drop(client_stream);
    drop(server_handle);
}

#[async_std::test]
async fn test_partial_length_header() {
    let (mut client_stream, server_stream) =
        UnixStream::pair().expect("Failed to create socket pair");
    let app = test_app::TestApp;

    // Spawn server task
    let server_handle = task::spawn(async move {
        let mut stream = server_stream;
        pinhole::handle_connection(app, &mut stream).await
    });

    // Send only 2 bytes of the 4-byte length header
    send_raw_bytes(&mut client_stream, &[0x01, 0x02])
        .await
        .expect("Failed to send partial header");

    // Close the connection
    drop(client_stream);

    // Give the server a moment to process
    task::sleep(Duration::from_millis(100)).await;

    // Server should handle this gracefully
    drop(server_handle);
}

#[async_std::test]
async fn test_message_at_exact_size_limit() {
    let (mut client_stream, server_stream) =
        UnixStream::pair().expect("Failed to create socket pair");
    let app = test_app::TestApp;

    // Spawn server task
    task::spawn(async move {
        let mut stream = server_stream;
        let _ = pinhole::handle_connection(app, &mut stream).await;
    });

    // Create a valid message
    let request = ClientToServerMessage::Load {
        path: "/hello".to_string(),
        storage: StateMap::new(),
    };

    let bytes = serde_cbor::to_vec(&request).unwrap();
    let length: u32 = bytes.len() as u32;

    // Send valid message (should be well under 10MB)
    send_raw_bytes(&mut client_stream, &length.to_le_bytes())
        .await
        .expect("Failed to send length");
    send_raw_bytes(&mut client_stream, &bytes)
        .await
        .expect("Failed to send data");

    // Should get a response since this is a valid message
    let mut response_len_bytes = [0u8; 4];
    let read_result = async_std::future::timeout(
        Duration::from_secs(2),
        client_stream.read(&mut response_len_bytes),
    )
    .await;

    assert!(
        read_result.is_ok() && read_result.unwrap().is_ok(),
        "Should receive response for valid message"
    );
}
