// Escape animation - character gradually moves then quickly exits the screen
//!
//! The escape animation simulates a character running away:
//! 1. Preparation phase: Small movement in escape direction (nervous step back)
//! 2. Acceleration phase: Rapid movement off-screen
//!
//! Presets:
//! - Small: Backs away slightly (move 100px off-screen)
//! - Medium: Runs away (move 400px off-screen)
//! - Large: Flees at full speed (move 800px off-screen)

use super::easing::EasingFunction;
use super::keyframe::{AnimationPhase, Keyframe, KeyframeAnimation};
use serde::{Deserialize, Serialize};

/// Direction for the escape animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscapeDirection {
    /// Escape to the left
    Left,
    /// Escape to the right
    Right,
    /// Escape upward
    Up,
    /// Escape downward
    Down,
}

impl Default for EscapeDirection {
    fn default() -> Self {
        Self::Right
    }
}

/// Preset intensity levels for escape animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscapePreset {
    /// Small escape (backs away slightly)
    Small,
    /// Medium escape (runs away)
    Medium,
    /// Large escape (flees at full speed)
    Large,
}

impl Default for EscapePreset {
    fn default() -> Self {
        Self::Medium
    }
}

impl EscapePreset {
    /// Get the preparation distance for this preset in pixels
    fn preparation_distance(self) -> f32 {
        match self {
            Self::Small => 10.0,
            Self::Medium => 20.0,
            Self::Large => 30.0,
        }
    }

    /// Get the escape distance for this preset in pixels
    fn escape_distance(self) -> f32 {
        match self {
            Self::Small => 100.0,
            Self::Medium => 400.0,
            Self::Large => 800.0,
        }
    }

    /// Get the preparation duration for this preset in seconds
    fn preparation_duration(self) -> f32 {
        match self {
            Self::Small => 0.15,
            Self::Medium => 0.2,
            Self::Large => 0.25,
        }
    }

    /// Get the escape duration for this preset in seconds
    fn escape_duration(self) -> f32 {
        match self {
            Self::Small => 0.3,
            Self::Medium => 0.3,
            Self::Large => 0.35,
        }
    }
}

/// Escape animation configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscapeAnimation {
    /// Direction of the escape
    #[serde(default)]
    pub direction: EscapeDirection,
    /// Preset configuration or custom parameters
    #[serde(flatten)]
    pub config: EscapeConfig,
}

/// Escape animation configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EscapeConfig {
    /// Use a preset configuration
    Preset { preset: EscapePreset },
    /// Custom configuration
    Custom {
        /// Distance to move during preparation phase (pixels)
        preparation_distance: f32,
        /// Distance to move during escape phase (pixels)
        escape_distance: f32,
        /// Duration of preparation phase (seconds)
        #[serde(default = "default_prep_duration")]
        preparation_duration: f32,
        /// Duration of escape phase (seconds)
        #[serde(default = "default_escape_duration")]
        escape_duration: f32,
        /// Whether to fade out during escape
        #[serde(default)]
        fade_out: bool,
    },
}

fn default_prep_duration() -> f32 {
    0.2
}

fn default_escape_duration() -> f32 {
    0.3
}

impl Default for EscapeConfig {
    fn default() -> Self {
        Self::Preset {
            preset: EscapePreset::default(),
        }
    }
}

impl EscapeAnimation {
    /// Create a new escape animation with preset
    pub fn new(direction: EscapeDirection, preset: EscapePreset) -> Self {
        Self {
            direction,
            config: EscapeConfig::Preset { preset },
        }
    }

    /// Create a custom escape animation
    pub fn custom(
        direction: EscapeDirection,
        preparation_distance: f32,
        escape_distance: f32,
        preparation_duration: f32,
        escape_duration: f32,
        fade_out: bool,
    ) -> Self {
        Self {
            direction,
            config: EscapeConfig::Custom {
                preparation_distance,
                escape_distance,
                preparation_duration,
                escape_duration,
                fade_out,
            },
        }
    }

    /// Get the preparation distance
    fn preparation_distance(&self) -> f32 {
        match &self.config {
            EscapeConfig::Preset { preset } => preset.preparation_distance(),
            EscapeConfig::Custom {
                preparation_distance,
                ..
            } => *preparation_distance,
        }
    }

    /// Get the escape distance
    fn escape_distance(&self) -> f32 {
        match &self.config {
            EscapeConfig::Preset { preset } => preset.escape_distance(),
            EscapeConfig::Custom { escape_distance, .. } => *escape_distance,
        }
    }

    /// Get the preparation duration
    fn preparation_duration(&self) -> f32 {
        match &self.config {
            EscapeConfig::Preset { preset } => preset.preparation_duration(),
            EscapeConfig::Custom {
                preparation_duration,
                ..
            } => *preparation_duration,
        }
    }

    /// Get the escape duration
    fn escape_duration(&self) -> f32 {
        match &self.config {
            EscapeConfig::Preset { preset } => preset.escape_duration(),
            EscapeConfig::Custom {
                escape_duration, ..
            } => *escape_duration,
        }
    }

    /// Check if fade out is enabled
    fn fade_out(&self) -> bool {
        match &self.config {
            EscapeConfig::Preset { .. } => false,
            EscapeConfig::Custom { fade_out, .. } => *fade_out,
        }
    }

    /// Convert direction-relative distances to absolute (x, y) offsets
    fn direction_offset(&self, distance: f32) -> (f32, f32) {
        match self.direction {
            EscapeDirection::Left => (-distance, 0.0),
            EscapeDirection::Right => (distance, 0.0),
            EscapeDirection::Up => (0.0, -distance),
            EscapeDirection::Down => (0.0, distance),
        }
    }
}

impl KeyframeAnimation for EscapeAnimation {
    fn keyframes(&self) -> Vec<Keyframe> {
        let prep_dist = self.preparation_distance();
        let escape_dist = self.escape_distance();
        let fade_out = self.fade_out();

        let (prep_x, prep_y) = self.direction_offset(prep_dist);
        let (escape_x, escape_y) = self.direction_offset(escape_dist);

        vec![
            Keyframe::new(0.0, 0.0, 0.0),
            Keyframe::new(0.333, prep_x, prep_y),
            Keyframe::with_properties(1.0, escape_x, escape_y, 0.0, 1.0, if fade_out { 0.0 } else { 1.0 }),
        ]
    }

    fn phases(&self) -> Vec<AnimationPhase> {
        let prep_dur = self.preparation_duration();
        let escape_dur = self.escape_duration();
        let prep_dist = self.preparation_distance();
        let escape_dist = self.escape_distance();
        let fade_out = self.fade_out();

        let (prep_x, prep_y) = self.direction_offset(prep_dist);
        let (escape_x, escape_y) = self.direction_offset(escape_dist);

        vec![
            // Phase 1: Preparation - small movement in escape direction
            AnimationPhase::new(
                prep_dur,
                EasingFunction::EaseOut,
                Keyframe::new(0.0, 0.0, 0.0),
                Keyframe::new(1.0, prep_x, prep_y),
            ),
            // Phase 2: Escape - rapid movement off screen
            AnimationPhase::new(
                escape_dur,
                EasingFunction::EaseIn,
                Keyframe::new(0.0, prep_x, prep_y),
                Keyframe::with_properties(
                    1.0,
                    escape_x,
                    escape_y,
                    0.0,
                    1.0,
                    if fade_out { 0.0 } else { 1.0 },
                ),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_preset_values() {
        assert_eq!(EscapePreset::Small.preparation_distance(), 10.0);
        assert_eq!(EscapePreset::Medium.preparation_distance(), 20.0);
        assert_eq!(EscapePreset::Large.preparation_distance(), 30.0);

        assert_eq!(EscapePreset::Small.escape_distance(), 100.0);
        assert_eq!(EscapePreset::Medium.escape_distance(), 400.0);
        assert_eq!(EscapePreset::Large.escape_distance(), 800.0);
    }

    #[test]
    fn test_escape_animation_preset() {
        let escape = EscapeAnimation::new(EscapeDirection::Right, EscapePreset::Medium);
        assert_eq!(escape.direction, EscapeDirection::Right);
        assert_eq!(escape.preparation_distance(), 20.0);
        assert_eq!(escape.escape_distance(), 400.0);
        assert!(!escape.fade_out());
    }

    #[test]
    fn test_escape_animation_custom() {
        let escape = EscapeAnimation::custom(
            EscapeDirection::Left,
            15.0,
            300.0,
            0.25,
            0.35,
            true,
        );
        assert_eq!(escape.direction, EscapeDirection::Left);
        assert_eq!(escape.preparation_distance(), 15.0);
        assert_eq!(escape.escape_distance(), 300.0);
        assert_eq!(escape.preparation_duration(), 0.25);
        assert_eq!(escape.escape_duration(), 0.35);
        assert!(escape.fade_out());
    }

    #[test]
    fn test_escape_direction_offset() {
        let escape_right = EscapeAnimation::new(EscapeDirection::Right, EscapePreset::Small);
        assert_eq!(escape_right.direction_offset(100.0), (100.0, 0.0));

        let escape_left = EscapeAnimation::new(EscapeDirection::Left, EscapePreset::Small);
        assert_eq!(escape_left.direction_offset(100.0), (-100.0, 0.0));

        let escape_up = EscapeAnimation::new(EscapeDirection::Up, EscapePreset::Small);
        assert_eq!(escape_up.direction_offset(100.0), (0.0, -100.0));

        let escape_down = EscapeAnimation::new(EscapeDirection::Down, EscapePreset::Small);
        assert_eq!(escape_down.direction_offset(100.0), (0.0, 100.0));
    }

    #[test]
    fn test_escape_animation_phases() {
        let escape = EscapeAnimation::new(EscapeDirection::Right, EscapePreset::Medium);
        let phases = escape.phases();

        assert_eq!(phases.len(), 2);
        assert_eq!(phases[0].duration, 0.2); // Preparation duration
        assert_eq!(phases[1].duration, 0.3); // Escape duration
    }

    #[test]
    fn test_escape_animation_total_duration() {
        let escape = EscapeAnimation::new(EscapeDirection::Right, EscapePreset::Medium);
        let total = escape.total_duration();
        assert_eq!(total, 0.5); // 0.2 + 0.3
    }

    #[test]
    fn test_escape_animation_transform_start() {
        let escape = EscapeAnimation::new(EscapeDirection::Right, EscapePreset::Small);
        let transform = escape.transform_at_progress(0.0);
        assert_eq!(transform.x, 0.0);
        assert_eq!(transform.y, 0.0);
        assert_eq!(transform.alpha, 1.0);
    }

    #[test]
    fn test_escape_animation_transform_midpoint() {
        let escape = EscapeAnimation::new(EscapeDirection::Right, EscapePreset::Small);
        let transform = escape.transform_at_progress(0.5);
        // Should be somewhere between prep and full escape
        assert!(transform.x > 0.0 && transform.x < 100.0);
    }

    #[test]
    fn test_escape_animation_transform_end() {
        let escape = EscapeAnimation::new(EscapeDirection::Right, EscapePreset::Small);
        let transform = escape.transform_at_progress(1.0);
        assert_eq!(transform.x, 100.0); // Small preset escape distance
        assert_eq!(transform.y, 0.0);
        assert_eq!(transform.alpha, 1.0); // No fade out for preset
    }

    #[test]
    fn test_escape_animation_fade_out() {
        let escape = EscapeAnimation::custom(
            EscapeDirection::Right,
            20.0,
            400.0,
            0.2,
            0.3,
            true,
        );
        let transform = escape.transform_at_progress(1.0);
        assert_eq!(transform.alpha, 0.0); // Should fade out
    }

    #[test]
    fn test_escape_serde_preset() {
        let escape = EscapeAnimation::new(EscapeDirection::Left, EscapePreset::Large);
        let json = serde_json::to_string(&escape).unwrap();
        let deserialized: EscapeAnimation = serde_json::from_str(&json).unwrap();
        assert_eq!(escape, deserialized);
    }

    #[test]
    fn test_escape_serde_custom() {
        let escape = EscapeAnimation::custom(
            EscapeDirection::Up,
            15.0,
            300.0,
            0.25,
            0.35,
            true,
        );
        let json = serde_json::to_string(&escape).unwrap();
        let deserialized: EscapeAnimation = serde_json::from_str(&json).unwrap();
        assert_eq!(escape, deserialized);
    }
}
