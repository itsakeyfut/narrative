// Keyframe-based animation system
//!
//! Provides a trait and types for defining complex character animations
//! using keyframes and phases.

use super::easing::EasingFunction;
use serde::{Deserialize, Serialize};

/// A single keyframe in an animation sequence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    /// Position in the animation timeline (0.0 to 1.0)
    pub time: f32,
    /// X offset in pixels (at reference resolution 1280x720)
    pub x: f32,
    /// Y offset in pixels (at reference resolution 1280x720)
    pub y: f32,
    /// Optional rotation in degrees
    #[serde(default)]
    pub rotation: f32,
    /// Optional scale factor (1.0 = normal size)
    #[serde(default = "default_scale")]
    pub scale: f32,
    /// Optional alpha/opacity (0.0 = transparent, 1.0 = opaque)
    #[serde(default = "default_alpha")]
    pub alpha: f32,
}

fn default_scale() -> f32 {
    1.0
}

fn default_alpha() -> f32 {
    1.0
}

impl Keyframe {
    /// Create a new keyframe with position offset
    pub fn new(time: f32, x: f32, y: f32) -> Self {
        Self {
            time,
            x,
            y,
            rotation: 0.0,
            scale: 1.0,
            alpha: 1.0,
        }
    }

    /// Create a keyframe with all properties
    pub fn with_properties(
        time: f32,
        x: f32,
        y: f32,
        rotation: f32,
        scale: f32,
        alpha: f32,
    ) -> Self {
        Self {
            time,
            x,
            y,
            rotation,
            scale,
            alpha,
        }
    }

    /// Interpolate between two keyframes
    pub fn interpolate(&self, other: &Self, t: f32, easing: EasingFunction) -> AnimationTransform {
        let t = t.clamp(0.0, 1.0);
        let eased_t = easing.apply(t);

        AnimationTransform {
            x: self.x + (other.x - self.x) * eased_t,
            y: self.y + (other.y - self.y) * eased_t,
            rotation: self.rotation + (other.rotation - self.rotation) * eased_t,
            scale: self.scale + (other.scale - self.scale) * eased_t,
            alpha: self.alpha + (other.alpha - self.alpha) * eased_t,
        }
    }
}

/// Animation transform at a specific point in time
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationTransform {
    /// X offset in pixels
    pub x: f32,
    /// Y offset in pixels
    pub y: f32,
    /// Rotation in degrees
    pub rotation: f32,
    /// Scale factor
    pub scale: f32,
    /// Alpha/opacity
    pub alpha: f32,
}

impl Default for AnimationTransform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale: 1.0,
            alpha: 1.0,
        }
    }
}

impl AnimationTransform {
    /// Create a new transform with just position offset
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            ..Default::default()
        }
    }

    /// Get the position offset as a tuple
    pub fn offset(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

/// A phase in a multi-phase animation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationPhase {
    /// Duration of this phase in seconds
    pub duration: f32,
    /// Easing function for this phase
    #[serde(default)]
    pub easing: EasingFunction,
    /// Start keyframe
    pub start: Keyframe,
    /// End keyframe
    pub end: Keyframe,
}

impl AnimationPhase {
    /// Create a new animation phase
    pub fn new(
        duration: f32,
        easing: EasingFunction,
        start: Keyframe,
        end: Keyframe,
    ) -> Self {
        Self {
            duration,
            easing,
            start,
            end,
        }
    }

    /// Calculate the transform for a given time within this phase
    /// time should be in range [0.0, duration]
    pub fn transform_at(&self, time: f32) -> AnimationTransform {
        if self.duration <= 0.0 {
            return AnimationTransform {
                x: self.end.x,
                y: self.end.y,
                rotation: self.end.rotation,
                scale: self.end.scale,
                alpha: self.end.alpha,
            };
        }

        let progress = (time / self.duration).clamp(0.0, 1.0);
        self.start.interpolate(&self.end, progress, self.easing)
    }
}

/// Trait for keyframe-based animations
pub trait KeyframeAnimation {
    /// Get the list of keyframes for this animation
    fn keyframes(&self) -> Vec<Keyframe>;

    /// Get the list of phases for this animation
    fn phases(&self) -> Vec<AnimationPhase>;

    /// Calculate the transform at a given progress (0.0 to 1.0)
    fn transform_at_progress(&self, progress: f32) -> AnimationTransform {
        let phases = self.phases();
        if phases.is_empty() {
            return AnimationTransform::default();
        }

        // Calculate total duration
        let total_duration: f32 = phases.iter().map(|p| p.duration).sum();
        if total_duration <= 0.0 {
            return AnimationTransform::default();
        }

        // Find which phase we're in
        let current_time = progress * total_duration;
        let mut elapsed = 0.0;

        for phase in &phases {
            if current_time <= elapsed + phase.duration {
                let phase_time = current_time - elapsed;
                return phase.transform_at(phase_time);
            }
            elapsed += phase.duration;
        }

        // If we've passed all phases, return the last keyframe
        phases
            .last()
            .map(|p| AnimationTransform {
                x: p.end.x,
                y: p.end.y,
                rotation: p.end.rotation,
                scale: p.end.scale,
                alpha: p.end.alpha,
            })
            .unwrap_or_default()
    }

    /// Get the total duration of the animation in seconds
    fn total_duration(&self) -> f32 {
        self.phases().iter().map(|p| p.duration).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_creation() {
        let kf = Keyframe::new(0.0, 10.0, 20.0);
        assert_eq!(kf.time, 0.0);
        assert_eq!(kf.x, 10.0);
        assert_eq!(kf.y, 20.0);
        assert_eq!(kf.rotation, 0.0);
        assert_eq!(kf.scale, 1.0);
        assert_eq!(kf.alpha, 1.0);
    }

    #[test]
    fn test_keyframe_interpolation() {
        let kf1 = Keyframe::new(0.0, 0.0, 0.0);
        let kf2 = Keyframe::new(1.0, 100.0, 50.0);

        let transform = kf1.interpolate(&kf2, 0.5, EasingFunction::Linear);
        assert_eq!(transform.x, 50.0);
        assert_eq!(transform.y, 25.0);
    }

    #[test]
    fn test_keyframe_interpolation_with_easing() {
        let kf1 = Keyframe::new(0.0, 0.0, 0.0);
        let kf2 = Keyframe::new(1.0, 100.0, 0.0);

        let linear = kf1.interpolate(&kf2, 0.5, EasingFunction::Linear);
        let ease_in = kf1.interpolate(&kf2, 0.5, EasingFunction::EaseIn);

        // EaseIn should be slower (less distance) than linear at t=0.5
        assert!(ease_in.x < linear.x);
    }

    #[test]
    fn test_animation_phase() {
        let start = Keyframe::new(0.0, 0.0, 0.0);
        let end = Keyframe::new(1.0, 100.0, 50.0);
        let phase = AnimationPhase::new(1.0, EasingFunction::Linear, start, end);

        let transform_start = phase.transform_at(0.0);
        assert_eq!(transform_start.x, 0.0);
        assert_eq!(transform_start.y, 0.0);

        let transform_mid = phase.transform_at(0.5);
        assert_eq!(transform_mid.x, 50.0);
        assert_eq!(transform_mid.y, 25.0);

        let transform_end = phase.transform_at(1.0);
        assert_eq!(transform_end.x, 100.0);
        assert_eq!(transform_end.y, 50.0);
    }

    #[test]
    fn test_animation_transform_default() {
        let transform = AnimationTransform::default();
        assert_eq!(transform.x, 0.0);
        assert_eq!(transform.y, 0.0);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, 1.0);
        assert_eq!(transform.alpha, 1.0);
    }

    #[test]
    fn test_animation_transform_offset() {
        let transform = AnimationTransform::new(10.0, 20.0);
        assert_eq!(transform.offset(), (10.0, 20.0));
    }

    #[test]
    fn test_keyframe_serde() {
        let kf = Keyframe::with_properties(0.5, 10.0, 20.0, 45.0, 1.2, 0.8);
        let json = serde_json::to_string(&kf).unwrap();
        let deserialized: Keyframe = serde_json::from_str(&json).unwrap();
        assert_eq!(kf, deserialized);
    }

    #[test]
    fn test_animation_phase_serde() {
        let start = Keyframe::new(0.0, 0.0, 0.0);
        let end = Keyframe::new(1.0, 100.0, 50.0);
        let phase = AnimationPhase::new(1.0, EasingFunction::EaseInOut, start, end);

        let json = serde_json::to_string(&phase).unwrap();
        let deserialized: AnimationPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(phase, deserialized);
    }

    // Mock implementation for testing KeyframeAnimation trait
    struct TestAnimation {
        phases: Vec<AnimationPhase>,
    }

    impl KeyframeAnimation for TestAnimation {
        fn keyframes(&self) -> Vec<Keyframe> {
            vec![]
        }

        fn phases(&self) -> Vec<AnimationPhase> {
            self.phases.clone()
        }
    }

    #[test]
    fn test_keyframe_animation_single_phase() {
        let start = Keyframe::new(0.0, 0.0, 0.0);
        let end = Keyframe::new(1.0, 100.0, 50.0);
        let phase = AnimationPhase::new(1.0, EasingFunction::Linear, start, end);

        let animation = TestAnimation {
            phases: vec![phase],
        };

        assert_eq!(animation.total_duration(), 1.0);

        let transform_start = animation.transform_at_progress(0.0);
        assert_eq!(transform_start.x, 0.0);

        let transform_mid = animation.transform_at_progress(0.5);
        assert_eq!(transform_mid.x, 50.0);

        let transform_end = animation.transform_at_progress(1.0);
        assert_eq!(transform_end.x, 100.0);
    }

    #[test]
    fn test_keyframe_animation_multi_phase() {
        // Phase 1: Move right slowly (0.0s to 0.5s)
        let phase1 = AnimationPhase::new(
            0.5,
            EasingFunction::Linear,
            Keyframe::new(0.0, 0.0, 0.0),
            Keyframe::new(0.5, 20.0, 0.0),
        );

        // Phase 2: Move right quickly (0.5s to 1.0s)
        let phase2 = AnimationPhase::new(
            0.5,
            EasingFunction::Linear,
            Keyframe::new(0.5, 20.0, 0.0),
            Keyframe::new(1.0, 200.0, 0.0),
        );

        let animation = TestAnimation {
            phases: vec![phase1, phase2],
        };

        assert_eq!(animation.total_duration(), 1.0);

        // At 25% progress (0.25s), we're in the middle of phase 1
        let transform_25 = animation.transform_at_progress(0.25);
        assert_eq!(transform_25.x, 10.0); // Halfway through phase 1: 20.0 * 0.5

        // At 75% progress (0.75s), we're in the middle of phase 2
        let transform_75 = animation.transform_at_progress(0.75);
        assert_eq!(transform_75.x, 110.0); // Halfway through phase 2: 20.0 + 180.0 * 0.5
    }

    #[test]
    fn test_keyframe_animation_empty_phases() {
        let animation = TestAnimation { phases: vec![] };
        assert_eq!(animation.total_duration(), 0.0);
        let transform = animation.transform_at_progress(0.5);
        assert_eq!(transform, AnimationTransform::default());
    }
}
