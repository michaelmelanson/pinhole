use pinhole_protocol::stylesheet::{ComputedStyle, PseudoState, StyleRule};

use crate::stylesheet::{
    convert_alignment, convert_colour, convert_font_weight, convert_length, convert_radius,
    Styleable, Stylesheet, Viewport,
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
    fn apply_stylesheet(
        self,
        stylesheet: &Stylesheet,
        classes: &[String],
        viewport: Option<Viewport>,
    ) -> Self {
        // Base-state computation, for the structural properties (font,
        // alignment) that iced's TextInput doesn't expose a Status for.
        let base_computed = ComputedStyle::compute_with_context(
            &stylesheet.0,
            classes,
            viewport,
            PseudoState::default(),
        );

        let align_x = base_computed
            .extract(|r| match r {
                StyleRule::AlignChildrenX(align) => Some(convert_alignment(*align)),
                _ => None,
            })
            .unwrap_or(iced::Alignment::Start);

        let mut font = iced::Font::DEFAULT;
        if let Some(weight) = base_computed.extract(|r| match r {
            StyleRule::FontWeight(w) => Some(convert_font_weight(*w)),
            _ => None,
        }) {
            font.weight = weight;
        }

        let stylesheet = stylesheet.0.clone();
        let classes = classes.to_vec();

        self.font(font.into())
            .align_x(align_x)
            .style(move |_theme, status| {
                let pseudo = PseudoState {
                    hovered: matches!(
                        status,
                        iced::widget::text_input::Status::Hovered
                            | iced::widget::text_input::Status::Focused { .. }
                    ),
                    focused: matches!(status, iced::widget::text_input::Status::Focused { .. }),
                };
                let computed =
                    ComputedStyle::compute_with_context(&stylesheet, &classes, viewport, pseudo);

                let background = computed
                    .extract(|r| match r {
                        StyleRule::BackgroundColour(colour) => {
                            Some(iced::Background::Color(convert_colour(*colour)))
                        }
                        _ => None,
                    })
                    .unwrap_or(iced::Background::Color(iced::Color::WHITE));

                let border_width = computed
                    .extract(|r| match r {
                        StyleRule::BorderWidth(width) => Some(convert_length(*width)),
                        _ => None,
                    })
                    .unwrap_or(1.0);

                let border_colour = computed
                    .extract(|r| match r {
                        StyleRule::BorderColour(colour) => Some(convert_colour(*colour)),
                        _ => None,
                    })
                    .unwrap_or(iced::Color::from_rgba(0., 0., 0., 0.5));

                let border_radius = computed
                    .extract(|r| match r {
                        StyleRule::BorderRadius(radius) => Some(convert_radius(*radius)),
                        _ => None,
                    })
                    .unwrap_or_default();

                let border = iced::Border {
                    width: border_width,
                    color: border_colour,
                    radius: border_radius,
                };

                iced::widget::text_input::Style {
                    background,
                    border,
                    icon: iced::Color::TRANSPARENT,
                    placeholder: iced::Color::from_rgba(0., 0., 0., 0.5),
                    value: iced::Color::BLACK,
                    selection: iced::Color::from_rgba(0., 0., 0.3, 0.5),
                }
            })
    }
}
