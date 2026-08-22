use pinhole_protocol::messages::{ErrorCode, ServerToClientMessage};

#[test]
fn test_error_code_as_u16() {
    assert_eq!(ErrorCode::BadRequest.as_u16(), 400);
    assert_eq!(ErrorCode::Unauthorized.as_u16(), 401);
    assert_eq!(ErrorCode::Forbidden.as_u16(), 403);
    assert_eq!(ErrorCode::NotFound.as_u16(), 404);
    assert_eq!(ErrorCode::UpgradeRequired.as_u16(), 426);
    assert_eq!(ErrorCode::TooManyRequests.as_u16(), 429);
    assert_eq!(ErrorCode::InternalServerError.as_u16(), 500);
}

#[test]
fn test_error_code_retryable_classification() {
    // Only TooManyRequests signals a transient, client-controlled condition
    assert!(ErrorCode::TooManyRequests.is_retryable());

    assert!(!ErrorCode::BadRequest.is_retryable());
    assert!(!ErrorCode::Unauthorized.is_retryable());
    assert!(!ErrorCode::Forbidden.is_retryable());
    assert!(!ErrorCode::NotFound.is_retryable());
    assert!(!ErrorCode::UpgradeRequired.is_retryable());
    assert!(!ErrorCode::InternalServerError.is_retryable());
}

#[test]
fn test_error_code_display() {
    assert_eq!(ErrorCode::BadRequest.to_string(), "400 Bad Request");
    assert_eq!(ErrorCode::Unauthorized.to_string(), "401 Unauthorized");
    assert_eq!(ErrorCode::Forbidden.to_string(), "403 Forbidden");
    assert_eq!(ErrorCode::NotFound.to_string(), "404 Not Found");
    assert_eq!(
        ErrorCode::UpgradeRequired.to_string(),
        "426 Upgrade Required"
    );
    assert_eq!(
        ErrorCode::TooManyRequests.to_string(),
        "429 Too Many Requests"
    );
    assert_eq!(
        ErrorCode::InternalServerError.to_string(),
        "500 Internal Server Error"
    );
}

#[test]
fn test_error_message_serialization() {
    let error_msg = ServerToClientMessage::Error {
        code: ErrorCode::NotFound,
        message: "Route not found".to_string(),
    };

    // Serialize to CBOR
    let serialized = serde_cbor::to_vec(&error_msg).unwrap();

    // Deserialize back
    let deserialized: ServerToClientMessage = serde_cbor::from_slice(&serialized).unwrap();

    match deserialized {
        ServerToClientMessage::Error { code, message } => {
            assert_eq!(code.as_u16(), 404);
            assert_eq!(message, "Route not found");
        }
        _ => panic!("Expected Error variant"),
    }
}

#[test]
fn test_error_code_serialization() {
    // Test that ErrorCode serializes and deserializes correctly
    let codes = vec![
        ErrorCode::BadRequest,
        ErrorCode::Unauthorized,
        ErrorCode::Forbidden,
        ErrorCode::NotFound,
        ErrorCode::UpgradeRequired,
        ErrorCode::TooManyRequests,
        ErrorCode::InternalServerError,
    ];

    for code in codes {
        let serialized = serde_cbor::to_vec(&code).unwrap();
        let deserialized: ErrorCode = serde_cbor::from_slice(&serialized).unwrap();
        assert_eq!(code.as_u16(), deserialized.as_u16());
    }
}

#[test]
fn test_all_error_codes_have_valid_status_codes() {
    // Ensure all error codes map to valid HTTP status codes
    assert!(ErrorCode::BadRequest.as_u16() >= 400);
    assert!(ErrorCode::BadRequest.as_u16() < 500);

    assert!(ErrorCode::Unauthorized.as_u16() >= 400);
    assert!(ErrorCode::Unauthorized.as_u16() < 500);

    assert!(ErrorCode::Forbidden.as_u16() >= 400);
    assert!(ErrorCode::Forbidden.as_u16() < 500);

    assert!(ErrorCode::NotFound.as_u16() >= 400);
    assert!(ErrorCode::NotFound.as_u16() < 500);

    assert!(ErrorCode::TooManyRequests.as_u16() >= 400);
    assert!(ErrorCode::TooManyRequests.as_u16() < 500);

    assert!(ErrorCode::InternalServerError.as_u16() >= 500);
    assert!(ErrorCode::InternalServerError.as_u16() < 600);
}
