use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::escape::{EscapeAnimation, EscapeDirection, EscapePreset};
use super::faint::{FaintAnimation, FaintPreset};
use super::keyframe::{AnimationTransform, KeyframeAnimation};

/// Character animation type for emotion expression and complex actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CharacterAnimation {
    /// No animation
    None,
    /// Shake animation (for surprise, anger)
    /// Horizontal oscillation effect
    Shake {
        #[serde(default)]
        intensity: AnimationIntensity,
        #[serde(default)]
        timing: AnimationTiming,
    },
    /// Jump animation (for joy, excitement)
    /// Vertical bounce effect
    Jump {
        #[serde(default)]
        intensity: AnimationIntensity,
        #[serde(default)]
        timing: AnimationTiming,
    },
    /// Tremble animation (for fear, anxiety)
    /// Rapid small vibration effect
    Tremble {
        #[serde(default)]
        intensity: AnimationIntensity,
        #[serde(default)]
        timing: AnimationTiming,
    },
    /// Escape animation (running away)
    /// Character gradually moves then quickly exits screen
    #[serde(rename = "escape")]
    Escape {
        #[serde(flatten)]
        animation: EscapeAnimation,
    },
    /// Faint animation (collapsing)
    /// Character sways and then collapses downward
    #[serde(rename = "faint")]
    Faint {
        #[serde(flatten)]
        animation: FaintAnimation,
    },
}

impl Default for CharacterAnimation {
    fn default() -> Self {
        Self::None
    }
}

impl CharacterAnimation {
    /// Create a shake animation with default settings
    pub fn shake() -> Self {
        Self::Shake {
            intensity: AnimationIntensity::default(),
            timing: AnimationTiming::default(),
        }
    }

    /// Create a jump animation with default settings
    pub fn jump() -> Self {
        Self::Jump {
            intensity: AnimationIntensity::default(),
            timing: AnimationTiming::default(),
        }
    }

    /// Create a tremble animation with default settings
    pub fn tremble() -> Self {
        Self::Tremble {
            intensity: AnimationIntensity::default(),
            timing: AnimationTiming::default(),
        }
    }

    /// Create a shake animation with custom intensity
    pub fn shake_with_intensity(intensity: AnimationIntensity) -> Self {
        Self::Shake {
            intensity,
            timing: AnimationTiming::default(),
        }
    }

    /// Create a jump animation with custom intensity
    pub fn jump_with_intensity(intensity: AnimationIntensity) -> Self {
        Self::Jump {
            intensity,
            timing: AnimationTiming::default(),
        }
    }

    /// Create a tremble animation with custom intensity
    pub fn tremble_with_intensity(intensity: AnimationIntensity) -> Self {
        Self::Tremble {
            intensity,
            timing: AnimationTiming::default(),
        }
    }

    /// Create animation with custom timing
    pub fn with_timing(mut self, timing: AnimationTiming) -> Self {
        match &mut self {
            Self::Shake { timing: t, .. }
            | Self::Jump { timing: t, .. }
            | Self::Tremble { timing: t, .. } => {
                *t = timing;
            }
            Self::None | Self::Escape { .. } | Self::Faint { .. } => {}
        }
        self
    }

    /// Create an escape animation with default settings
    pub fn escape(direction: EscapeDirection, preset: EscapePreset) -> Self {
        Self::Escape {
            animation: EscapeAnimation::new(direction, preset),
        }
    }

    /// Create a faint animation with default settings
    pub fn faint(preset: FaintPreset) -> Self {
        Self::Faint {
            animation: FaintAnimation::new(preset),
        }
    }

    /// Create a custom escape animation
    pub fn escape_custom(
        direction: EscapeDirection,
        preparation_distance: f32,
        escape_distance: f32,
        preparation_duration: f32,
        escape_duration: f32,
        fade_out: bool,
    ) -> Self {
        Self::Escape {
            animation: EscapeAnimation::custom(
                direction,
                preparation_distance,
                escape_distance,
                preparation_duration,
                escape_duration,
                fade_out,
            ),
        }
    }

    /// Create a custom faint animation
    pub fn faint_custom(
        sway_amplitude: f32,
        sway_cycles: u32,
        collapse_distance: f32,
        sway_duration: f32,
        collapse_duration: f32,
        rotate: bool,
    ) -> Self {
        Self::Faint {
            animation: FaintAnimation::custom(
                sway_amplitude,
                sway_cycles,
                collapse_distance,
                sway_duration,
                collapse_duration,
                rotate,
            ),
        }
    }

    /// Check if this is a non-None animation
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if this animation uses the keyframe system
    pub fn is_keyframe_based(&self) -> bool {
        matches!(self, Self::Escape { .. } | Self::Faint { .. })
    }

    /// Get the intensity of the animation (for legacy animations only)
    pub fn intensity(&self) -> Option<&AnimationIntensity> {
        match self {
            Self::Shake { intensity, .. }
            | Self::Jump { intensity, .. }
            | Self::Tremble { intensity, .. } => Some(intensity),
            Self::None | Self::Escape { .. } | Self::Faint { .. } => None,
        }
    }

    /// Get the timing mode of the animation (for legacy animations only)
    pub fn timing(&self) -> Option<&AnimationTiming> {
        match self {
            Self::Shake { timing, .. }
            | Self::Jump { timing, .. }
            | Self::Tremble { timing, .. } => Some(timing),
            Self::None | Self::Escape { .. } | Self::Faint { .. } => None,
        }
    }

    /// Get the total duration for keyframe-based animations
    /// Returns None for legacy animations (use timing().total_duration() instead)
    pub fn keyframe_duration(&self) -> Option<f32> {
        match self {
            Self::Escape { animation } => Some(animation.total_duration()),
            Self::Faint { animation } => Some(animation.total_duration()),
            _ => None,
        }
    }

    /// Get the transform at a given progress for keyframe-based animations
    /// Returns None for legacy animations
    pub fn keyframe_transform_at(&self, progress: f32) -> Option<AnimationTransform> {
        match self {
            Self::Escape { animation } => Some(animation.transform_at_progress(progress)),
            Self::Faint { animation } => Some(animation.transform_at_progress(progress)),
            _ => None,
        }
    }
}

/// Animation intensity configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimationIntensity {
    /// Preset intensity levels
    Preset(IntensityPreset),
    /// Custom intensity with direct numeric values
    Custom {
        /// Animation amplitude (displacement in pixels at reference resolution 1280x720)
        /// For shake/tremble: horizontal displacement
        /// For jump: vertical displacement
        amplitude: f32,
        /// Number of animation cycles (oscillations or bounces)
        #[serde(default = "default_count")]
        count: u32,
    },
}

fn default_count() -> u32 {
    3
}

impl Default for AnimationIntensity {
    fn default() -> Self {
        Self::Preset(IntensityPreset::default())
    }
}

impl AnimationIntensity {
    /// Create a small preset intensity
    pub fn small() -> Self {
        Self::Preset(IntensityPreset::Small)
    }

    /// Create a medium preset intensity
    pub fn medium() -> Self {
        Self::Preset(IntensityPreset::Medium)
    }

    /// Create a large preset intensity
    pub fn large() -> Self {
        Self::Preset(IntensityPreset::Large)
    }

    /// Create a custom intensity
    pub fn custom(amplitude: f32, count: u32) -> Self {
        Self::Custom { amplitude, count }
    }

    /// Get the amplitude value in pixels
    pub fn amplitude(&self) -> f32 {
        match self {
            Self::Preset(preset) => preset.amplitude(),
            Self::Custom { amplitude, .. } => *amplitude,
        }
    }

    /// Get the number of animation cycles
    pub fn count(&self) -> u32 {
        match self {
            Self::Preset(preset) => preset.count(),
            Self::Custom { count, .. } => *count,
        }
    }
}

/// Preset intensity levels with predefined amplitude and count values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntensityPreset {
    /// Small intensity (subtle animation)
    Small,
    /// Medium intensity (normal animation)
    Medium,
    /// Large intensity (dramatic animation)
    Large,
}

impl Default for IntensityPreset {
    fn default() -> Self {
        Self::Medium
    }
}

impl IntensityPreset {
    /// Get the amplitude for this preset in pixels
    pub fn amplitude(self) -> f32 {
        match self {
            Self::Small => 5.0,
            Self::Medium => 10.0,
            Self::Large => 20.0,
        }
    }

    /// Get the cycle count for this preset
    pub fn count(self) -> u32 {
        match self {
            Self::Small => 2,
            Self::Medium => 3,
            Self::Large => 4,
        }
    }
}

/// Animation timing mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum AnimationTiming {
    /// Auto mode: animation plays once when dialogue appears and completes automatically
    /// Duration is calculated based on intensity (count * base_duration_per_cycle)
    Auto,
    /// Duration mode: animation plays for a specified duration then stops
    Duration {
        /// Duration in seconds
        duration_secs: f32,
    },
    /// Continuous mode: animation loops continuously until next command
    /// Useful for long-lasting emotional states
    Continuous,
}

impl Default for AnimationTiming {
    fn default() -> Self {
        Self::Auto
    }
}

impl AnimationTiming {
    /// Create a duration timing mode
    pub fn duration(duration_secs: f32) -> Self {
        Self::Duration { duration_secs }
    }

    /// Create a continuous timing mode
    pub fn continuous() -> Self {
        Self::Continuous
    }

    /// Calculate the total animation duration
    /// Returns None for continuous mode
    pub fn total_duration(&self, animation_type: &CharacterAnimation) -> Option<Duration> {
        match self {
            Self::Auto => {
                // Calculate duration based on animation count
                // Base duration per cycle: 0.15 seconds
                const BASE_CYCLE_DURATION: f32 = 0.15;
                let count = animation_type
                    .intensity()
                    .map(|i| i.count())
                    .unwrap_or(3) as f32;
                Some(Duration::from_secs_f32(count * BASE_CYCLE_DURATION))
            }
            Self::Duration { duration_secs } => Some(Duration::from_secs_f32(*duration_secs)),
            Self::Continuous => None, // No fixed duration
        }
    }

    /// Check if this is continuous mode
    pub fn is_continuous(&self) -> bool {
        matches!(self, Self::Continuous)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_default() {
        assert_eq!(CharacterAnimation::default(), CharacterAnimation::None);
    }

    #[test]
    fn test_animation_constructors() {
        assert!(CharacterAnimation::shake().is_active());
        assert!(CharacterAnimation::jump().is_active());
        assert!(CharacterAnimation::tremble().is_active());
        assert!(!CharacterAnimation::None.is_active());
    }

    #[test]
    fn test_animation_with_intensity() {
        let shake = CharacterAnimation::shake_with_intensity(AnimationIntensity::large());
        assert_eq!(shake.intensity().unwrap().amplitude(), 20.0);
    }

    #[test]
    fn test_animation_with_timing() {
        let shake = CharacterAnimation::shake().with_timing(AnimationTiming::continuous());
        assert!(shake.timing().unwrap().is_continuous());
    }

    #[test]
    fn test_intensity_preset_values() {
        assert_eq!(IntensityPreset::Small.amplitude(), 5.0);
        assert_eq!(IntensityPreset::Medium.amplitude(), 10.0);
        assert_eq!(IntensityPreset::Large.amplitude(), 20.0);

        assert_eq!(IntensityPreset::Small.count(), 2);
        assert_eq!(IntensityPreset::Medium.count(), 3);
        assert_eq!(IntensityPreset::Large.count(), 4);
    }

    #[test]
    fn test_intensity_custom() {
        let custom = AnimationIntensity::custom(15.0, 5);
        assert_eq!(custom.amplitude(), 15.0);
        assert_eq!(custom.count(), 5);
    }

    #[test]
    fn test_intensity_default() {
        let default = AnimationIntensity::default();
        assert_eq!(default.amplitude(), 10.0); // Medium preset
        assert_eq!(default.count(), 3);
    }

    #[test]
    fn test_timing_auto_duration() {
        let timing = AnimationTiming::Auto;
        let shake = CharacterAnimation::shake_with_intensity(AnimationIntensity::medium());
        let duration = timing.total_duration(&shake).unwrap();
        let expected = Duration::from_secs_f32(0.45); // 3 cycles * 0.15s
        // Use approximate equality for floating point comparison
        let diff = if duration > expected {
            duration - expected
        } else {
            expected - duration
        };
        assert!(diff < Duration::from_micros(100), "Duration difference too large: {:?}", diff);
    }

    #[test]
    fn test_timing_duration_mode() {
        let timing = AnimationTiming::duration(1.0);
        let shake = CharacterAnimation::shake();
        let duration = timing.total_duration(&shake).unwrap();
        assert_eq!(duration, Duration::from_secs_f32(1.0));
    }

    #[test]
    fn test_timing_continuous_mode() {
        let timing = AnimationTiming::continuous();
        let shake = CharacterAnimation::shake();
        assert!(timing.total_duration(&shake).is_none());
        assert!(timing.is_continuous());
    }

    #[test]
    fn test_animation_serde_shake() {
        let shake = CharacterAnimation::Shake {
            intensity: AnimationIntensity::Preset(IntensityPreset::Large),
            timing: AnimationTiming::Auto,
        };

        let json = serde_json::to_string(&shake).unwrap();
        let deserialized: CharacterAnimation = serde_json::from_str(&json).unwrap();
        assert_eq!(shake, deserialized);
    }

    #[test]
    fn test_animation_serde_custom_intensity() {
        let shake = CharacterAnimation::Shake {
            intensity: AnimationIntensity::Custom {
                amplitude: 12.5,
                count: 4,
            },
            timing: AnimationTiming::Duration { duration_secs: 0.8 },
        };

        let json = serde_json::to_string(&shake).unwrap();
        let deserialized: CharacterAnimation = serde_json::from_str(&json).unwrap();
        assert_eq!(shake, deserialized);
    }

    #[test]
    fn test_animation_serde_toml_preset() {
        let toml_str = r#"
type = "shake"
intensity = "large"
timing = { mode = "auto" }
"#;
        let animation: CharacterAnimation = toml::from_str(toml_str).unwrap();
        assert_eq!(animation.intensity().unwrap().amplitude(), 20.0);
    }

    #[test]
    fn test_animation_serde_toml_custom() {
        let toml_str = r#"
type = "jump"
intensity = { amplitude = 15.0, count = 5 }
timing = { mode = "duration", duration_secs = 1.2 }
"#;
        let animation: CharacterAnimation = toml::from_str(toml_str).unwrap();
        assert_eq!(animation.intensity().unwrap().amplitude(), 15.0);
        assert_eq!(animation.intensity().unwrap().count(), 5);
    }

    #[test]
    fn test_animation_serde_toml_continuous() {
        let toml_str = r#"
type = "tremble"
intensity = "small"
timing = { mode = "continuous" }
"#;
        let animation: CharacterAnimation = toml::from_str(toml_str).unwrap();
        assert!(animation.timing().unwrap().is_continuous());
    }

    #[test]
    fn test_animation_types_distinct() {
        let shake = CharacterAnimation::shake();
        let jump = CharacterAnimation::jump();
        let tremble = CharacterAnimation::tremble();

        assert_ne!(shake, jump);
        assert_ne!(jump, tremble);
        assert_ne!(tremble, shake);
    }

    #[test]
    fn test_escape_animation_toml_deserialization() {
        let toml_str = r#"
type = "escape"
direction = "right"
preset = "small"
"#;
        let animation: CharacterAnimation = toml::from_str(toml_str).unwrap();
        assert!(animation.is_keyframe_based());
        assert!(animation.is_active());
    }

    #[test]
    fn test_faint_animation_toml_deserialization() {
        let toml_str = r#"
type = "faint"
preset = "medium"
"#;
        let animation: CharacterAnimation = toml::from_str(toml_str).unwrap();
        assert!(animation.is_keyframe_based());
        assert!(animation.is_active());
    }

    #[test]
    fn test_escape_animation_keyframe_methods() {
        let animation = CharacterAnimation::escape(
            super::EscapeDirection::Right,
            super::EscapePreset::Medium,
        );

        assert!(animation.is_keyframe_based());
        assert_eq!(animation.keyframe_duration(), Some(0.5));

        let transform = animation.keyframe_transform_at(0.0).unwrap();
        assert_eq!(transform.x, 0.0);
        assert_eq!(transform.y, 0.0);
    }

    #[test]
    fn test_faint_animation_keyframe_methods() {
        let animation = CharacterAnimation::faint(super::FaintPreset::Small);

        assert!(animation.is_keyframe_based());
        let duration = animation.keyframe_duration().unwrap();
        assert!((duration - 0.5).abs() < 0.001);

        let transform = animation.keyframe_transform_at(0.0).unwrap();
        assert_eq!(transform.x, 0.0);
        assert_eq!(transform.y, 0.0);
    }
}
