use serde::{Deserialize, Serialize};

use crate::document::Action;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextProps {
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonProps {
    pub label: String,
    pub on_click: Action,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckboxProps {
    pub id: String,
    pub label: String,
    pub checked: bool,
    pub on_change: Action,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputProps {
    pub id: String,
    pub label: String,
    pub password: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
    Empty,
    Container { children: Vec<Box<Node>> },
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
