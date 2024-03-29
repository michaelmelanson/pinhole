#![recursion_limit = "1024"]
mod network;
mod stylesheet;
mod ui_node;

use async_std::task;
use kv_log_macro as log;

use iced::{
    Application, Command,
    widget::Container, Length, Settings, Subscription, Theme, alignment::{Horizontal, Vertical},
};

use network::{NetworkSession, NetworkSessionEvent, NetworkSessionSubscription};
use pinhole_protocol::{action::Action, node::TextProps, storage::StateMap, storage::StateValue};
use std::{collections::HashMap, sync::Arc};
use stylesheet::Stylesheet;
use ui_node::UiNode;

#[derive(Clone, Default)]
pub enum ButtonState {
    #[default]
    Pressed
}

#[derive(Clone, Default)]
pub enum CheckboxState {
    Checked,

    #[default]
    Unchecked
}

#[derive(Clone, Default)]
pub struct TextInputState {
//    text: String
}

fn main() -> iced::Result {
    femme::with_level(::log::LevelFilter::Info);

    log::info!("📌 Pinhole starting up...");

    Pinhole::run(Settings {
        window: iced::window::Settings {
            size: (600, 400),
            ..Default::default()
        },
        default_text_size: 14.,
        ..Default::default()
    })
}

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
    button_state: HashMap<String, ButtonState>,
    text_input_state: HashMap<String, TextInputState>,
    state_map: StateMap,
}

impl Application for Pinhole {
    type Executor = iced::executor::Default;
    type Message = PinholeMessage;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
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
                    button_state: HashMap::new(),
                    text_input_state: HashMap::new(),
                },
            },
            Command::perform(async { "/".to_string() }, PinholeMessage::StartNavigation),
        )
    }

    fn title(&self) -> String {
        "Pinhole".to_string()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::from_recipe(NetworkSessionSubscription::new(
            self.network_session.clone(),
        ))
        .map(PinholeMessage::NetworkSessionEvent)
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        let mut command = Command::none();
        match message {
            PinholeMessage::StartNavigation(path) => {
                self.network_session.load(&path);
                command = Command::perform(async {}, |_| PinholeMessage::LoadStarted)
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

    fn view(&self) -> iced::Element<Self::Message> {
        let stylesheet = Stylesheet;
        Container::new(self.document.view(&stylesheet, &self.context.state_map))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}
