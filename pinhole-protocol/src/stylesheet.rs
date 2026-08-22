mod alignment;
mod colour;
mod computed_style;
mod direction;
mod font_weight;
mod length;
mod media_rule;
mod style_rule;
mod stylesheet_class;
mod viewport;

use serde::{Deserialize, Serialize};

pub use self::{
    alignment::Alignment,
    colour::Colour,
    computed_style::{ComputedStyle, PseudoState},
    direction::Direction,
    font_weight::FontWeight,
    length::Length,
    media_rule::MediaRule,
    style_rule::StyleRule,
    stylesheet_class::StylesheetClass,
    viewport::Viewport,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stylesheet {
    pub classes: Vec<StylesheetClass>,
}

impl Stylesheet {
    pub fn new(classes: Vec<StylesheetClass>) -> Self {
        Stylesheet { classes }
    }

    pub fn get(&self, class: &str) -> Option<&StylesheetClass> {
        self.classes.iter().find(|c| c.name == class)
    }
}
