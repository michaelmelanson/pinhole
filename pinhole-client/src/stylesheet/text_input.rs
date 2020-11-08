use iced::Color;

pub struct TextInputStylesheet;
impl iced_style::text_input::StyleSheet for TextInputStylesheet {
    fn active(&self) -> iced::text_input::Style {
        iced_style::text_input::Style {
            border_width: 1,
            border_color: Color::from_rgba(0., 0., 0., 0.3),
            border_radius: 3,
            ..Default::default()
        }
    }

    fn focused(&self) -> iced::text_input::Style {
        iced_style::text_input::Style {
            border_width: 1,
            border_color: Color::from_rgba(0., 0., 0., 0.6),
            border_radius: 3,
            ..Default::default()
        }
    }

    fn placeholder_color(&self) -> Color {
        Color::from_rgba(0.0, 0.0, 0.0, 0.3)
    }

    fn value_color(&self) -> Color {
        Color::from_rgba(0.0, 0.0, 0.0, 1.0)
    }

    fn selection_color(&self) -> Color {
        Color::from_rgba(0.0, 0.0, 1.0, 0.5)
    }
}
