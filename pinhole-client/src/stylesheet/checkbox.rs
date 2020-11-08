use iced::{Background, Color};

pub struct CheckboxStylesheet;
impl iced_style::checkbox::StyleSheet for CheckboxStylesheet {
    fn active(&self, _is_checked: bool) -> iced::checkbox::Style {
        iced::checkbox::Style {
            background: Background::Color(Color::from_rgb(0.95, 0.95, 0.95)),
            checkmark_color: Color::from_rgb(0.3, 0.3, 0.3),
            border_radius: 5,
            border_width: 1,
            border_color: Color::from_rgb(0.6, 0.6, 0.6),
        }
    }

    fn hovered(&self, is_checked: bool) -> iced::checkbox::Style {
        iced::checkbox::Style {
            background: Background::Color(Color::from_rgb(0.90, 0.90, 0.90)),
            ..self.active(is_checked)
        }
    }
}
