#![recursion_limit = "1024"]
mod error;
mod network;
mod storage;
mod stylesheet;
mod ui_node;

use futures::StreamExt;
use iced::{widget::Container, Alignment, Length, Subscription, Task};

use network::{NetworkSession, NetworkSessionEvent};
use pinhole_protocol::{
    action::Action,
    node::TextProps,
    storage::{StateMap, StateValue},
};
use std::sync::Arc;
use stylesheet::Stylesheet;
use ui_node::UiNode;

#[derive(Debug, Clone)]
pub enum PinholeMessage {
    StartNavigation(String),
    LoadStarted,
    NetworkSessionEvent(NetworkSessionEvent),
    PerformAction(Action),
    FormValueChanged {
        id: String,
        value: StateValue,
        action: Option<Action>,
    },
}

struct Pinhole {
    network_session: Arc<NetworkSession>,
    document_node: UiNode,
    stylesheet: Stylesheet,
    context: UiContext,
    error_message: Option<String>,
}

#[derive(Clone)]
struct UiContext {
    state_map: StateMap,
}

impl Pinhole {
    fn new() -> (Self, iced::Task<PinholeMessage>) {
        let address = "127.0.0.1:8080".to_string();
        let network_session = NetworkSession::new(address);
        let document_node = UiNode::Text(TextProps {
            text: "Loading...".to_string(),
            classes: vec![],
        });

        (
            Pinhole {
                network_session: Arc::new(network_session),
                document_node,
                stylesheet: Stylesheet::default(),
                context: UiContext {
                    state_map: StateMap::new(),
                },
                error_message: None,
            },
            Task::perform(async { "/".to_string() }, PinholeMessage::StartNavigation),
        )
    }

    fn subscription(&self) -> Subscription<PinholeMessage> {
        Subscription::run_with_id(
            "network_session",
            self.network_session
                .event_receiver()
                .filter_map(|result| async move { result.ok() }),
        )
        .map(PinholeMessage::NetworkSessionEvent)
    }

    fn update(&mut self, message: PinholeMessage) -> iced::Task<PinholeMessage> {
        let mut command = Task::none();
        match message {
            PinholeMessage::StartNavigation(path) => {
                if let Err(e) = self.network_session.load(&path) {
                    tracing::error!(error = %e, "Failed to load page");
                } else {
                    command = Task::perform(async {}, |_| PinholeMessage::LoadStarted)
                }
            }
            PinholeMessage::LoadStarted => {
                tracing::debug!("Load started");
            }
            PinholeMessage::NetworkSessionEvent(event) => match event {
                NetworkSessionEvent::DocumentUpdated(document) => {
                    tracing::debug!("Document updated");
                    self.document_node = document.node.into();
                    self.stylesheet = document.stylesheet.into();
                    self.error_message = None; // Clear any error when new document loads
                }
                NetworkSessionEvent::ServerError { code, message } => {
                    tracing::error!(code = code, message = %message, "Server error");
                    self.error_message = Some(format!("Error {}: {}", code, message));
                }
            },
            PinholeMessage::PerformAction(action) => {
                let network_session = self.network_session.clone();
                let state_map = self.context.state_map.clone();
                command = Task::perform(
                    async move {
                        if let Err(e) = network_session.action(&action, &state_map) {
                            tracing::error!(error = %e, "Failed to send action");
                        }
                    },
                    |_| PinholeMessage::LoadStarted,
                );
            }
            PinholeMessage::FormValueChanged { id, value, action } => {
                tracing::trace!(id = %id, "Form value changed");

                // Store in local context for immediate UI updates and local storage
                self.context.state_map.insert(id, value);

                if let Some(action) = action {
                    let network_session = self.network_session.clone();
                    let state_map = self.context.state_map.clone();
                    command = Task::perform(
                        async move {
                            if let Err(e) = network_session.action(&action, &state_map) {
                                tracing::error!(error = %e, "Failed to send action");
                            }
                        },
                        |_| PinholeMessage::LoadStarted,
                    );
                }
            }
        }

        command
    }

    fn view(&self) -> iced::Element<'_, PinholeMessage> {
        use iced::widget::{column, text};

        let content = if let Some(error) = &self.error_message {
            column![
                text(error).size(16).color([1.0, 0.0, 0.0]),
                self.document_node
                    .view(&self.stylesheet, &self.context.state_map),
            ]
            .spacing(10)
            .into()
        } else {
            self.document_node
                .view(&self.stylesheet, &self.context.state_map)
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}

fn main() -> iced::Result {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    iced::application("Pinhole", Pinhole::update, Pinhole::view)
        .subscription(Pinhole::subscription)
        .run_with(Pinhole::new)
}
