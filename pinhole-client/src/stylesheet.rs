mod button;
mod container;
mod text;
mod text_input;

use iced::advanced::text::Renderer;
use pinhole_protocol::{layout::Size, stylesheet};

#[derive(Default, Debug)]
pub struct Stylesheet(pub stylesheet::Stylesheet);

impl From<stylesheet::Stylesheet> for Stylesheet {
    fn from(value: stylesheet::Stylesheet) -> Self {
        Self(value)
    }
}

pub trait Styleable {
    fn apply_stylesheet(self, stylesheet: &Stylesheet, classes: &[String]) -> Self;
}

impl<M, T: iced::widget::checkbox::Catalog, R: Renderer> Styleable
    for iced::widget::Checkbox<'_, M, T, R>
{
    fn apply_stylesheet(self, _stylesheet: &Stylesheet, _classes: &[String]) -> Self {
        self
    }
}

fn convert_colour(colour: stylesheet::Colour) -> iced::Color {
    match colour {
        stylesheet::Colour::RGBA(r, g, b, a) => iced::Color::from_rgba(r, g, b, a),
    }
}

fn convert_radius(radius: stylesheet::Length) -> iced::border::Radius {
    match radius {
        stylesheet::Length::Pixels(px) => iced::border::Radius::from(px),
    }
}

fn convert_length(length: stylesheet::Length) -> f32 {
    match length {
        stylesheet::Length::Pixels(px) => f32::from(px),
    }
}

fn convert_size(size: Size) -> iced::Length {
    match size {
        Size::Fixed(value) => iced::Length::Fixed(f32::from(value)),
        Size::Fill => iced::Length::Fill,
        Size::Auto => iced::Length::Shrink, // ?
    }
}

fn convert_font_weight(weight: stylesheet::FontWeight) -> iced::font::Weight {
    match weight {
        stylesheet::FontWeight::Normal => iced::font::Weight::Normal,
        stylesheet::FontWeight::ExtraLight => iced::font::Weight::ExtraLight,
        stylesheet::FontWeight::Thin => iced::font::Weight::Thin,
        stylesheet::FontWeight::Light => iced::font::Weight::Light,
        stylesheet::FontWeight::Medium => iced::font::Weight::Medium,
        stylesheet::FontWeight::Bold => iced::font::Weight::Bold,
        stylesheet::FontWeight::ExtraBold => iced::font::Weight::ExtraBold,
        stylesheet::FontWeight::Black => iced::font::Weight::Black,
    }
}

fn convert_alignment(align: stylesheet::Alignment) -> iced::Alignment {
    match align {
        stylesheet::Alignment::Start => iced::Alignment::Start,
        stylesheet::Alignment::Centre => iced::Alignment::Center,
        stylesheet::Alignment::End => iced::Alignment::End,
    }
}
