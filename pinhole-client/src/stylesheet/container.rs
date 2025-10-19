use pinhole_protocol::stylesheet::StyleRule;

use crate::stylesheet::{convert_alignment, convert_colour};

use super::{convert_length, convert_size, Styleable, Stylesheet};

impl<'a, M, T, R> Styleable for iced::widget::Row<'a, M, T, R>
where
    T: Clone,
    R: iced::advanced::Renderer,
{
    fn apply_stylesheet(self, stylesheet: &Stylesheet, classes: &'_ [String]) -> Self {
        let mut spacing = 0.0;

        for class in classes {
            if let Some(class_def) = stylesheet.0.get(class) {
                for rule in &class_def.rules {
                    match rule {
                        StyleRule::Gap(length) => spacing = convert_length(*length),
                        _ => {}
                    }
                }
            }
        }

        self.spacing(spacing)
    }
}

impl<'a, M, T, R> Styleable for iced::widget::Column<'a, M, T, R>
where
    T: Clone,
    R: iced::advanced::Renderer,
{
    fn apply_stylesheet(self, stylesheet: &Stylesheet, classes: &'_ [String]) -> Self {
        let mut spacing = 0.0;

        for class in classes {
            if let Some(class_def) = stylesheet.0.get(class) {
                for rule in &class_def.rules {
                    match rule {
                        StyleRule::Gap(length) => spacing = convert_length(*length),
                        _ => {}
                    }
                }
            }
        }

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
        let mut align_x = iced::Alignment::Start;
        let mut align_y = iced::Alignment::Start;
        let mut width = iced::Length::Fill;
        let mut height = iced::Length::Fill;
        let mut border = iced::Border::default().width(0).color(iced::Color::BLACK);
        let mut background = None;
        let mut text_colour = None;
        let shadow = iced::Shadow::default();

        for class in classes {
            if let Some(class_def) = stylesheet.0.get(class) {
                for rule in &class_def.rules {
                    match rule {
                        StyleRule::AlignChildrenX(align) => align_x = convert_alignment(*align),
                        StyleRule::AlignChildrenY(align) => align_y = convert_alignment(*align),
                        StyleRule::BackgroundColour(colour) => {
                            background = Some(iced::Background::Color(convert_colour(*colour)))
                        }
                        StyleRule::Width(size) => width = convert_size(*size),
                        StyleRule::Height(size) => height = convert_size(*size),
                        StyleRule::BorderWidth(width) => border.width = convert_length(*width),
                        StyleRule::BorderColour(colour) => border.color = convert_colour(*colour),
                        StyleRule::TextColour(colour) => {
                            text_colour = Some(convert_colour(*colour))
                        }
                        _ => {}
                    }
                }
            }
        }

        self.align_x(align_x)
            .align_y(align_y)
            .width(width)
            .height(height)
            .style(move |_theme| iced::widget::container::Style {
                text_color: text_colour,
                background: background,
                border: border,
                shadow: shadow,
            })
    }
}
