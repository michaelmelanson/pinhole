use serde::{Deserialize, Serialize};

use crate::{
    layout::Size,
    stylesheet::{
        alignment::Alignment, colour::Colour, direction::Direction, font_weight::FontWeight,
        length::Length,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StyleRule {
    // Text
    TextColour(Colour),
    FontSize(Length),
    FontWeight(FontWeight),

    // Background
    BackgroundColour(Colour),

    // Border
    BorderWidth(Length),
    BorderColour(Colour),
    BorderRadius(Length),

    // Shadow
    ShadowOffsetX(Length),
    ShadowOffsetY(Length),
    ShadowBlurRadius(Length),
    ShadowColour(Colour),

    // Container layout
    Direction(Direction),
    AlignChildrenX(Alignment),
    AlignChildrenY(Alignment),
    Width(Size),
    Height(Size),
    Gap(Length),
}
