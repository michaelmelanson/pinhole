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

use pinhole_protocol::{network::{receive_response, send_request}, document::{Request, Document, Response}};
use std::time::Duration;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub enum NetworkSessionCommand {
  Action(String),
  Load(String)
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

  pub async fn action(&mut self, name: &String) {
    let name = name.clone();
    self.command_sender.send(NetworkSessionCommand::Action(name)).await;
  }

  pub async fn load(&mut self, path: &String) {
    let path = path.clone();
    self.command_sender.send(NetworkSessionCommand::Load(path)).await;
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
  let mut current_path = None;

  async fn connect(address: &String) -> Result<TcpStream> {
    loop {
      println!("Trying to connect to {}", address);
      match TcpStream::connect(&address).await {
        Ok(stream) => { return Ok(stream); },
        Err(err) => {
          println!("Error trying to connect: {:?}", err);
          task::sleep(Duration::from_millis(1000)).await;
        }
      }
    }
  }

  'main: loop {
    let mut stream: TcpStream = connect(&address).await?;
    
    println!("Connected to server");
    
    if let Some(current_path) = current_path.clone() {
      send_request(&mut stream, Request::Load { path: current_path }).await?;
    }

    'connection: loop {
      select! {
        command = command_receiver.recv().fuse() => {
          if let Some(command) = command {
            match command {
              NetworkSessionCommand::Action(action) => {
                send_request(&mut stream, Request::Action { action }).await?;
              },
              NetworkSessionCommand::Load(path) => {
                current_path = Some(path.clone());
                send_request(&mut stream, Request::Load { path }).await?;
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
                send_request(&mut stream, Request::Load { path }).await?;
              }
            }
          } else {
            println!("Received null response, terminating connection");
            break 'connection;
          }
        }
      }
    }
  }

  Ok(())
}
