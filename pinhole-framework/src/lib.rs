mod application;
mod context;
mod health;
mod route;
mod router;

use std::time::Duration;
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
pub use health::ServerHealth;
pub use pinhole_protocol::{
    action::Action,
    document::Document,
    layout::{Layout, Position, Size, Sizing},
    messages::ServerToClientMessage,
    node::{ButtonProps, CheckboxProps, ContainerProps, InputProps, Node, TextProps},
    storage::{StateMap, StateValue, StorageScope},
    stylesheet::{
        Alignment, Colour, Direction, FontWeight, Length, MediaRule, StyleRule, Stylesheet,
        StylesheetClass,
    },
    tls_config::ServerTlsConfig as TlsConfig,
};
pub use route::{Render, Route};
pub use router::{Params, RoutePattern};

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

/// How long to wait for in-flight connections to finish after a shutdown
/// signal before forcibly aborting them.
const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn run(
    application: impl Application + 'static,
    address: impl ToSocketAddrs,
    tls_config: ServerTlsConfig,
) -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    accept_loop(application, address, tls_config).await
}

/// Waits for a `SIGINT` (Ctrl+C) or, on Unix, `SIGTERM`.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn accept_loop(
    application: impl Application + 'static,
    addr: impl ToSocketAddrs,
    tls_config: ServerTlsConfig,
) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let acceptor = tls_config.build_acceptor()?;
    let health = ServerHealth::new();
    let mut connections = tokio::task::JoinSet::new();

    tracing::info!("Server listening with TLS enabled");

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                let (tcp_stream, peer_addr) = accept_result?;
                let acceptor = acceptor.clone();
                let health = health.clone();

                connections.spawn(async move {
                    let tls_stream = acceptor.accept(tcp_stream).await.map_err(|e| {
                        tracing::error!(error = %e, "TLS handshake failed");
                        e
                    });

                    if let Ok(stream) = tls_stream {
                        if let Err(e) = connection_loop(application, stream, peer_addr, health).await {
                            tracing::error!(error = %e, "Connection error");
                        }
                    }
                });
            }
            () = shutdown_signal() => {
                tracing::info!("Shutdown signal received, no longer accepting new connections");
                health.mark_draining();
                break;
            }
        }
    }

    let active = connections.len();
    if active > 0 {
        tracing::info!(
            active_connections = active,
            "Waiting for in-flight connections to finish"
        );
    }

    let drained = tokio::time::timeout(GRACEFUL_SHUTDOWN_TIMEOUT, async {
        while connections.join_next().await.is_some() {}
    })
    .await;

    if drained.is_err() {
        tracing::warn!(
            remaining = connections.len(),
            "Graceful shutdown timed out, aborting remaining connections"
        );
        connections.shutdown().await;
    } else {
        tracing::info!("Graceful shutdown complete");
    }

    Ok(())
}

/// Handle a single request and send response(s) to the stream
/// Returns Some(capabilities) if capabilities were renegotiated, None otherwise
pub async fn handle_request(
    application: impl Application,
    request: &ClientToServerMessage,
    stream: &mut impl MessageStream,
    capabilities: &CapabilitySet,
    health: &ServerHealth,
) -> Result<Option<CapabilitySet>> {
    let result = match request {
        ClientToServerMessage::ClientHello {
            capabilities: client_caps,
        } => {
            // Capability negotiation can happen at any time
            tracing::debug!(
                client_capabilities = client_caps.len(),
                "Received ClientHello"
            );

            let server_capabilities = supported_capabilities();
            let negotiated_capabilities = server_capabilities.intersect(client_caps);

            tracing::info!(
                capabilities = negotiated_capabilities.len(),
                "Capability negotiation successful"
            );
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

        ClientToServerMessage::HealthCheck => {
            let healthy = health.is_healthy();
            tracing::debug!(healthy, "Received health check");
            send_message_to_client(stream, ServerToClientMessage::HealthCheckResult { healthy })
                .await
                .map_err(|e| e.into())
        }

        ClientToServerMessage::Action {
            path,
            action,
            storage,
            storage_versions,
        } => {
            tracing::debug!(
                path = %path,
                action = %action.name,
                "Received action"
            );
            if let Some((route, params)) = application.route(path) {
                let mut context = Context {
                    storage: storage.clone(),
                    versions: storage_versions.clone(),
                    stream,
                    capabilities: capabilities.clone(),
                };
                route.action(action, &params, &mut context).await
            } else {
                tracing::warn!(path = %path, "Route not found");
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

        ClientToServerMessage::Load {
            path,
            storage,
            storage_versions: _,
        } => {
            tracing::debug!(path = %path, "Received load");
            if let Some((route, params)) = application.route(path) {
                match route.render(&params, storage).await {
                    Render::Document(document) => {
                        send_message_to_client(stream, ServerToClientMessage::Render { document })
                            .await
                            .map_err(|e| e.into())
                    }
                    Render::RedirectTo(redirect_path) => {
                        tracing::debug!(
                            from = %path,
                            to = %redirect_path,
                            "Redirecting"
                        );
                        send_message_to_client(
                            stream,
                            ServerToClientMessage::RedirectTo {
                                path: redirect_path,
                            },
                        )
                        .await
                        .map_err(|e| e.into())
                    }
                }
            } else {
                tracing::warn!(path = %path, "Route not found");
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
        tracing::warn!(error = %e, "Request handling error");
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
            tracing::error!(error = %send_err, "Failed to send error message");
            return Err(send_err.into());
        }
    }

    Ok(None)
}

/// Generic connection handler that works with any async stream (processes multiple requests)
#[tracing::instrument(skip_all)]
pub async fn handle_connection(
    application: impl Application,
    stream: &mut impl MessageStream,
    health: &ServerHealth,
) -> Result<()> {
    tracing::info!("Connection established");

    // Start with empty capabilities - client must negotiate
    let mut capabilities = CapabilitySet::new();
    let mut message_count = 0u64;

    loop {
        // Receive message - network errors are fatal and close connection
        let request = match receive_client_message(stream).await {
            Ok(Some(req)) => req,
            Ok(None) => {
                tracing::info!(
                    messages_processed = message_count,
                    "Client closed connection"
                );
                break;
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    messages_processed = message_count,
                    "Fatal network error"
                );
                return Err(e.into());
            }
        };

        message_count += 1;

        // Handle this request and update capabilities if renegotiated
        if let Some(new_capabilities) =
            handle_request(application, &request, stream, &capabilities, health).await?
        {
            capabilities = new_capabilities;
        }
    }

    Ok(())
}

/// TLS-specific connection handler wrapper
#[tracing::instrument(skip_all, fields(peer_addr = %peer_addr))]
async fn connection_loop(
    application: impl Application,
    mut stream: TlsStream<TcpStream>,
    peer_addr: std::net::SocketAddr,
    health: ServerHealth,
) -> Result<()> {
    handle_connection(application, &mut stream, &health).await
}
