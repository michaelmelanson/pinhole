use std::fmt;
use std::fs;
use std::io::Cursor;
use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, RootCertStore, ServerConfig, SignatureScheme};
use tokio_rustls::{TlsAcceptor, TlsConnector};

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

fn load_certs(pem_bytes: &[u8]) -> std::result::Result<Vec<CertificateDer<'static>>, String> {
    rustls_pemfile::certs(&mut Cursor::new(pem_bytes))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

/// rustls 0.23's multi-provider `CryptoProvider` architecture requires a
/// process-wide default to be installed before building any config, or it
/// panics at runtime. Installed once, lazily, the first time either side of
/// this module is used.
static CRYPTO_PROVIDER_INIT: std::sync::Once = std::sync::Once::new();

fn ensure_crypto_provider_installed() {
    CRYPTO_PROVIDER_INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

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
        ensure_crypto_provider_installed();

        let cert_pem =
            fs::read(&self.cert_path).map_err(|e| TlsConfigError::CertificateReadError {
                path: self.cert_path.clone(),
                source: e,
            })?;

        let key_pem = fs::read(&self.key_path).map_err(|e| TlsConfigError::KeyReadError {
            path: self.key_path.clone(),
            source: e,
        })?;

        let certs = load_certs(&cert_pem).map_err(TlsConfigError::IdentityParseError)?;

        let key = rustls_pemfile::private_key(&mut Cursor::new(&key_pem))
            .map_err(|e| TlsConfigError::IdentityParseError(e.to_string()))?
            .ok_or_else(|| {
                TlsConfigError::IdentityParseError("no private key found in key file".to_string())
            })?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| TlsConfigError::AcceptorBuildError(e.to_string()))?;

        Ok(TlsAcceptor::from(Arc::new(config)))
    }
}

/// Verifies nothing — used only for `ClientTlsConfig::new_danger_accept_invalid_certs()`,
/// for connecting to servers using self-signed development certificates.
#[derive(Debug)]
struct NoCertificateVerification(Arc<rustls::crypto::CryptoProvider>);

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
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
        ensure_crypto_provider_installed();

        let builder = ClientConfig::builder();

        let config = if self.accept_invalid_certs {
            builder
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoCertificateVerification(
                    rustls::crypto::CryptoProvider::get_default()
                        .expect("installed by ensure_crypto_provider_installed above")
                        .clone(),
                )))
                .with_no_client_auth()
        } else if let Some(ca_cert_path) = &self.ca_cert_path {
            let ca_cert_pem =
                fs::read(ca_cert_path).map_err(|e| TlsConfigError::CaCertificateReadError {
                    path: ca_cert_path.clone(),
                    source: e,
                })?;

            let ca_certs =
                load_certs(&ca_cert_pem).map_err(TlsConfigError::CaCertificateParseError)?;

            let mut root_store = RootCertStore::empty();
            for cert in ca_certs {
                root_store
                    .add(cert)
                    .map_err(|e| TlsConfigError::CaCertificateParseError(e.to_string()))?;
            }

            builder
                .with_root_certificates(root_store)
                .with_no_client_auth()
        } else {
            let mut root_store = RootCertStore::empty();
            let native_certs = rustls_native_certs::load_native_certs();

            for error in &native_certs.errors {
                tracing::warn!(%error, "Error reading a native certificate source");
            }

            let mut rejected = 0usize;
            for cert in native_certs.certs {
                if root_store.add(cert).is_err() {
                    rejected += 1;
                }
            }
            if rejected > 0 {
                tracing::warn!(
                    rejected,
                    trusted = root_store.len(),
                    "Platform trust store rejected some certificates"
                );
            }
            if root_store.is_empty() {
                return Err(TlsConfigError::CaCertificateParseError(
                    "no usable certificates found in the platform trust store".to_string(),
                ));
            }

            builder
                .with_root_certificates(root_store)
                .with_no_client_auth()
        };

        Ok(TlsConnector::from(Arc::new(config)))
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
