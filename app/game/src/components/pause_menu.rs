//! Pause menu UI component
//!
//! This component displays the in-game pause menu with:
//! - Resume
//! - Save
//! - Load
//! - Settings
//! - Return to Title
//!
//! Supports arrow key navigation and Enter/Space for confirmation.

use narrative_gui::Point;
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use narrative_gui::framework::input::{InputEvent, KeyCode};
use narrative_gui::framework::layout::Bounds;
use narrative_gui::theme::colors;
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

/// Pause menu item action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PauseMenuAction {
    /// Resume game
    Resume,
    /// Save game
    Save,
    /// Load saved game
    Load,
    /// Open settings
    Settings,
    /// Return to title screen
    Title,
}

/// Pause menu item
#[derive(Debug, Clone)]
struct MenuItem {
    /// Menu item label
    label: &'static str,
    /// Menu item action
    action: PauseMenuAction,
    /// Whether this item is available
    enabled: bool,
}

/// Pause menu element that displays the in-game menu
pub struct PauseMenuElement {
    /// Unique element ID
    id: ElementId,
    /// Taffy layout node
    layout_node: Option<NodeId>,
    /// Menu items
    menu_items: Vec<MenuItem>,
    /// Currently selected menu item index
    selected_index: usize,
    /// Whether a menu item has been confirmed
    action_confirmed: Option<PauseMenuAction>,
    /// Dirty flag to track if rendering needs update
    dirty: bool,
    /// Cached button bounds for click detection
    button_bounds: Vec<Bounds>,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl PauseMenuElement {
    /// Default button width
    const BUTTON_WIDTH: f32 = 400.0;
    /// Default button height
    const BUTTON_HEIGHT: f32 = 60.0;
    /// Spacing between buttons
    const BUTTON_SPACING: f32 = 16.0;
    /// Button corner radius
    const CORNER_RADIUS: f32 = 8.0;
    /// Button font size
    const FONT_SIZE: f32 = 24.0;
    /// Title font size
    const TITLE_FONT_SIZE: f32 = 36.0;
    /// Title offset from top
    const TITLE_OFFSET_Y: f32 = 80.0;
    /// Background overlay alpha
    const OVERLAY_ALPHA: f32 = 0.7;

    /// Create a new pause menu element
    pub fn new() -> Self {
        let menu_items = vec![
            MenuItem {
                label: "Resume",
                action: PauseMenuAction::Resume,
                enabled: true,
            },
            MenuItem {
                label: "Save",
                action: PauseMenuAction::Save,
                enabled: true,
            },
            MenuItem {
                label: "Load",
                action: PauseMenuAction::Load,
                enabled: true,
            },
            MenuItem {
                label: "Settings",
                action: PauseMenuAction::Settings,
                enabled: true,
            },
            MenuItem {
                label: "Title",
                action: PauseMenuAction::Title,
                enabled: true,
            },
        ];

        // Filter out disabled items
        let enabled_items: Vec<MenuItem> =
            menu_items.into_iter().filter(|item| item.enabled).collect();

        let button_bounds = vec![Bounds::default(); enabled_items.len()];

        Self {
            id: ElementId::new(),
            layout_node: None,
            menu_items: enabled_items,
            selected_index: 0,
            action_confirmed: None,
            dirty: true,
            button_bounds,
            animation_context: AnimationContext::default(),
            animations_enabled: None,
        }
    }

    /// Set the animation context
    pub fn with_animation_context(mut self, context: AnimationContext) -> Self {
        self.animation_context = context;
        self
    }

    /// Set component-specific animation override
    pub fn with_animations_enabled(mut self, enabled: impl Into<Option<bool>>) -> Self {
        self.animations_enabled = enabled.into();
        self
    }

    /// Get the confirmed action, if any
    pub fn confirmed_action(&self) -> Option<PauseMenuAction> {
        self.action_confirmed
    }

    /// Reset the confirmation state
    pub fn reset_confirmation(&mut self) {
        self.action_confirmed = None;
    }

    /// Move selection up
    fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.dirty = true;
        }
    }

    /// Move selection down
    fn select_next(&mut self) {
        if self.selected_index < self.menu_items.len().saturating_sub(1) {
            self.selected_index = self.selected_index.saturating_add(1);
            self.dirty = true;
        }
    }

    /// Confirm the current selection
    fn confirm_selection(&mut self) {
        if let Some(item) = self.menu_items.get(self.selected_index) {
            self.action_confirmed = Some(item.action);
            self.dirty = true;
        }
    }

    /// Calculate button bounds for layout
    fn calculate_button_bounds(&mut self, container_bounds: Bounds) {
        let total_height = (Self::BUTTON_HEIGHT * self.menu_items.len() as f32)
            + (Self::BUTTON_SPACING * (self.menu_items.len().saturating_sub(1)) as f32);

        // Center vertically
        let start_y =
            container_bounds.origin.y + (container_bounds.size.height - total_height) / 2.0;
        let start_x =
            container_bounds.origin.x + (container_bounds.size.width - Self::BUTTON_WIDTH) / 2.0;

        for i in 0..self.menu_items.len() {
            let y = start_y + (i as f32 * (Self::BUTTON_HEIGHT + Self::BUTTON_SPACING));
            self.button_bounds[i] = Bounds {
                origin: Point::new(start_x, y),
                size: narrative_gui::Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
            };
        }
    }
}

impl Default for PauseMenuElement {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for PauseMenuElement {
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

        // Take up full available space
        taffy::Style {
            size: taffy::geometry::Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: Some(AlignItems::Center),
            justify_content: Some(JustifyContent::Center),
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        // Draw semi-transparent background overlay
        let overlay_color = narrative_gui::Color::new(0.0, 0.0, 0.0, Self::OVERLAY_ALPHA);
        cx.fill_rect(cx.bounds, overlay_color);

        // Draw title
        let title = "Pause Menu";
        // Rough estimate for centering (TODO: use proper text measurement)
        let title_width = title.len() as f32 * Self::TITLE_FONT_SIZE * 0.6;
        let title_x = cx.bounds.origin.x + (cx.bounds.size.width - title_width) / 2.0;
        let title_y = cx.bounds.origin.y + Self::TITLE_OFFSET_Y;

        cx.draw_text(
            title,
            Point::new(title_x, title_y),
            colors::TEXT_PRIMARY,
            Self::TITLE_FONT_SIZE,
        );

        // Calculate button layout
        let total_height = (Self::BUTTON_HEIGHT * self.menu_items.len() as f32)
            + (Self::BUTTON_SPACING * (self.menu_items.len().saturating_sub(1)) as f32);

        let start_y = cx.bounds.origin.y + (cx.bounds.size.height - total_height) / 2.0;
        let start_x = cx.bounds.origin.x + (cx.bounds.size.width - Self::BUTTON_WIDTH) / 2.0;

        // Draw each menu button
        for (i, item) in self.menu_items.iter().enumerate() {
            let y = start_y + (i as f32 * (Self::BUTTON_HEIGHT + Self::BUTTON_SPACING));
            let button_bounds = Bounds {
                origin: Point::new(start_x, y),
                size: narrative_gui::Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
            };

            // Determine button appearance
            let is_selected = i == self.selected_index;
            let bg_color = if is_selected {
                colors::ACCENT_PRIMARY
            } else {
                colors::CARD_BG
            };
            let text_color = if is_selected {
                colors::BG_DARKEST
            } else {
                colors::TEXT_PRIMARY
            };

            // Draw button background
            cx.fill_rounded_rect(button_bounds, bg_color, Self::CORNER_RADIUS);

            // Draw button border for non-selected items
            if !is_selected {
                cx.stroke_rect(button_bounds, colors::BORDER_LIGHT, 1.0);
            }

            // Draw menu item text (centered)
            // TODO: Use cosmic-text's Buffer::shape() for accurate text width measurement
            let text_width = item.label.len() as f32 * Self::FONT_SIZE * 0.6;
            let text_x = button_bounds.origin.x + (Self::BUTTON_WIDTH - text_width) / 2.0;
            let text_y =
                button_bounds.origin.y + (Self::BUTTON_HEIGHT + Self::FONT_SIZE * 0.8) / 2.0;

            cx.draw_text(
                item.label,
                Point::new(text_x, text_y),
                text_color,
                Self::FONT_SIZE,
            );
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        // Update button bounds for click detection
        self.calculate_button_bounds(bounds);

        match event {
            InputEvent::KeyDown { key, .. } => match key {
                KeyCode::Up => {
                    self.select_previous();
                    true
                }
                KeyCode::Down => {
                    self.select_next();
                    true
                }
                KeyCode::Enter | KeyCode::Space => {
                    self.confirm_selection();
                    true
                }
                KeyCode::Escape => {
                    // ESC key acts as Resume
                    self.action_confirmed = Some(PauseMenuAction::Resume);
                    self.dirty = true;
                    true
                }
                _ => false,
            },
            InputEvent::MouseDown { position, .. } => {
                // Check if click is on any button
                for (i, button_bound) in self.button_bounds.iter().enumerate() {
                    if button_bound.contains(*position) {
                        self.selected_index = i;
                        self.confirm_selection();
                        self.dirty = true;
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn tick(&mut self, delta: Duration) -> bool {
        let _ = delta;
        // Reset dirty flag
        let was_dirty = self.dirty;
        self.dirty = false;
        was_dirty
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
    fn test_pause_menu_creation() {
        let menu = PauseMenuElement::new();

        // Should have 5 items (Resume, Save, Load, Settings, Title)
        assert_eq!(menu.menu_items.len(), 5);
        assert_eq!(menu.selected_index, 0);
        assert!(menu.confirmed_action().is_none());
    }

    #[test]
    fn test_selection_navigation() {
        let mut menu = PauseMenuElement::new();

        // Move down
        menu.select_next();
        assert_eq!(menu.selected_index, 1);

        // Move down again
        menu.select_next();
        assert_eq!(menu.selected_index, 2);

        // Move up
        menu.select_previous();
        assert_eq!(menu.selected_index, 1);

        // Move up to first
        menu.select_previous();
        assert_eq!(menu.selected_index, 0);

        // Try to move up past first (should stay at 0)
        menu.select_previous();
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn test_selection_navigation_boundary() {
        let mut menu = PauseMenuElement::new();

        // Move to last item (5 items total: Resume, Save, Load, Settings, Title)
        for _ in 0..10 {
            menu.select_next();
        }
        assert_eq!(menu.selected_index, 4); // Last item (Title) - index 4

        // Try to move down past last (should stay at last)
        menu.select_next();
        assert_eq!(menu.selected_index, 4);
    }

    #[test]
    fn test_confirm_selection() {
        let mut menu = PauseMenuElement::new();

        // Select Resume and confirm
        menu.confirm_selection();
        assert_eq!(menu.confirmed_action(), Some(PauseMenuAction::Resume));

        // Reset confirmation
        menu.reset_confirmation();
        assert!(menu.confirmed_action().is_none());

        // Select Settings and confirm (Resume, Save, Load, Settings - index 3)
        menu.select_next(); // Save (index 1)
        menu.select_next(); // Load (index 2)
        menu.select_next(); // Settings (index 3)
        menu.confirm_selection();
        assert_eq!(menu.confirmed_action(), Some(PauseMenuAction::Settings));
    }

    #[test]
    fn test_escape_key_acts_as_resume() {
        use narrative_gui::framework::input::Modifiers;

        let mut menu = PauseMenuElement::new();
        let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);

        let event = InputEvent::KeyDown {
            key: KeyCode::Escape,
            modifiers: Modifiers::none(),
        };

        assert!(menu.handle_event(&event, bounds));
        assert_eq!(menu.confirmed_action(), Some(PauseMenuAction::Resume));
    }
}
