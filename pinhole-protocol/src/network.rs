use async_native_tls::TlsStream;
use async_std::{net::TcpStream, prelude::*};

use crate::messages::{ClientToServerMessage, ServerToClientMessage};

use kv_log_macro as log;
use std::fmt;

/// Maximum message size: 10MB
/// This prevents DoS attacks where an attacker sends a message claiming to be gigabytes in size
const MAX_MESSAGE_SIZE: u32 = 10 * 1024 * 1024; // 10 MB

#[derive(Debug)]
pub enum NetworkError {
    /// Message exceeds maximum allowed size
    MessageTooLarge { size: u32, max: u32 },
    /// IO error
    IoError(std::io::Error),
    /// Serialization/deserialization error
    SerializationError(String),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::MessageTooLarge { size, max } => {
                write!(
                    f,
                    "Message size {} bytes exceeds maximum {} bytes",
                    size, max
                )
            }
            NetworkError::IoError(err) => write!(f, "IO error: {}", err),
            NetworkError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for NetworkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NetworkError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for NetworkError {
    fn from(err: std::io::Error) -> Self {
        NetworkError::IoError(err)
    }
}

impl From<serde_cbor::Error> for NetworkError {
    fn from(err: serde_cbor::Error) -> Self {
        NetworkError::SerializationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, NetworkError>;

/// Validates that a message length is within acceptable bounds
fn validate_message_size(length: u32) -> Result<()> {
    if length > MAX_MESSAGE_SIZE {
        log::error!(
            "Message size {} exceeds maximum {}",
            length,
            MAX_MESSAGE_SIZE
        );
        return Err(NetworkError::MessageTooLarge {
            size: length,
            max: MAX_MESSAGE_SIZE,
        });
    }
    Ok(())
}

pub async fn send_message_to_server(
    stream: &mut TlsStream<TcpStream>,
    request: ClientToServerMessage,
) -> Result<()> {
    log::debug!("Sending request: {:?}", request);
    let bytes = serde_cbor::to_vec(&request)?;

    let request_length: u32 = bytes.len() as u32;
    stream.write(&request_length.to_le_bytes()).await?;
    stream.write(&bytes).await?;

    Ok(())
}

pub async fn send_message_to_client(
    stream: &mut TlsStream<TcpStream>,
    response: ServerToClientMessage,
) -> Result<()> {
    log::debug!("Sending response: {:?}", response);

    let bytes = serde_cbor::to_vec(&response)?;

    let response_length: u32 = bytes.len() as u32;
    stream.write(&response_length.to_le_bytes()).await?;
    stream.write(&bytes).await?;

    Ok(())
}

pub async fn receive_server_message(
    stream: &mut TlsStream<TcpStream>,
) -> Result<Option<ServerToClientMessage>> {
    log::debug!("Waiting for response...");

    let mut bytes = [0u8; 4];
    stream.read(&mut bytes).await?;
    let response_length = u32::from_le_bytes(bytes);

    log::trace!("Incoming response", { length: response_length });

    if response_length > 0 {
        validate_message_size(response_length)?;

        let mut bytes = Vec::new();
        bytes.resize(response_length as usize, 0u8);
        stream.read(&mut bytes).await?;

        let response = serde_cbor::from_slice::<ServerToClientMessage>(&bytes)?;

        log::debug!("Received response", { response: response });
        Ok(Some(response))
    } else {
        log::debug!("Empty response");
        Ok(None)
    }
}

pub async fn receive_client_message(
    stream: &mut TlsStream<TcpStream>,
) -> Result<Option<ClientToServerMessage>> {
    log::debug!("Waiting for request...");

    let mut bytes = [0u8; 4];
    stream.read(&mut bytes).await?;
    let request_length = u32::from_le_bytes(bytes);

    log::trace!("Incoming request", { length: request_length });

    if request_length > 0 {
        validate_message_size(request_length)?;

        let mut bytes = Vec::new();
        bytes.resize(request_length as usize, 0u8);
        stream.read(&mut bytes).await?;

        let request = serde_cbor::from_slice::<ClientToServerMessage>(&bytes)?;
        log::debug!("Received request: {:?}", request);
        Ok(Some(request))
    } else {
        log::debug!("Received empty request");
        Ok(None)
    }
}
