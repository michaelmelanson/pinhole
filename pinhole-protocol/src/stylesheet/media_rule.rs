use serde::{Deserialize, Serialize};

use crate::stylesheet::{style_rule::StyleRule, viewport::Viewport};

/// A set of rules that only apply when the client's viewport falls within
/// the given bounds (CSS `@media` query), e.g. `min_width`/`max_width` for
/// responsive layouts. Bounds are inclusive; `None` means unbounded on that
/// side.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaRule {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub rules: Vec<StyleRule>,
}

impl MediaRule {
    pub fn new(rules: Vec<StyleRule>) -> Self {
        MediaRule {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            rules,
        }
    }

    pub const fn min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    pub const fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    pub const fn min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    pub const fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    pub fn matches(&self, viewport: Viewport) -> bool {
        self.min_width.is_none_or(|w| viewport.width >= w)
            && self.max_width.is_none_or(|w| viewport.width <= w)
            && self.min_height.is_none_or(|h| viewport.height >= h)
            && self.max_height.is_none_or(|h| viewport.height <= h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unbounded_matches_everything() {
        let rule = MediaRule::new(vec![]);
        assert!(rule.matches(Viewport {
            width: 0.0,
            height: 0.0
        }));
        assert!(rule.matches(Viewport {
            width: 10000.0,
            height: 10000.0
        }));
    }

    #[test]
    fn test_min_width_bound() {
        let rule = MediaRule::new(vec![]).min_width(600.0);
        assert!(!rule.matches(Viewport {
            width: 599.0,
            height: 0.0
        }));
        assert!(rule.matches(Viewport {
            width: 600.0,
            height: 0.0
        }));
        assert!(rule.matches(Viewport {
            width: 601.0,
            height: 0.0
        }));
    }

    #[test]
    fn test_max_width_bound() {
        let rule = MediaRule::new(vec![]).max_width(600.0);
        assert!(rule.matches(Viewport {
            width: 600.0,
            height: 0.0
        }));
        assert!(!rule.matches(Viewport {
            width: 601.0,
            height: 0.0
        }));
    }

    #[test]
    fn test_width_range() {
        let rule = MediaRule::new(vec![]).min_width(400.0).max_width(800.0);
        assert!(!rule.matches(Viewport {
            width: 399.0,
            height: 0.0
        }));
        assert!(rule.matches(Viewport {
            width: 400.0,
            height: 0.0
        }));
        assert!(rule.matches(Viewport {
            width: 800.0,
            height: 0.0
        }));
        assert!(!rule.matches(Viewport {
            width: 801.0,
            height: 0.0
        }));
    }

    #[test]
    fn test_height_bounds() {
        let rule = MediaRule::new(vec![]).min_height(300.0).max_height(500.0);
        assert!(!rule.matches(Viewport {
            width: 0.0,
            height: 299.0
        }));
        assert!(rule.matches(Viewport {
            width: 0.0,
            height: 400.0
        }));
        assert!(!rule.matches(Viewport {
            width: 0.0,
            height: 501.0
        }));
    }

    #[test]
    fn test_width_and_height_both_required() {
        let rule = MediaRule::new(vec![]).min_width(400.0).min_height(300.0);
        assert!(!rule.matches(Viewport {
            width: 500.0,
            height: 200.0
        }));
        assert!(!rule.matches(Viewport {
            width: 300.0,
            height: 400.0
        }));
        assert!(rule.matches(Viewport {
            width: 500.0,
            height: 400.0
        }));
    }
}
