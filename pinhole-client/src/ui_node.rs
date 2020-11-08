use iced::{
    button::State as ButtonState, text_input::State as TextInputState, Align, Button, Checkbox,
    Column, Container, Length, Row, Space, Text, TextInput,
};

use crate::{stylesheet::Stylesheet, LocalFormState, LocalFormValue, PinholeMessage};
use pinhole_protocol::{
    layout::{Layout, Position, Size},
    node::{ButtonProps, CheckboxProps, InputProps, Node, TextProps},
};

pub enum UiNode {
    Empty,
    Container(Layout, Vec<Box<UiNode>>),
    Text(TextProps),
    Button(ButtonProps, ButtonState),
    Checkbox(CheckboxProps),
    Input(InputProps, TextInputState),
}

impl From<Node> for UiNode {
    fn from(node: Node) -> Self {
        match node {
            Node::Empty => Self::Empty,
            Node::Container { layout, children } => {
                let mut nodes = Vec::new();
                for node in children {
                    nodes.push(Box::new(UiNode::from(*node)));
                }
                Self::Container(layout, nodes)
            }
            Node::Text(props) => UiNode::Text(props),
            Node::Button(props) => UiNode::Button(props, ButtonState::new()),
            Node::Checkbox(props) => UiNode::Checkbox(props),
            Node::Input(props) => UiNode::Input(props, TextInputState::new()),
        }
    }
}

impl UiNode {
    pub fn view(
        &mut self,
        stylesheet: &Stylesheet,
        form_state: &LocalFormState,
    ) -> iced::Element<PinholeMessage> {
        match self {
            UiNode::Empty => Space::new(Length::Fill, Length::Fill).into(),
            UiNode::Text(TextProps { text }) => Text::new(text.clone()).into(),
            UiNode::Button(ButtonProps { label, on_click }, state) => {
                Button::new::<Text>(state, Text::new(label.clone()).into())
                    .style(stylesheet)
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
                .style(stylesheet)
                .into()
            }

            UiNode::Container(layout, children) => {
                let mut elements = Vec::new();

                for element in children.iter_mut() {
                    elements.push(element.as_mut().view(stylesheet, form_state));
                }

                let container = Container::new(Column::with_children(elements))
                    .style(stylesheet)
                    .align_x(match layout.horizontal.position {
                        Position::Centre => Align::Center,
                        Position::Start => Align::Start,
                        Position::End => Align::End,
                    })
                    .align_y(match layout.vertical.position {
                        Position::Centre => Align::Center,
                        Position::Start => Align::Start,
                        Position::End => Align::End,
                    })
                    .width(match layout.horizontal.size {
                        Size::Auto => Length::Shrink,
                        Size::Fixed(size) => Length::Units(size),
                        Size::Fill => Length::Fill,
                    })
                    .height(match layout.vertical.size {
                        Size::Auto => Length::Shrink,
                        Size::Fixed(size) => Length::Units(size),
                        Size::Fill => Length::Fill,
                    });

                container.into()
            }

            UiNode::Input(
                InputProps {
                    id,
                    label,
                    password,
                    placeholder,
                },
                state,
            ) => {
                let value = match form_state.get(id) {
                    Some(value) => value.clone(),
                    None => LocalFormValue::String("".to_string()),
                };

                let id = id.clone();
                let placeholder = &placeholder.clone().unwrap_or("".to_string());
                let mut input =
                    TextInput::new(state, placeholder, &value.string(), move |new_value| {
                        PinholeMessage::FormValueChanged {
                            id: id.clone(),
                            value: LocalFormValue::String(new_value),
                            action: None,
                        }
                    })
                    .padding(5)
                    .style(stylesheet);

                if *password {
                    input = input.password();
                }

                Row::with_children(vec![Text::new(label.clone()).into(), input.into()])
                    .align_items(Align::Center)
                    .into()
            }
        }
    }
}
