use pinhole_protocol::stylesheet::{ComputedStyle, StyleRule};

use crate::stylesheet::{convert_font_weight, convert_length};

use super::{convert_colour, Styleable, Stylesheet};

impl<T: iced::widget::text::Catalog, R: iced::advanced::text::Renderer> Styleable
    for iced::advanced::widget::Text<'static, T, R>
where
    <T as iced::widget::text::Catalog>::Class<'static>:
        From<Box<dyn for<'a> std::ops::Fn(&'a T) -> iced::widget::text::Style>>,
    R::Font: From<iced::Font>,
{
    fn apply_stylesheet(self, stylesheet: &Stylesheet, classes: &[String]) -> Self {
        let computed = ComputedStyle::compute(&stylesheet.0, classes);

        let text_colour = computed
            .extract(|r| match r {
                StyleRule::TextColour(c) => Some(convert_colour(*c)),
                _ => None,
            })
            .unwrap_or(iced::Color::BLACK);

        let font_size = computed
            .extract(|r| match r {
                StyleRule::FontSize(s) => Some(convert_length(*s)),
                _ => None,
            })
            .unwrap_or(14.0);

        let mut font = iced::Font::DEFAULT;
        if let Some(weight) = computed.extract(|r| match r {
            StyleRule::FontWeight(w) => Some(convert_font_weight(*w)),
            _ => None,
        }) {
            font.weight = weight;
        }

        self.color(text_colour).size(font_size).font(font)
    }
}
