#![feature(associated_type_defaults)]

mod application;
mod context;
mod route;

use kv_log_macro as log;

use async_native_tls::TlsStream;
use async_std::{
    future::Future,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};

use pinhole_protocol::{
    messages::ClientToServerMessage,
    network::{receive_client_message, send_message_to_client},
    tls_config::ServerTlsConfig,
};

pub use application::Application;
pub use context::Context;
pub use pinhole_protocol::{
    action::Action,
    document::Document,
    layout::{Layout, Position, Size, Sizing},
    messages::ServerToClientMessage,
    node::{ButtonProps, CheckboxProps, ContainerProps, InputProps, Node, TextProps},
    storage::{StateMap, StateValue, StorageScope},
    stylesheet::{
        Alignment, Colour, Direction, FontWeight, Length, StyleRule, Stylesheet, StylesheetClass,
    },
    tls_config::ServerTlsConfig as TlsConfig,
};
pub use route::{Render, Route};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn run(
    application: impl Application + 'static,
    address: impl ToSocketAddrs,
    tls_config: ServerTlsConfig,
) -> Result<()> {
    femme::start();

    task::block_on(accept_loop(application, address, tls_config))
}

async fn accept_loop(
    application: impl Application + 'static,
    addr: impl ToSocketAddrs,
    tls_config: ServerTlsConfig,
) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let acceptor = tls_config.build_acceptor()?;

    log::info!("Server listening with TLS enabled");

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let tcp_stream = stream?;
        let acceptor = acceptor.clone();

        task::spawn(async move {
            let tls_stream = acceptor.accept(tcp_stream).await.map_err(|e| {
                log::error!("TLS handshake failed: {}", e);
                e
            });

            if let Ok(stream) = tls_stream {
                spawn_and_log_error(connection_loop(application, stream));
            }
        });
    }

    Ok(())
}

async fn connection_loop(
    application: impl Application,
    mut stream: TlsStream<TcpStream>,
) -> Result<()> {
    log::info!("New TLS connection");

    while let Some(ref request) = receive_client_message(&mut stream).await? {
        match request {
            ClientToServerMessage::Action {
                path,
                action,
                storage,
            } => {
                log::info!("Received action", {path: path, action: action});
                if let Some(route) = application.route(path) {
                    let mut context = Context {
                        storage: storage.clone(),
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
                        Render::Document(document) => send_message_to_client(
                            &mut stream,
                            ServerToClientMessage::Render { document },
                        ),
                        Render::RedirectTo(path) => send_message_to_client(
                            &mut stream,
                            ServerToClientMessage::RedirectTo { path },
                        ),
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
