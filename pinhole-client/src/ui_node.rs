use iced::{
    widget::{Button, Checkbox, Column, Container, Row, Space, Text, TextInput},
    Alignment, Length,
};

use crate::{stylesheet::Stylesheet, PinholeMessage};
use pinhole_protocol::{
    layout::{Layout, Position, Size},
    node::{ButtonProps, CheckboxProps, InputProps, Node, TextProps},
    storage::StateMap,
    storage::StateValue,
};

pub enum UiNode {
    Empty,
    Container(Layout, Vec<Box<UiNode>>),
    Text(TextProps),
    Button(ButtonProps),
    Checkbox(CheckboxProps),
    Input(InputProps),
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
            Node::Button(props) => UiNode::Button(props),
            Node::Checkbox(props) => UiNode::Checkbox(props),
            Node::Input(props) => UiNode::Input(props),
        }
    }
}

impl UiNode {
    pub fn view(
        &self,
        stylesheet: &Stylesheet,
        state_map: &StateMap,
    ) -> iced::Element<PinholeMessage> {
        match self {
            UiNode::Empty => Space::new(Length::Fill, Length::Fill).into(),
            UiNode::Text(TextProps { text }) => Text::new(text.clone()).into(),
            UiNode::Button(ButtonProps { label, on_click }) => {
                Button::new(Text::new(label.clone()))
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
                let default_value = StateValue::Boolean(checked);
                let value = state_map.get(&id).unwrap_or(&default_value);

                Checkbox::new(label.clone(), value.boolean())
                    .on_toggle(move |value| PinholeMessage::FormValueChanged {
                        id: id.clone(),
                        value: StateValue::Boolean(value),
                        action: Some(on_change.clone()),
                    })
                    .into()
            }

            UiNode::Container(layout, children) => {
                let mut elements = Vec::new();

                for element in children.iter() {
                    elements.push(element.as_ref().view(stylesheet, state_map));
                }

                let container = Container::new(Column::with_children(elements))
                    .align_x(match layout.horizontal.position {
                        Position::Centre => Alignment::Center,
                        Position::Start => Alignment::Start,
                        Position::End => Alignment::End,
                    })
                    .align_y(match layout.vertical.position {
                        Position::Centre => Alignment::Center,
                        Position::Start => Alignment::Start,
                        Position::End => Alignment::End,
                    })
                    .width(match layout.horizontal.size {
                        Size::Auto => Length::Shrink,
                        Size::Fixed(size) => Length::Fixed(size as f32),
                        Size::Fill => Length::Fill,
                    })
                    .height(match layout.vertical.size {
                        Size::Auto => Length::Shrink,
                        Size::Fixed(size) => Length::Fixed(size as f32),
                        Size::Fill => Length::Fill,
                    });

                container.into()
            }

            UiNode::Input(InputProps {
                id,
                label,
                password,
                placeholder,
            }) => {
                let value = match state_map.get(id) {
                    Some(value) => value.clone(),
                    None => StateValue::String("".to_string()),
                };

                let id = id.clone();
                let placeholder = &placeholder.clone().unwrap_or("".to_string());
                let mut input = TextInput::new(placeholder, &value.string())
                    .on_input(move |new_value| PinholeMessage::FormValueChanged {
                        id: id.clone(),
                        value: StateValue::String(new_value),
                        action: None,
                    })
                    .padding(5);

                if *password {
                    input = input.secure(true);
                }

                Row::with_children(vec![Text::new(label.clone()).into(), input.into()])
                    .align_y(Alignment::Center)
                    .into()
            }
        }
    }
}
