#![recursion_limit = "1024"]
mod network;
mod storage;
mod stylesheet;
mod ui_node;

use async_std::task;
use kv_log_macro as log;

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
            },
            Task::perform(async { "/".to_string() }, PinholeMessage::StartNavigation),
        )
    }

    fn subscription(&self) -> Subscription<PinholeMessage> {
        Subscription::run_with_id("network_session", self.network_session.event_receiver())
            .map(PinholeMessage::NetworkSessionEvent)
    }

    fn update(&mut self, message: PinholeMessage) -> iced::Task<PinholeMessage> {
        let mut command = Task::none();
        match message {
            PinholeMessage::StartNavigation(path) => {
                self.network_session.load(&path);
                command = Task::perform(async {}, |_| PinholeMessage::LoadStarted)
            }
            PinholeMessage::LoadStarted => {
                log::info!("Load started");
            }
            PinholeMessage::NetworkSessionEvent(event) => match event {
                NetworkSessionEvent::DocumentUpdated(document) => {
                    log::info!("Document updated", { document: format!("{:?}", document) });
                    self.document_node = document.node.into();
                    self.stylesheet = document.stylesheet.into();
                }
            },
            PinholeMessage::PerformAction(action) => {
                task::block_on(
                    self.network_session
                        .action(&action, &self.context.state_map),
                );
            }
            PinholeMessage::FormValueChanged { id, value, action } => {
                log::debug!("Form value changed", { id: id, value: value, action: action });

                // Store in local context for immediate UI updates and local storage
                self.context.state_map.insert(id, value);

                if let Some(action) = action {
                    task::block_on(
                        self.network_session
                            .action(&action, &self.context.state_map),
                    );
                }
            }
        }

        command
    }

    fn view(&self) -> iced::Element<'_, PinholeMessage> {
        Container::new(
            self.document_node
                .view(&self.stylesheet, &self.context.state_map),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
    }
}

fn main() -> iced::Result {
    env_logger::init();

    iced::application("Pinhole", Pinhole::update, Pinhole::view)
        .subscription(Pinhole::subscription)
        .run_with(Pinhole::new)
}
