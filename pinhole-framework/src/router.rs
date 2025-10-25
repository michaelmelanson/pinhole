use std::collections::HashMap;

pub type Params = HashMap<String, String>;

/// A route pattern that can match paths and extract parameters
#[derive(Debug, Clone)]
pub struct RoutePattern {
    pattern: String,
    segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq)]
enum Segment {
    Literal(String),
    Param(String),
}

impl RoutePattern {
    /// Create a new route pattern from a path like "/resources/:id/subpage"
    pub fn new(pattern: &str) -> Self {
        let segments = pattern
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|segment| {
                if let Some(param_name) = segment.strip_prefix(':') {
                    Segment::Param(param_name.to_string())
                } else {
                    Segment::Literal(segment.to_string())
                }
            })
            .collect();

        RoutePattern {
            pattern: pattern.to_string(),
            segments,
        }
    }

    /// Check if a path matches this pattern and extract parameters
    pub fn matches(&self, path: &str) -> Option<Params> {
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if path_segments.len() != self.segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_seg, path_seg) in self.segments.iter().zip(path_segments.iter()) {
            match pattern_seg {
                Segment::Literal(lit) => {
                    if lit != path_seg {
                        return None;
                    }
                }
                Segment::Param(name) => {
                    params.insert(name.clone(), path_seg.to_string());
                }
            }
        }

        Some(params)
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let pattern = RoutePattern::new("/users/list");
        assert!(pattern.matches("/users/list").is_some());
        assert!(pattern.matches("/users/other").is_none());
        assert!(pattern.matches("/users").is_none());
    }

    #[test]
    fn test_single_param() {
        let pattern = RoutePattern::new("/users/:id");

        let params = pattern.matches("/users/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));

        let params = pattern.matches("/users/abc").unwrap();
        assert_eq!(params.get("id"), Some(&"abc".to_string()));

        assert!(pattern.matches("/users").is_none());
        assert!(pattern.matches("/users/123/extra").is_none());
    }

    #[test]
    fn test_multiple_params() {
        let pattern = RoutePattern::new("/resources/:id/comments/:comment_id");

        let params = pattern.matches("/resources/123/comments/456").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("comment_id"), Some(&"456".to_string()));

        assert!(pattern.matches("/resources/123/comments").is_none());
    }

    #[test]
    fn test_param_with_suffix() {
        let pattern = RoutePattern::new("/resources/:id/subpage");

        let params = pattern.matches("/resources/42/subpage").unwrap();
        assert_eq!(params.get("id"), Some(&"42".to_string()));

        assert!(pattern.matches("/resources/42/other").is_none());
    }

    #[test]
    fn test_root_path() {
        let pattern = RoutePattern::new("/");
        assert!(pattern.matches("/").is_some());
        assert!(pattern.matches("/users").is_none());
    }

    #[test]
    fn test_trailing_slash() {
        let pattern = RoutePattern::new("/users/:id");
        // Both should work
        assert!(pattern.matches("/users/123").is_some());
        assert!(pattern.matches("/users/123/").is_some());
    }
}
