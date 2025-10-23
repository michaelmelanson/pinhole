use pinhole_protocol::network::NetworkError;

#[test]
fn test_max_message_size_constant() {
    // Verify the constant is set to 10MB as intended
    // We can't directly access MAX_MESSAGE_SIZE since it's private,
    // but we can verify behavior at the boundary

    // This test documents the expected limit
    const EXPECTED_MAX_SIZE: u32 = 10 * 1024 * 1024; // 10 MB

    // If we send a message claiming to be just over 10MB, it should fail
    let oversized = EXPECTED_MAX_SIZE + 1;

    // We can't easily test the actual network functions without setting up
    // a full TCP connection, but we verify the error type exists
    let err = NetworkError::MessageTooLarge {
        size: oversized,
        max: EXPECTED_MAX_SIZE,
    };

    let err_msg = format!("{}", err);
    assert!(err_msg.contains("exceeds maximum"));
    assert!(err_msg.contains(&oversized.to_string()));
    assert!(err_msg.contains(&EXPECTED_MAX_SIZE.to_string()));
}

#[test]
fn test_network_error_display() {
    let err = NetworkError::MessageTooLarge {
        size: 20_000_000,
        max: 10_000_000,
    };

    let msg = format!("{}", err);
    assert!(msg.contains("20000000"));
    assert!(msg.contains("10000000"));
    assert!(msg.contains("exceeds maximum"));
}

#[test]
fn test_network_error_io_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "test");
    let net_err: NetworkError = io_err.into();

    assert!(matches!(net_err, NetworkError::IoError(_)));
    assert!(format!("{}", net_err).contains("IO error"));
}

#[test]
fn test_network_error_serialization() {
    let err = NetworkError::SerializationError("bad data".to_string());
    let msg = format!("{}", err);

    assert!(msg.contains("Serialization error"));
    assert!(msg.contains("bad data"));
}

#[test]
fn test_network_error_to_boxed() {
    let err = NetworkError::MessageTooLarge { size: 100, max: 50 };

    let boxed: Box<dyn std::error::Error + Send + Sync> = err.into();
    assert!(boxed.to_string().contains("exceeds maximum"));
}
