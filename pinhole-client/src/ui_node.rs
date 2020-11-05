use iced::{
    button::State as ButtonState, text_input::State as TextInputState, Button, Checkbox, Column,
    Length, Row, Space, Text, TextInput,
};

use crate::{LocalFormState, LocalFormValue, PinholeMessage};
use pinhole_protocol::node::{ButtonProps, CheckboxProps, InputProps, Node, TextProps};

pub enum UiNode {
    Empty,
    Container(Vec<Box<UiNode>>),
    Text(TextProps),
    Button(ButtonProps, ButtonState),
    Checkbox(CheckboxProps),
    Input(InputProps, TextInputState),
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
            }
            Node::Text(props) => UiNode::Text(props),
            Node::Button(props) => UiNode::Button(props, ButtonState::new()),
            Node::Checkbox(props) => UiNode::Checkbox(props),
            Node::Input(props) => UiNode::Input(props, TextInputState::new()),
        }
    }
}

impl UiNode {
    pub fn view(&mut self, form_state: &LocalFormState) -> iced::Element<PinholeMessage> {
        match self {
            UiNode::Empty => Space::new(Length::Fill, Length::Fill).into(),
            UiNode::Text(TextProps { text }) => Text::new(text.clone()).into(),
            UiNode::Button(ButtonProps { label, on_click }, state) => {
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
                let value = form_state.get(&id).unwrap_or(&default_value);

                Checkbox::new(value.boolean(), label.clone(), move |value| {
                    PinholeMessage::FormValueChanged {
                        id: id.clone(),
                        value: LocalFormValue::Boolean(value),
                        action: Some(on_change.clone()),
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

            UiNode::Input(
                InputProps {
                    id,
                    label,
                    password,
                },
                state,
            ) => {
                let value = match form_state.get(id) {
                    Some(value) => value.clone(),
                    None => LocalFormValue::String("".to_string()),
                };

                let id = id.clone();
                let mut input = TextInput::new(state, "", &value.string(), move |new_value| {
                    PinholeMessage::FormValueChanged {
                        id: id.clone(),
                        value: LocalFormValue::String(new_value),
                        action: None,
                    }
                });

                if *password {
                    input = input.password();
                }

                Row::with_children(vec![Text::new(label.clone()).into(), input.into()]).into()
            }
        }
    }
}
