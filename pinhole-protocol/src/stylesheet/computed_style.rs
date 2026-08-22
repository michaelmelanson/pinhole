//! Computed style system - similar to CSSOM in browsers
//!
//! This module provides a flattened representation of styles that apply to a widget,
//! computed from a stylesheet and list of class names.

use super::{StyleRule, Stylesheet, Viewport};

/// Whether a widget is currently in an interactive pseudo-class state
/// (CSS `:hover`/`:focus`), used by `ComputedStyle::compute_with_context`
/// to decide whether a class's `hover_rules`/`focus_rules` apply.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PseudoState {
    pub hovered: bool,
    pub focused: bool,
}

/// Computed styles for a widget, with all applicable rules flattened
///
/// Later rules override earlier rules of the same type
#[derive(Debug, Clone, Default)]
pub struct ComputedStyle {
    rules: Vec<StyleRule>,
}

impl ComputedStyle {
    /// Compute styles for a widget with the given classes
    ///
    /// Rules from later classes override rules from earlier classes. This
    /// never applies `@media`/`:hover`/`:focus` rules — use
    /// `compute_with_context` for that.
    pub fn compute(stylesheet: &Stylesheet, classes: &[String]) -> Self {
        Self::compute_with_context(stylesheet, classes, None, PseudoState::default())
    }

    /// Compute styles for a widget with the given classes, additionally
    /// layering in any matching `@media` rules and, if `pseudo` indicates
    /// an active interactive state, `:hover`/`:focus` rules.
    ///
    /// Per class, layering order is: base rules, then matching
    /// `media_rules` (in declared order), then `hover_rules` if
    /// `pseudo.hovered`, then `focus_rules` if `pseudo.focused` — each
    /// layer overrides earlier rules of the same kind, same as across
    /// classes.
    pub fn compute_with_context(
        stylesheet: &Stylesheet,
        classes: &[String],
        viewport: Option<Viewport>,
        pseudo: PseudoState,
    ) -> Self {
        let mut rules = Vec::new();

        for class_name in classes {
            if let Some(class) = stylesheet.get(class_name) {
                Self::layer(&mut rules, &class.rules);

                for media in &class.media_rules {
                    if viewport.is_some_and(|v| media.matches(v)) {
                        Self::layer(&mut rules, &media.rules);
                    }
                }

                if pseudo.hovered {
                    Self::layer(&mut rules, &class.hover_rules);
                }
                if pseudo.focused {
                    Self::layer(&mut rules, &class.focus_rules);
                }
            }
        }

        ComputedStyle { rules }
    }

    /// Layer `new_rules` on top of `rules`, with each new rule overriding
    /// any existing rule of the same kind (same enum discriminant).
    fn layer(rules: &mut Vec<StyleRule>, new_rules: &[StyleRule]) {
        for rule in new_rules {
            rules.retain(|r| std::mem::discriminant(r) != std::mem::discriminant(rule));
            rules.push(rule.clone());
        }
    }

    /// Get a specific style rule by matching on the discriminant
    pub fn get<F>(&self, matcher: F) -> Option<&StyleRule>
    where
        F: Fn(&StyleRule) -> bool,
    {
        self.rules.iter().rev().find(|rule| matcher(rule))
    }

    /// Get all rules (for iteration)
    pub fn rules(&self) -> &[StyleRule] {
        &self.rules
    }

    /// Check if a specific rule type exists
    pub fn has<F>(&self, matcher: F) -> bool
    where
        F: Fn(&StyleRule) -> bool,
    {
        self.rules.iter().any(matcher)
    }

    /// Extract a value from a rule, applying a conversion function
    pub fn extract<T, F>(&self, matcher: F) -> Option<T>
    where
        F: Fn(&StyleRule) -> Option<T>,
    {
        self.rules.iter().rev().find_map(matcher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stylesheet::{Alignment, Colour, FontWeight, Length, StylesheetClass};

    #[test]
    fn test_empty_style() {
        let stylesheet = Stylesheet::new(vec![]);
        let style = ComputedStyle::compute(&stylesheet, &[]);

        assert_eq!(style.rules().len(), 0);
    }

    #[test]
    fn test_single_class() {
        let stylesheet = Stylesheet::new(vec![StylesheetClass::new(
            "heading",
            vec![
                StyleRule::FontSize(Length::Pixels(24.0)),
                StyleRule::FontWeight(FontWeight::Bold),
                StyleRule::TextColour(Colour::RGBA(0.0, 0.0, 0.0, 1.0)),
            ],
        )]);

        let style = ComputedStyle::compute(&stylesheet, &["heading".to_string()]);

        assert_eq!(style.rules().len(), 3);
        assert!(style.has(|r| matches!(r, StyleRule::FontSize(_))));
        assert!(style.has(|r| matches!(r, StyleRule::FontWeight(_))));
        assert!(style.has(|r| matches!(r, StyleRule::TextColour(_))));
    }

    #[test]
    fn test_multiple_classes_later_overrides() {
        let stylesheet = Stylesheet::new(vec![
            StylesheetClass::new(
                "base",
                vec![
                    StyleRule::FontSize(Length::Pixels(16.0)),
                    StyleRule::TextColour(Colour::RGBA(0.0, 0.0, 0.0, 1.0)),
                ],
            ),
            StylesheetClass::new("large", vec![StyleRule::FontSize(Length::Pixels(32.0))]),
        ]);

        let style = ComputedStyle::compute(&stylesheet, &["base".to_string(), "large".to_string()]);

        // Should have 2 rules: TextColour from base, FontSize from large (overriding base)
        assert_eq!(style.rules().len(), 2);

        // Font size from 'large' should override 'base'
        let font_size = style.get(|r| matches!(r, StyleRule::FontSize(_)));
        assert!(matches!(
            font_size,
            Some(StyleRule::FontSize(Length::Pixels(32.0)))
        ));

        // Text colour from 'base' should remain
        assert!(style.has(|r| matches!(r, StyleRule::TextColour(_))));
    }

    #[test]
    fn test_non_existent_class_ignored() {
        let stylesheet = Stylesheet::new(vec![StylesheetClass::new(
            "existing",
            vec![StyleRule::FontSize(Length::Pixels(20.0))],
        )]);

        let style = ComputedStyle::compute(
            &stylesheet,
            &["nonexistent".to_string(), "existing".to_string()],
        );

        assert_eq!(style.rules().len(), 1);
        assert!(style.has(|r| matches!(r, StyleRule::FontSize(_))));
    }

    #[test]
    fn test_class_order_matters() {
        let stylesheet = Stylesheet::new(vec![
            StylesheetClass::new(
                "red",
                vec![StyleRule::TextColour(Colour::RGBA(1.0, 0.0, 0.0, 1.0))],
            ),
            StylesheetClass::new(
                "blue",
                vec![StyleRule::TextColour(Colour::RGBA(0.0, 0.0, 1.0, 1.0))],
            ),
        ]);

        // red then blue - blue wins
        let style1 = ComputedStyle::compute(&stylesheet, &["red".to_string(), "blue".to_string()]);
        let color1 = style1.get(|r| matches!(r, StyleRule::TextColour(_)));
        assert!(matches!(
            color1,
            Some(StyleRule::TextColour(Colour::RGBA(0.0, 0.0, 1.0, 1.0)))
        ));

        // blue then red - red wins
        let style2 = ComputedStyle::compute(&stylesheet, &["blue".to_string(), "red".to_string()]);
        let color2 = style2.get(|r| matches!(r, StyleRule::TextColour(_)));
        assert!(matches!(
            color2,
            Some(StyleRule::TextColour(Colour::RGBA(1.0, 0.0, 0.0, 1.0)))
        ));
    }

    #[test]
    fn test_get_specific_rule() {
        let stylesheet = Stylesheet::new(vec![StylesheetClass::new(
            "styled",
            vec![
                StyleRule::FontSize(Length::Pixels(18.0)),
                StyleRule::TextColour(Colour::RGBA(1.0, 0.0, 0.0, 1.0)),
                StyleRule::FontWeight(FontWeight::Bold),
            ],
        )]);

        let style = ComputedStyle::compute(&stylesheet, &["styled".to_string()]);

        // Get specific rules
        let font_size = style.get(|r| matches!(r, StyleRule::FontSize(_)));
        assert!(matches!(
            font_size,
            Some(StyleRule::FontSize(Length::Pixels(18.0)))
        ));

        let text_color = style.get(|r| matches!(r, StyleRule::TextColour(_)));
        assert!(matches!(
            text_color,
            Some(StyleRule::TextColour(Colour::RGBA(1.0, 0.0, 0.0, 1.0)))
        ));

        // Check for non-existent rule
        let bg_color = style.get(|r| matches!(r, StyleRule::BackgroundColour(_)));
        assert!(bg_color.is_none());
    }

    #[test]
    fn test_multiple_overrides() {
        let stylesheet = Stylesheet::new(vec![
            StylesheetClass::new("small", vec![StyleRule::FontSize(Length::Pixels(12.0))]),
            StylesheetClass::new("medium", vec![StyleRule::FontSize(Length::Pixels(16.0))]),
            StylesheetClass::new("large", vec![StyleRule::FontSize(Length::Pixels(24.0))]),
        ]);

        let style = ComputedStyle::compute(
            &stylesheet,
            &[
                "small".to_string(),
                "medium".to_string(),
                "large".to_string(),
            ],
        );

        // Only one font size rule should remain (the last one)
        assert_eq!(style.rules().len(), 1);
        let font_size = style.get(|r| matches!(r, StyleRule::FontSize(_)));
        assert!(matches!(
            font_size,
            Some(StyleRule::FontSize(Length::Pixels(24.0)))
        ));
    }

    #[test]
    fn test_mixed_rule_types() {
        let stylesheet = Stylesheet::new(vec![
            StylesheetClass::new(
                "text",
                vec![
                    StyleRule::FontSize(Length::Pixels(16.0)),
                    StyleRule::TextColour(Colour::RGBA(0.0, 0.0, 0.0, 1.0)),
                ],
            ),
            StylesheetClass::new(
                "background",
                vec![StyleRule::BackgroundColour(Colour::RGBA(
                    1.0, 1.0, 1.0, 1.0,
                ))],
            ),
            StylesheetClass::new(
                "border",
                vec![
                    StyleRule::BorderWidth(Length::Pixels(2.0)),
                    StyleRule::BorderColour(Colour::RGBA(0.5, 0.5, 0.5, 1.0)),
                ],
            ),
        ]);

        let style = ComputedStyle::compute(
            &stylesheet,
            &[
                "text".to_string(),
                "background".to_string(),
                "border".to_string(),
            ],
        );

        // Should have all 5 rules
        assert_eq!(style.rules().len(), 5);
        assert!(style.has(|r| matches!(r, StyleRule::FontSize(_))));
        assert!(style.has(|r| matches!(r, StyleRule::TextColour(_))));
        assert!(style.has(|r| matches!(r, StyleRule::BackgroundColour(_))));
        assert!(style.has(|r| matches!(r, StyleRule::BorderWidth(_))));
        assert!(style.has(|r| matches!(r, StyleRule::BorderColour(_))));
    }

    #[test]
    fn test_direction_rule_included() {
        use crate::stylesheet::Direction;

        let stylesheet = Stylesheet::new(vec![StylesheetClass::new(
            "container",
            vec![
                StyleRule::Direction(Direction::Horizontal),
                StyleRule::Gap(Length::Pixels(10.0)),
            ],
        )]);

        let style = ComputedStyle::compute(&stylesheet, &["container".to_string()]);

        // Both rules should be present
        assert_eq!(style.rules().len(), 2);
        assert!(style.has(|r| matches!(r, StyleRule::Direction(_))));
        assert!(style.has(|r| matches!(r, StyleRule::Gap(_))));
    }

    #[test]
    fn test_layout_properties() {
        use crate::layout::Size;

        let stylesheet = Stylesheet::new(vec![StylesheetClass::new(
            "layout",
            vec![
                StyleRule::Width(Size::Fixed(300)),
                StyleRule::Height(Size::Fill),
                StyleRule::AlignChildrenX(Alignment::Centre),
                StyleRule::AlignChildrenY(Alignment::End),
                StyleRule::Gap(Length::Pixels(10.0)),
            ],
        )]);

        let style = ComputedStyle::compute(&stylesheet, &["layout".to_string()]);

        assert_eq!(style.rules().len(), 5);
        assert!(style.has(|r| matches!(r, StyleRule::Width(_))));
        assert!(style.has(|r| matches!(r, StyleRule::Height(_))));
        assert!(style.has(|r| matches!(r, StyleRule::AlignChildrenX(_))));
        assert!(style.has(|r| matches!(r, StyleRule::AlignChildrenY(_))));
        assert!(style.has(|r| matches!(r, StyleRule::Gap(_))));
    }

    #[test]
    fn test_align_items_and_flex_grow() {
        let stylesheet = Stylesheet::new(vec![StylesheetClass::new(
            "flexy",
            vec![
                StyleRule::AlignItems(Alignment::Centre),
                StyleRule::FlexGrow(2.0),
            ],
        )]);

        let style = ComputedStyle::compute(&stylesheet, &["flexy".to_string()]);

        assert_eq!(style.rules().len(), 2);
        assert!(matches!(
            style.get(|r| matches!(r, StyleRule::AlignItems(_))),
            Some(StyleRule::AlignItems(Alignment::Centre))
        ));
        assert!(matches!(
            style.get(|r| matches!(r, StyleRule::FlexGrow(_))),
            Some(StyleRule::FlexGrow(g)) if *g == 2.0
        ));
    }

    #[test]
    fn test_compute_ignores_media_hover_focus_rules() {
        use crate::stylesheet::media_rule::MediaRule;

        let stylesheet = Stylesheet::new(vec![StylesheetClass::new(
            "base",
            vec![StyleRule::FontSize(Length::Pixels(14.0))],
        )
        .with_hover(vec![StyleRule::FontSize(Length::Pixels(20.0))])
        .with_focus(vec![StyleRule::FontSize(Length::Pixels(24.0))])
        .with_media(vec![MediaRule::new(vec![StyleRule::FontSize(
            Length::Pixels(30.0),
        )])
        .min_width(0.0)])]);

        // Plain compute() must ignore hover/focus/media entirely, regardless
        // of viewport or interaction state.
        let style = ComputedStyle::compute(&stylesheet, &["base".to_string()]);
        assert!(matches!(
            style.get(|r| matches!(r, StyleRule::FontSize(_))),
            Some(StyleRule::FontSize(Length::Pixels(14.0)))
        ));
    }

    #[test]
    fn test_compute_with_context_media_rule_applies_when_matching() {
        use crate::stylesheet::media_rule::MediaRule;

        let stylesheet = Stylesheet::new(vec![StylesheetClass::new(
            "responsive",
            vec![StyleRule::FontSize(Length::Pixels(14.0))],
        )
        .with_media(vec![MediaRule::new(vec![StyleRule::FontSize(
            Length::Pixels(28.0),
        )])
        .min_width(600.0)])]);

        let narrow = ComputedStyle::compute_with_context(
            &stylesheet,
            &["responsive".to_string()],
            Some(Viewport {
                width: 400.0,
                height: 800.0,
            }),
            PseudoState::default(),
        );
        assert!(matches!(
            narrow.get(|r| matches!(r, StyleRule::FontSize(_))),
            Some(StyleRule::FontSize(Length::Pixels(14.0)))
        ));

        let wide = ComputedStyle::compute_with_context(
            &stylesheet,
            &["responsive".to_string()],
            Some(Viewport {
                width: 800.0,
                height: 800.0,
            }),
            PseudoState::default(),
        );
        assert!(matches!(
            wide.get(|r| matches!(r, StyleRule::FontSize(_))),
            Some(StyleRule::FontSize(Length::Pixels(28.0)))
        ));
    }

    #[test]
    fn test_compute_with_context_layering_order() {
        use crate::stylesheet::media_rule::MediaRule;

        // base < media < hover < focus: each later-applicable layer should
        // win over the earlier ones for the same rule kind.
        let stylesheet = Stylesheet::new(vec![StylesheetClass::new(
            "layered",
            vec![StyleRule::TextColour(Colour::RGBA(0.0, 0.0, 0.0, 1.0))],
        )
        .with_media(vec![MediaRule::new(vec![StyleRule::TextColour(
            Colour::RGBA(0.1, 0.0, 0.0, 1.0),
        )])
        .min_width(0.0)])
        .with_hover(vec![StyleRule::TextColour(Colour::RGBA(
            0.2, 0.0, 0.0, 1.0,
        ))])
        .with_focus(vec![StyleRule::TextColour(Colour::RGBA(
            0.3, 0.0, 0.0, 1.0,
        ))])]);

        let viewport = Some(Viewport {
            width: 800.0,
            height: 600.0,
        });

        let base_only = ComputedStyle::compute_with_context(
            &stylesheet,
            &["layered".to_string()],
            None,
            PseudoState::default(),
        );
        assert!(matches!(
            base_only.get(|r| matches!(r, StyleRule::TextColour(_))),
            Some(StyleRule::TextColour(Colour::RGBA(c, ..))) if *c == 0.0
        ));

        let media_only = ComputedStyle::compute_with_context(
            &stylesheet,
            &["layered".to_string()],
            viewport,
            PseudoState::default(),
        );
        assert!(matches!(
            media_only.get(|r| matches!(r, StyleRule::TextColour(_))),
            Some(StyleRule::TextColour(Colour::RGBA(c, ..))) if *c == 0.1
        ));

        let hovered = ComputedStyle::compute_with_context(
            &stylesheet,
            &["layered".to_string()],
            viewport,
            PseudoState {
                hovered: true,
                focused: false,
            },
        );
        assert!(matches!(
            hovered.get(|r| matches!(r, StyleRule::TextColour(_))),
            Some(StyleRule::TextColour(Colour::RGBA(c, ..))) if *c == 0.2
        ));

        let focused = ComputedStyle::compute_with_context(
            &stylesheet,
            &["layered".to_string()],
            viewport,
            PseudoState {
                hovered: true,
                focused: true,
            },
        );
        assert!(matches!(
            focused.get(|r| matches!(r, StyleRule::TextColour(_))),
            Some(StyleRule::TextColour(Colour::RGBA(c, ..))) if *c == 0.3
        ));
    }
}
