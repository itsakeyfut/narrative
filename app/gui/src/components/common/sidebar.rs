//! Sidebar navigation component

use crate::framework::Color;
use crate::framework::element::{
    Alignment, Container, Element, ElementId, FlexDirection, LayoutContext, PaintContext, Text,
};
use crate::framework::input::InputEvent;
use crate::framework::layout::{Bounds, Point};
use crate::theme::{colors, common, font_size, layout, radius, sidebar, spacing};
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

use super::icon::{Icon, IconType};

/// A sidebar navigation item
#[derive(Clone)]
pub struct SidebarItem {
    pub id: String,
    pub label: String,
    pub icon: IconType,
    pub badge: Option<String>,
}

impl SidebarItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>, icon: IconType) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon,
            badge: None,
        }
    }

    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

/// Sidebar navigation component
pub struct Sidebar {
    id: ElementId,
    layout_node: Option<NodeId>,
    items: Vec<SidebarItem>,
    selected_id: Option<String>,
    hovered_id: Option<String>,
    width: f32,
    header_content: Option<Box<dyn Element>>,
    footer_content: Option<Box<dyn Element>>,
    #[allow(clippy::type_complexity)]
    on_select: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            items: Vec::new(),
            selected_id: None,
            hovered_id: None,
            width: layout::SIDEBAR_WIDTH,
            header_content: None,
            footer_content: None,
            on_select: None,
        }
    }

    pub fn with_items(mut self, items: Vec<SidebarItem>) -> Self {
        self.items = items;
        self
    }

    pub fn with_selected(mut self, id: impl Into<String>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_header(mut self, content: Box<dyn Element>) -> Self {
        self.header_content = Some(content);
        self
    }

    pub fn with_footer(mut self, content: Box<dyn Element>) -> Self {
        self.footer_content = Some(content);
        self
    }

    pub fn with_on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(callback));
        self
    }

    fn get_item_y(&self, index: usize, base_y: f32) -> f32 {
        let header_height = if self.header_content.is_some() {
            sidebar::HEADER_HEIGHT
        } else {
            0.0
        };
        base_y + header_height + spacing::MD + (index as f32 * sidebar::ITEM_SPACING)
    }

    fn hit_test_item(&self, point: Point, bounds: Bounds) -> Option<usize> {
        let header_height = if self.header_content.is_some() {
            sidebar::HEADER_HEIGHT
        } else {
            0.0
        };
        let items_start_y = bounds.y() + header_height + spacing::MD;

        for (i, _) in self.items.iter().enumerate() {
            let item_y = items_start_y + (i as f32 * sidebar::ITEM_SPACING);
            let item_bounds = Bounds::new(
                bounds.x() + spacing::SM,
                item_y,
                self.width - spacing::SM * 2.0,
                sidebar::ITEM_HEIGHT,
            );
            if item_bounds.contains(point) {
                return Some(i);
            }
        }
        None
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Sidebar {
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
            size: taffy::Size {
                width: Dimension::length(self.width),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        // Draw sidebar background
        cx.fill_rect(cx.bounds, colors::SIDEBAR_BG);

        // Draw right border
        cx.fill_rect(
            Bounds::new(
                cx.bounds.x() + cx.bounds.width() - common::BORDER_THICKNESS,
                cx.bounds.y(),
                common::BORDER_THICKNESS,
                cx.bounds.height(),
            ),
            colors::BORDER,
        );

        let mut current_y = cx.bounds.y();

        // Draw header area if present (logo/brand)
        if self.header_content.is_some() {
            let header_height = sidebar::HEADER_HEIGHT;
            // Header background
            cx.fill_rect(
                Bounds::new(cx.bounds.x(), current_y, self.width, header_height),
                colors::SIDEBAR_BG,
            );

            // Draw brand text as placeholder
            cx.draw_text(
                "Narrative",
                Point::new(
                    cx.bounds.x() + spacing::LG,
                    current_y + sidebar::BRAND_OFFSET_Y,
                ),
                colors::ACCENT_PRIMARY,
                font_size::XL,
            );

            current_y += header_height;
        }

        // Draw items
        current_y += spacing::MD;

        for (i, item) in self.items.iter().enumerate() {
            let item_y = current_y + (i as f32 * sidebar::ITEM_SPACING);
            let item_bounds = Bounds::new(
                cx.bounds.x() + spacing::SM,
                item_y,
                self.width - spacing::SM * 2.0,
                sidebar::ITEM_HEIGHT,
            );

            let is_selected = self.selected_id.as_ref() == Some(&item.id);
            let is_hovered = self.hovered_id.as_ref() == Some(&item.id);

            // Item background
            let bg_color = if is_selected {
                colors::SIDEBAR_ITEM_ACTIVE
            } else if is_hovered {
                colors::SIDEBAR_ITEM_HOVER
            } else {
                Color::TRANSPARENT
            };

            if bg_color.a > 0.0 {
                cx.fill_rounded_rect(item_bounds, bg_color, radius::SM);
            }

            // Left accent for selected item
            if is_selected {
                cx.fill_rounded_rect(
                    Bounds::new(
                        cx.bounds.x(),
                        item_y + sidebar::ACCENT_OFFSET_Y,
                        sidebar::ACCENT_WIDTH,
                        sidebar::ACCENT_HEIGHT,
                    ),
                    colors::ACCENT_PRIMARY,
                    radius::XS,
                );
            }

            // Icon color
            let icon_color = if is_selected {
                colors::ACCENT_PRIMARY
            } else if is_hovered {
                colors::TEXT_PRIMARY
            } else {
                colors::TEXT_SECONDARY
            };

            // Draw icon placeholder (square for now)
            cx.fill_rounded_rect(
                Bounds::new(
                    item_bounds.x() + spacing::SM,
                    item_y + sidebar::ICON_OFFSET_Y,
                    sidebar::ICON_SIZE,
                    sidebar::ICON_SIZE,
                ),
                icon_color,
                radius::SM,
            );

            // Text color
            let text_color = if is_selected || is_hovered {
                colors::TEXT_PRIMARY
            } else {
                colors::TEXT_SECONDARY
            };

            // Draw label
            cx.draw_text(
                &item.label,
                Point::new(
                    item_bounds.x() + sidebar::LABEL_OFFSET_X,
                    item_y + sidebar::LABEL_OFFSET_Y,
                ),
                text_color,
                font_size::MD,
            );

            // Draw badge if present
            if let Some(ref badge) = item.badge {
                let badge_x = item_bounds.right() - sidebar::BADGE_OFFSET_X;
                cx.fill_rounded_rect(
                    Bounds::new(
                        badge_x,
                        item_y + sidebar::ICON_OFFSET_Y,
                        sidebar::BADGE_WIDTH,
                        sidebar::BADGE_HEIGHT,
                    ),
                    colors::ACCENT_MUTED,
                    sidebar::BADGE_RADIUS,
                );
                cx.draw_text(
                    badge,
                    Point::new(
                        badge_x + sidebar::BADGE_TEXT_OFFSET_X,
                        item_y + sidebar::BADGE_TEXT_OFFSET_Y,
                    ),
                    colors::TEXT_PRIMARY,
                    font_size::XS,
                );
            }
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        match event {
            InputEvent::MouseMove { position, .. } => {
                let old_hovered = self.hovered_id.clone();
                self.hovered_id = self
                    .hit_test_item(*position, bounds)
                    .map(|i| self.items[i].id.clone());
                old_hovered != self.hovered_id
            }
            InputEvent::MouseUp { position, .. } => {
                if let Some(index) = self.hit_test_item(*position, bounds) {
                    let item_id = self.items[index].id.clone();
                    self.selected_id = Some(item_id.clone());
                    if let Some(ref callback) = self.on_select {
                        callback(&item_id);
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
