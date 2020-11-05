#![feature(associated_type_defaults)]

mod application;
mod context;
mod route;

use kv_log_macro as log;

use async_std::{
    future::Future,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};

use pinhole_protocol::{
    messages::ClientToServerMessage,
    network::{receive_request, send_response},
};

pub use application::Application;
pub use context::Context;
pub use pinhole_protocol::{
    action::Action,
    document::Document,
    messages::ServerToClientMessage,
    node::{ButtonProps, CheckboxProps, InputProps, Node, TextProps},
    storage::StorageScope,
};
pub use route::{Render, Route, Storage};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn run(application: impl Application + 'static, address: impl ToSocketAddrs) -> Result<()> {
    femme::start();

    task::block_on(accept_loop(application, address))
}

async fn accept_loop(
    application: impl Application + 'static,
    addr: impl ToSocketAddrs,
) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        spawn_and_log_error(connection_loop(application, stream?));
    }

    Ok(())
}

async fn connection_loop(application: impl Application, mut stream: TcpStream) -> Result<()> {
    log::info!("New connection", {
        address: format!("{:?}", stream.peer_addr()?)
    });

    while let Some(ref request) = receive_request(&mut stream).await? {
        match request {
            ClientToServerMessage::Action {
                path,
                action,
                form_state,
            } => {
                log::info!("Received action", {path: path, action: action});
                if let Some(route) = application.route(path) {
                    let mut context = Context {
                        form_state: form_state.clone(),
                        stream: &mut stream,
                    };

                    route.action(action, &mut context).await?;
                } else {
                    log::error!("No route found", { path: path });
                }
            }

            ClientToServerMessage::Load { path, storage } => {
                if let Some(route) = application.route(path) {
                    match route.render(storage).await {
                        Render::Document(document) => {
                            send_response(&mut stream, ServerToClientMessage::Render { document })
                        }
                        Render::RedirectTo(path) => {
                            send_response(&mut stream, ServerToClientMessage::RedirectTo { path })
                        }
                    }
                    .await?
                } else {
                    log::error!("No route found", { path: path });
                }
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
            log::error!("Connection error {}", e);
        }
    })
}
