//! Title screen UI component
//!
//! This component displays the main menu with:
//! - New Game
//! - Continue (only if save data exists)
//! - Load
//! - Settings
//! - Exit
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

/// Title screen menu item action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TitleScreenAction {
    /// Start new game
    NewGame,
    /// Continue from last save
    Continue,
    /// Load saved game
    Load,
    /// Open CG Gallery
    CgGallery,
    /// Open settings
    Settings,
    /// Exit game
    Exit,
}

/// Title screen menu item
#[derive(Debug, Clone)]
struct MenuItem {
    /// Menu item label
    label: &'static str,
    /// Menu item action
    action: TitleScreenAction,
    /// Whether this item is available
    enabled: bool,
}

/// Title screen element that displays the main menu
pub struct TitleScreenElement {
    /// Unique element ID
    id: ElementId,
    /// Taffy layout node
    layout_node: Option<NodeId>,
    /// Menu items
    menu_items: Vec<MenuItem>,
    /// Currently selected menu item index
    selected_index: usize,
    /// Whether a menu item has been confirmed
    action_confirmed: Option<TitleScreenAction>,
    /// Dirty flag to track if rendering needs update
    dirty: bool,
    /// Cached button bounds for click detection
    button_bounds: Vec<Bounds>,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl TitleScreenElement {
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
    const TITLE_FONT_SIZE: f32 = 48.0;
    /// Title offset from top
    const TITLE_OFFSET_Y: f32 = 100.0;

    /// Create a new title screen element
    pub fn new(has_continue: bool) -> Self {
        let menu_items = vec![
            MenuItem {
                label: "New Game",
                action: TitleScreenAction::NewGame,
                enabled: true,
            },
            MenuItem {
                label: "Continue",
                action: TitleScreenAction::Continue,
                enabled: has_continue,
            },
            MenuItem {
                label: "Load",
                action: TitleScreenAction::Load,
                enabled: true,
            },
            MenuItem {
                label: "CG Gallery",
                action: TitleScreenAction::CgGallery,
                enabled: true,
            },
            MenuItem {
                label: "Settings",
                action: TitleScreenAction::Settings,
                enabled: true,
            },
            MenuItem {
                label: "Exit",
                action: TitleScreenAction::Exit,
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
    pub fn confirmed_action(&self) -> Option<TitleScreenAction> {
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

        // Center vertically, with slight offset downward
        let start_y =
            container_bounds.origin.y + (container_bounds.size.height - total_height) / 2.0 + 50.0;
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

impl Element for TitleScreenElement {
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
        // Draw title
        let title = "Narrative Novel";
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

        let start_y = cx.bounds.origin.y + (cx.bounds.size.height - total_height) / 2.0 + 50.0;
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
    fn test_title_screen_creation_without_continue() {
        let screen = TitleScreenElement::new(false);

        // Should have 5 items (New Game, Load, CG Gallery, Settings, Exit) - Continue disabled
        assert_eq!(screen.menu_items.len(), 5);
        assert_eq!(screen.selected_index, 0);
        assert!(screen.confirmed_action().is_none());
    }

    #[test]
    fn test_title_screen_creation_with_continue() {
        let screen = TitleScreenElement::new(true);

        // Should have 6 items (New Game, Continue, Load, CG Gallery, Settings, Exit)
        assert_eq!(screen.menu_items.len(), 6);
        assert_eq!(screen.selected_index, 0);
    }

    #[test]
    fn test_selection_navigation() {
        let mut screen = TitleScreenElement::new(true);

        // Move down
        screen.select_next();
        assert_eq!(screen.selected_index, 1);

        // Move down again
        screen.select_next();
        assert_eq!(screen.selected_index, 2);

        // Move up
        screen.select_previous();
        assert_eq!(screen.selected_index, 1);

        // Move up to first
        screen.select_previous();
        assert_eq!(screen.selected_index, 0);

        // Try to move up past first (should stay at 0)
        screen.select_previous();
        assert_eq!(screen.selected_index, 0);
    }

    #[test]
    fn test_selection_navigation_boundary() {
        let mut screen = TitleScreenElement::new(true);

        // Move to last item (6 items total: New Game, Continue, Load, CG Gallery, Settings, Exit)
        for _ in 0..10 {
            screen.select_next();
        }
        assert_eq!(screen.selected_index, 5); // Last item (Exit) - index 5

        // Try to move down past last (should stay at last)
        screen.select_next();
        assert_eq!(screen.selected_index, 5);
    }

    #[test]
    fn test_confirm_selection() {
        let mut screen = TitleScreenElement::new(true);

        // Select New Game and confirm
        screen.confirm_selection();
        assert_eq!(screen.confirmed_action(), Some(TitleScreenAction::NewGame));

        // Reset confirmation
        screen.reset_confirmation();
        assert!(screen.confirmed_action().is_none());

        // Select Continue and confirm
        screen.select_next();
        screen.confirm_selection();
        assert_eq!(screen.confirmed_action(), Some(TitleScreenAction::Continue));
    }
}
