//! Toggle/checkbox component for boolean settings

use crate::framework::Color;
use crate::framework::animation::{AnimationContext, Easing, PropertyAnimation};
use crate::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use crate::framework::input::InputEvent;
use crate::framework::layout::Bounds;
use crate::theme::{colors, font_size, radius, spacing};
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use taffy::NodeId;

/// Toggle visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleStyle {
    /// Checkbox style (24x24px square with checkmark)
    #[default]
    Checkbox,
    /// Slide switch style (48x24px pill with sliding knob)
    Switch,
}

/// Toggle component for boolean settings
pub struct Toggle {
    id: ElementId,
    layout_node: Option<NodeId>,
    label: String,
    value: Arc<Mutex<bool>>,
    style: ToggleStyle,
    width: Option<f32>,
    is_hovered: bool,
    on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
    /// Animation for switch knob slide (0.0 = left/OFF, 1.0 = right/ON)
    knob_animation: Option<PropertyAnimation<f32>>,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl Toggle {
    /// Create a new toggle with default checkbox style
    pub fn new(label: impl Into<String>, value: bool) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            label: label.into(),
            value: Arc::new(Mutex::new(value)),
            style: ToggleStyle::default(),
            width: None,
            is_hovered: false,
            on_change: None,
            knob_animation: None,
            animation_context: AnimationContext::default(),
            animations_enabled: None,
        }
    }

    /// Set the toggle style
    pub fn with_style(mut self, style: ToggleStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the change callback
    pub fn with_on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Set the animation context
    ///
    /// This allows the toggle to respect global animation settings.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let ctx = AnimationContext::from_enabled_and_speed(true, 1.0);
    /// let toggle = Toggle::new("Enable Animations", true)
    ///     .with_animation_context(ctx);
    /// ```
    pub fn with_animation_context(mut self, context: AnimationContext) -> Self {
        self.animation_context = context;
        self
    }

    /// Set component-specific animation override
    ///
    /// This allows disabling animations for this specific toggle
    /// even when global animations are enabled, or vice versa.
    ///
    /// # Arguments
    ///
    /// * `enabled` - `Some(true)` to force enable, `Some(false)` to force disable,
    ///   `None` to follow global settings
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Force disable animations for this toggle
    /// let toggle = Toggle::new("Silent Mode", false)
    ///     .with_animations_enabled(false);
    /// ```
    pub fn with_animations_enabled(mut self, enabled: impl Into<Option<bool>>) -> Self {
        self.animations_enabled = enabled.into();
        self
    }

    /// Get current value
    pub fn value(&self) -> bool {
        *self.value.lock().unwrap_or_else(|e| {
            tracing::warn!("Toggle value mutex poisoned, recovering: {}", e);
            e.into_inner()
        })
    }

    /// Set value (and trigger callback)
    pub fn set_value(&mut self, new_value: bool) {
        let mut value = self.value.lock().unwrap_or_else(|e| {
            tracing::warn!("Toggle value mutex poisoned, recovering: {}", e);
            e.into_inner()
        });

        if *value != new_value {
            *value = new_value;
            if let Some(callback) = &self.on_change {
                callback(new_value);
            }
        }
    }

    /// Toggle the value
    fn toggle(&mut self) {
        let current = self.value();
        let new_value = !current;
        self.set_value(new_value);

        // Start knob slide animation for switch style
        if self.style == ToggleStyle::Switch {
            let start = if current { 1.0 } else { 0.0 };
            let end = if new_value { 1.0 } else { 0.0 };
            tracing::debug!("Toggle animation: {} -> {} ({}ms)", start, end, 200);
            self.knob_animation = Some(PropertyAnimation::new_with_context(
                start,
                end,
                Duration::from_millis(200),
                Easing::QuadOut,
                &self.animation_context,
                self.animations_enabled,
            ));
        }
    }

    /// Get the toggle box bounds
    fn get_toggle_box_bounds(&self, bounds: Bounds) -> Bounds {
        let (width, height) = match self.style {
            ToggleStyle::Checkbox => (24.0, 24.0),
            ToggleStyle::Switch => (48.0, 24.0),
        };
        let x = bounds.x() + spacing::MD;
        let y = bounds.y() + (bounds.height() - height) / 2.0;
        Bounds::new(x, y, width, height)
    }

    /// Get the label bounds
    fn get_label_bounds(&self, bounds: Bounds) -> Bounds {
        let toggle_width = match self.style {
            ToggleStyle::Checkbox => 24.0,
            ToggleStyle::Switch => 48.0,
        };
        let x = bounds.x() + spacing::MD + toggle_width + spacing::MD;
        let y = bounds.y();
        let width = bounds.width() - spacing::MD - toggle_width - spacing::MD;
        Bounds::new(x, y, width, bounds.height())
    }

    /// Paint checkbox style toggle
    fn paint_checkbox(&self, cx: &mut PaintContext, toggle_bounds: Bounds, value: bool) {
        // Draw checkbox box background
        let box_color = if self.is_hovered {
            colors::BG_ELEVATED
        } else {
            colors::BG_DARK
        };
        cx.fill_rounded_rect(toggle_bounds, box_color, radius::SM);

        // Draw checkmark if enabled
        if value {
            // Fill the box with accent color
            cx.fill_rounded_rect(toggle_bounds, colors::ACCENT_PRIMARY, radius::SM);

            // Draw checkmark indicator (simple filled square)
            let check_padding = 6.0;
            let check_x = toggle_bounds.x() + check_padding;
            let check_y = toggle_bounds.y() + check_padding;
            let check_size = toggle_bounds.width() - check_padding * 2.0;

            let check_bounds = Bounds::new(check_x, check_y, check_size, check_size);
            cx.fill_rounded_rect(check_bounds, colors::BG_DARKEST, 2.0);
        }
    }

    /// Paint switch style toggle
    fn paint_switch(&self, cx: &mut PaintContext, toggle_bounds: Bounds, value: bool) {
        // Get animated knob progress (0.0 = left/OFF, 1.0 = right/ON)
        let knob_progress = self
            .knob_animation
            .as_ref()
            .map(|a| a.current_value())
            .unwrap_or(if value { 1.0 } else { 0.0 });

        // Switch background (pill shape) - animate color based on progress
        let bg_color = if knob_progress > 0.5 {
            // Mostly ON: accent color
            if self.is_hovered {
                colors::ACCENT_MUTED
            } else {
                colors::ACCENT_PRIMARY
            }
        } else {
            // Mostly OFF: dark gray
            if self.is_hovered {
                colors::BG_ELEVATED
            } else {
                colors::BG_DARK
            }
        };

        // Draw pill-shaped background (full height radius for pill shape)
        let pill_radius = toggle_bounds.height() / 2.0;
        cx.fill_rounded_rect(toggle_bounds, bg_color, pill_radius);

        // Draw sliding knob (circle) at animated position
        const KNOB_SIZE: f32 = 20.0;
        const KNOB_PADDING: f32 = 2.0;

        // Calculate knob position based on progress (0.0 to 1.0)
        let knob_travel = toggle_bounds.width() - KNOB_SIZE - KNOB_PADDING * 2.0;
        let knob_x = toggle_bounds.x() + KNOB_PADDING + knob_progress * knob_travel;

        let knob_y = toggle_bounds.y() + (toggle_bounds.height() - KNOB_SIZE) / 2.0;
        let knob_bounds = Bounds::new(knob_x, knob_y, KNOB_SIZE, KNOB_SIZE);

        // Knob color (always white/light)
        let knob_color = colors::TEXT_PRIMARY;
        cx.fill_rounded_rect(knob_bounds, knob_color, KNOB_SIZE / 2.0);
    }
}

impl Element for Toggle {
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

        let width = self.width.unwrap_or(300.0);

        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::Center),
            size: Size {
                width: Dimension::length(width),
                height: Dimension::length(40.0),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        let bounds = cx.bounds;
        let value = self.value();

        // Background (hover effect)
        if self.is_hovered {
            cx.fill_rounded_rect(bounds, colors::BG_HOVER, radius::MD);
        }

        // Toggle box bounds
        let toggle_bounds = self.get_toggle_box_bounds(bounds);

        // Draw based on style
        match self.style {
            ToggleStyle::Checkbox => {
                self.paint_checkbox(cx, toggle_bounds, value);
            }
            ToggleStyle::Switch => {
                self.paint_switch(cx, toggle_bounds, value);
            }
        }

        // Draw label
        let label_bounds = self.get_label_bounds(bounds);
        let label_x = label_bounds.x();
        let label_y = label_bounds.y() + label_bounds.height() / 2.0 + font_size::MD / 3.0;

        cx.draw_text(
            &self.label,
            crate::framework::layout::Point::new(label_x, label_y),
            colors::TEXT_PRIMARY,
            font_size::MD,
        );
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        match event {
            InputEvent::MouseMove { position, .. } => {
                let was_hovered = self.is_hovered;
                self.is_hovered = bounds.contains(*position);
                was_hovered != self.is_hovered
            }
            InputEvent::MouseDown { position, .. } => {
                if bounds.contains(*position) {
                    self.toggle();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn tick(&mut self, delta: Duration) -> bool {
        if let Some(anim) = &mut self.knob_animation {
            let progress_before = anim.current_value();
            let needs_redraw = anim.tick(delta);
            let progress_after = anim.current_value();

            if needs_redraw {
                tracing::trace!(
                    "Toggle tick: delta={:?}, progress: {:.3} -> {:.3}",
                    delta,
                    progress_before,
                    progress_after
                );
            }

            // Remove completed animations
            if anim.is_completed() {
                tracing::debug!("Toggle animation completed");
                self.knob_animation = None;
            }
            needs_redraw
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn children(&self) -> &[Box<dyn Element>] {
        &[]
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Element>] {
        &mut []
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_new() {
        let toggle = Toggle::new("Test", false);
        assert_eq!(toggle.label, "Test");
        assert!(!toggle.value());
    }

    #[test]
    fn test_toggle_new_true() {
        let toggle = Toggle::new("Test", true);
        assert!(toggle.value());
    }

    #[test]
    fn test_toggle_set_value() {
        let mut toggle = Toggle::new("Test", false);
        toggle.set_value(true);
        assert!(toggle.value());
        toggle.set_value(false);
        assert!(!toggle.value());
    }

    #[test]
    fn test_toggle_with_width() {
        let toggle = Toggle::new("Test", false).with_width(500.0);
        assert_eq!(toggle.width, Some(500.0));
    }

    #[test]
    fn test_toggle_callback() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);

        let mut toggle = Toggle::new("Test", false).with_on_change(move |value| {
            called_clone.store(value, Ordering::SeqCst);
        });

        assert!(!called.load(Ordering::SeqCst));
        toggle.set_value(true);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_toggle_style_checkbox() {
        let toggle = Toggle::new("Test", false);
        assert_eq!(toggle.style, ToggleStyle::Checkbox);
    }

    #[test]
    fn test_toggle_style_switch() {
        let toggle = Toggle::new("Test", false).with_style(ToggleStyle::Switch);
        assert_eq!(toggle.style, ToggleStyle::Switch);
    }

    #[test]
    fn test_toggle_bounds_checkbox() {
        let toggle = Toggle::new("Test", false).with_style(ToggleStyle::Checkbox);
        let bounds = Bounds::new(0.0, 0.0, 400.0, 40.0);
        let toggle_bounds = toggle.get_toggle_box_bounds(bounds);

        // Checkbox is 24x24
        assert_eq!(toggle_bounds.width(), 24.0);
        assert_eq!(toggle_bounds.height(), 24.0);
    }

    #[test]
    fn test_toggle_bounds_switch() {
        let toggle = Toggle::new("Test", false).with_style(ToggleStyle::Switch);
        let bounds = Bounds::new(0.0, 0.0, 400.0, 40.0);
        let toggle_bounds = toggle.get_toggle_box_bounds(bounds);

        // Switch is 48x24
        assert_eq!(toggle_bounds.width(), 48.0);
        assert_eq!(toggle_bounds.height(), 24.0);
    }

    #[test]
    fn test_toggle_with_animation_context() {
        let ctx = AnimationContext::disabled();
        let toggle = Toggle::new("Test", false)
            .with_style(ToggleStyle::Switch)
            .with_animation_context(ctx);

        // Animations should be disabled via context
        assert!(!toggle.animation_context.should_animate(None));
    }

    #[test]
    fn test_toggle_with_animations_enabled_override() {
        let toggle = Toggle::new("Test", false).with_animations_enabled(false);

        assert_eq!(toggle.animations_enabled, Some(false));
    }

    #[test]
    fn test_toggle_animation_uses_context() {
        let ctx = AnimationContext::disabled();
        let mut toggle = Toggle::new("Test", false)
            .with_style(ToggleStyle::Switch)
            .with_animation_context(ctx);

        // Toggle the switch
        toggle.toggle();

        // Animation should be instant (zero duration)
        if let Some(anim) = &toggle.knob_animation {
            assert!(anim.is_instant());
        }
    }
}
