#![feature(associated_type_defaults)]

mod application;
mod context;
mod route;

use kv_log_macro as log;

use std::future::Future;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_native_tls::TlsStream;

use pinhole_protocol::{
    messages::{ClientToServerMessage, ErrorCode},
    network::{receive_client_message, send_message_to_client},
    supported_capabilities,
    tls_config::ServerTlsConfig,
    CapabilitySet,
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

pub async fn run(
    application: impl Application + 'static,
    address: impl ToSocketAddrs,
    tls_config: ServerTlsConfig,
) -> Result<()> {
    femme::start();

    accept_loop(application, address, tls_config).await
}

async fn accept_loop(
    application: impl Application + 'static,
    addr: impl ToSocketAddrs,
    tls_config: ServerTlsConfig,
) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let acceptor = tls_config.build_acceptor()?;

    log::info!("Server listening with TLS enabled");

    loop {
        let (tcp_stream, _addr) = listener.accept().await?;
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            let tls_stream = acceptor.accept(tcp_stream).await.map_err(|e| {
                log::error!("TLS handshake failed: {}", e);
                e
            });

            if let Ok(stream) = tls_stream {
                spawn_and_log_error(connection_loop(application, stream));
            }
        });
    }
}

/// Handle a single request and send response(s) to the stream
/// Returns Some(capabilities) if capabilities were renegotiated, None otherwise
pub async fn handle_request(
    application: impl Application,
    request: &ClientToServerMessage,
    stream: &mut impl MessageStream,
    capabilities: &CapabilitySet,
) -> Result<Option<CapabilitySet>> {
    let result = match request {
        ClientToServerMessage::ClientHello {
            capabilities: client_caps,
        } => {
            // Capability negotiation can happen at any time
            log::info!("Received ClientHello, negotiating capabilities");

            let server_capabilities = supported_capabilities();
            let negotiated_capabilities = server_capabilities.intersect(client_caps);

            log::info!("Capability negotiation successful", {
                capabilities: negotiated_capabilities.len()
            });
            send_message_to_client(
                stream,
                ServerToClientMessage::ServerHello {
                    capabilities: negotiated_capabilities.clone(),
                },
            )
            .await?;

            // Return the new capabilities to update connection state
            return Ok(Some(negotiated_capabilities));
        }

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
                    capabilities: capabilities.clone(),
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

    Ok(None)
}

/// Generic connection handler that works with any async stream (processes multiple requests)
pub async fn handle_connection(
    application: impl Application,
    stream: &mut impl MessageStream,
) -> Result<()> {
    // Start with empty capabilities - client must negotiate
    let mut capabilities = CapabilitySet::new();

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

        // Handle this request and update capabilities if renegotiated
        match handle_request(application, &request, stream, &capabilities).await? {
            Some(new_capabilities) => {
                capabilities = new_capabilities;
            }
            None => {}
        }
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

fn spawn_and_log_error<F>(fut: F) -> tokio::task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    tokio::spawn(async move {
        if let Err(e) = fut.await {
            log::error!("Connection error {}", e);
        }
    })
}
