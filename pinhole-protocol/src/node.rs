use serde::{Deserialize, Serialize};

use crate::{action::Action, stylesheet::Direction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContainerProps {
    pub direction: Direction,
    pub children: Vec<Node>,
    pub classes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextProps {
    pub text: String,
    pub classes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonProps {
    pub label: String,
    pub on_click: Action,
    pub classes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckboxProps {
    pub id: String,
    pub label: String,
    pub checked: bool,
    pub on_change: Action,
    pub classes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputProps {
    pub id: String,
    pub label: String,
    pub password: bool,
    pub placeholder: Option<String>,
    pub label_classes: Vec<String>,
    pub input_classes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
    Empty,
    Container(ContainerProps),
    Text(TextProps),
    Button(ButtonProps),
    Checkbox(CheckboxProps),
    Input(InputProps),
}

impl Node {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
