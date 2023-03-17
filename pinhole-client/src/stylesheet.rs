mod button;
mod checkbox;
mod container;
mod text_input;

use iced::Color;

use self::{
    button::ButtonStylesheet, checkbox::{CheckboxStylesheet}, container::ContainerStylesheet,
    text_input::TextInputStylesheet,
};

#[derive(Default)]
pub struct Stylesheet;

impl iced::application::StyleSheet for Stylesheet {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> iced::application::Appearance {
        iced::application::Appearance { background_color: Color::WHITE, text_color: Color::BLACK }
    }
}

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

impl From<&Stylesheet> for Box<dyn iced::widget::button::StyleSheet<Style = ()>> {
    fn from(stylesheet: &Stylesheet) -> Self {
        Box::new(stylesheet.button_style())
    }
}

impl From<&Stylesheet> for Box<dyn iced::widget::checkbox::StyleSheet<Style=()>> {
    fn from(stylesheet: &Stylesheet) -> Self {
        Box::new(stylesheet.checkbox_style())
    }
}

impl From<&Stylesheet> for Box<dyn iced::widget::container::StyleSheet<Style = ()>> {
    fn from(stylesheet: &Stylesheet) -> Self {
        Box::new(stylesheet.container_style())
    }
}

impl From<&Stylesheet> for Box<dyn iced::widget::text_input::StyleSheet<Style = ()>> {
    fn from(stylesheet: &Stylesheet) -> Self {
        Box::new(stylesheet.text_input_style())
    }
}
