// Faint animation - character sways and then collapses
//!
//! The faint animation simulates a character losing consciousness:
//! 1. Sway phase: Character sways back and forth (dizzy/unsteady)
//! 2. Collapse phase: Character drops down rapidly
//!
//! Presets:
//! - Small: Staggers slightly (small sway, slight drop)
//! - Medium: Kneels down (medium sway, drop to "knee" level)
//! - Large: Complete collapse (large sway, drop off bottom of screen)

use super::easing::EasingFunction;
use super::keyframe::{AnimationPhase, Keyframe, KeyframeAnimation};
use serde::{Deserialize, Serialize};

/// Preset intensity levels for faint animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaintPreset {
    /// Small faint (staggers slightly)
    Small,
    /// Medium faint (kneels down)
    Medium,
    /// Large faint (complete collapse)
    Large,
}

impl Default for FaintPreset {
    fn default() -> Self {
        Self::Medium
    }
}

impl FaintPreset {
    /// Get the sway amplitude for this preset in pixels
    fn sway_amplitude(self) -> f32 {
        match self {
            Self::Small => 10.0,
            Self::Medium => 20.0,
            Self::Large => 30.0,
        }
    }

    /// Get the collapse distance for this preset in pixels
    fn collapse_distance(self) -> f32 {
        match self {
            Self::Small => 50.0,   // Slight drop
            Self::Medium => 200.0, // Drop to knee level
            Self::Large => 400.0,  // Complete collapse
        }
    }

    /// Get the sway duration for this preset in seconds
    fn sway_duration(self) -> f32 {
        match self {
            Self::Small => 0.3,
            Self::Medium => 0.4,
            Self::Large => 0.5,
        }
    }

    /// Get the collapse duration for this preset in seconds
    fn collapse_duration(self) -> f32 {
        match self {
            Self::Small => 0.2,
            Self::Medium => 0.3,
            Self::Large => 0.4,
        }
    }

    /// Get the number of sway cycles for this preset
    fn sway_cycles(self) -> u32 {
        match self {
            Self::Small => 1,
            Self::Medium => 2,
            Self::Large => 3,
        }
    }
}

/// Faint animation configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FaintAnimation {
    /// Use a preset configuration
    Preset { preset: FaintPreset },
    /// Custom configuration
    Custom {
        /// Amplitude of the swaying motion (pixels)
        sway_amplitude: f32,
        /// Number of sway cycles before collapse
        #[serde(default = "default_sway_cycles")]
        sway_cycles: u32,
        /// Distance to drop during collapse (pixels)
        collapse_distance: f32,
        /// Duration of sway phase (seconds)
        #[serde(default = "default_sway_duration")]
        sway_duration: f32,
        /// Duration of collapse phase (seconds)
        #[serde(default = "default_collapse_duration")]
        collapse_duration: f32,
        /// Whether to add rotation during collapse
        #[serde(default)]
        rotate: bool,
    },
}

fn default_sway_cycles() -> u32 {
    2
}

fn default_sway_duration() -> f32 {
    0.4
}

fn default_collapse_duration() -> f32 {
    0.3
}

impl Default for FaintAnimation {
    fn default() -> Self {
        Self::Preset {
            preset: FaintPreset::default(),
        }
    }
}

impl FaintAnimation {
    /// Create a new faint animation with preset
    pub fn new(preset: FaintPreset) -> Self {
        Self::Preset { preset }
    }

    /// Create a custom faint animation
    pub fn custom(
        sway_amplitude: f32,
        sway_cycles: u32,
        collapse_distance: f32,
        sway_duration: f32,
        collapse_duration: f32,
        rotate: bool,
    ) -> Self {
        Self::Custom {
            sway_amplitude,
            sway_cycles,
            collapse_distance,
            sway_duration,
            collapse_duration,
            rotate,
        }
    }

    /// Get the sway amplitude
    fn sway_amplitude(&self) -> f32 {
        match self {
            Self::Preset { preset } => preset.sway_amplitude(),
            Self::Custom { sway_amplitude, .. } => *sway_amplitude,
        }
    }

    /// Get the number of sway cycles
    fn sway_cycles(&self) -> u32 {
        match self {
            Self::Preset { preset } => preset.sway_cycles(),
            Self::Custom { sway_cycles, .. } => *sway_cycles,
        }
    }

    /// Get the collapse distance
    fn collapse_distance(&self) -> f32 {
        match self {
            Self::Preset { preset } => preset.collapse_distance(),
            Self::Custom {
                collapse_distance, ..
            } => *collapse_distance,
        }
    }

    /// Get the sway duration
    fn sway_duration(&self) -> f32 {
        match self {
            Self::Preset { preset } => preset.sway_duration(),
            Self::Custom { sway_duration, .. } => *sway_duration,
        }
    }

    /// Get the collapse duration
    fn collapse_duration(&self) -> f32 {
        match self {
            Self::Preset { preset } => preset.collapse_duration(),
            Self::Custom {
                collapse_duration, ..
            } => *collapse_duration,
        }
    }

    /// Check if rotation is enabled
    fn rotate(&self) -> bool {
        match self {
            Self::Preset { .. } => false,
            Self::Custom { rotate, .. } => *rotate,
        }
    }
}

impl KeyframeAnimation for FaintAnimation {
    fn keyframes(&self) -> Vec<Keyframe> {
        let sway_amp = self.sway_amplitude();
        let collapse_dist = self.collapse_distance();
        let _rotate = self.rotate();

        vec![
            Keyframe::new(0.0, 0.0, 0.0),
            Keyframe::new(0.5, sway_amp, 0.0), // Peak of sway
            Keyframe::with_properties(
                1.0,
                0.0,
                collapse_dist,
                0.0,
                1.0,
                1.0,
            ),
        ]
    }

    fn phases(&self) -> Vec<AnimationPhase> {
        let sway_amp = self.sway_amplitude();
        let sway_dur = self.sway_duration();
        let collapse_dist = self.collapse_distance();
        let collapse_dur = self.collapse_duration();
        let rotate = self.rotate();

        // Sway phase uses multiple sub-phases for each sway cycle
        let cycles = self.sway_cycles();
        let cycle_dur = sway_dur / (cycles as f32);
        let mut phases = Vec::new();

        // Create sway cycles (oscillate left-right)
        for i in 0..cycles {
            let is_even = i % 2 == 0;
            let start_x = if i == 0 {
                0.0
            } else if is_even {
                -sway_amp
            } else {
                sway_amp
            };
            let end_x = if is_even { sway_amp } else { -sway_amp };

            phases.push(AnimationPhase::new(
                cycle_dur,
                EasingFunction::EaseInOut,
                Keyframe::new(0.0, start_x, 0.0),
                Keyframe::new(1.0, end_x, 0.0),
            ));
        }

        // Collapse phase - rapid drop
        let final_rotation = if rotate { -90.0 } else { 0.0 };
        phases.push(AnimationPhase::new(
            collapse_dur,
            EasingFunction::EaseIn,
            Keyframe::new(0.0, 0.0, 0.0),
            Keyframe::with_properties(1.0, 0.0, collapse_dist, final_rotation, 1.0, 1.0),
        ));

        phases
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faint_preset_values() {
        assert_eq!(FaintPreset::Small.sway_amplitude(), 10.0);
        assert_eq!(FaintPreset::Medium.sway_amplitude(), 20.0);
        assert_eq!(FaintPreset::Large.sway_amplitude(), 30.0);

        assert_eq!(FaintPreset::Small.collapse_distance(), 50.0);
        assert_eq!(FaintPreset::Medium.collapse_distance(), 200.0);
        assert_eq!(FaintPreset::Large.collapse_distance(), 400.0);

        assert_eq!(FaintPreset::Small.sway_cycles(), 1);
        assert_eq!(FaintPreset::Medium.sway_cycles(), 2);
        assert_eq!(FaintPreset::Large.sway_cycles(), 3);
    }

    #[test]
    fn test_faint_animation_preset() {
        let faint = FaintAnimation::new(FaintPreset::Medium);
        assert_eq!(faint.sway_amplitude(), 20.0);
        assert_eq!(faint.collapse_distance(), 200.0);
        assert_eq!(faint.sway_cycles(), 2);
        assert!(!faint.rotate());
    }

    #[test]
    fn test_faint_animation_custom() {
        let faint = FaintAnimation::custom(25.0, 3, 250.0, 0.5, 0.35, true);
        assert_eq!(faint.sway_amplitude(), 25.0);
        assert_eq!(faint.sway_cycles(), 3);
        assert_eq!(faint.collapse_distance(), 250.0);
        assert_eq!(faint.sway_duration(), 0.5);
        assert_eq!(faint.collapse_duration(), 0.35);
        assert!(faint.rotate());
    }

    #[test]
    fn test_faint_animation_phases() {
        let faint = FaintAnimation::new(FaintPreset::Medium);
        let phases = faint.phases();

        // Should have sway_cycles + 1 collapse phase
        assert_eq!(phases.len(), 3); // 2 sway cycles + 1 collapse

        // Last phase should be collapse
        let last_phase = phases.last().unwrap();
        assert_eq!(last_phase.end.y, 200.0); // Medium collapse distance
    }

    #[test]
    fn test_faint_animation_total_duration() {
        let faint = FaintAnimation::new(FaintPreset::Medium);
        let total = faint.total_duration();
        assert!((total - 0.7).abs() < 0.001); // 0.4 sway + 0.3 collapse
    }

    #[test]
    fn test_faint_animation_transform_start() {
        let faint = FaintAnimation::new(FaintPreset::Small);
        let transform = faint.transform_at_progress(0.0);
        assert_eq!(transform.x, 0.0);
        assert_eq!(transform.y, 0.0);
        assert_eq!(transform.rotation, 0.0);
    }

    #[test]
    fn test_faint_animation_transform_during_sway() {
        let faint = FaintAnimation::new(FaintPreset::Medium);
        // Progress within sway phase (before collapse)
        let transform = faint.transform_at_progress(0.3);
        // Should have some horizontal offset from swaying
        assert_ne!(transform.x, 0.0);
        // Should not have dropped yet
        assert_eq!(transform.y, 0.0);
    }

    #[test]
    fn test_faint_animation_transform_end() {
        let faint = FaintAnimation::new(FaintPreset::Small);
        let transform = faint.transform_at_progress(1.0);
        assert!((transform.y - 50.0).abs() < 0.001); // Small preset collapse distance
        assert!((transform.rotation - 0.0).abs() < 0.001); // No rotation for preset
    }

    #[test]
    fn test_faint_animation_rotation() {
        let faint = FaintAnimation::custom(20.0, 2, 200.0, 0.4, 0.3, true);
        let transform = faint.transform_at_progress(1.0);
        assert_eq!(transform.rotation, -90.0); // Should rotate when enabled
    }

    #[test]
    fn test_faint_serde_preset() {
        let faint = FaintAnimation::new(FaintPreset::Large);
        let json = serde_json::to_string(&faint).unwrap();
        let deserialized: FaintAnimation = serde_json::from_str(&json).unwrap();
        assert_eq!(faint, deserialized);
    }

    #[test]
    fn test_faint_serde_custom() {
        let faint = FaintAnimation::custom(25.0, 3, 250.0, 0.5, 0.35, true);
        let json = serde_json::to_string(&faint).unwrap();
        let deserialized: FaintAnimation = serde_json::from_str(&json).unwrap();
        assert_eq!(faint, deserialized);
    }

    #[test]
    fn test_faint_sway_cycles_create_correct_phases() {
        let faint = FaintAnimation::new(FaintPreset::Large); // 3 sway cycles
        let phases = faint.phases();

        // 3 sway cycles + 1 collapse = 4 phases
        assert_eq!(phases.len(), 4);

        // Each sway cycle should have the same duration
        let sway_cycle_duration = 0.5 / 3.0; // total sway / cycles
        for i in 0..3 {
            assert!((phases[i].duration - sway_cycle_duration).abs() < 0.001);
        }
    }
}
