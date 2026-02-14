//! Animation system for GUI framework
//!
//! Provides a comprehensive animation system with:
//! - Generic Animation<T> for any interpolatable type
//! - Rich easing function library
//! - Property animations with callbacks
//! - Animation context for settings propagation
//! - Frame-rate independent timing

mod context;
mod core;
mod easing;
mod property;

pub use context::AnimationContext;
pub use core::{Animation, AnimationState, Interpolate};
pub use easing::Easing;
pub use property::PropertyAnimation;

use super::{
    Color,
    layout::{Bounds, Point, Size},
};

// Interpolate implementation for Color (RGBA linear interpolation)
impl Interpolate for Color {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
}

// Interpolate implementation for Point (x, y independent interpolation)
impl Interpolate for Point {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Point {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

// Interpolate implementation for Size (width, height independent interpolation)
impl Interpolate for Size {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Size {
            width: self.width + (other.width - self.width) * t,
            height: self.height + (other.height - self.height) * t,
        }
    }
}

// Interpolate implementation for Bounds (origin and size interpolation)
impl Interpolate for Bounds {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Bounds {
            origin: self.origin.lerp(&other.origin, t),
            size: self.size.lerp(&other.size, t),
        }
    }
}

// Interpolate implementation for tuples (for animating multiple properties together)
impl<T, U> Interpolate for (T, U)
where
    T: Interpolate,
    U: Interpolate,
{
    fn lerp(&self, other: &Self, t: f32) -> Self {
        (self.0.lerp(&other.0, t), self.1.lerp(&other.1, t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_color_interpolate() {
        let black = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let white = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };

        let gray = black.lerp(&white, 0.5);
        assert_eq!(gray.r, 0.5);
        assert_eq!(gray.g, 0.5);
        assert_eq!(gray.b, 0.5);
        assert_eq!(gray.a, 1.0);
    }

    #[test]
    fn test_point_interpolate() {
        let start = Point { x: 0.0, y: 0.0 };
        let end = Point { x: 100.0, y: 50.0 };

        let mid = start.lerp(&end, 0.5);
        assert_eq!(mid.x, 50.0);
        assert_eq!(mid.y, 25.0);
    }

    #[test]
    fn test_size_interpolate() {
        let small = Size {
            width: 10.0,
            height: 10.0,
        };
        let large = Size {
            width: 100.0,
            height: 100.0,
        };

        let mid = small.lerp(&large, 0.5);
        assert_eq!(mid.width, 55.0);
        assert_eq!(mid.height, 55.0);
    }

    #[test]
    fn test_bounds_interpolate() {
        let start = Bounds {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 10.0,
                height: 10.0,
            },
        };
        let end = Bounds {
            origin: Point { x: 100.0, y: 100.0 },
            size: Size {
                width: 50.0,
                height: 50.0,
            },
        };

        let mid = start.lerp(&end, 0.5);
        assert_eq!(mid.origin.x, 50.0);
        assert_eq!(mid.origin.y, 50.0);
        assert_eq!(mid.size.width, 30.0);
        assert_eq!(mid.size.height, 30.0);
    }

    #[test]
    fn test_tuple_interpolate() {
        let start = (0.0_f32, 100.0_f32);
        let end = (100.0_f32, 0.0_f32);

        let mid = start.lerp(&end, 0.5);
        assert_eq!(mid.0, 50.0);
        assert_eq!(mid.1, 50.0);
    }

    #[test]
    fn test_color_animation() {
        let black = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let white = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };

        let mut anim = Animation::new(black, white, Duration::from_millis(100), Easing::Linear);
        anim.tick(Duration::from_millis(50));

        let current = anim.current_value();
        assert!((current.r - 0.5).abs() < 0.01);
        assert!((current.g - 0.5).abs() < 0.01);
        assert!((current.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_point_animation() {
        let start = Point { x: 0.0, y: 0.0 };
        let end = Point { x: 100.0, y: 100.0 };

        let mut anim = Animation::new(start, end, Duration::from_millis(100), Easing::Linear);
        anim.tick(Duration::from_millis(50));

        let current = anim.current_value();
        assert!((current.x - 50.0).abs() < 0.01);
        assert!((current.y - 50.0).abs() < 0.01);
    }
}
