#![recursion_limit = "1024"]
mod network;
mod stylesheet;
mod ui_node;

use async_std::task;
use kv_log_macro as log;

use iced::{widget::Container, Alignment, Length, Subscription, Task};

use network::{NetworkSession, NetworkSessionEvent};
use pinhole_protocol::{action::Action, node::TextProps, storage::StateMap, storage::StateValue};
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
    document: UiNode,
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
        let document = UiNode::Text(TextProps {
            text: "Loading...".to_string(),
        });

        (
            Pinhole {
                network_session: Arc::new(network_session),
                document,
                context: UiContext {
                    state_map: StateMap::new(),
                },
            },
            Task::perform(async { "/".to_string() }, PinholeMessage::StartNavigation),
        )
    }

    fn title(&self) -> String {
        "Pinhole".to_string()
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
                    self.document = document.0.into();
                }
            },
            PinholeMessage::PerformAction(action) => {
                task::block_on(
                    self.network_session
                        .action(&action, &self.context.state_map),
                );
            }
            PinholeMessage::FormValueChanged { id, value, action } => {
                log::info!("Form value changed", { id: id, value: value, action: action });
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

    fn view(&self) -> iced::Element<PinholeMessage> {
        let stylesheet = Stylesheet;
        Container::new(self.document.view(&stylesheet, &self.context.state_map))
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
