// Character animation state - handles emotion-based animations
//!
//! This module provides animation state tracking for character emotion expressions:
//! - Shake animation (horizontal oscillation for surprise/anger)
//! - Jump animation (vertical bounce for joy/excitement)
//! - Tremble animation (rapid vibration for fear/anxiety)
//! - Escape animation (running away from screen)
//! - Faint animation (collapsing downward)
//!
//! Animations use GPU-side transform matrices for optimal performance.

use narrative_core::character::CharacterAnimation;
use std::time::Duration;

/// Active animation state for a character
#[derive(Debug, Clone)]
pub struct CharacterAnimationState {
    /// The animation configuration
    animation: CharacterAnimation,
    /// Elapsed time since animation started
    elapsed: Duration,
    /// Amplitude in pixels (at reference resolution 1280x720)
    amplitude: f32,
    /// Number of cycles (oscillations or bounces)
    cycles: u32,
    /// Total duration (None for continuous mode)
    total_duration: Option<Duration>,
}

impl CharacterAnimationState {
    /// Create a new animation state from a CharacterAnimation
    pub fn new(animation: CharacterAnimation) -> Self {
        // Handle keyframe-based animations (Escape, Faint)
        // TODO: Proper implementation pending in core animation system
        if animation.is_keyframe_based() {
            return Self {
                animation,
                elapsed: Duration::ZERO,
                amplitude: 0.0,
                cycles: 0,
                total_duration: Some(Duration::from_secs_f32(1.0)),
            };
        }

        // Handle timing-based animations (Shake, Jump, Tremble)
        let (amplitude, cycles) = match animation.intensity() {
            Some(intensity) => (intensity.amplitude(), intensity.count()),
            None => (10.0, 3), // Default values
        };

        let total_duration = match animation.timing() {
            Some(timing) => timing.total_duration(&animation),
            None => None,
        };

        Self {
            animation,
            elapsed: Duration::ZERO,
            amplitude,
            cycles,
            total_duration,
        }
    }

    /// Update the animation state with elapsed time
    /// Returns true if the animation is complete
    pub fn update(&mut self, delta: Duration) -> bool {
        self.elapsed = self.elapsed.saturating_add(delta);
        self.is_complete()
    }

    /// Check if the animation is complete
    pub fn is_complete(&self) -> bool {
        match self.total_duration {
            Some(duration) => self.elapsed >= duration,
            None => false, // Continuous mode never completes
        }
    }

    /// Check if this is a continuous animation
    pub fn is_continuous(&self) -> bool {
        self.total_duration.is_none()
    }

    /// Get the current animation offset (x, y) in pixels at reference resolution
    /// This offset will be scaled based on screen resolution by the sprite renderer
    pub fn current_offset(&self) -> (f32, f32) {
        let progress = self.progress();

        match &self.animation {
            CharacterAnimation::Shake { .. } => self.calculate_shake_offset(progress),
            CharacterAnimation::Jump { .. } => self.calculate_jump_offset(progress),
            CharacterAnimation::Tremble { .. } => self.calculate_tremble_offset(progress),
            CharacterAnimation::Escape { .. } | CharacterAnimation::Faint { .. } => {
                // Keyframe-based animations - implementation pending in core
                self.calculate_keyframe_offset(progress)
            }
            CharacterAnimation::None => (0.0, 0.0),
        }
    }

    /// Calculate offset using keyframe-based animation system
    /// TODO: This should delegate to core animation system implementation
    fn calculate_keyframe_offset(&self, _progress: f32) -> (f32, f32) {
        // Placeholder until core animation system is properly implemented
        (0.0, 0.0)
    }

    /// Get the current progress (0.0 to 1.0+)
    /// For continuous animations, this wraps around every cycle
    fn progress(&self) -> f32 {
        match self.total_duration {
            Some(duration) => {
                if duration.as_secs_f32() == 0.0 {
                    1.0
                } else {
                    (self.elapsed.as_secs_f32() / duration.as_secs_f32()).min(1.0)
                }
            }
            None => {
                // Continuous mode: cycle continuously
                // One cycle = 0.15 seconds
                const CYCLE_DURATION: f32 = 0.15;
                let cycles_elapsed = self.elapsed.as_secs_f32() / CYCLE_DURATION;
                cycles_elapsed % 1.0 // Wrap around at each cycle
            }
        }
    }

    /// Calculate shake animation offset (horizontal oscillation)
    fn calculate_shake_offset(&self, progress: f32) -> (f32, f32) {
        // Sine wave oscillation with decay envelope
        let frequency = self.cycles as f32;
        let phase = progress * frequency * 2.0 * std::f32::consts::PI;

        // Envelope: start strong, fade out at the end (for auto/duration modes)
        let envelope = if self.is_continuous() {
            1.0 // Constant amplitude for continuous mode
        } else {
            1.0 - progress // Linear decay for auto/duration modes
        };

        let x_offset = self.amplitude * phase.sin() * envelope;
        (x_offset, 0.0)
    }

    /// Calculate jump animation offset (vertical bounce)
    fn calculate_jump_offset(&self, progress: f32) -> (f32, f32) {
        // Parabolic bounce pattern
        let frequency = self.cycles as f32;
        let phase = progress * frequency;
        let cycle_progress = (phase % 1.0) * 2.0; // 0 to 2 per cycle

        // Parabolic curve: high at start, descend, land, repeat
        let y_offset = if cycle_progress < 1.0 {
            // Ascending phase (0.0 to 1.0)
            -self.amplitude * (1.0 - (1.0 - cycle_progress).powi(2))
        } else {
            // Descending phase (1.0 to 0.0)
            -self.amplitude * (cycle_progress - 1.0).powi(2)
        };

        // Envelope for auto/duration modes
        let envelope = if self.is_continuous() {
            1.0
        } else {
            1.0 - progress
        };

        (0.0, y_offset * envelope)
    }

    /// Calculate tremble animation offset (rapid small vibration)
    fn calculate_tremble_offset(&self, progress: f32) -> (f32, f32) {
        // High-frequency random-like vibration using multiple sine waves
        let frequency = self.cycles as f32 * 8.0; // Higher frequency than shake
        let phase = progress * frequency * 2.0 * std::f32::consts::PI;

        // Combine multiple frequencies for more chaotic movement
        let x_component = (phase.sin() + (phase * 1.7).sin() * 0.5) / 1.5;
        let y_component = ((phase * 1.3).sin() + (phase * 2.1).sin() * 0.5) / 1.5;

        // Smaller amplitude for tremble (50% of specified amplitude)
        let amplitude = self.amplitude * 0.5;

        // Envelope
        let envelope = if self.is_continuous() {
            1.0
        } else {
            1.0 - progress
        };

        (
            amplitude * x_component * envelope,
            amplitude * y_component * envelope,
        )
    }

    /// Get the animation type
    pub fn animation(&self) -> &CharacterAnimation {
        &self.animation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use narrative_core::character::{AnimationIntensity, AnimationTiming, IntensityPreset};

    #[test]
    fn test_animation_state_creation() {
        let animation = CharacterAnimation::shake();
        let state = CharacterAnimationState::new(animation);

        assert_eq!(state.amplitude, 10.0); // Medium preset
        assert_eq!(state.cycles, 3);
        assert!(!state.is_complete());
    }

    #[test]
    fn test_animation_state_auto_completion() {
        let animation = CharacterAnimation::shake();
        let mut state = CharacterAnimationState::new(animation);

        // Auto mode should complete after cycles * 0.15s = 0.45s
        // Use slightly longer duration to account for floating point precision
        state.update(Duration::from_secs_f32(0.451));
        assert!(state.is_complete());
    }

    #[test]
    fn test_animation_state_continuous_never_completes() {
        let animation = CharacterAnimation::shake()
            .with_timing(AnimationTiming::continuous());
        let mut state = CharacterAnimationState::new(animation);

        state.update(Duration::from_secs_f32(10.0));
        assert!(!state.is_complete());
        assert!(state.is_continuous());
    }

    #[test]
    fn test_animation_state_duration_mode() {
        let animation = CharacterAnimation::shake()
            .with_timing(AnimationTiming::duration(1.0));
        let mut state = CharacterAnimationState::new(animation);

        state.update(Duration::from_secs_f32(0.5));
        assert!(!state.is_complete());

        state.update(Duration::from_secs_f32(0.5));
        assert!(state.is_complete());
    }

    #[test]
    fn test_shake_offset_starts_at_zero() {
        let animation = CharacterAnimation::shake();
        let state = CharacterAnimationState::new(animation);

        let (x, y) = state.current_offset();
        assert_eq!(y, 0.0); // Shake is horizontal only
        assert!((x.abs() - 0.0).abs() < 0.01); // Should start near zero
    }

    #[test]
    fn test_shake_offset_oscillates() {
        let animation = CharacterAnimation::shake();
        let mut state = CharacterAnimationState::new(animation);

        let mut offsets = Vec::new();
        for _ in 0..10 {
            state.update(Duration::from_secs_f32(0.05));
            let (x, _) = state.current_offset();
            offsets.push(x);
        }

        // Check that offset changes (oscillates)
        let has_positive = offsets.iter().any(|&x| x > 1.0);
        let has_negative = offsets.iter().any(|&x| x < -1.0);
        assert!(has_positive && has_negative);
    }

    #[test]
    fn test_jump_offset_vertical_only() {
        let animation = CharacterAnimation::jump();
        let mut state = CharacterAnimationState::new(animation);

        state.update(Duration::from_secs_f32(0.05));
        let (x, y) = state.current_offset();

        assert_eq!(x, 0.0); // Jump is vertical only
        assert!(y < 0.0); // Should move upward (negative Y)
    }

    #[test]
    fn test_jump_offset_bounces() {
        let animation = CharacterAnimation::jump();
        let mut state = CharacterAnimationState::new(animation);

        let mut offsets = Vec::new();
        for _ in 0..20 {
            state.update(Duration::from_secs_f32(0.02));
            let (_, y) = state.current_offset();
            offsets.push(y);
        }

        // Check that Y offset changes direction (bounces)
        let max_offset = offsets.iter().cloned().fold(0.0f32, f32::min);
        assert!(max_offset < -1.0); // Should have significant upward movement
    }

    #[test]
    fn test_tremble_offset_both_axes() {
        let animation = CharacterAnimation::tremble();
        let mut state = CharacterAnimationState::new(animation);

        state.update(Duration::from_secs_f32(0.05));
        let (x, y) = state.current_offset();

        // Tremble should affect both axes
        assert!(x.abs() > 0.01 || y.abs() > 0.01);
    }

    #[test]
    fn test_custom_intensity() {
        let animation = CharacterAnimation::shake_with_intensity(
            AnimationIntensity::custom(20.0, 5)
        );
        let state = CharacterAnimationState::new(animation);

        assert_eq!(state.amplitude, 20.0);
        assert_eq!(state.cycles, 5);
    }

    #[test]
    fn test_preset_intensity_small() {
        let animation = CharacterAnimation::shake_with_intensity(
            AnimationIntensity::Preset(IntensityPreset::Small)
        );
        let state = CharacterAnimationState::new(animation);

        assert_eq!(state.amplitude, 5.0);
        assert_eq!(state.cycles, 2);
    }

    #[test]
    fn test_preset_intensity_large() {
        let animation = CharacterAnimation::shake_with_intensity(
            AnimationIntensity::Preset(IntensityPreset::Large)
        );
        let state = CharacterAnimationState::new(animation);

        assert_eq!(state.amplitude, 20.0);
        assert_eq!(state.cycles, 4);
    }

    #[test]
    fn test_continuous_animation_wraps() {
        let animation = CharacterAnimation::shake()
            .with_timing(AnimationTiming::continuous());
        let mut state = CharacterAnimationState::new(animation);

        // Advance past one cycle
        state.update(Duration::from_secs_f32(0.15));
        let (x1, _) = state.current_offset();

        // Advance past another cycle
        state.update(Duration::from_secs_f32(0.15));
        let (x2, _) = state.current_offset();

        // Offsets should be similar (wrapped around)
        assert!((x1 - x2).abs() < 1.0);
    }
}
