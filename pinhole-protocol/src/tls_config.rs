use std::fmt;
use std::fs;
use tokio_native_tls::{
    native_tls::{Certificate, Identity},
    TlsAcceptor, TlsConnector,
};

/// TLS configuration errors
#[derive(Debug)]
pub enum TlsConfigError {
    /// Failed to read certificate file
    CertificateReadError {
        path: String,
        source: std::io::Error,
    },
    /// Failed to read key file
    KeyReadError {
        path: String,
        source: std::io::Error,
    },
    /// Failed to read CA certificate file
    CaCertificateReadError {
        path: String,
        source: std::io::Error,
    },
    /// Failed to parse certificate/key
    IdentityParseError(String),
    /// Failed to parse CA certificate
    CaCertificateParseError(String),
    /// Failed to build TLS acceptor
    AcceptorBuildError(String),
}

impl fmt::Display for TlsConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TlsConfigError::CertificateReadError { path, source } => {
                write!(f, "Failed to read certificate file '{}': {}", path, source)
            }
            TlsConfigError::KeyReadError { path, source } => {
                write!(f, "Failed to read key file '{}': {}", path, source)
            }
            TlsConfigError::CaCertificateReadError { path, source } => {
                write!(
                    f,
                    "Failed to read CA certificate file '{}': {}",
                    path, source
                )
            }
            TlsConfigError::IdentityParseError(msg) => {
                write!(f, "Failed to parse certificate/key: {}", msg)
            }
            TlsConfigError::CaCertificateParseError(msg) => {
                write!(f, "Failed to parse CA certificate: {}", msg)
            }
            TlsConfigError::AcceptorBuildError(msg) => {
                write!(f, "Failed to build TLS acceptor: {}", msg)
            }
        }
    }
}

impl std::error::Error for TlsConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TlsConfigError::CertificateReadError { source, .. } => Some(source),
            TlsConfigError::KeyReadError { source, .. } => Some(source),
            TlsConfigError::CaCertificateReadError { source, .. } => Some(source),
            _ => None,
        }
    }
}

type Result<T> = std::result::Result<T, TlsConfigError>;

/// Server-side TLS configuration
#[derive(Clone)]
pub struct ServerTlsConfig {
    /// Path to the PEM-encoded certificate file
    pub cert_path: String,
    /// Path to the PEM-encoded private key file
    pub key_path: String,
}

impl ServerTlsConfig {
    /// Create a new ServerTlsConfig with certificate and key file paths
    pub fn new(cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        ServerTlsConfig {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
        }
    }

    /// Load the certificate and key from disk and create a TlsAcceptor
    pub fn build_acceptor(&self) -> Result<TlsAcceptor> {
        let cert_pem = fs::read_to_string(&self.cert_path).map_err(|e| {
            TlsConfigError::CertificateReadError {
                path: self.cert_path.clone(),
                source: e,
            }
        })?;

        let key_pem =
            fs::read_to_string(&self.key_path).map_err(|e| TlsConfigError::KeyReadError {
                path: self.key_path.clone(),
                source: e,
            })?;

        // Parse certificate and key into identity
        let identity = Identity::from_pkcs8(cert_pem.as_bytes(), key_pem.as_bytes())
            .map_err(|e| TlsConfigError::IdentityParseError(e.to_string()))?;

        let acceptor = TlsAcceptor::from(
            native_tls::TlsAcceptor::builder(identity)
                .build()
                .map_err(|e| TlsConfigError::AcceptorBuildError(e.to_string()))?,
        );

        Ok(acceptor)
    }
}

/// Client-side TLS configuration
#[derive(Clone)]
pub struct ClientTlsConfig {
    /// Whether to accept invalid certificates (for development with self-signed certs)
    pub accept_invalid_certs: bool,
    /// Optional custom CA certificate path for validating server certificates
    pub ca_cert_path: Option<String>,
}

impl ClientTlsConfig {
    /// Create a new ClientTlsConfig with default settings (strict certificate validation)
    pub fn new() -> Self {
        ClientTlsConfig {
            accept_invalid_certs: false,
            ca_cert_path: None,
        }
    }

    /// Create a ClientTlsConfig that accepts invalid certificates (for development)
    pub fn new_danger_accept_invalid_certs() -> Self {
        ClientTlsConfig {
            accept_invalid_certs: true,
            ca_cert_path: None,
        }
    }

    /// Set a custom CA certificate for validating server certificates
    pub fn with_ca_cert(mut self, ca_cert_path: impl Into<String>) -> Self {
        self.ca_cert_path = Some(ca_cert_path.into());
        self
    }

    /// Build a TlsConnector from this configuration
    pub fn build_connector(&self) -> Result<TlsConnector> {
        let mut builder = native_tls::TlsConnector::builder();

        if self.accept_invalid_certs {
            builder
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true);
        }

        if let Some(ca_cert_path) = &self.ca_cert_path {
            let ca_cert_pem =
                fs::read(ca_cert_path).map_err(|e| TlsConfigError::CaCertificateReadError {
                    path: ca_cert_path.clone(),
                    source: e,
                })?;

            let ca_cert = Certificate::from_pem(&ca_cert_pem)
                .map_err(|e| TlsConfigError::CaCertificateParseError(e.to_string()))?;

            builder.add_root_certificate(ca_cert);
        }

        let native_connector = builder
            .build()
            .map_err(|e| TlsConfigError::AcceptorBuildError(e.to_string()))?;
        let connector = TlsConnector::from(native_connector);

        Ok(connector)
    }
}

impl Default for ClientTlsConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_creation() {
        let config = ServerTlsConfig::new("cert.pem", "key.pem");
        assert_eq!(config.cert_path, "cert.pem");
        assert_eq!(config.key_path, "key.pem");
    }

    #[test]
    fn test_client_config_defaults() {
        let config = ClientTlsConfig::new();
        assert!(!config.accept_invalid_certs);
        assert!(config.ca_cert_path.is_none());
    }

    #[test]
    fn test_client_config_danger_mode() {
        let config = ClientTlsConfig::new_danger_accept_invalid_certs();
        assert!(config.accept_invalid_certs);
    }

    #[test]
    fn test_client_config_with_ca() {
        let config = ClientTlsConfig::new().with_ca_cert("ca.pem");
        assert_eq!(config.ca_cert_path, Some("ca.pem".to_string()));
    }
}
