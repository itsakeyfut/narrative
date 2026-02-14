//! Slider component for numeric value selection
//!
//! A draggable slider control with:
//! - Customizable value range (min, max)
//! - Optional step/granularity
//! - Visual feedback (hover, drag states)
//! - Click-to-jump on track
//! - Callback on value change

use crate::framework::Color;
use crate::framework::animation::AnimationContext;
use crate::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use crate::framework::input::InputEvent;
use crate::framework::layout::{Bounds, Point};
use crate::theme::{colors, font_size, radius, spacing};
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

/// Callback type for slider value changes
type ValueChangeCallback = Box<dyn Fn(f32) + Send + Sync>;

/// Slider component for numeric value selection
pub struct Slider {
    id: ElementId,
    layout_node: Option<NodeId>,
    /// Current value
    value: f32,
    /// Minimum value
    min: f32,
    /// Maximum value
    max: f32,
    /// Optional step size (None = continuous)
    step: Option<f32>,
    /// Label text (displayed above slider)
    label: String,
    /// Width in pixels (None = auto)
    width: Option<f32>,
    /// Height of the slider track
    track_height: f32,
    /// Thumb (handle) radius
    thumb_radius: f32,
    /// Whether the thumb is currently hovered
    is_hovered: bool,
    /// Whether the slider is being dragged
    is_dragging: bool,
    /// Value change callback
    on_change: Option<ValueChangeCallback>,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl Slider {
    /// Default track height
    const DEFAULT_TRACK_HEIGHT: f32 = 6.0;
    /// Default thumb radius
    const DEFAULT_THUMB_RADIUS: f32 = 10.0;
    /// Minimum component height (thumb + padding)
    const MIN_HEIGHT: f32 = 40.0;
    /// Thumb border thickness
    const THUMB_BORDER_THICKNESS: f32 = 2.0;

    /// Create a new slider with specified range
    pub fn new(label: impl Into<String>, min: f32, max: f32) -> Self {
        let min_val = min.min(max);
        let max_val = min.max(max);

        Self {
            id: ElementId::new(),
            layout_node: None,
            value: min_val,
            min: min_val,
            max: max_val,
            step: None,
            label: label.into(),
            width: None,
            track_height: Self::DEFAULT_TRACK_HEIGHT,
            thumb_radius: Self::DEFAULT_THUMB_RADIUS,
            is_hovered: false,
            is_dragging: false,
            on_change: None,
            animation_context: AnimationContext::default(),
            animations_enabled: None,
        }
    }

    /// Set the current value
    pub fn with_value(mut self, value: f32) -> Self {
        self.value = self.clamp_value(value);
        self
    }

    /// Set step size for discrete values
    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Set custom width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set value change callback
    pub fn with_on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Set the animation context
    ///
    /// This allows the slider to respect global animation settings for future animations.
    pub fn with_animation_context(mut self, context: AnimationContext) -> Self {
        self.animation_context = context;
        self
    }

    /// Set component-specific animation override
    ///
    /// This allows disabling animations for this specific slider
    /// even when global animations are enabled, or vice versa.
    pub fn with_animations_enabled(mut self, enabled: impl Into<Option<bool>>) -> Self {
        self.animations_enabled = enabled.into();
        self
    }

    /// Get the current value
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Set the value programmatically (triggers callback)
    pub fn set_value(&mut self, value: f32) {
        let new_value = self.clamp_value(value);
        if (new_value - self.value).abs() > f32::EPSILON {
            self.value = new_value;
            if let Some(ref callback) = self.on_change {
                callback(self.value);
            }
        }
    }

    /// Clamp and optionally snap value to step
    fn clamp_value(&self, value: f32) -> f32 {
        let clamped = value.clamp(self.min, self.max);

        if let Some(step) = self.step
            && step > 0.0
        {
            // Snap to nearest step
            let steps = ((clamped - self.min) / step).round();
            return self.min + steps * step;
        }

        clamped
    }

    /// Calculate normalized value (0.0 to 1.0)
    fn normalized_value(&self) -> f32 {
        let range = self.max - self.min;
        if range > 0.0 {
            (self.value - self.min) / range
        } else {
            0.0
        }
    }

    /// Calculate thumb position from value
    fn thumb_position(&self, track_bounds: Bounds) -> f32 {
        let usable_width = track_bounds.width() - self.thumb_radius * 2.0;
        track_bounds.x() + self.thumb_radius + usable_width * self.normalized_value()
    }

    /// Calculate value from x position
    fn value_from_position(&self, x: f32, track_bounds: Bounds) -> f32 {
        let usable_width = track_bounds.width() - self.thumb_radius * 2.0;
        let local_x = (x - track_bounds.x() - self.thumb_radius)
            .max(0.0)
            .min(usable_width);
        let ratio = if usable_width > 0.0 {
            local_x / usable_width
        } else {
            0.0
        };

        self.min + ratio * (self.max - self.min)
    }

    /// Get track bounds from component bounds
    fn track_bounds(&self, bounds: Bounds) -> Bounds {
        let label_height = if self.label.is_empty() {
            0.0
        } else {
            font_size::MD + spacing::XS
        };

        let track_y =
            bounds.y() + label_height + (bounds.height() - label_height - self.track_height) / 2.0;

        Bounds::new(
            bounds.x() + self.thumb_radius,
            track_y,
            bounds.width() - self.thumb_radius * 2.0,
            self.track_height,
        )
    }

    /// Get thumb bounds
    fn thumb_bounds(&self, bounds: Bounds) -> Bounds {
        let track = self.track_bounds(bounds);
        let thumb_x = self.thumb_position(track);
        let thumb_y = track.y() + track.height() / 2.0;

        Bounds::new(
            thumb_x - self.thumb_radius,
            thumb_y - self.thumb_radius,
            self.thumb_radius * 2.0,
            self.thumb_radius * 2.0,
        )
    }

    /// Check if point is on thumb
    fn is_on_thumb(&self, point: Point, bounds: Bounds) -> bool {
        let thumb = self.thumb_bounds(bounds);
        thumb.contains(point)
    }
}

impl Element for Slider {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout_node(&self) -> Option<NodeId> {
        self.layout_node
    }

    fn set_layout_node(&mut self, node: NodeId) {
        self.layout_node = Some(node);
    }

    fn layout(&mut self, _cx: &mut LayoutContext) -> taffy::Style {
        use taffy::prelude::*;

        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: taffy::Size {
                width: self
                    .width
                    .map(Dimension::length)
                    .unwrap_or(Dimension::auto()),
                height: Dimension::length(Self::MIN_HEIGHT),
            },
            min_size: taffy::Size {
                width: Dimension::length(100.0),
                height: Dimension::length(Self::MIN_HEIGHT),
            },
            padding: taffy::Rect {
                top: LengthPercentage::length(spacing::SM),
                right: LengthPercentage::length(spacing::MD),
                bottom: LengthPercentage::length(spacing::SM),
                left: LengthPercentage::length(spacing::MD),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        let bounds = cx.bounds;

        // Draw label
        if !self.label.is_empty() {
            cx.draw_text(
                &self.label,
                Point::new(bounds.x(), bounds.y() + font_size::MD),
                colors::TEXT_SECONDARY,
                font_size::MD,
            );
        }

        let track = self.track_bounds(bounds);

        // Draw track background
        cx.fill_rounded_rect(track, colors::BG_DARK, radius::SM);

        // Draw filled portion (value indicator)
        let filled_width = (track.width() * self.normalized_value()).max(0.0);
        if filled_width > 0.0 {
            let filled_track = Bounds::new(track.x(), track.y(), filled_width, track.height());
            cx.fill_rounded_rect(filled_track, colors::ACCENT_PRIMARY, radius::SM);
        }

        // Draw thumb
        let thumb = self.thumb_bounds(bounds);
        let thumb_color = if self.is_dragging {
            colors::ACCENT_GRADIENT_START
        } else if self.is_hovered {
            colors::BUTTON_PRIMARY_HOVER
        } else {
            colors::BUTTON_PRIMARY
        };

        // Thumb shadow (for depth)
        if !self.is_dragging {
            cx.fill_rounded_rect(
                Bounds::new(
                    thumb.x() + 1.0,
                    thumb.y() + 2.0,
                    thumb.width(),
                    thumb.height(),
                ),
                Color::new(0.0, 0.0, 0.0, 0.3),
                radius::FULL,
            );
        }

        // Thumb with border effect (outer circle is border, inner is color)
        cx.fill_rounded_rect(thumb, colors::BG_DARKEST, radius::FULL);

        let inner_thumb = Bounds::new(
            thumb.x() + Self::THUMB_BORDER_THICKNESS,
            thumb.y() + Self::THUMB_BORDER_THICKNESS,
            thumb.width() - Self::THUMB_BORDER_THICKNESS * 2.0,
            thumb.height() - Self::THUMB_BORDER_THICKNESS * 2.0,
        );
        cx.fill_rounded_rect(inner_thumb, thumb_color, radius::FULL);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        match event {
            InputEvent::MouseMove { position, .. } => {
                let was_hovered = self.is_hovered;
                self.is_hovered = self.is_on_thumb(*position, bounds);

                if self.is_dragging {
                    // Update value while dragging
                    let track = self.track_bounds(bounds);
                    let new_value = self.value_from_position(position.x, track);
                    self.set_value(new_value);
                    return true;
                }

                was_hovered != self.is_hovered
            }
            InputEvent::MouseDown { position, .. } => {
                if self.is_on_thumb(*position, bounds) {
                    // Start dragging thumb
                    self.is_dragging = true;
                    true
                } else if bounds.contains(*position) {
                    // Click on track - jump to position
                    let track = self.track_bounds(bounds);
                    if track.contains(*position) {
                        let new_value = self.value_from_position(position.x, track);
                        self.set_value(new_value);
                        self.is_dragging = true; // Start dragging from clicked position
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            InputEvent::MouseUp { .. } => {
                let was_dragging = self.is_dragging;
                self.is_dragging = false;
                was_dragging
            }
            _ => false,
        }
    }

    fn tick(&mut self, delta: Duration) -> bool {
        let _ = delta;
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_new() {
        let slider = Slider::new("Test", 0.0, 100.0);
        assert_eq!(slider.min, 0.0);
        assert_eq!(slider.max, 100.0);
        assert_eq!(slider.value, 0.0);
        assert_eq!(slider.label, "Test");
    }

    #[test]
    fn test_slider_with_value() {
        let slider = Slider::new("Test", 0.0, 100.0).with_value(50.0);
        assert_eq!(slider.value, 50.0);
    }

    #[test]
    fn test_slider_clamp_value() {
        let slider = Slider::new("Test", 0.0, 100.0);
        assert_eq!(slider.clamp_value(-10.0), 0.0);
        assert_eq!(slider.clamp_value(50.0), 50.0);
        assert_eq!(slider.clamp_value(150.0), 100.0);
    }

    #[test]
    fn test_slider_with_step() {
        let slider = Slider::new("Test", 0.0, 100.0).with_step(10.0);
        assert_eq!(slider.step, Some(10.0));
        assert_eq!(slider.clamp_value(23.0), 20.0);
        assert_eq!(slider.clamp_value(27.0), 30.0);
    }

    #[test]
    fn test_slider_normalized_value() {
        let slider = Slider::new("Test", 0.0, 100.0).with_value(50.0);
        assert!((slider.normalized_value() - 0.5).abs() < f32::EPSILON);

        let slider = Slider::new("Test", 0.0, 100.0).with_value(0.0);
        assert!((slider.normalized_value() - 0.0).abs() < f32::EPSILON);

        let slider = Slider::new("Test", 0.0, 100.0).with_value(100.0);
        assert!((slider.normalized_value() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_slider_value_from_position() {
        let slider = Slider::new("Test", 0.0, 100.0);
        let track = Bounds::new(20.0, 0.0, 200.0, 6.0);

        // Test positions (accounting for thumb radius)
        let value = slider.value_from_position(120.0, track); // Middle
        assert!((value - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_slider_min_max_swap() {
        // Should handle reversed min/max
        let slider = Slider::new("Test", 100.0, 0.0);
        assert_eq!(slider.min, 0.0);
        assert_eq!(slider.max, 100.0);
    }
}
