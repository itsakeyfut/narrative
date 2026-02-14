//! Animation context for efficient settings propagation

use std::time::Duration;

/// Animation context for efficient settings propagation
///
/// This is a lightweight Copy type (8 bytes) that efficiently propagates
/// animation settings through the component tree without repeated file I/O.
#[derive(Debug, Clone, Copy)]
pub struct AnimationContext {
    global_enabled: bool,
    speed_multiplier: f32,
}

impl AnimationContext {
    /// Create a new animation context
    ///
    /// # Arguments
    ///
    /// * `global_enabled` - Global animation enable/disable flag
    /// * `speed_multiplier` - Speed multiplier (0.5 = half speed, 2.0 = double speed)
    ///
    /// # Examples
    ///
    /// ```
    /// use narrative_gui::framework::animation::AnimationContext;
    ///
    /// let ctx = AnimationContext::new(true, 1.0);
    /// assert!(ctx.should_animate(None));
    /// ```
    pub fn new(global_enabled: bool, speed_multiplier: f32) -> Self {
        Self {
            global_enabled,
            speed_multiplier: speed_multiplier.max(0.0),
        }
    }

    /// Create animation context from enabled flag and speed
    ///
    /// This is a convenience method for creating a context from settings values.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether animations are globally enabled
    /// * `speed` - Animation speed multiplier
    ///
    /// # Examples
    ///
    /// ```
    /// use narrative_gui::framework::animation::AnimationContext;
    ///
    /// // Typically called with values from UserSettings
    /// let ctx = AnimationContext::from_enabled_and_speed(true, 1.5);
    /// assert!(ctx.should_animate(None));
    /// ```
    pub fn from_enabled_and_speed(enabled: bool, speed: f32) -> Self {
        Self::new(enabled, speed)
    }

    /// Check if animations should play for a component
    ///
    /// # Arguments
    ///
    /// * `component_enabled` - Component-specific override (None = follow global)
    ///
    /// # Returns
    ///
    /// * `true` if animations should play
    /// * `false` if animations should be skipped
    ///
    /// # Decision Logic
    ///
    /// - If `component_enabled` is `Some(false)`, always returns `false`
    /// - If `component_enabled` is `Some(true)`, returns `global_enabled`
    /// - If `component_enabled` is `None`, returns `global_enabled`
    ///
    /// # Examples
    ///
    /// ```
    /// use narrative_gui::framework::animation::AnimationContext;
    ///
    /// let ctx = AnimationContext::new(true, 1.0);
    ///
    /// // Follow global setting
    /// assert!(ctx.should_animate(None));
    ///
    /// // Component explicitly enabled
    /// assert!(ctx.should_animate(Some(true)));
    ///
    /// // Component explicitly disabled
    /// assert!(!ctx.should_animate(Some(false)));
    /// ```
    pub fn should_animate(&self, component_enabled: Option<bool>) -> bool {
        match component_enabled {
            Some(enabled) => self.global_enabled && enabled,
            None => self.global_enabled,
        }
    }

    /// Adjust duration based on speed multiplier
    ///
    /// Returns `Duration::ZERO` if animations are disabled, otherwise
    /// returns the base duration divided by the speed multiplier.
    ///
    /// # Arguments
    ///
    /// * `base_duration` - Original animation duration
    /// * `component_enabled` - Component-specific override (None = follow global)
    ///
    /// # Returns
    ///
    /// * `Duration::ZERO` if animations disabled
    /// * Adjusted duration based on speed multiplier if enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use narrative_gui::framework::animation::AnimationContext;
    ///
    /// let ctx = AnimationContext::new(true, 2.0); // 2x speed
    /// let duration = Duration::from_millis(200);
    ///
    /// let adjusted = ctx.adjust_duration(duration, None);
    /// let expected = Duration::from_millis(100);
    /// // Allow small floating point error (within 10 microseconds)
    /// assert!((adjusted.as_micros() as i128 - expected.as_micros() as i128).abs() < 10);
    /// ```
    pub fn adjust_duration(
        &self,
        base_duration: Duration,
        component_enabled: Option<bool>,
    ) -> Duration {
        if !self.should_animate(component_enabled) {
            Duration::ZERO
        } else {
            base_duration.mul_f32(1.0 / self.speed_multiplier)
        }
    }

    /// Create a disabled animation context
    ///
    /// This is a convenience method for creating a context with animations disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use narrative_gui::framework::animation::AnimationContext;
    ///
    /// let ctx = AnimationContext::disabled();
    /// assert!(!ctx.should_animate(None));
    ///
    /// let duration = Duration::from_millis(200);
    /// assert_eq!(ctx.adjust_duration(duration, None), Duration::ZERO);
    /// ```
    pub fn disabled() -> Self {
        Self {
            global_enabled: false,
            speed_multiplier: 1.0,
        }
    }
}

impl Default for AnimationContext {
    /// Default animation context with animations enabled at normal speed
    fn default() -> Self {
        Self::new(true, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ctx = AnimationContext::new(true, 1.5);
        assert!(ctx.should_animate(None));
    }

    #[test]
    fn test_new_negative_speed() {
        // Negative speed should be clamped to 0.0
        let ctx = AnimationContext::new(true, -1.0);
        assert_eq!(ctx.speed_multiplier, 0.0);
    }

    #[test]
    fn test_from_enabled_and_speed() {
        let ctx = AnimationContext::from_enabled_and_speed(true, 1.5);
        assert!(ctx.should_animate(None));
    }

    #[test]
    fn test_should_animate_global_enabled_component_none() {
        let ctx = AnimationContext::new(true, 1.0);
        assert!(ctx.should_animate(None));
    }

    #[test]
    fn test_should_animate_global_enabled_component_true() {
        let ctx = AnimationContext::new(true, 1.0);
        assert!(ctx.should_animate(Some(true)));
    }

    #[test]
    fn test_should_animate_global_enabled_component_false() {
        let ctx = AnimationContext::new(true, 1.0);
        assert!(!ctx.should_animate(Some(false)));
    }

    #[test]
    fn test_should_animate_global_disabled_component_none() {
        let ctx = AnimationContext::new(false, 1.0);
        assert!(!ctx.should_animate(None));
    }

    #[test]
    fn test_should_animate_global_disabled_component_true() {
        let ctx = AnimationContext::new(false, 1.0);
        assert!(!ctx.should_animate(Some(true)));
    }

    #[test]
    fn test_should_animate_global_disabled_component_false() {
        let ctx = AnimationContext::new(false, 1.0);
        assert!(!ctx.should_animate(Some(false)));
    }

    #[test]
    fn test_adjust_duration_disabled() {
        let ctx = AnimationContext::disabled();
        let duration = Duration::from_millis(200);
        assert_eq!(ctx.adjust_duration(duration, None), Duration::ZERO);
    }

    #[test]
    fn test_adjust_duration_enabled_normal_speed() {
        let ctx = AnimationContext::new(true, 1.0);
        let duration = Duration::from_millis(200);
        let result = ctx.adjust_duration(duration, None);
        // Allow small floating point error (within 10 microseconds)
        assert!((result.as_micros() as i128 - duration.as_micros() as i128).abs() < 10);
    }

    #[test]
    fn test_adjust_duration_enabled_half_speed() {
        let ctx = AnimationContext::new(true, 0.5); // 0.5x = double the time
        let duration = Duration::from_millis(200);
        let expected = Duration::from_millis(400);
        let result = ctx.adjust_duration(duration, None);
        // Allow small floating point error (within 10 microseconds)
        assert!((result.as_micros() as i128 - expected.as_micros() as i128).abs() < 10);
    }

    #[test]
    fn test_adjust_duration_enabled_double_speed() {
        let ctx = AnimationContext::new(true, 2.0); // 2x = half the time
        let duration = Duration::from_millis(200);
        let expected = Duration::from_millis(100);
        let result = ctx.adjust_duration(duration, None);
        // Allow small floating point error (within 10 microseconds)
        assert!((result.as_micros() as i128 - expected.as_micros() as i128).abs() < 10);
    }

    #[test]
    fn test_adjust_duration_component_disabled() {
        let ctx = AnimationContext::new(true, 1.0);
        let duration = Duration::from_millis(200);
        assert_eq!(ctx.adjust_duration(duration, Some(false)), Duration::ZERO);
    }

    #[test]
    fn test_disabled() {
        let ctx = AnimationContext::disabled();
        assert!(!ctx.should_animate(None));
        assert!(!ctx.should_animate(Some(true)));
    }

    #[test]
    fn test_default() {
        let ctx = AnimationContext::default();
        assert!(ctx.should_animate(None));
        let duration = Duration::from_millis(200);
        let result = ctx.adjust_duration(duration, None);
        // Allow small floating point error (within 10 microseconds)
        assert!((result.as_micros() as i128 - duration.as_micros() as i128).abs() < 10);
    }
}
