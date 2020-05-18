
use async_std::{
  prelude::*,
  net::TcpStream
};

use crate::document::{Request, Response};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


pub async fn send_request(stream: &mut TcpStream, request: Request) -> Result<()> {
  println!("Sending request: {:?}", request);
  let bytes = serde_cbor::to_vec(&request)?;

  let request_length: u32 = bytes.len() as u32;
  stream.write(&request_length.to_le_bytes()).await?;
  stream.write(&bytes).await?;
  
  println!("Request sent");

  Ok(())
}

pub async fn send_response(stream: &mut TcpStream, response: Response) -> Result<()> {
  println!("Sending response: {:?}", response);

  let bytes = serde_cbor::to_vec(&response)?;

  let response_length: u32 = bytes.len() as u32;
  stream.write(&response_length.to_le_bytes()).await?;
  stream.write(&bytes).await?;
  
  println!("Response sent");

  Ok(())
}


pub async fn receive_response(stream: &mut TcpStream) -> Result<Option<Response>> {
  println!("Waiting for response...");

  let mut bytes = [0u8;4];
  stream.read(&mut bytes).await?;
  let response_length = u32::from_le_bytes(bytes);

  println!("Incoming response with length {:?}", response_length);

  if response_length > 0 {
    let mut bytes = Vec::new();
    bytes.resize(response_length as usize, 0u8);
    stream.read(&mut bytes).await?;

    let response = serde_cbor::from_slice::<Response>(&bytes)?;

    println!("Received response: {:?}", response);
    Ok(Some(response))
  } else {
    println!("Empty response");
    Ok(None)
  }
}

pub async fn receive_request(stream: &mut TcpStream) -> Result<Option<Request>> {
  println!("Waiting for request...");

  let mut bytes = [0u8;4];
  stream.read(&mut bytes).await?;
  let request_length = u32::from_le_bytes(bytes);

  println!("Incoming request with length length {:?}", request_length);

  if request_length > 0 {
    let mut bytes = Vec::new();
    bytes.resize(request_length as usize, 0u8);
    stream.read(&mut bytes).await?;

    let request = serde_cbor::from_slice::<Request>(&bytes)?;
    println!("Received request: {:?}", request);
    Ok(Some(request))
  } else {
    println!("Received empty request");
    Ok(None)
  }
}
