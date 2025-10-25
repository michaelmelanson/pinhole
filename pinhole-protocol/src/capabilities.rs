use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A capability URI identifying a protocol feature
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Capability(String);

impl Capability {
    /// Create a new capability from a URI string
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    /// Get the capability URI as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Core protocol capability (version 1)
    pub const CORE_V1: &'static str = "pinhole:core:v1";
}

impl From<&str> for Capability {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Capability {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Set of capabilities supported by a client or server
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
}

impl CapabilitySet {
    /// Create a new empty capability set
    pub fn new() -> Self {
        Self {
            capabilities: HashSet::new(),
        }
    }

    /// Create a capability set with the given capabilities
    pub fn from_capabilities(capabilities: impl IntoIterator<Item = Capability>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }

    /// Add a capability to the set
    pub fn add(&mut self, capability: impl Into<Capability>) {
        self.capabilities.insert(capability.into());
    }

    /// Check if the set contains a capability
    pub fn contains(&self, capability: &str) -> bool {
        self.capabilities.contains(&Capability::new(capability))
    }

    /// Get the intersection of two capability sets
    pub fn intersect(&self, other: &CapabilitySet) -> CapabilitySet {
        Self {
            capabilities: self
                .capabilities
                .intersection(&other.capabilities)
                .cloned()
                .collect(),
        }
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    /// Get the number of capabilities
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Iterate over capabilities
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.iter()
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the hardcoded set of capabilities supported by this implementation
pub fn supported_capabilities() -> CapabilitySet {
    let mut caps = CapabilitySet::new();
    caps.add(Capability::CORE_V1);
    caps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new("pinhole:core:v1");
        assert_eq!(cap.as_str(), "pinhole:core:v1");
    }

    #[test]
    fn test_capability_set_operations() {
        let mut set1 = CapabilitySet::new();
        set1.add("pinhole:core:v1");
        set1.add("pinhole:storage:arrays");

        let mut set2 = CapabilitySet::new();
        set2.add("pinhole:core:v1");
        set2.add("pinhole:stylesheet:pseudo-classes");

        assert!(set1.contains("pinhole:core:v1"));
        assert!(set1.contains("pinhole:storage:arrays"));
        assert!(!set1.contains("pinhole:stylesheet:pseudo-classes"));

        let intersection = set1.intersect(&set2);
        assert_eq!(intersection.len(), 1);
        assert!(intersection.contains("pinhole:core:v1"));
    }

    #[test]
    fn test_supported_capabilities() {
        let caps = supported_capabilities();
        assert!(caps.contains(Capability::CORE_V1));
        assert_eq!(caps.len(), 1);
    }
}
