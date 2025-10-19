use pinhole_protocol::stylesheet::StyleRule;

use crate::stylesheet::{
    convert_alignment, convert_colour, convert_font_weight, convert_length, convert_radius,
    Styleable, Stylesheet,
};

impl<M, T, R> Styleable for iced::widget::TextInput<'static, M, T, R>
where
    T: iced::widget::text_input::Catalog,
    <T as iced::widget::text_input::Catalog>::Class<'static>: From<
        Box<
            dyn for<'a> std::ops::Fn(
                &'a T,
                iced::widget::text_input::Status,
            ) -> iced::widget::text_input::Style,
        >,
    >,
    R: iced::advanced::text::Renderer,
    <R as iced::advanced::text::Renderer>::Font: From<iced::Font>,
    M: Clone,
{
    fn apply_stylesheet(self, stylesheet: &Stylesheet, classes: &[String]) -> Self {
        let mut align_x = iced::Alignment::Start;
        let mut font = iced::Font::DEFAULT;
        let mut background = iced::Background::Color(iced::Color::WHITE);
        let mut border = iced::Border::default()
            .width(1)
            .color(iced::Color::from_rgba(0., 0., 0., 0.5));

        for class in classes {
            if let Some(class_def) = stylesheet.0.get(class) {
                for rule in &class_def.rules {
                    match rule {
                        StyleRule::BackgroundColour(colour) => {
                            background = iced::Background::Color(convert_colour(*colour));
                        }
                        StyleRule::BorderRadius(radius) => border.radius = convert_radius(*radius),
                        StyleRule::BorderColour(colour) => border.color = convert_colour(*colour),
                        StyleRule::BorderWidth(width) => border.width = convert_length(*width),
                        StyleRule::AlignChildrenX(align) => align_x = convert_alignment(*align),
                        StyleRule::FontWeight(weight) => font.weight = convert_font_weight(*weight),
                        _ => {}
                    }
                }
            }
        }

        self.font(font.into())
            .align_x(align_x)
            .style(move |_theme, _status| iced::widget::text_input::Style {
                background: background,
                border: border,
                icon: iced::Color::TRANSPARENT,
                placeholder: iced::Color::from_rgba(0., 0., 0., 0.5),
                value: iced::Color::BLACK,
                selection: iced::Color::from_rgba(0., 0., 0.3, 0.5),
            })
    }
}
