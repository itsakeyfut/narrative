//! Easing functions for animations
//!
//! Provides a comprehensive library of easing functions for smooth animations.
//! All easing functions take a normalized time value (0.0 to 1.0) and return
//! an eased value (also typically 0.0 to 1.0, though some may overshoot).

use std::f32::consts::PI;

/// Easing function types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    /// No easing - constant speed
    Linear,

    // Quadratic easing (power of 2)
    QuadIn,
    QuadOut,
    QuadInOut,

    // Cubic easing (power of 3)
    CubicIn,
    CubicOut,
    CubicInOut,

    // Quartic easing (power of 4)
    QuartIn,
    QuartOut,
    QuartInOut,

    // Sine easing
    SineIn,
    SineOut,
    SineInOut,

    // Exponential easing
    ExpoIn,
    ExpoOut,
    ExpoInOut,

    // Back easing (overshoots then returns)
    BackIn,
    BackOut,
    BackInOut,

    // Elastic easing (spring-like oscillation)
    ElasticIn,
    ElasticOut,
    ElasticInOut,

    // Bounce easing (bouncing ball effect)
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl Easing {
    /// Apply the easing function to a normalized time value
    ///
    /// # Parameters
    /// * `t` - Progress from 0.0 to 1.0
    ///
    /// # Returns
    /// Eased progress value (typically 0.0 to 1.0, but some functions may overshoot)
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => linear(t),

            Self::QuadIn => quad_in(t),
            Self::QuadOut => quad_out(t),
            Self::QuadInOut => quad_in_out(t),

            Self::CubicIn => cubic_in(t),
            Self::CubicOut => cubic_out(t),
            Self::CubicInOut => cubic_in_out(t),

            Self::QuartIn => quart_in(t),
            Self::QuartOut => quart_out(t),
            Self::QuartInOut => quart_in_out(t),

            Self::SineIn => sine_in(t),
            Self::SineOut => sine_out(t),
            Self::SineInOut => sine_in_out(t),

            Self::ExpoIn => expo_in(t),
            Self::ExpoOut => expo_out(t),
            Self::ExpoInOut => expo_in_out(t),

            Self::BackIn => back_in(t),
            Self::BackOut => back_out(t),
            Self::BackInOut => back_in_out(t),

            Self::ElasticIn => elastic_in(t),
            Self::ElasticOut => elastic_out(t),
            Self::ElasticInOut => elastic_in_out(t),

            Self::BounceIn => bounce_in(t),
            Self::BounceOut => bounce_out(t),
            Self::BounceInOut => bounce_in_out(t),
        }
    }
}

// Linear easing

pub fn linear(t: f32) -> f32 {
    t
}

// Quadratic easing (power of 2)

pub fn quad_in(t: f32) -> f32 {
    t * t
}

pub fn quad_out(t: f32) -> f32 {
    t * (2.0 - t)
}

pub fn quad_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

// Cubic easing (power of 3)

pub fn cubic_in(t: f32) -> f32 {
    t * t * t
}

pub fn cubic_out(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}

pub fn cubic_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t = 2.0 * t - 2.0;
        1.0 + t * t * t / 2.0
    }
}

// Quartic easing (power of 4)

pub fn quart_in(t: f32) -> f32 {
    t * t * t * t
}

pub fn quart_out(t: f32) -> f32 {
    let t = t - 1.0;
    1.0 - t * t * t * t
}

pub fn quart_in_out(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        let t = t - 1.0;
        1.0 - 8.0 * t * t * t * t
    }
}

// Sine easing

pub fn sine_in(t: f32) -> f32 {
    1.0 - (t * PI / 2.0).cos()
}

pub fn sine_out(t: f32) -> f32 {
    (t * PI / 2.0).sin()
}

pub fn sine_in_out(t: f32) -> f32 {
    -(PI * t).cos() / 2.0 + 0.5
}

// Exponential easing

pub fn expo_in(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else {
        2.0_f32.powf(10.0 * (t - 1.0))
    }
}

pub fn expo_out(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0_f32.powf(-10.0 * t)
    }
}

pub fn expo_in_out(t: f32) -> f32 {
    if t == 0.0 {
        return 0.0;
    }
    if t == 1.0 {
        return 1.0;
    }

    if t < 0.5 {
        2.0_f32.powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
    }
}

// Back easing (overshoots and returns)

const BACK_CONSTANT: f32 = 1.70158;

pub fn back_in(t: f32) -> f32 {
    let c = BACK_CONSTANT;
    (c + 1.0) * t * t * t - c * t * t
}

pub fn back_out(t: f32) -> f32 {
    let c = BACK_CONSTANT;
    let t = t - 1.0;
    1.0 + (c + 1.0) * t * t * t + c * t * t
}

pub fn back_in_out(t: f32) -> f32 {
    let c = BACK_CONSTANT * 1.525;

    if t < 0.5 {
        let t = 2.0 * t;
        t * t * ((c + 1.0) * t - c) / 2.0
    } else {
        let t = 2.0 * t - 2.0;
        (t * t * ((c + 1.0) * t + c) + 2.0) / 2.0
    }
}

// Elastic easing (spring-like oscillation)

pub fn elastic_in(t: f32) -> f32 {
    if t == 0.0 {
        return 0.0;
    }
    if t == 1.0 {
        return 1.0;
    }

    let c = (2.0 * PI) / 3.0;
    -2.0_f32.powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * c).sin()
}

pub fn elastic_out(t: f32) -> f32 {
    if t == 0.0 {
        return 0.0;
    }
    if t == 1.0 {
        return 1.0;
    }

    let c = (2.0 * PI) / 3.0;
    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c).sin() + 1.0
}

pub fn elastic_in_out(t: f32) -> f32 {
    if t == 0.0 {
        return 0.0;
    }
    if t == 1.0 {
        return 1.0;
    }

    let c = (2.0 * PI) / 4.5;

    if t < 0.5 {
        -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c).sin()) / 2.0
    } else {
        2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c).sin() / 2.0 + 1.0
    }
}

// Bounce easing (bouncing ball effect)

pub fn bounce_out(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;

    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

pub fn bounce_in(t: f32) -> f32 {
    1.0 - bounce_out(1.0 - t)
}

pub fn bounce_in_out(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        assert_eq!(linear(0.0), 0.0);
        assert_eq!(linear(0.5), 0.5);
        assert_eq!(linear(1.0), 1.0);
    }

    #[test]
    fn test_quad_in() {
        assert_eq!(quad_in(0.0), 0.0);
        assert_eq!(quad_in(1.0), 1.0);
        // Quad in should be slower at start (below linear)
        assert!(quad_in(0.5) < 0.5);
    }

    #[test]
    fn test_quad_out() {
        assert_eq!(quad_out(0.0), 0.0);
        assert_eq!(quad_out(1.0), 1.0);
        // Quad out should be faster at start (above linear)
        assert!(quad_out(0.5) > 0.5);
    }

    #[test]
    fn test_cubic_in() {
        assert_eq!(cubic_in(0.0), 0.0);
        assert_eq!(cubic_in(1.0), 1.0);
        assert!(cubic_in(0.5) < 0.5);
    }

    #[test]
    fn test_sine_in() {
        assert_eq!(sine_in(0.0), 0.0);
        assert_eq!(sine_in(1.0), 1.0);
    }

    #[test]
    fn test_expo_in() {
        assert_eq!(expo_in(0.0), 0.0);
        assert_eq!(expo_in(1.0), 1.0);
    }

    #[test]
    fn test_back_out() {
        assert_eq!(back_out(0.0), 0.0);
        assert_eq!(back_out(1.0), 1.0);
        // Back out should overshoot past 1.0 at some point
        let mut max: f32 = 0.0;
        for i in 0..100 {
            let t = i as f32 / 100.0;
            max = max.max(back_out(t));
        }
        assert!(max > 1.0);
    }

    #[test]
    fn test_bounce_out() {
        assert_eq!(bounce_out(0.0), 0.0);
        assert_eq!(bounce_out(1.0), 1.0);
    }

    #[test]
    fn test_easing_enum() {
        assert_eq!(Easing::Linear.apply(0.5), 0.5);
        assert_eq!(Easing::QuadIn.apply(0.0), 0.0);
        assert_eq!(Easing::QuadIn.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_clamping() {
        // All easing functions should clamp input to [0.0, 1.0]
        assert_eq!(Easing::Linear.apply(-0.5), 0.0);
        assert_eq!(Easing::Linear.apply(1.5), 1.0);
        assert_eq!(Easing::QuadOut.apply(-0.1), 0.0);
        assert_eq!(Easing::QuadOut.apply(1.1), 1.0);
    }
}
