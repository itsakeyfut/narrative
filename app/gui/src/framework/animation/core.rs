//! Core animation types and interpolation trait

use std::time::Duration;

use super::easing::Easing;

/// Trait for types that can be interpolated (linearly interpolated/lerp)
pub trait Interpolate: Clone + Send + Sync + 'static {
    /// Linear interpolation between self and other
    /// t should be in range [0.0, 1.0]
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

/// Interpolate implementation for f32
impl Interpolate for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

/// Interpolate implementation for f64
impl Interpolate for f64 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }
}

/// Animation state lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    /// Animation has not started yet
    Pending,
    /// Animation is currently playing
    Playing,
    /// Animation is paused
    Paused,
    /// Animation has completed
    Completed,
}

/// Generic animation for any interpolatable type
pub struct Animation<T: Interpolate> {
    start_value: T,
    end_value: T,
    duration: Duration,
    easing: Easing,
    elapsed: Duration,
    state: AnimationState,
}

impl<T: Interpolate> Animation<T> {
    /// Create a new animation
    pub fn new(start: T, end: T, duration: Duration, easing: Easing) -> Self {
        Self {
            start_value: start,
            end_value: end,
            duration,
            easing,
            elapsed: Duration::ZERO,
            state: AnimationState::Playing,
        }
    }

    /// Update animation by delta time
    /// Returns true if animation needs redraw
    pub fn tick(&mut self, delta: Duration) -> bool {
        if self.state != AnimationState::Playing {
            return false;
        }

        self.elapsed = self.elapsed.saturating_add(delta);

        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.state = AnimationState::Completed;
        }

        true
    }

    /// Get the current interpolated value
    pub fn current_value(&self) -> T {
        if self.duration.is_zero() {
            return self.end_value.clone();
        }

        let progress = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let progress = progress.clamp(0.0, 1.0);
        let eased_progress = self.easing.apply(progress);

        self.start_value.lerp(&self.end_value, eased_progress)
    }

    /// Reset animation to beginning
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.state = AnimationState::Playing;
    }

    /// Reverse the animation (swap start and end)
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.start_value, &mut self.end_value);
        self.elapsed = Duration::ZERO;
        self.state = AnimationState::Playing;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.state == AnimationState::Playing {
            self.state = AnimationState::Paused;
        }
    }

    /// Resume the animation
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Playing;
        }
    }

    /// Check if animation is completed
    pub fn is_completed(&self) -> bool {
        self.state == AnimationState::Completed
    }

    /// Check if animation is playing
    pub fn is_playing(&self) -> bool {
        self.state == AnimationState::Playing
    }

    /// Get animation state
    pub fn state(&self) -> AnimationState {
        self.state
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0)
    }

    /// Get the animation duration
    ///
    /// This method returns the total duration of the animation.
    /// A duration of `Duration::ZERO` indicates an instant animation
    /// that completes immediately without interpolation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use narrative_gui::framework::animation::{Animation, Easing};
    ///
    /// let anim = Animation::new(0.0_f32, 100.0, Duration::from_millis(200), Easing::Linear);
    /// assert_eq!(anim.duration(), Duration::from_millis(200));
    /// ```
    pub fn duration(&self) -> Duration {
        self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_interpolate() {
        let start = 0.0_f32;
        let end = 100.0_f32;

        assert_eq!(start.lerp(&end, 0.0), 0.0);
        assert_eq!(start.lerp(&end, 0.5), 50.0);
        assert_eq!(start.lerp(&end, 1.0), 100.0);
    }

    #[test]
    fn test_animation_new() {
        let anim = Animation::new(0.0_f32, 100.0, Duration::from_millis(100), Easing::Linear);
        assert_eq!(anim.state(), AnimationState::Playing);
        assert_eq!(anim.current_value(), 0.0);
    }

    #[test]
    fn test_animation_tick() {
        let mut anim = Animation::new(0.0_f32, 100.0, Duration::from_millis(100), Easing::Linear);

        // At 0ms
        assert_eq!(anim.current_value(), 0.0);
        assert!(!anim.is_completed());

        // At 50ms (halfway)
        anim.tick(Duration::from_millis(50));
        assert!((anim.current_value() - 50.0).abs() < 0.01);
        assert!(!anim.is_completed());

        // At 100ms (complete)
        anim.tick(Duration::from_millis(50));
        assert_eq!(anim.current_value(), 100.0);
        assert!(anim.is_completed());

        // Further ticks don't change value
        anim.tick(Duration::from_millis(50));
        assert_eq!(anim.current_value(), 100.0);
    }

    #[test]
    fn test_animation_reset() {
        let mut anim = Animation::new(0.0_f32, 100.0, Duration::from_millis(100), Easing::Linear);
        anim.tick(Duration::from_millis(100));
        assert!(anim.is_completed());

        anim.reset();
        assert!(anim.is_playing());
        assert_eq!(anim.current_value(), 0.0);
    }

    #[test]
    fn test_animation_reverse() {
        let mut anim = Animation::new(0.0_f32, 100.0, Duration::from_millis(100), Easing::Linear);
        anim.tick(Duration::from_millis(50));

        anim.reverse();
        assert_eq!(anim.current_value(), 100.0); // Now starts at 100
        assert!(anim.is_playing());
    }

    #[test]
    fn test_animation_pause_resume() {
        let mut anim = Animation::new(0.0_f32, 100.0, Duration::from_millis(100), Easing::Linear);
        anim.tick(Duration::from_millis(50));

        anim.pause();
        assert_eq!(anim.state(), AnimationState::Paused);

        // Tick during pause doesn't advance
        let value_before = anim.current_value();
        anim.tick(Duration::from_millis(10));
        assert_eq!(anim.current_value(), value_before);

        anim.resume();
        assert!(anim.is_playing());
    }

    #[test]
    fn test_animation_duration() {
        let anim = Animation::new(0.0_f32, 100.0, Duration::from_millis(200), Easing::Linear);
        assert_eq!(anim.duration(), Duration::from_millis(200));

        let instant = Animation::new(0.0_f32, 100.0, Duration::ZERO, Easing::Linear);
        assert_eq!(instant.duration(), Duration::ZERO);
    }
}
