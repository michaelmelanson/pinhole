use serde::{Deserialize, Serialize};

use crate::{
    layout::Size,
    stylesheet::{
        alignment::Alignment, colour::Colour, direction::Direction, font_weight::FontWeight,
        length::Length,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    /// Cross-axis alignment of children within a Row/Column (CSS `align-items`).
    /// Distinct from `AlignChildrenX`/`AlignChildrenY`, which position the
    /// Row/Column block itself within its parent Container.
    AlignItems(Alignment),

    /// Proportional share of the remaining main-axis space this child should
    /// take, relative to its siblings (CSS `flex-grow`). Applies to a node as
    /// a child of a Container, not to the Container itself. Unlike CSS,
    /// there's no separate flex-basis: this maps directly to iced's
    /// `Length::FillPortion`, so a value of `0.0` collapses the child to zero
    /// size along the main axis rather than leaving it at its natural size.
    FlexGrow(f32),
}
