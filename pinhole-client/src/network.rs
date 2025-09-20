use async_std::{
    channel::{unbounded, Receiver, Sender},
    net::TcpStream,
    task,
};
use futures::{select, stream::BoxStream, FutureExt};

use kv_log_macro as log;
use rustc_hash::FxHasher;

use pinhole_protocol::{
    action::Action,
    document::Document,
    messages::{ClientToServerMessage, ServerToClientMessage},
    network::{receive_server_message, send_message_to_server},
    storage::StateMap,
    storage::StorageScope,
};
use std::{collections::HashMap, sync::Arc, time::Duration};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub enum NetworkSessionCommand {
    Action { action: Action, storage: StateMap },
    Load { path: String },
}

impl ::log::kv::ToValue for NetworkSessionCommand {
    fn to_value(&self) -> ::log::kv::Value {
        ::log::kv::Value::from_debug(self)
    }
}

#[derive(Debug, Clone)]
pub enum NetworkSessionEvent {
    DocumentUpdated(Document),
}

#[derive(Clone)]
pub struct NetworkSession {
    address: String,
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
            address,
            command_sender,
            event_receiver,
        }
    }

    pub async fn action(&self, action: &Action, storage: &StateMap) {
        let action = action.clone();
        self.command_sender
            .send(NetworkSessionCommand::Action {
                action,
                storage: storage.clone(),
            })
            .await;
    }

    pub fn load(&self, path: &str) {
        let path = path.to_string();

        task::block_on(async {
            self.command_sender
                .send(NetworkSessionCommand::Load { path })
                .await;
        });
    }

    pub fn event_receiver(&self) -> Receiver<NetworkSessionEvent> {
        self.event_receiver.clone()
    }
}

#[derive(Clone)]
pub struct NetworkSessionSubscription {
    session: Arc<NetworkSession>,
}

impl NetworkSessionSubscription {
    pub fn new(session: Arc<NetworkSession>) -> NetworkSessionSubscription {
        NetworkSessionSubscription {
            session: session.clone(),
        }
    }
}

impl iced::advanced::subscription::Recipe for NetworkSessionSubscription {
    type Output = NetworkSessionEvent;

    fn hash(&self, state: &mut FxHasher) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, iced::advanced::subscription::Event>,
    ) -> BoxStream<'static, Self::Output> {
        Box::pin(self.session.event_receiver.clone())
    }
}

async fn session_loop(
    address: String,
    command_receiver: Receiver<NetworkSessionCommand>,
    event_sender: Sender<NetworkSessionEvent>,
) -> Result<()> {
    let mut current_path: Option<String> = None;
    let mut session_storage: StateMap = HashMap::new();

    async fn connect(address: &String) -> Result<TcpStream> {
        loop {
            log::debug!("Trying to connect to {}", address);
            match TcpStream::connect(&address).await {
                Ok(stream) => {
                    return Ok(stream);
                }
                Err(err) => {
                    log::warn!("Error trying to connect (will retry in 1s): {:?}", err);
                    task::sleep(Duration::from_millis(1000)).await;
                }
            }
        }
    }

    'main: loop {
        let mut stream: TcpStream = connect(&address).await?;

        log::info!("Connected to server");

        if let Some(path) = current_path.clone() {
            let storage = session_storage.clone();
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
                            let storage = session_storage.clone();
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
                      event_sender.send(NetworkSessionEvent::DocumentUpdated(document)).await;
                    },
                    ServerToClientMessage::RedirectTo { path } => {
                      current_path = Some(path.clone());
                      let storage = session_storage.clone();
                      send_message_to_server(&mut stream, ClientToServerMessage::Load { path, storage }).await?;
                    }
                    ServerToClientMessage::Store { scope, key, value } => {
                      match scope {
                        StorageScope::Session => { session_storage.insert(key, value); },
                        _ => todo!("scope {:?}", scope)
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
    }

    Ok(())
}
