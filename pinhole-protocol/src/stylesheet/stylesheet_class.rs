use serde::{Deserialize, Serialize};

use crate::stylesheet::{media_rule::MediaRule, style_rule::StyleRule};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StylesheetClass {
    pub name: String,
    pub rules: Vec<StyleRule>,

    /// Rules applied in addition to (and overriding) `rules` while the
    /// widget is hovered (CSS `:hover`). Only meaningful for widgets that
    /// have a hover state, e.g. buttons and text inputs.
    #[serde(default)]
    pub hover_rules: Vec<StyleRule>,

    /// Rules applied in addition to (and overriding) `rules`/`hover_rules`
    /// while the widget is focused (CSS `:focus`). Only meaningful for
    /// widgets that have a focus state, e.g. text inputs.
    #[serde(default)]
    pub focus_rules: Vec<StyleRule>,

    /// Rules applied in addition to (and overriding) `rules` when the
    /// client's viewport matches (CSS `@media`), in declared order.
    #[serde(default)]
    pub media_rules: Vec<MediaRule>,
}

impl StylesheetClass {
    pub fn new(name: impl ToString, rules: Vec<StyleRule>) -> Self {
        StylesheetClass {
            name: name.to_string(),
            rules,
            hover_rules: Vec::new(),
            focus_rules: Vec::new(),
            media_rules: Vec::new(),
        }
    }

    pub fn with_hover(mut self, rules: Vec<StyleRule>) -> Self {
        self.hover_rules = rules;
        self
    }

    pub fn with_focus(mut self, rules: Vec<StyleRule>) -> Self {
        self.focus_rules = rules;
        self
    }

    pub fn with_media(mut self, rules: Vec<MediaRule>) -> Self {
        self.media_rules = rules;
        self
    }
}
