#![recursion_limit = "1024"]
mod form;
mod network;

use async_std::task;
use iced::{Application, Button, Checkbox, Column, Command, Length, Row, Settings, Space, Subscription, Text, TextInput, button::State as ButtonState, text_input::State as TextInputState};
use kv_log_macro as log;

use form::{LocalFormState, LocalFormValue, convert_form_state};
use network::{NetworkSession, NetworkSessionEvent, NetworkSessionSubscription};
use pinhole_protocol::document::{Action, ButtonProps, CheckboxProps, InputProps, Node, TextProps};
use std::{sync::Arc, collections::HashMap};

fn main() -> iced::Result {
    femme::with_level(::log::LevelFilter::Debug);

    log::info!("📌 Pinhole starting up...");

    Pinhole::run(Settings::default())
}

#[derive(Debug, Clone)]
enum PinholeMessage {
    StartNavigation(String),
    LoadStarted(()),
    NetworkSessionEvent(NetworkSessionEvent),
    PerformAction(Action),
    FormValueChanged { id: String, value: LocalFormValue, action: Option<Action> },
}

enum UiNode {
    Empty,
    Container(Vec<Box<UiNode>>),
    Text(TextProps),
    Button(ButtonProps, ButtonState),
    Checkbox(CheckboxProps),
    Input(InputProps, TextInputState)
}

impl From<Node> for UiNode {
    fn from(node: Node) -> Self {
        match node {
            Node::Empty => Self::Empty,
            Node::Container { children } => {
                let mut nodes = Vec::new();
                for node in children {
                    nodes.push(Box::new(UiNode::from(*node)));
                }
                Self::Container(nodes)
            },
            Node::Text(props) => UiNode::Text(props),
            Node::Button(props) => UiNode::Button(props, ButtonState::new()),
            Node::Checkbox(props) => UiNode::Checkbox(props),
            Node::Input(props) => UiNode::Input(props, TextInputState::new())
        }
    }
}

impl UiNode {
    fn view(&mut self, form_state: &LocalFormState) -> iced::Element<PinholeMessage> {
        match self {
            UiNode::Empty => Space::new(Length::Fill, Length::Fill).into(),
            UiNode::Text(TextProps { text }) => Text::new(text.clone()).into(),
            UiNode::Button(ButtonProps {
                label,
                on_click,
            }, state) => {
                Button::new::<Text>(state, Text::new(label.clone()).into())
                    .on_press(PinholeMessage::PerformAction(on_click.clone()))
                    .into()
            }

            UiNode::Checkbox(CheckboxProps {
                id,
                label,
                checked,
                on_change,
            }) => {
                let id = id.clone();
                let checked = *checked;
                let on_change = on_change.clone();
                let default_value = LocalFormValue::Boolean(checked);
                let value = form_state
                    .get(&id)
                    .unwrap_or(&default_value);

                Checkbox::new(value.boolean(), label.clone(), move |value| {
                    PinholeMessage::FormValueChanged { 
                        id: id.clone(),
                        value: LocalFormValue::Boolean(value), 
                        action: Some(on_change.clone()) 
                    }
                })
                .into()
            }

            UiNode::Container(children) => {
                let mut elements = Vec::new();

                for element in children.iter_mut() {
                    elements.push(element.as_mut().view(form_state));
                }

                Column::with_children(elements).into()
            }

            UiNode::Input(InputProps {
                id,
                label,
                password,
            }, state) => {
                let value = match form_state.get(id) {
                    Some(value) => value.clone(),
                    None => LocalFormValue::String("".to_string())
                };

                let id = id.clone();
                let mut input = TextInput::new(state, "", &value.string(), move |new_value| {
                    PinholeMessage::FormValueChanged {
                        id: id.clone(),
                        value: LocalFormValue::String(new_value),
                        action: None
                    }
                });

                if *password {
                    input = input.password();
                }

                Row::with_children(vec![
                    Text::new(label.clone()).into(),
                    input.into()
                ]).into()
            }
        }
    }
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
    form_state: LocalFormState,
}

impl Application for Pinhole {
    type Executor = iced::executor::Default;
    type Message = PinholeMessage;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
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
                    form_state: HashMap::new(),
                    button_state: HashMap::new(),
                    text_input_state: HashMap::new(),
                }
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
                command = Command::perform(async {}, PinholeMessage::LoadStarted)
            }
            PinholeMessage::LoadStarted(()) => {
                log::info!("Load started");
            }
            PinholeMessage::NetworkSessionEvent(event) => match event {
                NetworkSessionEvent::DocumentUpdated(document) => {
                    log::info!("Document updated: {:?}", document);
                    self.document = document.0.into();
                }
                NetworkSessionEvent::Error(error) => log::warn!("Network error: {:?}", error),
            },

            PinholeMessage::PerformAction(action) => {
                task::block_on(self.network_session.action(&action, &convert_form_state(&self.context.form_state)));
            },
            PinholeMessage::FormValueChanged { id, value, action } => {
                self.context.form_state.insert(id, value);

                if let Some(action) = action {
                    task::block_on(self.network_session.action(&action, &convert_form_state(&self.context.form_state)));
                }
            }
        }

        // Command::perform(self.network_session.recv(), PinholeMessage::NetworkSessionEvent)
        command
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        self.document.view(&self.context.form_state)
    }
}
