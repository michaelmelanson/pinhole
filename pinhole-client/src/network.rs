use async_native_tls::TlsStream;
use async_std::{
    channel::{unbounded, Receiver, Sender},
    net::TcpStream,
    task,
};
use futures::{select, FutureExt};

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
}

#[derive(Clone)]
pub struct NetworkSession {
    command_sender: Sender<NetworkSessionCommand>,
    event_receiver: Receiver<NetworkSessionEvent>,
}

impl NetworkSession {
    pub fn new(address: String) -> NetworkSession {
        let (command_sender, command_receiver) = unbounded::<NetworkSessionCommand>();
        let (event_sender, event_receiver) = unbounded::<NetworkSessionEvent>();

        let address = address.clone();
        task::spawn(session_loop(
            address.clone(),
            command_receiver,
            event_sender,
        ));

        NetworkSession {
            command_sender,
            event_receiver,
        }
    }

    pub async fn action(&self, action: &Action, storage: &StateMap) {
        let action = action.clone();
        let _ = self
            .command_sender
            .send(NetworkSessionCommand::Action {
                action,
                storage: storage.clone(),
            })
            .await;
    }

    pub fn load(&self, path: &str) {
        let path = path.to_string();

        task::block_on(async {
            let _ = self
                .command_sender
                .send(NetworkSessionCommand::Load { path })
                .await;
        });
    }

    pub fn event_receiver(&self) -> Receiver<NetworkSessionEvent> {
        self.event_receiver.clone()
    }
}

async fn session_loop(
    address: String,
    command_receiver: Receiver<NetworkSessionCommand>,
    event_sender: Sender<NetworkSessionEvent>,
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
                    task::sleep(Duration::from_millis(1000)).await;
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
                if let Ok(command) = command {
                    log::info!("Received command from app", {command: command});
                    match command {
                        NetworkSessionCommand::Action { action, storage } => {
                            let path = current_path.clone().expect("Can't fire actions without a path set");
                            send_message_to_server(&mut stream, ClientToServerMessage::Action { path, action, storage }).await?;
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
                      let _ = event_sender.send(NetworkSessionEvent::DocumentUpdated(document)).await;
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
