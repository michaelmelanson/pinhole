use serde::{Deserialize, Serialize};

use crate::{action::Action, layout::Layout};

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
    pub placeholder: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
    Empty,
    Container {
        layout: Layout,
        children: Vec<Box<Node>>,
    },
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
