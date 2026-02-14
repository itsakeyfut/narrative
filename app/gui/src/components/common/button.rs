//! Button component with various styles

use crate::framework::Color;
use crate::framework::animation::AnimationContext;
use crate::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use crate::framework::input::InputEvent;
use crate::framework::layout::{Bounds, Point};
use crate::theme::{button, colors, common, font_size, radius, spacing};
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

/// Button visual style variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ButtonVariant {
    /// Primary accent button (teal gradient)
    #[default]
    Primary,
    /// Secondary muted button
    Secondary,
    /// Ghost/transparent button
    Ghost,
    /// Text-only button
    Text,
}

/// Button style configuration
#[derive(Debug, Clone)]
pub struct ButtonStyle {
    pub variant: ButtonVariant,
    pub corner_radius: f32,
    pub font_size: f32,
    pub padding_h: f32,
    pub padding_v: f32,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            variant: ButtonVariant::Primary,
            corner_radius: radius::MD,
            font_size: font_size::MD,
            padding_h: spacing::LG,
            padding_v: spacing::SM,
        }
    }
}

/// Interactive button component
pub struct Button {
    id: ElementId,
    layout_node: Option<NodeId>,
    label: String,
    style: ButtonStyle,
    width: Option<f32>,
    height: Option<f32>,
    is_hovered: bool,
    is_pressed: bool,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            label: label.into(),
            style: ButtonStyle::default(),
            width: None,
            height: None,
            is_hovered: false,
            is_pressed: false,
            on_click: None,
            animation_context: AnimationContext::default(),
            animations_enabled: None,
        }
    }

    pub fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.style.variant = variant;
        self
    }

    pub fn with_style(mut self, style: ButtonStyle) -> Self {
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

    pub fn with_on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(callback));
        self
    }

    /// Set the animation context
    ///
    /// This allows the button to respect global animation settings for future animations.
    pub fn with_animation_context(mut self, context: AnimationContext) -> Self {
        self.animation_context = context;
        self
    }

    /// Set component-specific animation override
    ///
    /// This allows disabling animations for this specific button
    /// even when global animations are enabled, or vice versa.
    pub fn with_animations_enabled(mut self, enabled: impl Into<Option<bool>>) -> Self {
        self.animations_enabled = enabled.into();
        self
    }

    fn get_background_color(&self) -> Color {
        match self.style.variant {
            ButtonVariant::Primary => {
                if self.is_pressed {
                    colors::ACCENT_MUTED
                } else if self.is_hovered {
                    colors::BUTTON_PRIMARY_HOVER
                } else {
                    colors::BUTTON_PRIMARY
                }
            }
            ButtonVariant::Secondary => {
                if self.is_pressed {
                    colors::BG_SELECTED
                } else if self.is_hovered {
                    colors::BUTTON_SECONDARY_HOVER
                } else {
                    colors::BUTTON_SECONDARY
                }
            }
            ButtonVariant::Ghost => {
                if self.is_pressed {
                    colors::BG_SELECTED
                } else if self.is_hovered {
                    colors::BG_HOVER
                } else {
                    Color::TRANSPARENT
                }
            }
            ButtonVariant::Text => Color::TRANSPARENT,
        }
    }

    fn get_text_color(&self) -> Color {
        match self.style.variant {
            ButtonVariant::Primary => colors::BG_DARKEST,
            ButtonVariant::Secondary => colors::TEXT_PRIMARY,
            ButtonVariant::Ghost => {
                if self.is_hovered {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_SECONDARY
                }
            }
            ButtonVariant::Text => {
                if self.is_hovered {
                    colors::ACCENT_PRIMARY
                } else {
                    colors::TEXT_SECONDARY
                }
            }
        }
    }
}

impl Element for Button {
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

        // Calculate minimum size based on text
        let text_width =
            self.label.chars().count() as f32 * self.style.font_size * common::CHAR_WIDTH_RATIO;
        let min_width = text_width + self.style.padding_h * 2.0;
        let min_height = self.style.font_size + self.style.padding_v * 2.0;

        Style {
            display: Display::Flex,
            align_items: Some(AlignItems::Center),
            justify_content: Some(JustifyContent::Center),
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
            min_size: taffy::Size {
                width: Dimension::length(min_width),
                height: Dimension::length(min_height),
            },
            padding: taffy::Rect {
                top: LengthPercentage::length(self.style.padding_v),
                right: LengthPercentage::length(self.style.padding_h),
                bottom: LengthPercentage::length(self.style.padding_v),
                left: LengthPercentage::length(self.style.padding_h),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        let bg = self.get_background_color();
        let text_color = self.get_text_color();

        // Draw background
        if bg.a > 0.0 {
            cx.fill_rounded_rect(cx.bounds, bg, self.style.corner_radius);
        }

        // Draw border for secondary variant
        if self.style.variant == ButtonVariant::Secondary {
            cx.stroke_rect(cx.bounds, colors::BORDER_LIGHT, common::BORDER_THICKNESS);
        }

        // Draw text centered
        let text_x = cx.bounds.x()
            + (cx.bounds.width()
                - self.label.len() as f32 * self.style.font_size * common::CHAR_WIDTH_RATIO)
                / 2.0;
        let text_y = cx.bounds.y()
            + (cx.bounds.height() + self.style.font_size * button::TEXT_VERTICAL_RATIO) / 2.0;

        cx.draw_text(
            &self.label,
            Point::new(text_x, text_y),
            text_color,
            self.style.font_size,
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
                    self.is_pressed = true;
                    true
                } else {
                    false
                }
            }
            InputEvent::MouseUp { position, .. } => {
                let was_pressed = self.is_pressed;
                self.is_pressed = false;
                if was_pressed && bounds.contains(*position) {
                    if let Some(ref callback) = self.on_click {
                        callback();
                    }
                    true
                } else {
                    was_pressed
                }
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
