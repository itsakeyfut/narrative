//! Property animation wrapper with callbacks

use std::time::Duration;

use super::context::AnimationContext;
use super::core::{Animation, AnimationState, Interpolate};
use super::easing::Easing;

/// Property animation with optional callbacks
///
/// This is a wrapper around Animation<T> that adds callback support
/// for integration with UI components.
pub struct PropertyAnimation<T: Interpolate> {
    animation: Animation<T>,
    #[allow(clippy::type_complexity)]
    on_update: Option<Box<dyn Fn(&T) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_complete: Option<Box<dyn Fn() + Send + Sync>>,
}

impl<T: Interpolate> PropertyAnimation<T> {
    /// Create a new property animation
    pub fn new(start: T, end: T, duration: Duration, easing: Easing) -> Self {
        Self {
            animation: Animation::new(start, end, duration, easing),
            on_update: None,
            on_complete: None,
        }
    }

    /// Create a new property animation with animation context
    ///
    /// This method creates an animation with context-aware duration adjustment.
    /// If animations are disabled or the component is disabled, the duration
    /// will be set to `Duration::ZERO`, causing the animation to complete instantly.
    ///
    /// # Arguments
    ///
    /// * `start` - Starting value
    /// * `end` - Ending value
    /// * `duration` - Base animation duration
    /// * `easing` - Easing function
    /// * `context` - Animation context with global settings
    /// * `component_enabled` - Component-specific override (None = follow global)
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use narrative_gui::framework::animation::{AnimationContext, PropertyAnimation, Easing};
    ///
    /// let ctx = AnimationContext::new(true, 1.0);
    /// let anim = PropertyAnimation::new_with_context(
    ///     0.0_f32,
    ///     100.0,
    ///     Duration::from_millis(200),
    ///     Easing::Linear,
    ///     &ctx,
    ///     None,
    /// );
    /// ```
    pub fn new_with_context(
        start: T,
        end: T,
        duration: Duration,
        easing: Easing,
        context: &AnimationContext,
        component_enabled: Option<bool>,
    ) -> Self {
        let effective_duration = context.adjust_duration(duration, component_enabled);

        Self {
            animation: Animation::new(start, end, effective_duration, easing),
            on_update: None,
            on_complete: None,
        }
    }

    /// Check if this animation is instant (duration is zero)
    ///
    /// Returns `true` if the animation has zero duration, meaning it will
    /// complete immediately without interpolation. This typically occurs
    /// when animations are disabled via `AnimationContext`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use narrative_gui::framework::animation::{AnimationContext, PropertyAnimation, Easing};
    ///
    /// let ctx = AnimationContext::disabled();
    /// let anim = PropertyAnimation::new_with_context(
    ///     0.0_f32,
    ///     100.0,
    ///     Duration::from_millis(200),
    ///     Easing::Linear,
    ///     &ctx,
    ///     None,
    /// );
    ///
    /// assert!(anim.is_instant());
    /// ```
    pub fn is_instant(&self) -> bool {
        self.animation.duration().is_zero()
    }

    /// Set the update callback
    ///
    /// This callback is called every time tick() is called while the animation is playing.
    pub fn with_on_update<F>(mut self, callback: F) -> Self
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.on_update = Some(Box::new(callback));
        self
    }

    /// Set the completion callback
    ///
    /// This callback is called once when the animation completes.
    pub fn with_on_complete<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_complete = Some(Box::new(callback));
        self
    }

    /// Update the animation by delta time
    ///
    /// Returns true if the element needs to be repainted.
    /// Automatically calls update and complete callbacks.
    pub fn tick(&mut self, delta: Duration) -> bool {
        let was_completed = self.animation.is_completed();

        let needs_redraw = self.animation.tick(delta);

        if needs_redraw {
            // Call update callback with current value
            if let Some(callback) = &self.on_update {
                callback(&self.animation.current_value());
            }
        }

        // Call complete callback if just completed
        if !was_completed
            && self.animation.is_completed()
            && let Some(callback) = &self.on_complete
        {
            callback();
        }

        needs_redraw
    }

    /// Get the current interpolated value
    pub fn current_value(&self) -> T {
        self.animation.current_value()
    }

    /// Reset animation to beginning
    pub fn reset(&mut self) {
        self.animation.reset();
    }

    /// Reverse the animation
    pub fn reverse(&mut self) {
        self.animation.reverse();
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        self.animation.pause();
    }

    /// Resume the animation
    pub fn resume(&mut self) {
        self.animation.resume();
    }

    /// Check if animation is completed
    pub fn is_completed(&self) -> bool {
        self.animation.is_completed();
        self.animation.is_completed()
    }

    /// Check if animation is playing
    pub fn is_playing(&self) -> bool {
        self.animation.is_playing()
    }

    /// Get animation state
    pub fn state(&self) -> AnimationState {
        self.animation.state()
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        self.animation.progress()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_property_animation_new() {
        let anim =
            PropertyAnimation::new(0.0_f32, 100.0, Duration::from_millis(100), Easing::Linear);
        assert_eq!(anim.current_value(), 0.0);
        assert!(anim.is_playing());
    }

    #[test]
    fn test_property_animation_tick() {
        let mut anim =
            PropertyAnimation::new(0.0_f32, 100.0, Duration::from_millis(100), Easing::Linear);

        anim.tick(Duration::from_millis(50));
        assert!((anim.current_value() - 50.0).abs() < 0.01);

        anim.tick(Duration::from_millis(50));
        assert_eq!(anim.current_value(), 100.0);
        assert!(anim.is_completed());
    }

    #[test]
    fn test_on_update_callback() {
        let updates = Arc::new(Mutex::new(Vec::new()));
        let updates_clone = Arc::clone(&updates);

        let mut anim =
            PropertyAnimation::new(0.0_f32, 100.0, Duration::from_millis(100), Easing::Linear)
                .with_on_update(move |value| {
                    updates_clone.lock().unwrap().push(*value);
                });

        anim.tick(Duration::from_millis(25));
        anim.tick(Duration::from_millis(25));
        anim.tick(Duration::from_millis(50));

        let updates = updates.lock().unwrap();
        assert_eq!(updates.len(), 3);
        // Values should be approximately 25, 50, 100
        assert!((updates[0] - 25.0).abs() < 1.0);
        assert!((updates[1] - 50.0).abs() < 1.0);
        assert!((updates[2] - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_on_complete_callback() {
        let completed = Arc::new(Mutex::new(false));
        let completed_clone = Arc::clone(&completed);

        let mut anim =
            PropertyAnimation::new(0.0_f32, 100.0, Duration::from_millis(100), Easing::Linear)
                .with_on_complete(move || {
                    *completed_clone.lock().unwrap() = true;
                });

        anim.tick(Duration::from_millis(50));
        assert!(!*completed.lock().unwrap());

        anim.tick(Duration::from_millis(50));
        assert!(*completed.lock().unwrap());
    }

    #[test]
    fn test_reset_and_replay() {
        let mut anim =
            PropertyAnimation::new(0.0_f32, 100.0, Duration::from_millis(100), Easing::Linear);

        anim.tick(Duration::from_millis(100));
        assert!(anim.is_completed());

        anim.reset();
        assert!(anim.is_playing());
        assert_eq!(anim.current_value(), 0.0);
    }

    #[test]
    fn test_new_with_context_enabled() {
        let ctx = AnimationContext::new(true, 1.0);
        let anim = PropertyAnimation::new_with_context(
            0.0_f32,
            100.0,
            Duration::from_millis(200),
            Easing::Linear,
            &ctx,
            None,
        );

        assert!(!anim.is_instant());
        assert_eq!(anim.current_value(), 0.0);
        assert!(anim.is_playing());
    }

    #[test]
    fn test_new_with_context_disabled() {
        let ctx = AnimationContext::disabled();
        let mut anim = PropertyAnimation::new_with_context(
            0.0_f32,
            100.0,
            Duration::from_millis(200),
            Easing::Linear,
            &ctx,
            None,
        );

        assert!(anim.is_instant());
        // With zero duration, current_value should jump to end immediately
        assert_eq!(anim.current_value(), 100.0);
        // But needs tick() to transition to Completed state
        anim.tick(Duration::ZERO);
        assert!(anim.is_completed());
    }

    #[test]
    fn test_new_with_context_component_override() {
        let ctx = AnimationContext::new(true, 1.0);
        let mut anim = PropertyAnimation::new_with_context(
            0.0_f32,
            100.0,
            Duration::from_millis(200),
            Easing::Linear,
            &ctx,
            Some(false), // Component explicitly disabled
        );

        assert!(anim.is_instant());
        assert_eq!(anim.current_value(), 100.0);
        // But needs tick() to transition to Completed state
        anim.tick(Duration::ZERO);
        assert!(anim.is_completed());
    }

    #[test]
    fn test_new_with_context_speed_multiplier() {
        let ctx = AnimationContext::new(true, 2.0); // 2x speed = half duration
        let anim = PropertyAnimation::new_with_context(
            0.0_f32,
            100.0,
            Duration::from_millis(200),
            Easing::Linear,
            &ctx,
            None,
        );

        assert!(!anim.is_instant());
        // Animation should be faster (half duration)
    }

    #[test]
    fn test_is_instant() {
        let normal =
            PropertyAnimation::new(0.0_f32, 100.0, Duration::from_millis(200), Easing::Linear);
        assert!(!normal.is_instant());

        let instant = PropertyAnimation::new(0.0_f32, 100.0, Duration::ZERO, Easing::Linear);
        assert!(instant.is_instant());
    }
}
