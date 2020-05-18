use async_std::{
  net::TcpStream,
  task
};

use pinhole_protocol::{network::{receive_response, send_request}, document::{Request, Document, Response}};
use crossbeam::{Receiver, TryRecvError, Sender, channel};
use std::time::Duration;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub enum NetworkSessionCommand {
  Load(String)
}

#[derive(Debug)]
pub enum LoadError {
  UrlParseError(url::ParseError),
}

impl From<url::ParseError> for LoadError {
  fn from(error: url::ParseError) -> LoadError {
    LoadError::UrlParseError(error)
  }
}

pub enum NetworkSessionEvent {
  DocumentUpdated(Document)
}

pub struct NetworkSession {
  command_sender: channel::Sender<NetworkSessionCommand>,
  event_receiver: channel::Receiver<NetworkSessionEvent>,
}

impl NetworkSession {
  pub fn new(address: String) -> NetworkSession {
    let (command_sender, command_receiver) = channel::unbounded::<NetworkSessionCommand>();
    let (event_sender, event_receiver) = channel::unbounded::<NetworkSessionEvent>();

    task::spawn(session_loop(address, command_receiver, event_sender));

    NetworkSession {
      command_sender,
      event_receiver
    }
  }

  pub fn load(&mut self, path: String) {
    self.command_sender.send(NetworkSessionCommand::Load(path)).expect("Failed to send network session command");
  }

  pub fn try_recv(&mut self) -> Option<NetworkSessionEvent> {
    match self.event_receiver.try_recv() {
      Ok(event) => Some(event),
      Err(TryRecvError::Empty) => None,
      Err(TryRecvError::Disconnected) => panic!("Network session terminated prematurely")
    }
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
      send_request(&mut stream, Request::Load(current_path)).await?;
    }

    'connection: loop {
      'command: loop {
        match command_receiver.try_recv() {
          Ok(command) => match command {
            NetworkSessionCommand::Load(path) => {
              current_path = Some(path.clone());
              send_request(&mut stream, Request::Load(path)).await?;
            }
          },

          Err(TryRecvError::Empty) => break 'command,
          Err(TryRecvError::Disconnected) => break 'main
        }
      }

      let response = receive_response(&mut stream).await?;
      if let Some(response) = response {        
        match response {
          Response::UpdateDocument(document) => {
            event_sender.send(NetworkSessionEvent::DocumentUpdated(document))?;
          }
        }
      } else {
        println!("Received null response, terminating connection");
        break 'connection;
      }
    }
  }

  Ok(())
}
