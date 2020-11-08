use iced::{Background, Color, Vector};

pub struct ButtonStylesheet;
impl iced_style::button::StyleSheet for ButtonStylesheet {
    fn active(&self) -> iced::button::Style {
        iced_style::button::Style {
            background: Some(Background::Color(Color::WHITE)),
            border_color: Color::from_rgba(0., 0., 0., 0.3),
            border_radius: 3,
            border_width: 1,
            shadow_offset: Vector::new(1., 1.),
            ..Default::default()
        }
    }

    fn hovered(&self) -> iced::button::Style {
        let active = self.active();

        iced::button::Style {
            shadow_offset: active.shadow_offset + iced::Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self) -> iced::button::Style {
        let active = self.active();
        iced::button::Style {
            shadow_offset: iced::Vector::default(),
            ..active
        }
    }

    fn disabled(&self) -> iced::button::Style {
        let active = self.active();

        iced::button::Style {
            shadow_offset: iced::Vector::default(),
            background: active.background.map(|background| match background {
                iced::Background::Color(color) => iced::Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}
