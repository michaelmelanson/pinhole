mod alignment;
mod colour;
mod direction;
mod font_weight;
mod length;
mod style_rule;
mod stylesheet_class;

use serde::{Deserialize, Serialize};

pub use self::{
    alignment::Alignment, colour::Colour, direction::Direction, font_weight::FontWeight,
    length::Length, style_rule::StyleRule, stylesheet_class::StylesheetClass,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
