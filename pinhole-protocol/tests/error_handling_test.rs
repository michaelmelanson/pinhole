use pinhole_protocol::messages::{ErrorCode, ServerToClientMessage};

#[test]
fn test_error_code_as_u16() {
    assert_eq!(ErrorCode::BadRequest.as_u16(), 400);
    assert_eq!(ErrorCode::NotFound.as_u16(), 404);
    assert_eq!(ErrorCode::InternalServerError.as_u16(), 500);
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
        ErrorCode::NotFound,
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

    assert!(ErrorCode::NotFound.as_u16() >= 400);
    assert!(ErrorCode::NotFound.as_u16() < 500);

    assert!(ErrorCode::InternalServerError.as_u16() >= 500);
    assert!(ErrorCode::InternalServerError.as_u16() < 600);
}
