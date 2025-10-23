use pinhole_protocol::tls_config::{ClientTlsConfig, ServerTlsConfig, TlsConfigError};

#[test]
fn test_server_config_creation() {
    let config = ServerTlsConfig::new("cert.pem", "key.pem");
    assert_eq!(config.cert_path, "cert.pem");
    assert_eq!(config.key_path, "key.pem");
}

#[test]
fn test_client_config_danger_mode() {
    let config = ClientTlsConfig::new_danger_accept_invalid_certs();
    assert!(config.accept_invalid_certs);
}

#[test]
fn test_client_config_default() {
    let config = ClientTlsConfig::new();
    assert!(!config.accept_invalid_certs);
    assert!(config.ca_cert_path.is_none());
}

#[test]
fn test_client_config_with_ca() {
    let config = ClientTlsConfig::new().with_ca_cert("ca.pem");
    assert_eq!(config.ca_cert_path, Some("ca.pem".to_string()));
}

#[test]
fn test_client_config_build_connector() {
    let config = ClientTlsConfig::new_danger_accept_invalid_certs();
    let result = config.build_connector();
    assert!(
        result.is_ok(),
        "Should be able to build TLS connector: {:?}",
        result.err()
    );
}

#[test]
fn test_server_config_missing_cert_file() {
    let config = ServerTlsConfig::new("/nonexistent/path/cert.pem", "/nonexistent/path/key.pem");
    let result = config.build_acceptor();

    assert!(result.is_err(), "Should fail with missing cert file");

    let err = result.unwrap_err();
    assert!(
        matches!(err, TlsConfigError::CertificateReadError { .. }),
        "Should be CertificateReadError, got: {:?}",
        err
    );

    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("/nonexistent/path/cert.pem"),
        "Error should mention the missing file, got: {}",
        err_msg
    );
}

#[test]
fn test_server_config_missing_key_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a temporary cert file with valid content
    let mut cert_file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(cert_file, "-----BEGIN CERTIFICATE-----").expect("Failed to write");
    writeln!(cert_file, "fake cert content").expect("Failed to write");
    writeln!(cert_file, "-----END CERTIFICATE-----").expect("Failed to write");
    cert_file.flush().expect("Failed to flush");

    let config = ServerTlsConfig::new(
        cert_file.path().to_str().unwrap(),
        "/nonexistent/path/key.pem",
    );

    let result = config.build_acceptor();
    assert!(result.is_err(), "Should fail with missing key file");

    let err = result.unwrap_err();
    assert!(
        matches!(err, TlsConfigError::KeyReadError { .. }),
        "Should be KeyReadError, got: {:?}",
        err
    );

    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("/nonexistent/path/key.pem"),
        "Error should mention the missing file, got: {}",
        err_msg
    );
}

#[test]
fn test_client_config_missing_ca_file() {
    let config = ClientTlsConfig::new().with_ca_cert("/nonexistent/path/ca.pem");
    let result = config.build_connector();

    assert!(result.is_err(), "Should fail with missing CA cert file");

    let err = result.unwrap_err();
    assert!(
        matches!(err, TlsConfigError::CaCertificateReadError { .. }),
        "Should be CaCertificateReadError, got: {:?}",
        err
    );

    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("/nonexistent/path/ca.pem"),
        "Error should mention the missing file, got: {}",
        err_msg
    );
}
