use futures::{select, FutureExt};
use tokio::{
    net::TcpStream,
    sync::broadcast::{channel as broadcast_channel, Sender as BroadcastSender},
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use tokio_native_tls::TlsStream;
use tokio_stream::wrappers::BroadcastStream;

use kv_log_macro as log;

use crate::error::NetworkError;
use crate::storage::StorageManager;
use pinhole_protocol::{
    action::Action,
    document::Document,
    messages::{ClientToServerMessage, ServerToClientMessage},
    network::{receive_server_message, send_message_to_server},
    storage::StateMap,
    tls_config::ClientTlsConfig,
};
use std::time::Duration;

type Result<T> = std::result::Result<T, NetworkError>;

#[derive(Debug)]
pub enum NetworkSessionCommand {
    Action { action: Action, storage: StateMap },
    Load { path: String },
}

impl ::log::kv::ToValue for NetworkSessionCommand {
    fn to_value(&self) -> ::log::kv::Value<'_> {
        ::log::kv::Value::from_debug(self)
    }
}

#[derive(Debug, Clone)]
pub enum NetworkSessionEvent {
    DocumentUpdated(Document),
    ServerError { code: u16, message: String },
}

#[derive(Clone)]
pub struct NetworkSession {
    command_sender: UnboundedSender<NetworkSessionCommand>,
    event_sender: BroadcastSender<NetworkSessionEvent>,
}

impl NetworkSession {
    pub fn new(address: String) -> NetworkSession {
        let (command_sender, command_receiver) = unbounded_channel::<NetworkSessionCommand>();
        let (event_sender, _event_receiver) = broadcast_channel::<NetworkSessionEvent>(100);

        let address = address.clone();
        tokio::spawn(session_loop(
            address.clone(),
            command_receiver,
            event_sender.clone(),
        ));

        NetworkSession {
            command_sender,
            event_sender,
        }
    }

    pub fn action(&self, action: &Action, storage: &StateMap) -> Result<()> {
        let action = action.clone();
        self.command_sender
            .send(NetworkSessionCommand::Action {
                action,
                storage: storage.clone(),
            })
            .map_err(|e| {
                log::error!("Network session thread is dead: {:?}", e);
                NetworkError::ProtocolError("Network session is not running".to_string())
            })
    }

    pub fn load(&self, path: &str) -> Result<()> {
        let path = path.to_string();

        self.command_sender
            .send(NetworkSessionCommand::Load { path })
            .map_err(|e| {
                log::error!("Network session thread is dead: {:?}", e);
                NetworkError::ProtocolError("Network session is not running".to_string())
            })
    }

    pub fn event_receiver(&self) -> BroadcastStream<NetworkSessionEvent> {
        BroadcastStream::new(self.event_sender.subscribe())
    }
}

async fn session_loop(
    address: String,
    mut command_receiver: UnboundedReceiver<NetworkSessionCommand>,
    event_sender: BroadcastSender<NetworkSessionEvent>,
) -> Result<()> {
    let mut current_path: Option<String> = None;
    let mut storage_manager = StorageManager::new(address.clone())
        .map_err(|e| NetworkError::StorageError(e.to_string()))?;

    async fn connect(address: &String) -> Result<TlsStream<TcpStream>> {
        // Create TLS connector that accepts invalid certificates for development
        let tls_config = ClientTlsConfig::new_danger_accept_invalid_certs();
        let connector = tls_config.build_connector()?;

        loop {
            log::debug!("Trying to connect to {}", address);
            match TcpStream::connect(&address).await {
                Ok(tcp_stream) => {
                    log::debug!("TCP connection established, starting TLS handshake");

                    // Extract hostname from address (before the colon)
                    let hostname = address
                        .split(':')
                        .next()
                        .ok_or_else(|| NetworkError::InvalidAddress(address.clone()))?;

                    // TLS handshake failures are usually configuration errors
                    // (bad certs, protocol mismatch, etc.) that won't fix themselves
                    let tls_stream =
                        connector
                            .connect(hostname, tcp_stream)
                            .await
                            .map_err(|err| {
                                log::error!("TLS handshake failed: {:?}", err);
                                NetworkError::TlsHandshakeFailed(err.to_string())
                            })?;

                    log::info!("TLS connection established");
                    return Ok(tls_stream);
                }
                Err(err) => {
                    log::warn!("Error trying to connect (will retry in 1s): {:?}", err);
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }
        }
    }

    'main: loop {
        let mut stream: TlsStream<TcpStream> = connect(&address).await?;

        log::info!("Connected to server");

        if let Some(path) = current_path.clone() {
            storage_manager.navigate_to(path.clone());
            storage_manager.clear_local_storage();
            let storage = storage_manager.get_all_storage();
            send_message_to_server(&mut stream, ClientToServerMessage::Load { path, storage })
                .await?;
        }

        'connection: loop {
            select! {
              command = command_receiver.recv().fuse() => {
                if let Some(command) = command {
                    log::info!("Received command from app", {command: command});
                    match command {
                        NetworkSessionCommand::Action { action, storage } => {
                            if let Some(path) = current_path.clone() {
                                send_message_to_server(&mut stream, ClientToServerMessage::Action { path, action, storage }).await?;
                            } else {
                                log::error!("Attempted to fire action without a path set, ignoring");
                            }
                        },
                        NetworkSessionCommand::Load { path } => {
                            current_path = Some(path.clone());
                            storage_manager.navigate_to(path.clone());
                            storage_manager.clear_local_storage();
                            let storage = storage_manager.get_all_storage();
                            send_message_to_server(&mut stream, ClientToServerMessage::Load { path, storage }).await?;
                        }
                    }
                } else {
                  break 'main;
                }
              },

              message = receive_server_message(&mut stream).fuse() => {
                if let Some(message) = message? {
                log::info!("Received message from server", {message: message});
                  match message {
                    ServerToClientMessage::Render { document } => {
                      if let Err(e) = event_sender.send(NetworkSessionEvent::DocumentUpdated(document)) {
                        log::error!("UI thread closed, shutting down network session: {:?}", e);
                        break 'main;
                      }
                    },
                    ServerToClientMessage::RedirectTo { path } => {
                      current_path = Some(path.clone());
                      storage_manager.navigate_to(path.clone());
                      storage_manager.clear_local_storage();
                      let storage = storage_manager.get_all_storage();
                      send_message_to_server(&mut stream, ClientToServerMessage::Load { path, storage }).await?;
                    }
                    ServerToClientMessage::Store { scope, key, value } => {
                      if let Err(e) = storage_manager.store(scope, key, value) {
                        log::warn!("Failed to store value: {:?}", e);
                      }
                    }
                    ServerToClientMessage::Error { code, message } => {
                      log::error!("Server error {}: {}", code.as_u16(), message);
                      if let Err(e) = event_sender.send(NetworkSessionEvent::ServerError {
                        code: code.as_u16(),
                        message,
                      }) {
                        log::error!("UI thread closed, shutting down network session: {:?}", e);
                        break 'main;
                      }
                    }
                  }
                } else {
                  log::info!("Received null response, terminating connection");
                  break 'connection;
                }
              }
            }
        }

        // Connection lost, clear session storage before attempting to reconnect
        storage_manager.clear_session_storage();
    }

    Ok(())
}
