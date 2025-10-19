use pinhole_protocol::stylesheet::StyleRule;

use super::{convert_colour, convert_length, convert_radius, Styleable, Stylesheet};

impl<M, T, R> Styleable for iced::widget::Button<'static, M, T, R>
where
    T: iced::widget::button::Catalog,
    <T as iced::widget::button::Catalog>::Class<'static>: From<
        Box<
            dyn for<'a> std::ops::Fn(
                &'a T,
                iced::widget::button::Status,
            ) -> iced::widget::button::Style,
        >,
    >,
    R: iced::advanced::renderer::Renderer,
{
    fn apply_stylesheet(self, stylesheet: &Stylesheet, classes: &'_ [String]) -> Self {
        let mut background_colour = iced::Color::TRANSPARENT;
        let mut text_colour = iced::Color::BLACK;
        let mut border_colour = iced::Color::TRANSPARENT;
        let mut border_width = 0.0;
        let mut border_radius = iced::border::Radius::default();
        let mut shadow_offset_x = 0.0;
        let mut shadow_offset_y = 0.0;
        let mut shadow_blur_radius = 0.0;
        let mut shadow_colour = iced::Color::TRANSPARENT;

        for class in classes {
            if let Some(class_def) = stylesheet.0.get(class) {
                for rule in &class_def.rules {
                    match rule {
                        StyleRule::BackgroundColour(colour) => {
                            background_colour = convert_colour(*colour);
                        }
                        StyleRule::TextColour(colour) => text_colour = convert_colour(*colour),
                        StyleRule::BorderRadius(radius) => border_radius = convert_radius(*radius),
                        StyleRule::BorderColour(colour) => border_colour = convert_colour(*colour),
                        StyleRule::BorderWidth(width) => border_width = convert_length(*width),
                        StyleRule::ShadowOffsetX(offset) => {
                            shadow_offset_x = convert_length(*offset)
                        }
                        StyleRule::ShadowOffsetY(offset) => {
                            shadow_offset_y = convert_length(*offset)
                        }
                        StyleRule::ShadowBlurRadius(radius) => {
                            shadow_blur_radius = convert_length(*radius)
                        }
                        StyleRule::ShadowColour(colour) => shadow_colour = convert_colour(*colour),

                        #[allow(unreachable_patterns)]
                        _ => {}
                    }
                }
            }
        }

        self.style(move |_theme, _status| iced::widget::button::Style {
            background: Some(iced::Background::Color(background_colour)),
            text_color: text_colour,
            border: iced::Border {
                color: border_colour,
                width: border_width,
                radius: border_radius,
            },
            shadow: iced::Shadow {
                color: shadow_colour,
                offset: iced::Vector::new(shadow_offset_x, shadow_offset_y),
                blur_radius: shadow_blur_radius,
            },
        })
    }
}
