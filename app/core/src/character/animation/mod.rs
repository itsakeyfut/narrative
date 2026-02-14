// Character animation system module
//!
//! This module provides a comprehensive character animation system including:
//! - Emotion-based animations (shake, jump, tremble)
//! - Action-based animations (escape, faint)
//! - Keyframe-based animation infrastructure
//! - Easing functions for smooth transitions

pub mod easing;
pub mod escape;
pub mod faint;
pub mod keyframe;
pub mod types;

// Re-export public API
pub use easing::EasingFunction;
pub use escape::{EscapeAnimation, EscapeDirection, EscapePreset};
pub use faint::{FaintAnimation, FaintPreset};
pub use keyframe::{AnimationPhase, AnimationTransform, Keyframe, KeyframeAnimation};
pub use types::{
    AnimationIntensity, AnimationTiming, CharacterAnimation, IntensityPreset,
};
