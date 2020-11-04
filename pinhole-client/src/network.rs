use async_std::{
    net::TcpStream,
    sync::{channel, Receiver, Sender},
    task,
};
use futures::{select, stream::BoxStream, FutureExt};

use pinhole_protocol::{
    document::{Action, Document, FormState, Request, Response, Scope},
    network::{receive_response, send_request},
};
use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub enum NetworkSessionCommand {
    Action {
        action: Action,
        form_state: FormState,
    },
    Load {
        path: String,
    },
}

#[derive(Debug, Clone)]
pub enum NetworkSessionEvent {
    DocumentUpdated(Document),
    Error(NetworkSessionError),
}

#[derive(Debug, Clone)]
pub enum NetworkSessionError {
    Terminated,
}

#[derive(Clone)]
pub struct NetworkSession {
    address: String,
    command_sender: Sender<NetworkSessionCommand>,
    event_receiver: Receiver<NetworkSessionEvent>,
}

unsafe impl Send for NetworkSession {}

impl NetworkSession {
    pub fn new(address: String) -> NetworkSession {
        let (command_sender, command_receiver) = channel::<NetworkSessionCommand>(10);
        let (event_sender, event_receiver) = channel::<NetworkSessionEvent>(10);

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

    pub async fn action(&self, action: &Action, form_state: &FormState) {
        let action = action.clone();
        self.command_sender
            .send(NetworkSessionCommand::Action {
                action,
                form_state: form_state.clone(),
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

impl<H, I> iced_native::subscription::Recipe<H, I> for NetworkSessionSubscription
where
    H: std::hash::Hasher,
{
    type Output = NetworkSessionEvent;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<'static, I>) -> BoxStream<'static, Self::Output> {
        Box::pin(self.session.event_receiver.clone())
    }
}

async fn session_loop(
    address: String,
    command_receiver: Receiver<NetworkSessionCommand>,
    event_sender: Sender<NetworkSessionEvent>,
) -> Result<()> {
    let mut current_path: Option<String> = None;
    let mut session_storage = HashMap::new();

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
            send_request(&mut stream, Request::Load { path, storage }).await?;
        }

        'connection: loop {
            select! {
              command = command_receiver.recv().fuse() => {
                if let Ok(command) = command {
                  match command {
                    NetworkSessionCommand::Action { action, form_state } => {
                      let path = current_path.clone().expect("Can't fire actions without a path set");
                      send_request(&mut stream, Request::Action { path, action, form_state }).await?;
                    },
                    NetworkSessionCommand::Load { path } => {
                      current_path = Some(path.clone());
                      let storage = session_storage.clone();
                      send_request(&mut stream, Request::Load { path, storage }).await?;
                    }
                  }
                } else {
                  break 'main;
                }
              },

              response = receive_response(&mut stream).fuse() => {
                let response = response?;
                if let Some(response) = response {
                  match response {
                    Response::Render { document } => {
                      event_sender.send(NetworkSessionEvent::DocumentUpdated(document)).await;
                    },
                    Response::RedirectTo { path } => {
                      current_path = Some(path.clone());
                      let storage = session_storage.clone();
                      send_request(&mut stream, Request::Load { path, storage }).await?;
                    }
                    Response::Store { scope, key, value } => {
                      match scope {
                        Scope::Session => { session_storage.insert(key, value); },
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
