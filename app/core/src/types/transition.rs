use serde::{Deserialize, Serialize};

/// Transition effect kind for scene/background changes
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransitionKind {
    /// No transition, instant change
    #[default]
    None,
    /// Fade to black then fade in
    Fade,
    /// Fade to white then fade in
    FadeWhite,
    /// Crossfade between old and new
    Crossfade,
    /// Slide in from direction
    Slide(SlideDirection),
    /// Dissolve/pixelate effect
    Dissolve,
    /// Wipe from direction
    Wipe(WipeDirection),
}

/// Direction for slide transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlideDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Direction for wipe transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WipeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Transition configuration with duration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub kind: TransitionKind,
    /// Duration in seconds
    pub duration: f32,
}

impl Transition {
    /// Create a new transition
    pub const fn new(kind: TransitionKind, duration: f32) -> Self {
        Self { kind, duration }
    }

    /// Instant transition (no effect)
    pub const fn instant() -> Self {
        Self::new(TransitionKind::None, 0.0)
    }

    /// Quick fade (0.3 seconds)
    pub const fn quick_fade() -> Self {
        Self::new(TransitionKind::Fade, 0.3)
    }

    /// Normal fade (0.5 seconds)
    pub const fn fade() -> Self {
        Self::new(TransitionKind::Fade, 0.5)
    }

    /// Slow fade (1.0 second)
    pub const fn slow_fade() -> Self {
        Self::new(TransitionKind::Fade, 1.0)
    }

    /// Crossfade with default duration
    pub const fn crossfade() -> Self {
        Self::new(TransitionKind::Crossfade, 0.5)
    }

    /// Create a transition from a string name (for scenario files)
    ///
    /// Supported names:
    /// - "fade_in" / "fade_out" -> Fade
    /// - "crossfade" / "dissolve" -> Crossfade
    /// - "slide_left" -> Slide(Left)
    /// - "slide_right" -> Slide(Right)
    /// - "slide_up" -> Slide(Up)
    /// - "slide_down" -> Slide(Down)
    /// - "none" / "instant" -> None
    pub fn from_name(name: &str, duration: f32) -> Self {
        let kind = match name {
            "fade_in" | "fade_out" | "fade" | "fade_black" => TransitionKind::Fade,
            "fade_white" => TransitionKind::FadeWhite,
            "crossfade" | "dissolve" => TransitionKind::Crossfade,
            "slide_left" => TransitionKind::Slide(SlideDirection::Left),
            "slide_right" => TransitionKind::Slide(SlideDirection::Right),
            "slide_up" => TransitionKind::Slide(SlideDirection::Up),
            "slide_down" => TransitionKind::Slide(SlideDirection::Down),
            "wipe_left" => TransitionKind::Wipe(WipeDirection::Left),
            "wipe_right" => TransitionKind::Wipe(WipeDirection::Right),
            "wipe_up" => TransitionKind::Wipe(WipeDirection::Up),
            "wipe_down" => TransitionKind::Wipe(WipeDirection::Down),
            "none" | "instant" => TransitionKind::None,
            _ => TransitionKind::None,
        };
        Self::new(kind, duration)
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self::instant()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_kind_none() {
        let kind = TransitionKind::None;
        assert_eq!(kind, TransitionKind::default());
    }

    #[test]
    fn test_transition_kind_fade() {
        let kind = TransitionKind::Fade;
        assert_ne!(kind, TransitionKind::None);
    }

    #[test]
    fn test_transition_kind_slide() {
        let left = TransitionKind::Slide(SlideDirection::Left);
        let right = TransitionKind::Slide(SlideDirection::Right);
        assert_ne!(left, right);
    }

    #[test]
    fn test_transition_kind_wipe() {
        let up = TransitionKind::Wipe(WipeDirection::Up);
        let down = TransitionKind::Wipe(WipeDirection::Down);
        assert_ne!(up, down);
    }

    #[test]
    fn test_transition_kind_serialization() {
        let kind = TransitionKind::Fade;
        let serialized = serde_json::to_string(&kind).unwrap();
        let deserialized: TransitionKind = serde_json::from_str(&serialized).unwrap();
        assert_eq!(kind, deserialized);
    }

    #[test]
    fn test_slide_direction_variants() {
        assert_eq!(SlideDirection::Left, SlideDirection::Left);
        assert_ne!(SlideDirection::Left, SlideDirection::Right);
        assert_ne!(SlideDirection::Up, SlideDirection::Down);
    }

    #[test]
    fn test_wipe_direction_variants() {
        assert_eq!(WipeDirection::Right, WipeDirection::Right);
        assert_ne!(WipeDirection::Left, WipeDirection::Right);
        assert_ne!(WipeDirection::Up, WipeDirection::Down);
    }

    #[test]
    fn test_transition_new() {
        let t = Transition::new(TransitionKind::Fade, 1.5);
        assert_eq!(t.kind, TransitionKind::Fade);
        assert_eq!(t.duration, 1.5);
    }

    #[test]
    fn test_transition_instant() {
        let t = Transition::instant();
        assert_eq!(t.kind, TransitionKind::None);
        assert_eq!(t.duration, 0.0);
    }

    #[test]
    fn test_transition_quick_fade() {
        let t = Transition::quick_fade();
        assert_eq!(t.kind, TransitionKind::Fade);
        assert_eq!(t.duration, 0.3);
    }

    #[test]
    fn test_transition_fade() {
        let t = Transition::fade();
        assert_eq!(t.kind, TransitionKind::Fade);
        assert_eq!(t.duration, 0.5);
    }

    #[test]
    fn test_transition_slow_fade() {
        let t = Transition::slow_fade();
        assert_eq!(t.kind, TransitionKind::Fade);
        assert_eq!(t.duration, 1.0);
    }

    #[test]
    fn test_transition_crossfade() {
        let t = Transition::crossfade();
        assert_eq!(t.kind, TransitionKind::Crossfade);
        assert_eq!(t.duration, 0.5);
    }

    #[test]
    fn test_transition_default() {
        let t = Transition::default();
        assert_eq!(t.kind, TransitionKind::None);
        assert_eq!(t.duration, 0.0);
    }

    #[test]
    fn test_transition_serialization() {
        let t = Transition::new(TransitionKind::Crossfade, 0.7);
        let serialized = serde_json::to_string(&t).unwrap();
        let deserialized: Transition = serde_json::from_str(&serialized).unwrap();
        assert_eq!(t, deserialized);
    }

    #[test]
    fn test_transition_with_slide() {
        let t = Transition::new(TransitionKind::Slide(SlideDirection::Right), 0.8);
        assert_eq!(t.duration, 0.8);
        if let TransitionKind::Slide(dir) = t.kind {
            assert_eq!(dir, SlideDirection::Right);
        } else {
            panic!("Expected Slide transition");
        }
    }

    #[test]
    fn test_transition_with_wipe() {
        let t = Transition::new(TransitionKind::Wipe(WipeDirection::Down), 1.2);
        assert_eq!(t.duration, 1.2);
        if let TransitionKind::Wipe(dir) = t.kind {
            assert_eq!(dir, WipeDirection::Down);
        } else {
            panic!("Expected Wipe transition");
        }
    }
}
