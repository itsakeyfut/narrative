//! Character transition system - handles animation of character appearance/disappearance
//!
//! This module provides transition animations for character sprites including:
//! - Fade in/out (opacity animation)
//! - Slide in (position animation from off-screen)
//! - Cross-fade (switching between expressions)
//! - Configurable duration and easing

use narrative_core::character::CharacterPosition;
use narrative_core::{SlideDirection, Transition, TransitionKind};
use std::time::Duration;

/// Easing function type for smooth animation curves
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingFunction {
    /// Linear interpolation (constant speed)
    Linear,
    /// Ease in (slow start, accelerate)
    EaseIn,
    /// Ease out (fast start, decelerate)
    EaseOut,
    /// Ease in-out (slow start, slow end)
    EaseInOut,
}

impl EasingFunction {
    /// Apply easing function to a linear progress value (0.0-1.0)
    /// Returns the eased progress value (0.0-1.0)
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => t * (2.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
        }
    }
}

/// Active transition state for a character
#[derive(Debug, Clone)]
pub struct CharacterTransitionState {
    /// The transition configuration
    transition: Transition,
    /// Elapsed time since transition started
    elapsed: Duration,
    /// Easing function to use
    easing: EasingFunction,
    /// Transition-specific data
    data: TransitionData,
}

/// Transition-specific data
#[derive(Debug, Clone)]
enum TransitionData {
    /// Fade in/out transition (animates opacity)
    Fade {
        /// Starting opacity
        from_opacity: f32,
        /// Target opacity
        to_opacity: f32,
    },
    /// Slide in transition (animates position)
    Slide {
        /// Slide direction
        direction: SlideDirection,
        /// Target position after slide completes
        ///
        /// NOTE: This field is currently only used for documentation purposes.
        /// The actual character position is managed by the CharacterSpriteElement.
        /// Kept for potential future use in position validation or state tracking.
        #[allow(dead_code)] // Intentionally kept for future extensibility
        target_position: CharacterPosition,
        /// Starting offset multiplier (screen widths/heights)
        offset: f32,
    },
    /// Crossfade between two textures (for expression changes)
    Crossfade {
        /// Old texture ID
        old_texture_id: u64,
        /// New texture ID
        new_texture_id: u64,
    },
    /// Move transition (animates position between two CharacterPosition values)
    Move {
        /// Starting position
        from_position: CharacterPosition,
        /// Target position
        to_position: CharacterPosition,
    },
}

impl CharacterTransitionState {
    /// Create a fade in transition
    pub fn fade_in(transition: Transition) -> Self {
        Self {
            transition,
            elapsed: Duration::ZERO,
            easing: EasingFunction::EaseOut,
            data: TransitionData::Fade {
                from_opacity: 0.0,
                to_opacity: 1.0,
            },
        }
    }

    /// Create a fade out transition
    pub fn fade_out(transition: Transition) -> Self {
        Self {
            transition,
            elapsed: Duration::ZERO,
            easing: EasingFunction::EaseIn,
            data: TransitionData::Fade {
                from_opacity: 1.0,
                to_opacity: 0.0,
            },
        }
    }

    /// Create a slide in transition
    pub fn slide_in(
        transition: Transition,
        direction: SlideDirection,
        target_position: CharacterPosition,
    ) -> Self {
        Self {
            transition,
            elapsed: Duration::ZERO,
            easing: EasingFunction::EaseOut,
            data: TransitionData::Slide {
                direction,
                target_position,
                offset: 1.0, // Start one screen width/height away
            },
        }
    }

    /// Create a crossfade transition between two textures
    pub fn crossfade(transition: Transition, old_texture_id: u64, new_texture_id: u64) -> Self {
        Self {
            transition,
            elapsed: Duration::ZERO,
            easing: EasingFunction::Linear,
            data: TransitionData::Crossfade {
                old_texture_id,
                new_texture_id,
            },
        }
    }

    /// Create a move transition between two positions
    pub fn move_to(
        transition: Transition,
        from_position: CharacterPosition,
        to_position: CharacterPosition,
    ) -> Self {
        Self {
            transition,
            elapsed: Duration::ZERO,
            easing: EasingFunction::EaseOut,
            data: TransitionData::Move {
                from_position,
                to_position,
            },
        }
    }

    /// Update the transition state with elapsed time
    /// Returns true if the transition is complete
    pub fn update(&mut self, delta: Duration) -> bool {
        self.elapsed = self.elapsed.saturating_add(delta);
        self.is_complete()
    }

    /// Check if the transition is complete
    pub fn is_complete(&self) -> bool {
        if self.transition.duration < 0.0 {
            tracing::warn!(
                "Negative transition duration detected: {}. Treating as complete.",
                self.transition.duration
            );
            return true;
        }
        self.elapsed.as_secs_f32() >= self.transition.duration
    }

    /// Get the current progress (0.0 to 1.0)
    fn progress(&self) -> f32 {
        if self.transition.duration < 0.0 {
            tracing::warn!(
                "Negative transition duration detected: {}. Returning 1.0 progress.",
                self.transition.duration
            );
            return 1.0;
        }
        if self.transition.duration == 0.0 {
            return 1.0;
        }
        (self.elapsed.as_secs_f32() / self.transition.duration).min(1.0)
    }

    /// Get the eased progress (0.0 to 1.0)
    fn eased_progress(&self) -> f32 {
        self.easing.apply(self.progress())
    }

    /// Calculate the current opacity based on transition progress
    pub fn current_opacity(&self, base_opacity: f32) -> f32 {
        match &self.data {
            TransitionData::Fade {
                from_opacity,
                to_opacity,
            } => {
                let progress = self.eased_progress();
                lerp(*from_opacity, *to_opacity, progress)
            }
            TransitionData::Crossfade { .. } => {
                // For crossfade, both textures are rendered at varying opacities
                // The base sprite uses the fade-out opacity
                let progress = self.eased_progress();
                lerp(1.0, 0.0, progress) * base_opacity
            }
            _ => base_opacity,
        }
    }

    /// Calculate the current position offset for slide and move transitions
    /// Returns (x_offset, y_offset) in pixels
    pub fn position_offset(&self, screen_width: f32, screen_height: f32) -> (f32, f32) {
        match &self.data {
            TransitionData::Slide {
                direction, offset, ..
            } => {
                let progress = self.eased_progress();
                let remaining = 1.0 - progress;
                let distance = offset * remaining;

                match direction {
                    SlideDirection::Left => (-screen_width * distance, 0.0),
                    SlideDirection::Right => (screen_width * distance, 0.0),
                    SlideDirection::Up => (0.0, -screen_height * distance),
                    SlideDirection::Down => (0.0, screen_height * distance),
                }
            }
            TransitionData::Move {
                from_position,
                to_position,
            } => {
                let progress = self.eased_progress();

                // Calculate x positions for from and to
                let from_x = self.calculate_position_x(*from_position, screen_width);
                let to_x = self.calculate_position_x(*to_position, screen_width);

                // Interpolate between positions
                let current_x = lerp(from_x, to_x, progress);
                let offset_x = current_x - to_x; // Offset from final position

                (offset_x, 0.0) // Only horizontal movement for now
            }
            _ => (0.0, 0.0),
        }
    }

    /// Calculate the x position in pixels for a CharacterPosition
    fn calculate_position_x(&self, position: CharacterPosition, screen_width: f32) -> f32 {
        // Reference resolution for fixed positions
        const REFERENCE_WIDTH: f32 = 1280.0;

        match position {
            CharacterPosition::Fixed(fixed_x) => {
                // Scale fixed pixel position based on screen width
                let x_scale = screen_width / REFERENCE_WIDTH;
                fixed_x * x_scale
            }
            _ => {
                // Use percentage-based positioning
                screen_width * position.x_percent()
            }
        }
    }

    /// Get crossfade texture IDs if this is a crossfade transition
    pub fn crossfade_textures(&self) -> Option<(u64, u64, f32)> {
        match &self.data {
            TransitionData::Crossfade {
                old_texture_id,
                new_texture_id,
            } => {
                let progress = self.eased_progress();
                Some((*old_texture_id, *new_texture_id, progress))
            }
            _ => None,
        }
    }

    /// Get the transition kind
    pub fn kind(&self) -> TransitionKind {
        self.transition.kind
    }
}

/// Linear interpolation between two values
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_linear() {
        let easing = EasingFunction::Linear;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.5), 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_in() {
        let easing = EasingFunction::EaseIn;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.5) < 0.5); // Slower at start
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_out() {
        let easing = EasingFunction::EaseOut;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.5) > 0.5); // Faster at start
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_clamping() {
        let easing = EasingFunction::Linear;
        assert_eq!(easing.apply(-0.5), 0.0);
        assert_eq!(easing.apply(1.5), 1.0);
    }

    #[test]
    fn test_fade_in_transition() {
        let transition = Transition::fade();
        let mut state = CharacterTransitionState::fade_in(transition);

        assert_eq!(state.current_opacity(1.0), 0.0); // Starts at 0

        state.update(Duration::from_secs_f32(0.25)); // Half way
        let mid_opacity = state.current_opacity(1.0);
        assert!(mid_opacity > 0.0 && mid_opacity < 1.0);

        state.update(Duration::from_secs_f32(0.25)); // Complete
        assert_eq!(state.current_opacity(1.0), 1.0); // Ends at 1
        assert!(state.is_complete());
    }

    #[test]
    fn test_fade_out_transition() {
        let transition = Transition::fade();
        let mut state = CharacterTransitionState::fade_out(transition);

        assert_eq!(state.current_opacity(1.0), 1.0); // Starts at 1

        state.update(Duration::from_secs_f32(0.5)); // Complete
        assert_eq!(state.current_opacity(1.0), 0.0); // Ends at 0
        assert!(state.is_complete());
    }

    #[test]
    fn test_slide_in_left() {
        let transition = Transition::new(TransitionKind::Slide(SlideDirection::Left), 0.5);
        let mut state = CharacterTransitionState::slide_in(
            transition,
            SlideDirection::Left,
            CharacterPosition::Center,
        );

        let (x_offset, y_offset) = state.position_offset(1280.0, 720.0);
        assert!(x_offset < 0.0); // Starts off-screen to the left
        assert_eq!(y_offset, 0.0);

        state.update(Duration::from_secs_f32(0.5)); // Complete
        let (x_offset, y_offset) = state.position_offset(1280.0, 720.0);
        assert_eq!(x_offset, 0.0); // Ends at target position
        assert_eq!(y_offset, 0.0);
    }

    #[test]
    fn test_slide_in_right() {
        let transition = Transition::new(TransitionKind::Slide(SlideDirection::Right), 0.5);
        let mut state = CharacterTransitionState::slide_in(
            transition,
            SlideDirection::Right,
            CharacterPosition::Center,
        );

        let (x_offset, _) = state.position_offset(1280.0, 720.0);
        assert!(x_offset > 0.0); // Starts off-screen to the right

        state.update(Duration::from_secs_f32(0.5));
        let (x_offset, _) = state.position_offset(1280.0, 720.0);
        assert_eq!(x_offset, 0.0); // Ends at target position
    }

    #[test]
    fn test_crossfade_transition() {
        let transition = Transition::crossfade();
        let state = CharacterTransitionState::crossfade(transition, 100, 200);

        let textures = state.crossfade_textures();
        assert!(textures.is_some());
        let (old_id, new_id, progress) = textures.unwrap();
        assert_eq!(old_id, 100);
        assert_eq!(new_id, 200);
        assert_eq!(progress, 0.0); // Just started
    }

    #[test]
    fn test_instant_transition() {
        let transition = Transition::instant();
        let state = CharacterTransitionState::fade_in(transition);

        assert!(state.is_complete()); // Already complete
        assert_eq!(state.current_opacity(1.0), 1.0); // Progress is 1.0
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(5.0, 15.0, 0.5), 10.0);
    }

    #[test]
    fn test_transition_progress() {
        let transition = Transition::new(TransitionKind::Fade, 1.0);
        let mut state = CharacterTransitionState::fade_in(transition);

        assert_eq!(state.progress(), 0.0);

        state.update(Duration::from_secs_f32(0.5));
        assert_eq!(state.progress(), 0.5);

        state.update(Duration::from_secs_f32(0.5));
        assert_eq!(state.progress(), 1.0);

        state.update(Duration::from_secs_f32(1.0)); // Over-update
        assert_eq!(state.progress(), 1.0); // Clamped at 1.0
    }

    #[test]
    fn test_move_transition_left_to_center() {
        let transition = Transition::new(TransitionKind::Fade, 0.5);
        let mut state = CharacterTransitionState::move_to(
            transition,
            CharacterPosition::Left,
            CharacterPosition::Center,
        );

        // At start (progress = 0), should be at Left position (25%)
        let (x_offset, y_offset) = state.position_offset(1280.0, 720.0);
        assert!(x_offset < 0.0); // Not yet at Center, offset is negative
        assert_eq!(y_offset, 0.0);

        // At completion (progress = 1), should be at Center position (50%)
        state.update(Duration::from_secs_f32(0.5));
        let (x_offset, y_offset) = state.position_offset(1280.0, 720.0);
        assert_eq!(x_offset, 0.0); // At final position, no offset
        assert_eq!(y_offset, 0.0);
        assert!(state.is_complete());
    }

    #[test]
    fn test_move_transition_center_to_right() {
        let transition = Transition::new(TransitionKind::Fade, 0.5);
        let mut state = CharacterTransitionState::move_to(
            transition,
            CharacterPosition::Center,
            CharacterPosition::Right,
        );

        // At start, should be at Center (50%), offset from Right (75%)
        let (x_offset, _) = state.position_offset(1280.0, 720.0);
        assert!(x_offset < 0.0); // Moving right, so initial offset is negative

        // Halfway through
        state.update(Duration::from_secs_f32(0.25));
        let (x_offset_mid, _) = state.position_offset(1280.0, 720.0);
        assert!(x_offset_mid < 0.0 && x_offset_mid > x_offset); // Progressing towards zero

        // Complete
        state.update(Duration::from_secs_f32(0.25));
        let (x_offset_end, _) = state.position_offset(1280.0, 720.0);
        assert_eq!(x_offset_end, 0.0); // At final position
    }

    #[test]
    fn test_move_transition_right_to_left() {
        let transition = Transition::new(TransitionKind::Fade, 0.5);
        let mut state = CharacterTransitionState::move_to(
            transition,
            CharacterPosition::Right,
            CharacterPosition::Left,
        );

        // At start (Right: 75%), moving to Left (25%)
        let (x_offset, _) = state.position_offset(1280.0, 720.0);
        assert!(x_offset > 0.0); // Moving left, so initial offset is positive

        // Complete
        state.update(Duration::from_secs_f32(0.5));
        let (x_offset, _) = state.position_offset(1280.0, 720.0);
        assert_eq!(x_offset, 0.0); // At final position
    }

    #[test]
    fn test_move_transition_fixed_positions() {
        let transition = Transition::new(TransitionKind::Fade, 0.5);
        let mut state = CharacterTransitionState::move_to(
            transition,
            CharacterPosition::Fixed(100.0),
            CharacterPosition::Fixed(300.0),
        );

        // At start, should be at 100px, target is 300px
        let (x_offset, _) = state.position_offset(1280.0, 720.0);
        assert!(x_offset < 0.0); // Not yet at target

        // Complete
        state.update(Duration::from_secs_f32(0.5));
        let (x_offset, _) = state.position_offset(1280.0, 720.0);
        assert_eq!(x_offset, 0.0); // At final position
    }

    #[test]
    fn test_move_transition_uses_ease_out() {
        let transition = Transition::new(TransitionKind::Fade, 1.0);
        let state = CharacterTransitionState::move_to(
            transition,
            CharacterPosition::Left,
            CharacterPosition::Right,
        );

        // Check that EaseOut is being used
        assert_eq!(state.easing, EasingFunction::EaseOut);
    }
}
