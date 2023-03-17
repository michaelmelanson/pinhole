use iced::{Background, Color, widget::checkbox};

pub struct CheckboxStylesheet;
impl checkbox::StyleSheet for CheckboxStylesheet {
    type Style = ();

    fn active(&self, _style: &Self::Style, _is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: Background::Color(Color::from_rgb(0.95, 0.95, 0.95)),
            icon_color: Color::from_rgb(0.3, 0.3, 0.3),
            text_color: Some(Color::from_rgb(0.3, 0.3, 0.3)),
            border_radius: 5.,
            border_width: 1.,
            border_color: Color::from_rgb(0.6, 0.6, 0.6),
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: Background::Color(Color::from_rgb(0.90, 0.90, 0.90)),
            ..self.active(style, is_checked)
        }
    }
}
