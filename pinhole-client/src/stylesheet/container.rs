use pinhole_protocol::stylesheet::{ComputedStyle, StyleRule};

use crate::stylesheet::{convert_alignment, convert_colour};

use super::{convert_length, convert_size, Styleable, Stylesheet};

impl<'a, M, T, R> Styleable for iced::widget::Row<'a, M, T, R>
where
    T: Clone,
    R: iced::advanced::Renderer,
{
    fn apply_stylesheet(self, stylesheet: &Stylesheet, classes: &'_ [String]) -> Self {
        let computed = ComputedStyle::compute(&stylesheet.0, classes);

        let spacing = computed
            .extract(|r| match r {
                StyleRule::Gap(length) => Some(convert_length(*length)),
                _ => None,
            })
            .unwrap_or(0.0);

        self.spacing(spacing)
    }
}

impl<'a, M, T, R> Styleable for iced::widget::Column<'a, M, T, R>
where
    T: Clone,
    R: iced::advanced::Renderer,
{
    fn apply_stylesheet(self, stylesheet: &Stylesheet, classes: &'_ [String]) -> Self {
        let computed = ComputedStyle::compute(&stylesheet.0, classes);

        let spacing = computed
            .extract(|r| match r {
                StyleRule::Gap(length) => Some(convert_length(*length)),
                _ => None,
            })
            .unwrap_or(0.0);

        self.spacing(spacing)
    }
}

impl<M, T, R> Styleable for iced::widget::Container<'static, M, T, R>
where
    T: iced::widget::container::Catalog,
    R: iced::advanced::Renderer,
    <T as iced::widget::container::Catalog>::Class<'static>:
        From<Box<dyn for<'b> std::ops::Fn(&'b T) -> iced::widget::container::Style>>,
{
    fn apply_stylesheet(self, stylesheet: &Stylesheet, classes: &'_ [String]) -> Self {
        let computed = ComputedStyle::compute(&stylesheet.0, classes);

        let align_x = computed
            .extract(|r| match r {
                StyleRule::AlignChildrenX(align) => Some(convert_alignment(*align)),
                _ => None,
            })
            .unwrap_or(iced::Alignment::Start);

        let align_y = computed
            .extract(|r| match r {
                StyleRule::AlignChildrenY(align) => Some(convert_alignment(*align)),
                _ => None,
            })
            .unwrap_or(iced::Alignment::Start);

        let width = computed
            .extract(|r| match r {
                StyleRule::Width(size) => Some(convert_size(*size)),
                _ => None,
            })
            .unwrap_or(iced::Length::Fill);

        let height = computed
            .extract(|r| match r {
                StyleRule::Height(size) => Some(convert_size(*size)),
                _ => None,
            })
            .unwrap_or(iced::Length::Fill);

        let background = computed.extract(|r| match r {
            StyleRule::BackgroundColour(colour) => {
                Some(iced::Background::Color(convert_colour(*colour)))
            }
            _ => None,
        });

        let text_colour = computed.extract(|r| match r {
            StyleRule::TextColour(colour) => Some(convert_colour(*colour)),
            _ => None,
        });

        let border_width = computed
            .extract(|r| match r {
                StyleRule::BorderWidth(width) => Some(convert_length(*width)),
                _ => None,
            })
            .unwrap_or(0.0);

        let border_colour = computed
            .extract(|r| match r {
                StyleRule::BorderColour(colour) => Some(convert_colour(*colour)),
                _ => None,
            })
            .unwrap_or(iced::Color::BLACK);

        let border = iced::Border::default()
            .width(border_width)
            .color(border_colour);

        let shadow = iced::Shadow::default();

        self.align_x(align_x)
            .align_y(align_y)
            .width(width)
            .height(height)
            .style(move |_theme| iced::widget::container::Style {
                text_color: text_colour,
                background,
                border,
                shadow,
            })
    }
}
