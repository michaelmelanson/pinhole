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
        ButtonStylesheet::default()
    }

    pub fn checkbox_style(&self) -> CheckboxStylesheet {
        CheckboxStylesheet::default()
    }

    pub fn container_style(&self) -> ContainerStylesheet {
        ContainerStylesheet::default()
    }

    pub fn text_input_style(&self) -> TextInputStylesheet {
        TextInputStylesheet::default()
    }
}
