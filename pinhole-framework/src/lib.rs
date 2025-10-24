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
    messages::{ClientToServerMessage, ErrorCode},
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

/// Requirements for a stream that can handle Pinhole protocol messages
pub trait MessageStream:
    pinhole_protocol::network::ReadStream + pinhole_protocol::network::WriteStream + Send
{
}

/// Blanket implementation for any type that satisfies the requirements
impl<T> MessageStream for T where
    T: pinhole_protocol::network::ReadStream + pinhole_protocol::network::WriteStream + Send
{
}

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

/// Handle a single request and send response(s) to the stream
pub async fn handle_request(
    application: impl Application,
    request: &ClientToServerMessage,
    stream: &mut impl MessageStream,
) -> Result<()> {
    let result = match request {
        ClientToServerMessage::Action {
            path,
            action,
            storage,
        } => {
            log::info!("Received action", {path: path, action: action});
            if let Some(route) = application.route(path) {
                let mut context = Context {
                    storage: storage.clone(),
                    stream,
                };
                route.action(action, &mut context).await
            } else {
                log::error!("No route found", { path: path });
                send_message_to_client(
                    stream,
                    ServerToClientMessage::Error {
                        code: ErrorCode::NotFound,
                        message: format!("Route not found: {}", path),
                    },
                )
                .await
                .map_err(|e| e.into())
            }
        }

        ClientToServerMessage::Load { path, storage } => {
            if let Some(route) = application.route(path) {
                match route.render(storage).await {
                    Render::Document(document) => {
                        send_message_to_client(stream, ServerToClientMessage::Render { document })
                            .await
                            .map_err(|e| e.into())
                    }
                    Render::RedirectTo(redirect_path) => send_message_to_client(
                        stream,
                        ServerToClientMessage::RedirectTo {
                            path: redirect_path,
                        },
                    )
                    .await
                    .map_err(|e| e.into()),
                }
            } else {
                log::error!("No route found", { path: path });
                send_message_to_client(
                    stream,
                    ServerToClientMessage::Error {
                        code: ErrorCode::NotFound,
                        message: format!("Route not found: {}", path),
                    },
                )
                .await
                .map_err(|e| e.into())
            }
        }
    };

    // Send error message to client if request handling failed
    if let Err(e) = result {
        log::warn!("Request handling error: {}", e);
        let error_result = send_message_to_client(
            stream,
            ServerToClientMessage::Error {
                code: ErrorCode::InternalServerError,
                message: e.to_string(),
            },
        )
        .await;

        // If we can't send the error message, the connection is broken
        if let Err(send_err) = error_result {
            log::error!("Failed to send error message: {}", send_err);
            return Err(send_err.into());
        }
    }

    Ok(())
}

/// Generic connection handler that works with any async stream (processes multiple requests)
pub async fn handle_connection(
    application: impl Application,
    stream: &mut impl MessageStream,
) -> Result<()> {
    loop {
        // Receive message - network errors are fatal and close connection
        let request = match receive_client_message(stream).await {
            Ok(Some(req)) => req,
            Ok(None) => {
                log::info!("Client closed connection");
                break;
            }
            Err(e) => {
                log::error!("Fatal network error: {}", e);
                return Err(e.into());
            }
        };

        // Handle this request
        handle_request(application, &request, stream).await?;
    }

    Ok(())
}

/// TLS-specific connection handler wrapper
async fn connection_loop(
    application: impl Application,
    mut stream: TlsStream<TcpStream>,
) -> Result<()> {
    log::info!("New TLS connection");
    handle_connection(application, &mut stream).await
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
