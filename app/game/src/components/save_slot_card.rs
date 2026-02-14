//! Save slot card component
//!
//! Displays individual save slot information with thumbnail, metadata, and action buttons.

use narrative_engine::runtime::LayoutMode;
use narrative_engine::save::SlotInfo;
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use narrative_gui::framework::input::InputEvent;
use narrative_gui::framework::layout::Bounds;
use narrative_gui::theme::{colors, font_size, radius, spacing};
use narrative_gui::{Color, Point, Size};
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

/// Save slot card element
pub struct SaveSlotCard {
    /// Unique element ID
    id: ElementId,
    /// Taffy layout node
    layout_node: Option<NodeId>,
    /// Slot information
    slot_info: SlotInfo,
    /// Whether this slot is selected
    is_selected: bool,
    /// Whether in save mode (true) or load mode (false)
    is_save_mode: bool,
    /// Layout mode
    layout_mode: LayoutMode,
    /// Animation context
    animation_context: AnimationContext,
}

impl SaveSlotCard {
    // Card dimensions for list layout
    const CARD_WIDTH_LIST: f32 = 800.0;
    const CARD_HEIGHT_LIST: f32 = 120.0;

    // Card dimensions for grid layout
    const CARD_WIDTH_GRID: f32 = 280.0;
    const CARD_HEIGHT_GRID: f32 = 200.0;

    // Thumbnail dimensions
    const THUMBNAIL_WIDTH_LIST: f32 = 160.0;
    const THUMBNAIL_HEIGHT_LIST: f32 = 90.0;
    #[allow(dead_code)]
    const THUMBNAIL_WIDTH_GRID: f32 = 256.0;
    const THUMBNAIL_HEIGHT_GRID: f32 = 144.0;

    /// Create a new save slot card
    pub fn new(
        slot_info: SlotInfo,
        is_selected: bool,
        is_save_mode: bool,
        layout_mode: LayoutMode,
    ) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            slot_info,
            is_selected,
            is_save_mode,
            layout_mode,
            animation_context: AnimationContext::default(),
        }
    }

    /// Set animation context
    pub fn with_animation_context(mut self, ctx: AnimationContext) -> Self {
        self.animation_context = ctx;
        self
    }

    /// Paint thumbnail (or placeholder)
    fn paint_thumbnail(&self, cx: &mut PaintContext, bounds: Bounds) {
        // For now, always draw placeholder
        // TODO: Load actual thumbnail texture when available
        self.paint_thumbnail_placeholder(cx, bounds);
    }

    /// Paint thumbnail placeholder
    fn paint_thumbnail_placeholder(&self, cx: &mut PaintContext, bounds: Bounds) {
        // Draw dark gray background
        let placeholder_color = Color::new(0.2, 0.2, 0.2, 1.0);
        cx.fill_rounded_rect(bounds, placeholder_color, radius::SM);

        // Draw "No Preview" text in center
        if self.slot_info.exists {
            let text = "Preview";
            let text_x = bounds.x() + bounds.width() / 2.0 - 30.0; // Rough centering
            let text_y = bounds.y() + bounds.height() / 2.0 - 8.0;
            cx.draw_text(
                text,
                Point::new(text_x, text_y),
                colors::TEXT_SECONDARY,
                font_size::SM,
            );
        }
    }

    /// Paint list layout
    fn paint_list_layout(&self, cx: &mut PaintContext, bounds: Bounds) {
        // Layout: [Thumbnail | Slot Info | Buttons]

        // Thumbnail (left)
        let thumbnail_bounds = Bounds {
            origin: Point::new(bounds.x() + spacing::MD, bounds.y() + spacing::MD),
            size: Size::new(Self::THUMBNAIL_WIDTH_LIST, Self::THUMBNAIL_HEIGHT_LIST),
        };
        self.paint_thumbnail(cx, thumbnail_bounds);

        // Slot info (center)
        let info_x = thumbnail_bounds.x() + Self::THUMBNAIL_WIDTH_LIST + spacing::MD;
        let info_y = bounds.y() + spacing::MD;

        if self.slot_info.exists {
            // Slot number
            let slot_text = format!("Slot {:02}", self.slot_info.slot + 1);
            cx.draw_text(
                &slot_text,
                Point::new(info_x, info_y),
                colors::TEXT_PRIMARY,
                font_size::LG,
            );

            // Scene name
            cx.draw_text(
                &self.slot_info.scene_name,
                Point::new(info_x, info_y + 30.0),
                colors::TEXT_SECONDARY,
                font_size::MD,
            );

            // Date/Time
            cx.draw_text(
                &self.slot_info.formatted_date(),
                Point::new(info_x, info_y + 55.0),
                colors::TEXT_SECONDARY,
                font_size::SM,
            );

            // Play time
            let play_time_text = format!("Play Time: {}", self.slot_info.formatted_play_time());
            cx.draw_text(
                &play_time_text,
                Point::new(info_x, info_y + 75.0),
                colors::TEXT_SECONDARY,
                font_size::SM,
            );
        } else {
            // Empty slot
            let empty_text = format!("Slot {:02} - Empty", self.slot_info.slot + 1);
            cx.draw_text(
                &empty_text,
                Point::new(info_x, info_y + 40.0),
                colors::TEXT_SECONDARY,
                font_size::LG,
            );
        }

        // Buttons (right)
        // TODO: Implement actual buttons when needed
        // For now, just show text hints
        let button_x = bounds.x() + bounds.width() - 120.0;
        let button_y = bounds.y() + spacing::MD;

        if self.is_save_mode {
            cx.draw_text(
                "[Save]",
                Point::new(button_x, button_y),
                colors::ACCENT_PRIMARY,
                font_size::MD,
            );
        } else if self.slot_info.exists {
            cx.draw_text(
                "[Load]",
                Point::new(button_x, button_y),
                colors::ACCENT_PRIMARY,
                font_size::MD,
            );
            cx.draw_text(
                "[Delete]",
                Point::new(button_x, button_y + 25.0),
                colors::TEXT_SECONDARY,
                font_size::SM,
            );
        }
    }

    /// Paint grid layout
    fn paint_grid_layout(&self, cx: &mut PaintContext, bounds: Bounds) {
        // Layout: Vertical stack [Thumbnail, Slot Info]

        // Thumbnail (top)
        let thumbnail_bounds = Bounds {
            origin: Point::new(bounds.x() + spacing::SM, bounds.y() + spacing::SM),
            size: Size::new(
                bounds.width() - spacing::SM * 2.0,
                Self::THUMBNAIL_HEIGHT_GRID,
            ),
        };
        self.paint_thumbnail(cx, thumbnail_bounds);

        // Slot info (bottom)
        let info_y = thumbnail_bounds.y() + thumbnail_bounds.height() + spacing::SM;

        if self.slot_info.exists {
            let slot_text = format!("#{:02}", self.slot_info.slot + 1);
            cx.draw_text(
                &slot_text,
                Point::new(bounds.x() + spacing::SM, info_y),
                colors::TEXT_PRIMARY,
                font_size::MD,
            );

            cx.draw_text(
                &self.slot_info.scene_name_short(),
                Point::new(bounds.x() + spacing::SM, info_y + 20.0),
                colors::TEXT_SECONDARY,
                font_size::SM,
            );

            cx.draw_text(
                &self.slot_info.formatted_date_short(),
                Point::new(bounds.x() + spacing::SM, info_y + 38.0),
                colors::TEXT_SECONDARY,
                font_size::XS,
            );
        } else {
            let empty_text = format!("#{:02} Empty", self.slot_info.slot + 1);
            cx.draw_text(
                &empty_text,
                Point::new(bounds.x() + spacing::SM, info_y + 20.0),
                colors::TEXT_SECONDARY,
                font_size::MD,
            );
        }
    }
}

impl Element for SaveSlotCard {
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

        let (width, height) = match self.layout_mode {
            LayoutMode::List => (Self::CARD_WIDTH_LIST, Self::CARD_HEIGHT_LIST),
            LayoutMode::Grid => (Self::CARD_WIDTH_GRID, Self::CARD_HEIGHT_GRID),
        };

        Style {
            display: Display::Flex,
            size: Size {
                width: Dimension::length(width),
                height: Dimension::length(height),
            },
            margin: Rect::length(spacing::SM),
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        let bounds = cx.bounds;

        // Card background
        let bg_color = if self.is_selected {
            Color::new(0.3, 0.4, 0.6, 1.0) // Highlighted blue
        } else {
            colors::CARD_BG
        };
        cx.fill_rounded_rect(bounds, bg_color, radius::MD);

        // Paint based on layout mode
        match self.layout_mode {
            LayoutMode::List => self.paint_list_layout(cx, bounds),
            LayoutMode::Grid => self.paint_grid_layout(cx, bounds),
        }

        // Selection indicator
        if self.is_selected {
            // Draw border
            let border_color = colors::ACCENT_PRIMARY;
            let border_width = 2.0;
            // Simple border using rectangles (top, right, bottom, left)
            // Top
            cx.fill_rect(
                Bounds {
                    origin: bounds.origin,
                    size: Size::new(bounds.width(), border_width),
                },
                border_color,
            );
            // Right
            cx.fill_rect(
                Bounds {
                    origin: Point::new(bounds.x() + bounds.width() - border_width, bounds.y()),
                    size: Size::new(border_width, bounds.height()),
                },
                border_color,
            );
            // Bottom
            cx.fill_rect(
                Bounds {
                    origin: Point::new(bounds.x(), bounds.y() + bounds.height() - border_width),
                    size: Size::new(bounds.width(), border_width),
                },
                border_color,
            );
            // Left
            cx.fill_rect(
                Bounds {
                    origin: bounds.origin,
                    size: Size::new(border_width, bounds.height()),
                },
                border_color,
            );
        }
    }

    fn handle_event(&mut self, _event: &InputEvent, _bounds: Bounds) -> bool {
        // Events are handled by parent SaveLoadMenuElement
        false
    }

    fn tick(&mut self, _delta: Duration) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
