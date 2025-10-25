use pinhole_protocol::stylesheet::{ComputedStyle, StyleRule};

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
        let computed = ComputedStyle::compute(&stylesheet.0, classes);

        // Extract values with defaults
        let background_colour = computed
            .extract(|r| match r {
                StyleRule::BackgroundColour(c) => Some(convert_colour(*c)),
                _ => None,
            })
            .unwrap_or(iced::Color::TRANSPARENT);

        let text_colour = computed
            .extract(|r| match r {
                StyleRule::TextColour(c) => Some(convert_colour(*c)),
                _ => None,
            })
            .unwrap_or(iced::Color::BLACK);

        let border_colour = computed
            .extract(|r| match r {
                StyleRule::BorderColour(c) => Some(convert_colour(*c)),
                _ => None,
            })
            .unwrap_or(iced::Color::TRANSPARENT);

        let border_width = computed
            .extract(|r| match r {
                StyleRule::BorderWidth(w) => Some(convert_length(*w)),
                _ => None,
            })
            .unwrap_or(0.0);

        let border_radius = computed
            .extract(|r| match r {
                StyleRule::BorderRadius(r) => Some(convert_radius(*r)),
                _ => None,
            })
            .unwrap_or_default();

        let shadow_offset_x = computed
            .extract(|r| match r {
                StyleRule::ShadowOffsetX(x) => Some(convert_length(*x)),
                _ => None,
            })
            .unwrap_or(0.0);

        let shadow_offset_y = computed
            .extract(|r| match r {
                StyleRule::ShadowOffsetY(y) => Some(convert_length(*y)),
                _ => None,
            })
            .unwrap_or(0.0);

        let shadow_blur_radius = computed
            .extract(|r| match r {
                StyleRule::ShadowBlurRadius(r) => Some(convert_length(*r)),
                _ => None,
            })
            .unwrap_or(0.0);

        let shadow_colour = computed
            .extract(|r| match r {
                StyleRule::ShadowColour(c) => Some(convert_colour(*c)),
                _ => None,
            })
            .unwrap_or(iced::Color::TRANSPARENT);

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
