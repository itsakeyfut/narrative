//! Card component for displaying project cards, action cards, etc.

use crate::framework::Color;
use crate::framework::animation::AnimationContext;
use crate::framework::element::{
    Container, Element, ElementId, FlexDirection, LayoutContext, PaintContext,
};
use crate::framework::input::InputEvent;
use crate::framework::layout::{Bounds, Point};
use crate::theme::{colors, font_size, radius, spacing};
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

/// Card visual style
#[derive(Debug, Clone)]
pub struct CardStyle {
    pub background: Color,
    pub background_hover: Color,
    pub border_color: Color,
    pub corner_radius: f32,
    pub has_gradient: bool,
    pub gradient_start: Color,
    pub gradient_end: Color,
}

impl Default for CardStyle {
    fn default() -> Self {
        Self {
            background: colors::CARD_BG,
            background_hover: colors::CARD_HOVER,
            border_color: colors::CARD_BORDER,
            corner_radius: radius::LG,
            has_gradient: false,
            gradient_start: colors::ACCENT_GRADIENT_START,
            gradient_end: colors::ACCENT_GRADIENT_END,
        }
    }
}

impl CardStyle {
    /// Create a gradient card style (like Filmora's "New Project" button)
    pub fn gradient() -> Self {
        Self {
            has_gradient: true,
            background: colors::ACCENT_PRIMARY,
            background_hover: colors::ACCENT_SECONDARY,
            border_color: Color::TRANSPARENT,
            corner_radius: radius::LG,
            gradient_start: colors::ACCENT_GRADIENT_START,
            gradient_end: colors::ACCENT_GRADIENT_END,
        }
    }

    /// Create a subtle card style
    pub fn subtle() -> Self {
        Self {
            background: colors::BG_ELEVATED,
            background_hover: colors::BG_HOVER,
            border_color: colors::BORDER,
            corner_radius: radius::MD,
            has_gradient: false,
            gradient_start: Color::TRANSPARENT,
            gradient_end: Color::TRANSPARENT,
        }
    }
}

/// A card component for displaying content in a bordered container
pub struct Card {
    id: ElementId,
    layout_node: Option<NodeId>,
    children: Vec<Box<dyn Element>>,
    style: CardStyle,
    width: Option<f32>,
    height: Option<f32>,
    flex_grow: f32,
    is_hovered: bool,
    is_interactive: bool,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    padding: f32,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            children: Vec::new(),
            style: CardStyle::default(),
            width: None,
            height: None,
            flex_grow: 0.0,
            is_hovered: false,
            is_interactive: false,
            on_click: None,
            padding: spacing::LG,
            animation_context: AnimationContext::default(),
            animations_enabled: None,
        }
    }

    pub fn with_style(mut self, style: CardStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_flex_grow(mut self, grow: f32) -> Self {
        self.flex_grow = grow;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn interactive(mut self) -> Self {
        self.is_interactive = true;
        self
    }

    pub fn with_on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(callback));
        self.is_interactive = true;
        self
    }

    /// Set the animation context
    ///
    /// This allows the card to respect global animation settings for future animations.
    pub fn with_animation_context(mut self, context: AnimationContext) -> Self {
        self.animation_context = context;
        self
    }

    /// Set component-specific animation override
    ///
    /// This allows disabling animations for this specific card
    /// even when global animations are enabled, or vice versa.
    pub fn with_animations_enabled(mut self, enabled: impl Into<Option<bool>>) -> Self {
        self.animations_enabled = enabled.into();
        self
    }

    pub fn with_child(mut self, child: Box<dyn Element>) -> Self {
        self.children.push(child);
        self
    }

    fn get_background_color(&self) -> Color {
        if self.is_interactive && self.is_hovered {
            self.style.background_hover
        } else {
            self.style.background
        }
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Card {
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
            flex_direction: taffy::FlexDirection::Column,
            flex_grow: self.flex_grow,
            size: taffy::Size {
                width: self
                    .width
                    .map(Dimension::length)
                    .unwrap_or(Dimension::auto()),
                height: self
                    .height
                    .map(Dimension::length)
                    .unwrap_or(Dimension::auto()),
            },
            padding: taffy::Rect {
                top: LengthPercentage::length(self.padding),
                right: LengthPercentage::length(self.padding),
                bottom: LengthPercentage::length(self.padding),
                left: LengthPercentage::length(self.padding),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        let bg = self.get_background_color();

        // For gradient cards, we simulate a gradient with two overlapping rects
        if self.style.has_gradient {
            // Draw gradient background (simplified as solid color for now)
            // In a real implementation, we'd use a shader for proper gradients
            cx.fill_rounded_rect(
                cx.bounds,
                self.style.gradient_start,
                self.style.corner_radius,
            );

            // Overlay to simulate gradient
            let overlay_bounds = Bounds::new(
                cx.bounds.x() + cx.bounds.width() * 0.3,
                cx.bounds.y(),
                cx.bounds.width() * 0.7,
                cx.bounds.height(),
            );
            cx.fill_rounded_rect(
                overlay_bounds,
                Color::new(
                    self.style.gradient_end.r,
                    self.style.gradient_end.g,
                    self.style.gradient_end.b,
                    0.5,
                ),
                self.style.corner_radius,
            );
        } else {
            // Draw solid background
            cx.fill_rounded_rect(cx.bounds, bg, self.style.corner_radius);

            // Draw border
            if self.style.border_color.a > 0.0 {
                cx.stroke_rect(cx.bounds, self.style.border_color, 1.0);
            }
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        if !self.is_interactive {
            return false;
        }

        match event {
            InputEvent::MouseMove { position, .. } => {
                let was_hovered = self.is_hovered;
                self.is_hovered = bounds.contains(*position);
                was_hovered != self.is_hovered
            }
            InputEvent::MouseUp { position, .. } => {
                if bounds.contains(*position) {
                    if let Some(ref callback) = self.on_click {
                        callback();
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn tick(&mut self, delta: Duration) -> bool {
        let _ = delta;
        false
    }

    fn children(&self) -> &[Box<dyn Element>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Element>] {
        &mut self.children
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
