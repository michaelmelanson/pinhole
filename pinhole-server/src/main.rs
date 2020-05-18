use async_std::{
    future::Future,
    prelude::*,
    task,
    net::{TcpListener, ToSocketAddrs, TcpStream}
};

use pinhole_protocol::{
    document::{Document, Node, Request, Response},
    network::{send_response, receive_request}
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> { 
    task::block_on(accept_loop("0.0.0.0:8080"))
}

async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;

        spawn_and_log_error(connection_loop(stream));
    }

    Ok(())
}

async fn connection_loop(mut stream: TcpStream) -> Result<()> {
    println!("New connection from {}", stream.peer_addr()?);

    while let Some(ref request) = receive_request(&mut stream).await? {
        match request {
            Request::Load(path) => {
                let document = match path.as_str() {
                    "/" => Document(
                        Node::Container(vec![
                            Node::Text("Hello from pinhole!".to_string()).boxed(),
                            Node::Text("This text has been updated".to_string()).boxed(),
                        ])
                    ),
                    _ => Document(Node::Text("Route not found".to_string()))
                };

                send_response(&mut stream, Response::UpdateDocument(document)).await?;
            }
        }
    }

    Ok(())
}

fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}