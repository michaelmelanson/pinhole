mod button;
mod checkbox;
mod container;
mod text_input;

use self::{
    button::ButtonStylesheet, checkbox::CheckboxStylesheet, container::ContainerStylesheet,
    text_input::TextInputStylesheet,
};

pub struct Stylesheet;

impl Stylesheet {
    pub fn button_style(&self) -> ButtonStylesheet {
        ButtonStylesheet
    }

    pub fn checkbox_style(&self) -> CheckboxStylesheet {
        CheckboxStylesheet
    }

    pub fn container_style(&self) -> ContainerStylesheet {
        ContainerStylesheet
    }

    pub fn text_input_style(&self) -> TextInputStylesheet {
        TextInputStylesheet
    }
}

impl From<&Stylesheet> for Box<dyn iced_style::button::StyleSheet> {
    fn from(stylesheet: &Stylesheet) -> Self {
        Box::new(stylesheet.button_style())
    }
}

impl From<&Stylesheet> for Box<dyn iced_style::checkbox::StyleSheet> {
    fn from(stylesheet: &Stylesheet) -> Self {
        Box::new(stylesheet.checkbox_style())
    }
}

impl From<&Stylesheet> for Box<dyn iced_style::container::StyleSheet> {
    fn from(stylesheet: &Stylesheet) -> Self {
        Box::new(stylesheet.container_style())
    }
}

impl From<&Stylesheet> for Box<dyn iced_style::text_input::StyleSheet> {
    fn from(stylesheet: &Stylesheet) -> Self {
        Box::new(stylesheet.text_input_style())
    }
}
