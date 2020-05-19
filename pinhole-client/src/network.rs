use async_std::{
  net::TcpStream,
  task,
  sync::{
    channel,
    Sender,
    Receiver
  }
};
use futures::{select, FutureExt};

use pinhole_protocol::{network::{receive_response, send_request}, document::{Request, Document, Response, Action, Scope, FormState}};
use std::{collections::HashMap, time::Duration};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub enum NetworkSessionCommand {
  Action { action: Action, form_state: FormState },
  Load { path: String }
}

pub enum NetworkSessionEvent {
  DocumentUpdated(Document)
}

pub struct NetworkSession {
  command_sender: Sender<NetworkSessionCommand>,
  event_receiver: Receiver<NetworkSessionEvent>,
}

impl NetworkSession {
  pub fn new(address: String) -> NetworkSession {
    let (command_sender, command_receiver) = channel::<NetworkSessionCommand>(10);
    let (event_sender, event_receiver) = channel::<NetworkSessionEvent>(10);

    task::spawn(session_loop(address, command_receiver, event_sender));

    NetworkSession {
      command_sender,
      event_receiver
    }
  }

  pub async fn action(&mut self, action: &Action, form_state: &FormState) {
    let action = action.clone();
    self.command_sender.send(NetworkSessionCommand::Action { action, form_state: form_state.clone() }).await;
  }

  pub async fn load(&mut self, path: &String) {
    let path = path.clone();
    self.command_sender.send(NetworkSessionCommand::Load { path }).await;
  }

  pub fn try_recv(&mut self) -> Option<NetworkSessionEvent> {
    if self.event_receiver.is_empty() {
      return None;
    }

    let event = task::block_on(self.event_receiver.recv());
    event
  }
}

async fn session_loop(address: String, command_receiver: Receiver<NetworkSessionCommand>, event_sender: Sender<NetworkSessionEvent>) -> Result<()> {
  let mut current_path: Option<String> = None;

  let mut session_storage = HashMap::new();


  async fn connect(address: &String) -> Result<TcpStream> {
    loop {
      log::debug!("Trying to connect to {}", address);
      match TcpStream::connect(&address).await {
        Ok(stream) => { return Ok(stream); },
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
          if let Some(command) = command {
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
