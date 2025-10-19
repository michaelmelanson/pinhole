use pinhole::{
    Alignment, Colour, FontWeight, Length, Size, StyleRule, Stylesheet, StylesheetClass,
};

pub fn stylesheet() -> Stylesheet {
    Stylesheet::new(vec![
        StylesheetClass::new(
            "login-container",
            vec![
                StyleRule::Gap(Length::Pixels(10.)),
                StyleRule::Width(Size::Fixed(300)),
                StyleRule::AlignChildrenY(Alignment::Centre),
            ],
        ),
        StylesheetClass::new("container", vec![StyleRule::Gap(Length::Pixels(10.))]),
        StylesheetClass::new("header-container", vec![StyleRule::Height(Size::Fixed(70))]),
        StylesheetClass::new(
            "title",
            vec![
                StyleRule::FontSize(Length::Pixels(22.)),
                StyleRule::FontWeight(FontWeight::ExtraBold),
            ],
        ),
        StylesheetClass::new(
            "account-info",
            vec![StyleRule::AlignChildrenX(Alignment::End)],
        ),
        StylesheetClass::new(
            "todo-list",
            vec![StyleRule::AlignChildrenY(Alignment::Start)],
        ),
        StylesheetClass::new(
            "primary-action",
            vec![
                StyleRule::BorderRadius(Length::Pixels(5.0)),
                StyleRule::TextColour(Colour::RGBA(1.0, 1.0, 1.0, 1.0)),
                StyleRule::BackgroundColour(Colour::RGBA(0.0, 0.3, 0.7, 1.0)),
            ],
        ),
        StylesheetClass::new(
            "destructive-action",
            vec![
                StyleRule::BorderRadius(Length::Pixels(5.0)),
                StyleRule::TextColour(Colour::RGBA(1.0, 1.0, 1.0, 1.0)),
                StyleRule::BackgroundColour(Colour::RGBA(0.7, 0.0, 0.0, 1.0)),
            ],
        ),
        StylesheetClass::new(
            "input",
            vec![
                StyleRule::BorderRadius(Length::Pixels(10.0)),
                StyleRule::TextColour(Colour::RGBA(1.0, 1.0, 1.0, 1.0)),
                StyleRule::BackgroundColour(Colour::RGBA(0.0, 0.3, 0.7, 1.0)),
            ],
        ),
    ])
}
