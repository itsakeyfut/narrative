//! Dropdown Menu Component
//!
//! A reusable dropdown menu component with support for:
//! - Menu items with labels and keyboard shortcuts
//! - Separators between item groups
//! - Hover highlighting
//! - Click handling

use crate::framework::Color;
use crate::framework::animation::AnimationContext;
use crate::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use crate::framework::input::{InputEvent, MouseButton};
use crate::framework::layout::{Bounds, Point};
use crate::theme::{colors, dropdown, font_size, layout, radius, spacing, typography};
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

/// Callback type for menu item clicks
type ItemClickCallback = Box<dyn Fn(&str) + Send + Sync>;
/// Callback type for menu close events
type CloseCallback = Box<dyn Fn() + Send + Sync>;

/// A single item in a dropdown menu
#[derive(Clone, Debug)]
pub struct DropdownItem {
    /// Unique identifier for this item
    pub id: String,
    /// Display label
    pub label: String,
    /// Optional keyboard shortcut display
    pub shortcut: Option<String>,
    /// Whether this is a separator (visual divider)
    pub is_separator: bool,
    /// Whether this item is disabled
    pub disabled: bool,
}

impl DropdownItem {
    /// Create a new menu item
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            is_separator: false,
            disabled: false,
        }
    }

    /// Add a keyboard shortcut display
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Create a separator item
    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            shortcut: None,
            is_separator: true,
            disabled: false,
        }
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Dropdown menu state for tracking open menus
#[derive(Clone, Debug, Default)]
pub struct DropdownState {
    /// Whether the dropdown is visible
    pub is_open: bool,
    /// Position where the dropdown should appear
    pub anchor_bounds: Bounds,
    /// Currently hovered item index
    pub hovered_item: Option<usize>,
    /// The items to display
    pub items: Vec<DropdownItem>,
    /// Calculated bounds of the dropdown
    pub dropdown_bounds: Bounds,
}

impl DropdownState {
    /// Create a new dropdown state
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the dropdown at the specified anchor position
    pub fn open(&mut self, anchor_bounds: Bounds, items: Vec<DropdownItem>) {
        self.is_open = true;
        self.anchor_bounds = anchor_bounds;
        self.items = items;
        self.hovered_item = None;
        self.dropdown_bounds = self.calculate_bounds();
    }

    /// Close the dropdown
    pub fn close(&mut self) {
        self.is_open = false;
        self.hovered_item = None;
    }

    /// Calculate dropdown bounds based on anchor and items
    fn calculate_bounds(&self) -> Bounds {
        if self.items.is_empty() {
            return Bounds::ZERO;
        }

        // Calculate width based on longest item
        let mut max_width: f32 = dropdown::MIN_WIDTH;
        for item in &self.items {
            if !item.is_separator {
                let label_width = item.label.len() as f32 * layout::MENU_CHAR_WIDTH;
                let shortcut_width = item
                    .shortcut
                    .as_ref()
                    .map(|s| {
                        s.len() as f32 * dropdown::SHORTCUT_CHAR_WIDTH + dropdown::SHORTCUT_PADDING
                    })
                    .unwrap_or(0.0);
                let total_width = label_width + shortcut_width + spacing::LG * 2.0;
                max_width = max_width.max(total_width);
            }
        }

        // Calculate height
        let mut total_height = spacing::XS * 2.0; // Top and bottom padding
        for item in &self.items {
            if item.is_separator {
                total_height += dropdown::SEPARATOR_HEIGHT;
            } else {
                total_height += dropdown::ITEM_HEIGHT;
            }
        }

        Bounds::new(
            self.anchor_bounds.x(),
            self.anchor_bounds.bottom(),
            max_width,
            total_height,
        )
    }

    /// Get the item bounds at the given index
    fn get_item_bounds(&self, index: usize) -> Option<Bounds> {
        if index >= self.items.len() {
            return None;
        }

        let mut y = self.dropdown_bounds.y() + spacing::XS;
        for (i, item) in self.items.iter().enumerate() {
            let height = if item.is_separator {
                dropdown::SEPARATOR_HEIGHT
            } else {
                dropdown::ITEM_HEIGHT
            };

            if i == index && !item.is_separator {
                return Some(Bounds::new(
                    self.dropdown_bounds.x() + dropdown::ITEM_PADDING_X,
                    y,
                    self.dropdown_bounds.width() - dropdown::ITEM_PADDING_X * 2.0,
                    height,
                ));
            }

            y += height;
        }

        None
    }

    /// Find item index at point
    fn item_at_point(&self, point: Point) -> Option<usize> {
        if !self.dropdown_bounds.contains(point) {
            return None;
        }

        let mut y = self.dropdown_bounds.y() + spacing::XS;
        for (i, item) in self.items.iter().enumerate() {
            let height = if item.is_separator {
                dropdown::SEPARATOR_HEIGHT
            } else {
                dropdown::ITEM_HEIGHT
            };

            let item_bounds = Bounds::new(
                self.dropdown_bounds.x() + dropdown::ITEM_PADDING_X,
                y,
                self.dropdown_bounds.width() - dropdown::ITEM_PADDING_X * 2.0,
                height,
            );

            if item_bounds.contains(point) && !item.is_separator && !item.disabled {
                return Some(i);
            }

            y += height;
        }

        None
    }
}

/// Dropdown menu overlay component
///
/// This component should be rendered last (as an overlay) to ensure
/// it appears on top of all other UI elements.
pub struct DropdownMenu {
    id: ElementId,
    layout_node: Option<NodeId>,
    state: DropdownState,
    /// Callback when an item is clicked (receives item id)
    on_item_click: Option<ItemClickCallback>,
    /// Callback when dropdown is closed
    on_close: Option<CloseCallback>,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl DropdownMenu {
    /// Create a new dropdown menu
    pub fn new() -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            state: DropdownState::new(),
            on_item_click: None,
            on_close: None,
            animation_context: AnimationContext::default(),
            animations_enabled: None,
        }
    }

    /// Set item click callback
    pub fn with_on_item_click<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_item_click = Some(Box::new(callback));
        self
    }

    /// Set close callback
    pub fn with_on_close<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_close = Some(Box::new(callback));
        self
    }

    /// Set the animation context
    ///
    /// This allows the dropdown to respect global animation settings for future animations.
    pub fn with_animation_context(mut self, context: AnimationContext) -> Self {
        self.animation_context = context;
        self
    }

    /// Set component-specific animation override
    ///
    /// This allows disabling animations for this specific dropdown
    /// even when global animations are enabled, or vice versa.
    pub fn with_animations_enabled(mut self, enabled: impl Into<Option<bool>>) -> Self {
        self.animations_enabled = enabled.into();
        self
    }

    /// Open the dropdown
    pub fn open(&mut self, anchor_bounds: Bounds, items: Vec<DropdownItem>) {
        self.state.open(anchor_bounds, items);
    }

    /// Close the dropdown
    pub fn close(&mut self) {
        self.state.close();
        if let Some(ref callback) = self.on_close {
            callback();
        }
    }

    /// Check if dropdown is open
    pub fn is_open(&self) -> bool {
        self.state.is_open
    }

    /// Get the dropdown bounds
    pub fn bounds(&self) -> Bounds {
        self.state.dropdown_bounds
    }

    /// Update hovered item based on mouse position
    pub fn update_hover(&mut self, position: Point) {
        if self.state.is_open {
            self.state.hovered_item = self.state.item_at_point(position);
        }
    }

    /// Handle click at position, returns clicked item id if any
    pub fn handle_click(&mut self, position: Point) -> Option<String> {
        if !self.state.is_open {
            return None;
        }

        if let Some(idx) = self.state.item_at_point(position) {
            let item_id = self.state.items[idx].id.clone();
            self.close();
            return Some(item_id);
        }

        // Click outside dropdown closes it
        if !self.state.dropdown_bounds.contains(position) {
            self.close();
        }

        None
    }

    /// Paint the dropdown (call this last to render as overlay)
    pub fn paint_overlay(&self, cx: &mut PaintContext) {
        if !self.state.is_open {
            return;
        }

        let bounds = self.state.dropdown_bounds;

        // Shadow (multiple layers for softer effect)
        for i in 1..=4 {
            let offset = i as f32 * 2.0;
            let alpha = 0.15 - (i as f32 * 0.03);
            cx.fill_rounded_rect(
                Bounds::new(
                    bounds.x() + offset,
                    bounds.y() + offset,
                    bounds.width(),
                    bounds.height(),
                ),
                Color::new(0.0, 0.0, 0.0, alpha),
                radius::SM,
            );
        }

        // Background (solid, opaque)
        cx.fill_rounded_rect(bounds, colors::BG_PANEL, radius::SM);

        // Border
        let border_color = colors::BORDER_LIGHT;
        // Top
        cx.fill_rect(
            Bounds::new(bounds.x(), bounds.y(), bounds.width(), 1.0),
            border_color,
        );
        // Bottom
        cx.fill_rect(
            Bounds::new(bounds.x(), bounds.bottom() - 1.0, bounds.width(), 1.0),
            border_color,
        );
        // Left
        cx.fill_rect(
            Bounds::new(bounds.x(), bounds.y(), 1.0, bounds.height()),
            border_color,
        );
        // Right
        cx.fill_rect(
            Bounds::new(bounds.right() - 1.0, bounds.y(), 1.0, bounds.height()),
            border_color,
        );

        // Draw items
        let mut y = bounds.y() + spacing::XS;
        for (i, item) in self.state.items.iter().enumerate() {
            if item.is_separator {
                // Separator line
                let sep_y = y + dropdown::SEPARATOR_HEIGHT / 2.0;
                cx.fill_rect(
                    Bounds::new(
                        bounds.x() + spacing::SM,
                        sep_y,
                        bounds.width() - spacing::SM * 2.0,
                        1.0,
                    ),
                    colors::BORDER,
                );
                y += dropdown::SEPARATOR_HEIGHT;
            } else {
                let item_bounds = Bounds::new(
                    bounds.x() + dropdown::ITEM_PADDING_X,
                    y,
                    bounds.width() - dropdown::ITEM_PADDING_X * 2.0,
                    dropdown::ITEM_HEIGHT,
                );

                // Hover highlight
                let is_hovered = self.state.hovered_item == Some(i);
                if is_hovered && !item.disabled {
                    cx.fill_rounded_rect(item_bounds, colors::ACCENT_PRIMARY, radius::XS);
                }

                // Label
                let text_color = if item.disabled {
                    colors::TEXT_MUTED
                } else if is_hovered {
                    colors::BG_DARKEST
                } else {
                    colors::TEXT_PRIMARY
                };

                let text_y =
                    y + dropdown::ITEM_HEIGHT / 2.0 + font_size::LG * typography::BASELINE_OFFSET;
                cx.draw_text(
                    &item.label,
                    Point::new(item_bounds.x() + spacing::MD, text_y),
                    text_color,
                    font_size::LG,
                );

                // Shortcut
                if let Some(ref shortcut) = item.shortcut {
                    let shortcut_color = if item.disabled {
                        colors::TEXT_MUTED
                    } else if is_hovered {
                        colors::BG_DARK
                    } else {
                        colors::TEXT_MUTED
                    };
                    let shortcut_width = shortcut.len() as f32 * dropdown::SHORTCUT_CHAR_WIDTH;
                    cx.draw_text(
                        shortcut,
                        Point::new(item_bounds.right() - shortcut_width - spacing::MD, text_y),
                        shortcut_color,
                        font_size::MD,
                    );
                }

                y += dropdown::ITEM_HEIGHT;
            }
        }
    }
}

impl Default for DropdownMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for DropdownMenu {
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

        // Dropdown doesn't participate in normal layout
        // It's rendered as an overlay at absolute position
        Style {
            display: Display::None,
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        // Use paint_overlay instead for proper z-ordering
        self.paint_overlay(cx);
    }

    fn handle_event(&mut self, event: &InputEvent, _bounds: Bounds) -> bool {
        if !self.state.is_open {
            return false;
        }

        match event {
            InputEvent::MouseMove { position, .. } => {
                self.update_hover(*position);
                self.state.dropdown_bounds.contains(*position)
            }
            InputEvent::MouseDown {
                button: MouseButton::Left,
                position,
                ..
            } => {
                if let Some(item_id) = self.handle_click(*position) {
                    if let Some(ref callback) = self.on_item_click {
                        callback(&item_id);
                    }
                    return true;
                }
                self.state.dropdown_bounds.contains(*position)
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

/// Menu bar state for managing multiple dropdown menus
#[derive(Clone, Debug, Default)]
pub struct MenuBarState {
    /// Currently open menu index
    pub open_menu: Option<usize>,
    /// Currently hovered menu index
    pub hovered_menu: Option<usize>,
}

impl MenuBarState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open a specific menu
    pub fn open(&mut self, index: usize) {
        self.open_menu = Some(index);
    }

    /// Close the current menu
    pub fn close(&mut self) {
        self.open_menu = None;
    }

    /// Check if any menu is open
    pub fn is_open(&self) -> bool {
        self.open_menu.is_some()
    }

    /// Toggle a menu (close if open, open if closed)
    pub fn toggle(&mut self, index: usize) {
        if self.open_menu == Some(index) {
            self.close();
        } else {
            self.open(index);
        }
    }

    /// Switch to different menu if one is already open
    pub fn switch_if_open(&mut self, index: usize) {
        if self.open_menu.is_some() {
            self.open(index);
        }
    }
}

/// Menu definition for a top-level menu
#[derive(Clone, Debug)]
pub struct MenuDefinition {
    /// Menu label (displayed in menu bar)
    pub label: String,
    /// Menu items
    pub items: Vec<DropdownItem>,
}

impl MenuDefinition {
    /// Create a new menu definition
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
        }
    }

    /// Add an item to the menu
    pub fn with_item(mut self, item: DropdownItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add a separator
    pub fn with_separator(mut self) -> Self {
        self.items.push(DropdownItem::separator());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropdown_item_new() {
        let item = DropdownItem::new("test_id", "Test Label");
        assert_eq!(item.id, "test_id");
        assert_eq!(item.label, "Test Label");
        assert!(item.shortcut.is_none());
        assert!(!item.is_separator);
        assert!(!item.disabled);
    }

    #[test]
    fn test_dropdown_item_with_shortcut() {
        let item = DropdownItem::new("save", "Save").with_shortcut("Ctrl+S");
        assert_eq!(item.shortcut, Some("Ctrl+S".to_string()));
    }

    #[test]
    fn test_dropdown_item_separator() {
        let sep = DropdownItem::separator();
        assert!(sep.is_separator);
        assert!(sep.id.is_empty());
    }

    #[test]
    fn test_dropdown_state_initial() {
        let state = DropdownState::new();
        assert!(!state.is_open);
        assert!(state.items.is_empty());
        assert!(state.hovered_item.is_none());
    }

    #[test]
    fn test_dropdown_state_open_close() {
        let mut state = DropdownState::new();
        let anchor = Bounds::new(100.0, 50.0, 80.0, 30.0);
        let items = vec![
            DropdownItem::new("item1", "Item 1"),
            DropdownItem::new("item2", "Item 2"),
        ];

        state.open(anchor, items);
        assert!(state.is_open);
        assert_eq!(state.items.len(), 2);

        state.close();
        assert!(!state.is_open);
        assert!(state.hovered_item.is_none());
    }

    #[test]
    fn test_dropdown_state_calculate_bounds_empty() {
        let state = DropdownState::new();
        let bounds = state.calculate_bounds();
        assert_eq!(bounds, Bounds::ZERO);
    }

    #[test]
    fn test_dropdown_state_calculate_bounds_with_items() {
        let mut state = DropdownState::new();
        state.anchor_bounds = Bounds::new(100.0, 50.0, 80.0, 30.0);
        state.items = vec![
            DropdownItem::new("item1", "Item 1"),
            DropdownItem::separator(),
            DropdownItem::new("item2", "Item 2"),
        ];

        let bounds = state.calculate_bounds();
        assert!(bounds.width() >= dropdown::MIN_WIDTH);
        assert!(bounds.height() > 0.0);
        assert_eq!(bounds.x(), 100.0);
        assert_eq!(bounds.y(), 80.0); // anchor bottom = 50 + 30
    }

    #[test]
    fn test_dropdown_state_item_at_point() {
        let mut state = DropdownState::new();
        state.anchor_bounds = Bounds::new(0.0, 0.0, 100.0, 30.0);
        state.items = vec![
            DropdownItem::new("item1", "Item 1"),
            DropdownItem::new("item2", "Item 2"),
        ];
        state.dropdown_bounds = state.calculate_bounds();

        // Point inside first item
        let point_in_first = Point::new(50.0, state.dropdown_bounds.y() + 15.0);
        assert_eq!(state.item_at_point(point_in_first), Some(0));

        // Point outside dropdown
        let point_outside = Point::new(500.0, 500.0);
        assert_eq!(state.item_at_point(point_outside), None);
    }

    #[test]
    fn test_menu_bar_state() {
        let mut state = MenuBarState::new();
        assert!(!state.is_open());
        assert!(state.open_menu.is_none());

        state.open(0);
        assert!(state.is_open());
        assert_eq!(state.open_menu, Some(0));

        state.toggle(0);
        assert!(!state.is_open());

        state.toggle(1);
        assert_eq!(state.open_menu, Some(1));

        state.switch_if_open(2);
        assert_eq!(state.open_menu, Some(2));

        state.close();
        assert!(!state.is_open());
    }

    #[test]
    fn test_menu_definition() {
        let menu = MenuDefinition::new("File")
            .with_item(DropdownItem::new("new", "New"))
            .with_separator()
            .with_item(DropdownItem::new("exit", "Exit"));

        assert_eq!(menu.label, "File");
        assert_eq!(menu.items.len(), 3);
        assert!(!menu.items[0].is_separator);
        assert!(menu.items[1].is_separator);
    }
}
