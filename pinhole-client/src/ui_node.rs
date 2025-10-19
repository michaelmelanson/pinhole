use iced::{
    widget::{Button, Checkbox, Column, Container, Row, Space, Text, TextInput},
    Alignment, Length,
};

use crate::{stylesheet::Styleable, stylesheet::Stylesheet, PinholeMessage};
use pinhole_protocol::{
    node::{ButtonProps, CheckboxProps, ContainerProps, InputProps, Node, TextProps},
    storage::{StateMap, StateValue},
    stylesheet::Direction,
};

pub struct UiContainerProps {
    pub direction: Direction,
    pub children: Vec<UiNode>,
    pub classes: Vec<String>,
}

pub enum UiNode {
    Empty,
    Container(UiContainerProps),
    Text(TextProps),
    Button(ButtonProps),
    Checkbox(CheckboxProps),
    Input(InputProps),
}

impl From<Node> for UiNode {
    fn from(node: Node) -> Self {
        match node {
            Node::Empty => Self::Empty,
            Node::Container(ContainerProps {
                direction,
                children,
                classes,
            }) => {
                let mut nodes = Vec::new();
                for node in children {
                    nodes.push(UiNode::from(node));
                }
                Self::Container(UiContainerProps {
                    direction,
                    children: nodes,
                    classes,
                })
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
    ) -> iced::Element<'static, PinholeMessage> {
        match self {
            UiNode::Empty => Space::new(Length::Fill, Length::Fill).into(),
            UiNode::Text(TextProps { text, classes }) => Text::new(text.clone())
                .apply_stylesheet(stylesheet, classes)
                .into(),
            UiNode::Button(ButtonProps {
                label,
                on_click,
                classes,
            }) => Button::new(Text::new(label.clone()))
                .on_press(PinholeMessage::PerformAction(on_click.clone()))
                .apply_stylesheet(stylesheet, classes)
                .into(),

            UiNode::Checkbox(CheckboxProps {
                id,
                label,
                checked,
                on_change,
                classes,
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
                    .apply_stylesheet(stylesheet, classes)
                    .into()
            }

            UiNode::Container(UiContainerProps {
                direction,
                children,
                classes,
            }) => {
                let mut elements = Vec::new();

                for element in children.iter() {
                    elements.push(element.view(stylesheet, state_map));
                }

                let content: iced::Element<PinholeMessage> = match direction {
                    Direction::Horizontal => Row::with_children(elements)
                        .apply_stylesheet(stylesheet, classes)
                        .into(),
                    Direction::Vertical => Column::with_children(elements)
                        .apply_stylesheet(stylesheet, classes)
                        .into(),
                };

                Container::new(content)
                    .apply_stylesheet(stylesheet, classes)
                    .into()
            }

            UiNode::Input(InputProps {
                id,
                label,
                password,
                placeholder,
                label_classes,
                input_classes,
            }) => {
                let value = match state_map.get(id) {
                    Some(value) => value.clone(),
                    None => StateValue::String("".to_string()),
                };

                let id = id.clone();
                let placeholder = &placeholder.clone().unwrap_or("".to_string());
                let mut input_child = TextInput::new(placeholder, &value.string())
                    .on_input(move |new_value| PinholeMessage::FormValueChanged {
                        id: id.clone(),
                        value: StateValue::String(new_value),
                        action: None,
                    })
                    .padding(5);

                if *password {
                    input_child = input_child.secure(true);
                }

                input_child = input_child.apply_stylesheet(stylesheet, input_classes);

                let label_child = Text::new(label.clone())
                    .apply_stylesheet(stylesheet, label_classes)
                    .into();

                Column::with_children(vec![label_child, input_child.into()])
                    .align_x(Alignment::Start)
                    .into()
            }
        }
    }
}
