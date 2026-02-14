//! Narrative GUI Framework
//!
//! A custom wgpu-based GUI framework designed for visual novel applications.
//! Inspired by Zed's GPUI but tailored for interactive narrative experiences.
//!
//! ## Performance Optimizations (Issue #250)
//!
//! This framework implements several performance optimizations for 60+ FPS rendering:
//! - **Metrics System**: Frame time measurement and FPS tracking (`metrics` module)
//! - **GPU Buffer Pooling**: Reuse of GPU buffers to reduce allocation overhead
//! - **Dirty Tracking**: Only repaint elements that have changed
//! - **LRU Glyph Cache**: Efficient text atlas with least-recently-used eviction
//! - **Incremental Layout**: Only recompute layout for changed subtrees
//!
//! ## Phase 2 Optimizations (Issue #250)
//!
//! - **Frame Pacing**: VSync and adaptive sync support (`window` module)
//! - **Batched Draw Calls**: Command sorting and grouping (`renderer::batch` module)
//! - **Async Layout**: Background layout computation (`async_layout` module)
//!
//! ## Phase 3 Optimizations (Issue #250)
//!
//! - **Reactive System**: Signals/Effects for fine-grained reactivity (`reactive` module)
//! - **Render Graph**: Rendering pass optimization (`render_graph` module)

pub mod animation;
pub mod app;
pub mod async_layout;
pub mod dev_assets;
pub mod dirty;
pub mod element;
pub mod error;
pub mod input;
pub mod layout;
pub mod menu;
pub mod metrics;
pub mod reactive;
pub mod render_graph;
pub mod renderer;
pub mod window;

// Re-exports
pub use animation::{
    Animation, AnimationContext, AnimationState, Easing, Interpolate, PropertyAnimation,
};
pub use app::{App, AppContext};
pub use async_layout::{AsyncLayoutConfig, AsyncLayoutManager, LayoutStatus};
pub use dirty::{DirtyState, DirtyTracker};
pub use element::{
    Alignment, BackgroundTextureLoader, Container, Element, ElementId, FlexDirection, Text,
    VideoElement, WindowOperation,
};
pub use error::{FrameworkError, FrameworkResult};
pub use input::{InputEvent, KeyCode, MouseButton};
pub use layout::{Bounds, Point, Size};
pub use menu::{AppMenu, MenuEventHandler, MenuId};
pub use metrics::{FrameMetrics, FrameTiming, PerformanceStats};
pub use reactive::{
    Effect, EffectId, ReactiveRuntime, RuntimeStats, Signal, SignalId, SubscriptionId,
    create_effect, create_signal,
};
pub use render_graph::{
    ExecutionOrder, GraphStats, PassContext, PassId, RenderGraph, RenderGraphError, RenderPass,
    Resource, ResourceAccess, ResourceId, ResourceType, ResourceUsage,
};
pub use renderer::{BatchBuilder, BatchStats, Renderer, ZLayer};
pub use window::{PresentMode, Window, WindowContext, WindowOptions};

/// Color representation (RGBA, 0.0-1.0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self { r, g, b, a: 1.0 }
    }

    pub fn from_hex_with_alpha(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let a = (hex & 0xFF) as f32 / 255.0;
        Self { r, g, b, a }
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    // Common colors
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
}

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        wgpu::Color {
            r: color.r as f64,
            g: color.g as f64,
            b: color.b as f64,
            a: color.a as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex(0xFF0000);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);

        let color = Color::from_hex(0x00FF00);
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 0.0);

        let color = Color::from_hex(0x0000FF);
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 1.0);

        let color = Color::from_hex(0x808080);
        assert!((color.r - 0.502).abs() < 0.01);
        assert!((color.g - 0.502).abs() < 0.01);
        assert!((color.b - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_color_from_hex_with_alpha() {
        let color = Color::from_hex_with_alpha(0xFF000080);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert!((color.a - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_color_to_array() {
        let color = Color::new(0.25, 0.5, 0.75, 1.0);
        let arr = color.to_array();
        assert_eq!(arr, [0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::BLACK.to_array(), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(Color::WHITE.to_array(), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(Color::RED.to_array(), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(Color::GREEN.to_array(), [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(Color::BLUE.to_array(), [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(Color::TRANSPARENT.to_array(), [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_color_into_wgpu() {
        let color = Color::new(0.5, 0.25, 0.75, 1.0);
        let wgpu_color: wgpu::Color = color.into();
        assert_eq!(wgpu_color.r, 0.5);
        assert_eq!(wgpu_color.g, 0.25);
        assert_eq!(wgpu_color.b, 0.75);
        assert_eq!(wgpu_color.a, 1.0);
    }
}
