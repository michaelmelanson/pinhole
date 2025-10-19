use pinhole_protocol::stylesheet::StyleRule;

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
        let mut text_colour = iced::Color::BLACK;
        let mut font_size = 14.;
        let mut font = iced::Font::DEFAULT;

        for class in classes {
            if let Some(class_def) = stylesheet.0.get(class) {
                for rule in &class_def.rules {
                    match rule {
                        StyleRule::TextColour(colour) => text_colour = convert_colour(*colour),
                        StyleRule::FontSize(size) => font_size = convert_length(*size),
                        StyleRule::FontWeight(weight) => font.weight = convert_font_weight(*weight),
                        _ => {}
                    }
                }
            }
        }

        self.color(text_colour).size(font_size).font(font)
    }
}
