use pinhole::{
    Alignment, Colour, FontWeight, Length, MediaRule, Size, StyleRule, Stylesheet, StylesheetClass,
};

pub fn stylesheet() -> Stylesheet {
    Stylesheet::new(vec![
        StylesheetClass::new(
            "page",
            vec![
                StyleRule::Width(Size::Fill),
                StyleRule::Height(Size::Fill),
                StyleRule::Gap(Length::Pixels(24.0)),
            ],
        ),
        StylesheetClass::new(
            "title",
            vec![
                StyleRule::FontSize(Length::Pixels(28.0)),
                StyleRule::FontWeight(FontWeight::ExtraBold),
            ],
        ),
        StylesheetClass::new(
            "subtitle",
            vec![
                StyleRule::FontSize(Length::Pixels(14.0)),
                StyleRule::TextColour(Colour::RGBA(0.4, 0.4, 0.4, 1.0)),
            ],
        ),
        StylesheetClass::new(
            "section-title",
            vec![
                StyleRule::FontSize(Length::Pixels(18.0)),
                StyleRule::FontWeight(FontWeight::Bold),
            ],
        ),
        // --- Flexbox: AlignItems + FlexGrow ---
        StylesheetClass::new(
            "flex-row",
            vec![
                StyleRule::Gap(Length::Pixels(12.0)),
                StyleRule::Width(Size::Fill),
                StyleRule::Height(Size::Fixed(120)),
                StyleRule::AlignItems(Alignment::Centre),
                StyleRule::BackgroundColour(Colour::RGBA(0.92, 0.92, 0.92, 1.0)),
                StyleRule::BorderRadius(Length::Pixels(8.0)),
            ],
        ),
        StylesheetClass::new(
            "flex-item-a",
            vec![
                StyleRule::FlexGrow(1.0),
                StyleRule::Height(Size::Fixed(60)),
                StyleRule::AlignChildrenX(Alignment::Centre),
                StyleRule::AlignChildrenY(Alignment::Centre),
                StyleRule::BackgroundColour(Colour::RGBA(0.2, 0.4, 0.9, 1.0)),
                StyleRule::BorderRadius(Length::Pixels(6.0)),
            ],
        ),
        StylesheetClass::new(
            "flex-item-b",
            vec![
                StyleRule::FlexGrow(2.0),
                StyleRule::Height(Size::Fixed(90)),
                StyleRule::AlignChildrenX(Alignment::Centre),
                StyleRule::AlignChildrenY(Alignment::Centre),
                StyleRule::BackgroundColour(Colour::RGBA(0.2, 0.7, 0.4, 1.0)),
                StyleRule::BorderRadius(Length::Pixels(6.0)),
            ],
        ),
        StylesheetClass::new(
            "flex-item-c",
            vec![
                StyleRule::FlexGrow(1.0),
                StyleRule::Height(Size::Fixed(45)),
                StyleRule::AlignChildrenX(Alignment::Centre),
                StyleRule::AlignChildrenY(Alignment::Centre),
                StyleRule::BackgroundColour(Colour::RGBA(0.9, 0.6, 0.2, 1.0)),
                StyleRule::BorderRadius(Length::Pixels(6.0)),
            ],
        ),
        StylesheetClass::new(
            "flex-item-label",
            vec![StyleRule::TextColour(Colour::RGBA(1.0, 1.0, 1.0, 1.0))],
        ),
        StylesheetClass::new(
            "button-row",
            vec![
                StyleRule::Gap(Length::Pixels(12.0)),
                StyleRule::AlignItems(Alignment::Centre),
            ],
        ),
        // --- Pseudo-classes: :hover / :focus ---
        StylesheetClass::new(
            "hover-button",
            vec![
                StyleRule::BackgroundColour(Colour::RGBA(0.2, 0.4, 0.9, 1.0)),
                StyleRule::TextColour(Colour::RGBA(1.0, 1.0, 1.0, 1.0)),
                StyleRule::BorderRadius(Length::Pixels(6.0)),
            ],
        )
        .with_hover(vec![StyleRule::BackgroundColour(Colour::RGBA(
            0.35, 0.55, 1.0, 1.0,
        ))]),
        StylesheetClass::new(
            "focus-input",
            vec![
                StyleRule::BorderWidth(Length::Pixels(3.0)),
                StyleRule::BorderColour(Colour::RGBA(0.85, 0.1, 0.1, 1.0)),
                StyleRule::BorderRadius(Length::Pixels(4.0)),
                StyleRule::BackgroundColour(Colour::RGBA(1.0, 0.92, 0.92, 1.0)),
            ],
        )
        .with_focus(vec![
            StyleRule::BorderColour(Colour::RGBA(0.1, 0.3, 0.9, 1.0)),
            StyleRule::BackgroundColour(Colour::RGBA(0.92, 0.95, 1.0, 1.0)),
        ]),
        // --- Responsive: @media ---
        // Narrow (< 700px): red. Medium (700-1099px): amber. Wide (>= 1100px): green.
        // Both media rules use only min_width, so at wide viewports both
        // match and the later one (1100px) wins, giving three effective bands.
        // Breakpoints are deliberately desktop-scale (not the usual
        // phone-sized 500/800 CSS convention) so the effect is visible by
        // resizing an ordinary window, rather than requiring it to be
        // shrunk down to near phone width.
        StylesheetClass::new(
            "media-box",
            vec![
                StyleRule::Height(Size::Fixed(80)),
                StyleRule::Width(Size::Fill),
                StyleRule::AlignChildrenX(Alignment::Centre),
                StyleRule::AlignChildrenY(Alignment::Centre),
                StyleRule::BackgroundColour(Colour::RGBA(0.85, 0.1, 0.1, 1.0)),
                StyleRule::BorderRadius(Length::Pixels(8.0)),
            ],
        )
        .with_media(vec![
            MediaRule::new(vec![StyleRule::BackgroundColour(Colour::RGBA(
                0.95, 0.65, 0.0, 1.0,
            ))])
            .min_width(700.0),
            MediaRule::new(vec![StyleRule::BackgroundColour(Colour::RGBA(
                0.05, 0.65, 0.15, 1.0,
            ))])
            .min_width(1100.0),
        ]),
        StylesheetClass::new(
            "media-box-label",
            vec![
                StyleRule::TextColour(Colour::RGBA(1.0, 1.0, 1.0, 1.0)),
                StyleRule::FontWeight(FontWeight::Bold),
            ],
        ),
    ])
}
