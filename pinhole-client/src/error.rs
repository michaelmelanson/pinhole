use pinhole_protocol::tls_config::TlsConfigError;
use std::fmt;

/// Client-side network errors
#[derive(Debug)]
pub enum NetworkError {
    /// Invalid address format (expected host:port)
    InvalidAddress(String),
    /// TLS handshake failed
    TlsHandshakeFailed(String),
    /// TLS connector build failed
    TlsConnectorBuildFailed(String),
    /// TCP connection failed
    TcpConnectionFailed(std::io::Error),
    /// Protocol error (serialization, deserialization)
    ProtocolError(String),
    /// Storage error
    StorageError(String),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::InvalidAddress(addr) => {
                write!(
                    f,
                    "Invalid address format '{}' (expected 'host:port')",
                    addr
                )
            }
            NetworkError::TlsHandshakeFailed(msg) => {
                write!(f, "TLS handshake failed: {}", msg)
            }
            NetworkError::TcpConnectionFailed(err) => {
                write!(f, "TCP connection failed: {}", err)
            }
            NetworkError::ProtocolError(msg) => {
                write!(f, "Protocol error: {}", msg)
            }
            NetworkError::StorageError(msg) => {
                write!(f, "Storage error: {}", msg)
            }
            NetworkError::TlsConnectorBuildFailed(msg) => {
                write!(f, "Failed to build TLS connector: {}", msg)
            }
        }
    }
}

impl std::error::Error for NetworkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NetworkError::TcpConnectionFailed(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for NetworkError {
    fn from(err: std::io::Error) -> Self {
        NetworkError::TcpConnectionFailed(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for NetworkError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        // All boxed errors from protocol layer are protocol errors
        // IO errors should be caught and converted directly at call sites
        NetworkError::ProtocolError(err.to_string())
    }
}

impl From<TlsConfigError> for NetworkError {
    fn from(err: TlsConfigError) -> Self {
        NetworkError::TlsConnectorBuildFailed(err.to_string())
    }
}

impl From<pinhole_protocol::network::NetworkError> for NetworkError {
    fn from(err: pinhole_protocol::network::NetworkError) -> Self {
        NetworkError::ProtocolError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_address_display() {
        let err = NetworkError::InvalidAddress("bad".to_string());
        assert!(err.to_string().contains("Invalid address format"));
        assert!(err.to_string().contains("bad"));
    }

    #[test]
    fn test_tls_handshake_display() {
        let err = NetworkError::TlsHandshakeFailed("cert invalid".to_string());
        assert!(err.to_string().contains("TLS handshake failed"));
        assert!(err.to_string().contains("cert invalid"));
    }

    #[test]
    fn test_tcp_connection_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let err = NetworkError::TcpConnectionFailed(io_err);
        assert!(err.to_string().contains("TCP connection failed"));
    }

    #[test]
    fn test_protocol_error_display() {
        let err = NetworkError::ProtocolError("bad message".to_string());
        assert!(err.to_string().contains("Protocol error"));
        assert!(err.to_string().contains("bad message"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let net_err: NetworkError = io_err.into();
        assert!(matches!(net_err, NetworkError::TcpConnectionFailed(_)));
    }
}
