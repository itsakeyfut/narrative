//! Skip mode configuration

use serde::{Deserialize, Serialize};

/// Skip mode type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkipMode {
    /// Skip is disabled
    Disabled,
    /// Skip only read text (default for visual novels)
    #[default]
    ReadOnly,
    /// Skip all text (including unread)
    All,
}

impl SkipMode {
    /// Check if skip mode is enabled (any mode except Disabled)
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Disabled)
    }

    /// Check if this mode allows skipping unread text
    pub fn allows_unread(&self) -> bool {
        matches!(self, Self::All)
    }

    /// Check if this mode requires text to be read
    pub fn requires_read(&self) -> bool {
        matches!(self, Self::ReadOnly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_mode_default() {
        let mode = SkipMode::default();
        assert_eq!(mode, SkipMode::ReadOnly);
    }

    #[test]
    fn test_is_enabled() {
        assert!(!SkipMode::Disabled.is_enabled());
        assert!(SkipMode::ReadOnly.is_enabled());
        assert!(SkipMode::All.is_enabled());
    }

    #[test]
    fn test_allows_unread() {
        assert!(!SkipMode::Disabled.allows_unread());
        assert!(!SkipMode::ReadOnly.allows_unread());
        assert!(SkipMode::All.allows_unread());
    }

    #[test]
    fn test_requires_read() {
        assert!(!SkipMode::Disabled.requires_read());
        assert!(SkipMode::ReadOnly.requires_read());
        assert!(!SkipMode::All.requires_read());
    }

    #[test]
    fn test_serialization() {
        // Test each variant
        let disabled = SkipMode::Disabled;
        let serialized = ron::to_string(&disabled).unwrap();
        assert_eq!(serialized, "disabled");

        let read_only = SkipMode::ReadOnly;
        let serialized = ron::to_string(&read_only).unwrap();
        assert_eq!(serialized, "read_only");

        let all = SkipMode::All;
        let serialized = ron::to_string(&all).unwrap();
        assert_eq!(serialized, "all");
    }

    #[test]
    fn test_deserialization() {
        let disabled: SkipMode = ron::from_str("disabled").unwrap();
        assert_eq!(disabled, SkipMode::Disabled);

        let read_only: SkipMode = ron::from_str("read_only").unwrap();
        assert_eq!(read_only, SkipMode::ReadOnly);

        let all: SkipMode = ron::from_str("all").unwrap();
        assert_eq!(all, SkipMode::All);
    }

    #[test]
    fn test_clone() {
        let mode = SkipMode::ReadOnly;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_equality() {
        assert_eq!(SkipMode::Disabled, SkipMode::Disabled);
        assert_eq!(SkipMode::ReadOnly, SkipMode::ReadOnly);
        assert_eq!(SkipMode::All, SkipMode::All);

        assert_ne!(SkipMode::Disabled, SkipMode::ReadOnly);
        assert_ne!(SkipMode::ReadOnly, SkipMode::All);
    }
}
