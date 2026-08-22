use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::messages::{ClientToServerMessage, ServerToClientMessage};

/// Trait alias for readable streams (supports trait objects via ?Sized impl)
pub trait ReadStream: AsyncRead + Unpin {}
impl<T: AsyncRead + Unpin + ?Sized> ReadStream for T {}

/// Trait alias for writable streams (supports trait objects via ?Sized impl)
pub trait WriteStream: AsyncWrite + Unpin {}
impl<T: AsyncWrite + Unpin + ?Sized> WriteStream for T {}

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
        return Err(NetworkError::MessageTooLarge {
            size: length,
            max: MAX_MESSAGE_SIZE,
        });
    }
    Ok(())
}

pub async fn send_message_to_server<S: WriteStream + ?Sized>(
    stream: &mut S,
    request: ClientToServerMessage,
) -> Result<()> {
    let bytes = serde_cbor::to_vec(&request)?;

    let request_length: u32 = bytes.len() as u32;
    stream.write_all(&request_length.to_le_bytes()).await?;
    stream.write_all(&bytes).await?;

    Ok(())
}

pub async fn send_message_to_client<S: WriteStream + ?Sized>(
    stream: &mut S,
    response: ServerToClientMessage,
) -> Result<()> {
    let bytes = serde_cbor::to_vec(&response)?;

    let response_length: u32 = bytes.len() as u32;
    stream.write_all(&response_length.to_le_bytes()).await?;
    stream.write_all(&bytes).await?;

    Ok(())
}

/// Reads the 4-byte length prefix, or `None` if the stream closed before any
/// bytes arrived (a graceful disconnect). A close in the middle of the prefix
/// is a genuine error, not a graceful one, so it's reported as such.
async fn read_length_prefix<S: ReadStream + ?Sized>(stream: &mut S) -> Result<Option<u32>> {
    let mut bytes = [0u8; 4];
    let mut filled = 0;

    while filled < bytes.len() {
        let read = stream.read(&mut bytes[filled..]).await?;
        if read == 0 {
            if filled == 0 {
                return Ok(None);
            }
            return Err(NetworkError::IoError(std::io::Error::from(
                std::io::ErrorKind::UnexpectedEof,
            )));
        }
        filled += read;
    }

    Ok(Some(u32::from_le_bytes(bytes)))
}

pub async fn receive_server_message<S: ReadStream + ?Sized>(
    stream: &mut S,
) -> Result<Option<ServerToClientMessage>> {
    let Some(response_length) = read_length_prefix(stream).await? else {
        return Ok(None);
    };

    if response_length > 0 {
        validate_message_size(response_length)?;

        let mut bytes = vec![0u8; response_length as usize];
        stream.read_exact(&mut bytes).await?;

        let response = serde_cbor::from_slice::<ServerToClientMessage>(&bytes)?;
        Ok(Some(response))
    } else {
        Ok(None)
    }
}

pub async fn receive_client_message<S: ReadStream + ?Sized>(
    stream: &mut S,
) -> Result<Option<ClientToServerMessage>> {
    let Some(request_length) = read_length_prefix(stream).await? else {
        return Ok(None);
    };

    if request_length > 0 {
        validate_message_size(request_length)?;

        let mut bytes = vec![0u8; request_length as usize];
        stream.read_exact(&mut bytes).await?;

        let request = serde_cbor::from_slice::<ClientToServerMessage>(&bytes)?;
        Ok(Some(request))
    } else {
        Ok(None)
    }
}
