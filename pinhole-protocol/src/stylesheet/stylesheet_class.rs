use serde::{Deserialize, Serialize};

use crate::stylesheet::style_rule::StyleRule;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StylesheetClass {
    pub name: String,
    pub rules: Vec<StyleRule>,
}

impl StylesheetClass {
    pub fn new(name: impl ToString, rules: Vec<StyleRule>) -> Self {
        StylesheetClass {
            name: name.to_string(),
            rules,
        }
    }
}
