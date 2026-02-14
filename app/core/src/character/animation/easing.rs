// Easing functions for animations
//!
//! Provides standard easing functions for smooth animation transitions.
//! Reference: https://easings.net/

use serde::{Deserialize, Serialize};

/// Standard easing function types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EasingFunction {
    /// Linear interpolation (constant speed)
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    EaseOut,
    /// Ease in and out (slow start and end)
    EaseInOut,
    /// Quadratic ease in
    EaseInQuad,
    /// Quadratic ease out
    EaseOutQuad,
    /// Quadratic ease in-out
    EaseInOutQuad,
    /// Cubic ease in
    EaseInCubic,
    /// Cubic ease out
    EaseOutCubic,
    /// Cubic ease in-out
    EaseInOutCubic,
    /// Quartic ease in
    EaseInQuart,
    /// Quartic ease out
    EaseOutQuart,
    /// Quartic ease in-out
    EaseInOutQuart,
    /// Bounce effect at end
    Bounce,
    /// Elastic spring effect
    Elastic,
    /// Back (overshoot) effect
    Back,
}

impl Default for EasingFunction {
    fn default() -> Self {
        Self::Linear
    }
}

impl EasingFunction {
    /// Apply the easing function to a progress value (0.0 to 1.0)
    /// Returns the eased progress value
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Linear => t,
            Self::EaseIn => self.ease_in_quad(t),
            Self::EaseOut => self.ease_out_quad(t),
            Self::EaseInOut => self.ease_in_out_quad(t),
            Self::EaseInQuad => self.ease_in_quad(t),
            Self::EaseOutQuad => self.ease_out_quad(t),
            Self::EaseInOutQuad => self.ease_in_out_quad(t),
            Self::EaseInCubic => self.ease_in_cubic(t),
            Self::EaseOutCubic => self.ease_out_cubic(t),
            Self::EaseInOutCubic => self.ease_in_out_cubic(t),
            Self::EaseInQuart => self.ease_in_quart(t),
            Self::EaseOutQuart => self.ease_out_quart(t),
            Self::EaseInOutQuart => self.ease_in_out_quart(t),
            Self::Bounce => self.ease_out_bounce(t),
            Self::Elastic => self.ease_out_elastic(t),
            Self::Back => self.ease_in_out_back(t),
        }
    }

    // Quadratic easing functions
    fn ease_in_quad(self, t: f32) -> f32 {
        t * t
    }

    fn ease_out_quad(self, t: f32) -> f32 {
        t * (2.0 - t)
    }

    fn ease_in_out_quad(self, t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }

    // Cubic easing functions
    fn ease_in_cubic(self, t: f32) -> f32 {
        t * t * t
    }

    fn ease_out_cubic(self, t: f32) -> f32 {
        let t1 = t - 1.0;
        t1 * t1 * t1 + 1.0
    }

    fn ease_in_out_cubic(self, t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            let t1 = 2.0 * t - 2.0;
            1.0 + t1 * t1 * t1 / 2.0
        }
    }

    // Quartic easing functions
    fn ease_in_quart(self, t: f32) -> f32 {
        t * t * t * t
    }

    fn ease_out_quart(self, t: f32) -> f32 {
        let t1 = t - 1.0;
        1.0 - t1 * t1 * t1 * t1
    }

    fn ease_in_out_quart(self, t: f32) -> f32 {
        if t < 0.5 {
            8.0 * t * t * t * t
        } else {
            let t1 = t - 1.0;
            1.0 - 8.0 * t1 * t1 * t1 * t1
        }
    }

    // Bounce easing
    fn ease_out_bounce(self, t: f32) -> f32 {
        const N1: f32 = 7.5625;
        const D1: f32 = 2.75;

        if t < 1.0 / D1 {
            N1 * t * t
        } else if t < 2.0 / D1 {
            let t1 = t - 1.5 / D1;
            N1 * t1 * t1 + 0.75
        } else if t < 2.5 / D1 {
            let t1 = t - 2.25 / D1;
            N1 * t1 * t1 + 0.9375
        } else {
            let t1 = t - 2.625 / D1;
            N1 * t1 * t1 + 0.984375
        }
    }

    // Elastic easing
    fn ease_out_elastic(self, t: f32) -> f32 {
        use std::f32::consts::PI;

        if t == 0.0 {
            0.0
        } else if t == 1.0 {
            1.0
        } else {
            let c4 = (2.0 * PI) / 3.0;
            2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
        }
    }

    // Back easing (overshoot)
    fn ease_in_out_back(self, t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C2: f32 = C1 * 1.525;

        if t < 0.5 {
            ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
        } else {
            let t1 = 2.0 * t - 2.0;
            (t1.powi(2) * ((C2 + 1.0) * t1 + C2) + 2.0) / 2.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_easing() {
        assert_eq!(EasingFunction::Linear.apply(0.0), 0.0);
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
        assert_eq!(EasingFunction::Linear.apply(1.0), 1.0);
    }

    #[test]
    fn test_ease_in_starts_slow() {
        let ease_in = EasingFunction::EaseIn;
        let linear_mid = 0.5;
        let ease_in_mid = ease_in.apply(0.5);

        // Ease in should be slower than linear at midpoint
        assert!(ease_in_mid < linear_mid);
    }

    #[test]
    fn test_ease_out_ends_slow() {
        let ease_out = EasingFunction::EaseOut;
        let linear_mid = 0.5;
        let ease_out_mid = ease_out.apply(0.5);

        // Ease out should be faster than linear at midpoint
        assert!(ease_out_mid > linear_mid);
    }

    #[test]
    fn test_easing_bounds() {
        let easings = [
            EasingFunction::Linear,
            EasingFunction::EaseIn,
            EasingFunction::EaseOut,
            EasingFunction::EaseInOut,
            EasingFunction::EaseInQuad,
            EasingFunction::EaseOutQuad,
            EasingFunction::EaseInOutQuad,
            EasingFunction::EaseInCubic,
            EasingFunction::EaseOutCubic,
            EasingFunction::EaseInOutCubic,
        ];

        for easing in easings {
            // All easing functions should start at 0 and end at 1
            assert!((easing.apply(0.0) - 0.0).abs() < 0.01, "{:?} should start at 0", easing);
            assert!((easing.apply(1.0) - 1.0).abs() < 0.01, "{:?} should end at 1", easing);
        }
    }

    #[test]
    fn test_bounce_easing() {
        let bounce = EasingFunction::Bounce;
        assert_eq!(bounce.apply(0.0), 0.0);
        assert!((bounce.apply(1.0) - 1.0).abs() < 0.01);

        // Bounce should have values greater than 0 in the middle
        let mid = bounce.apply(0.5);
        assert!(mid > 0.0 && mid < 1.0);
    }

    #[test]
    fn test_elastic_easing() {
        let elastic = EasingFunction::Elastic;
        assert_eq!(elastic.apply(0.0), 0.0);
        assert!((elastic.apply(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_clamp_input() {
        // Values outside [0, 1] should be clamped
        assert_eq!(EasingFunction::Linear.apply(-0.5), 0.0);
        assert_eq!(EasingFunction::Linear.apply(1.5), 1.0);
    }

    #[test]
    fn test_serde_easing_function() {
        let easing = EasingFunction::EaseInOut;
        let json = serde_json::to_string(&easing).unwrap();
        let deserialized: EasingFunction = serde_json::from_str(&json).unwrap();
        assert_eq!(easing, deserialized);
    }

    #[test]
    fn test_serde_toml_easing() {
        let toml_str = r#"easing = "ease_in_out""#;
        #[derive(Deserialize)]
        struct Test {
            easing: EasingFunction,
        }
        let test: Test = toml::from_str(toml_str).unwrap();
        assert_eq!(test.easing, EasingFunction::EaseInOut);
    }
}
