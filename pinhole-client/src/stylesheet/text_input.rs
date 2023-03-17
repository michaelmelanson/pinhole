use iced::{Color, widget::text_input, Background};

pub struct TextInputStylesheet;
impl text_input::StyleSheet for TextInputStylesheet {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            border_color: Color::from_rgba(0., 0., 0., 0.3),
            border_width: 1.,
            border_radius: 3.,
            background: Background::Color(Color::WHITE)
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            border_color: Color::from_rgba(0., 0., 0., 0.6),
            border_width: 1.,
            border_radius: 3.,
            background: Background::Color(Color::WHITE)
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.0, 0.0, 0.0, 0.3)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.0, 0.0, 0.0, 1.0)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.0, 0.0, 1.0, 0.5)
    }
}
