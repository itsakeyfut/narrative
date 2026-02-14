//! Quick menu UI component
//!
//! This component displays a horizontal quick access menu near the dialogue box with buttons for:
//! - Skip mode toggle
//! - Auto mode toggle
//! - Backlog viewer
//! - Quick save
//! - Pause menu
//!
//! Buttons show their active/inactive state with visual feedback.

use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use narrative_gui::framework::input::InputEvent;
use narrative_gui::framework::layout::Bounds;
use narrative_gui::theme::colors;
use narrative_gui::{Color, Point, Size};
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use taffy::NodeId;

/// Quick menu button action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuickMenuAction {
    /// Toggle skip mode
    ToggleSkip,
    /// Toggle auto mode
    ToggleAuto,
    /// Open backlog
    OpenBacklog,
    /// Quick save
    QuickSave,
    /// Open pause menu
    OpenMenu,
}

/// Quick menu button state
#[derive(Debug, Clone)]
struct QuickMenuButton {
    /// Button label
    label: &'static str,
    /// Button action
    action: QuickMenuAction,
    /// Whether this button is in active state (for toggles)
    is_active: bool,
    /// Whether this button is enabled
    enabled: bool,
}

/// Quick menu element that displays quick access buttons
pub struct QuickMenuElement {
    /// Unique element ID
    id: ElementId,
    /// Taffy layout node
    layout_node: Option<NodeId>,
    /// Menu buttons
    buttons: Vec<QuickMenuButton>,
    /// Currently hovered button index
    hovered_index: Option<usize>,
    /// Action to perform (set when button is clicked)
    pending_action: Option<QuickMenuAction>,
    /// Dirty flag to track if rendering needs update
    dirty: bool,
    /// Cached button bounds for click detection (absolute window coordinates, updated in paint)
    button_bounds: Arc<Mutex<Vec<Bounds>>>,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Whether skip mode is active
    skip_active: bool,
    /// Whether auto mode is active
    auto_active: bool,
}

impl QuickMenuElement {
    /// Button width
    const BUTTON_WIDTH: f32 = 80.0;
    /// Button height
    const BUTTON_HEIGHT: f32 = 40.0;
    /// Spacing between buttons
    const BUTTON_SPACING: f32 = 8.0;
    /// Button corner radius
    const CORNER_RADIUS: f32 = 6.0;
    /// Button font size
    const FONT_SIZE: f32 = 14.0;
    /// Padding around the menu
    const MENU_PADDING: f32 = 12.0;
    /// Background alpha
    const BG_ALPHA: f32 = 0.85;

    /// Create a new quick menu element
    pub fn new() -> Self {
        let buttons = vec![
            QuickMenuButton {
                label: "Skip",
                action: QuickMenuAction::ToggleSkip,
                is_active: false,
                enabled: true,
            },
            QuickMenuButton {
                label: "Auto",
                action: QuickMenuAction::ToggleAuto,
                is_active: false,
                enabled: true,
            },
            QuickMenuButton {
                label: "Log",
                action: QuickMenuAction::OpenBacklog,
                is_active: false,
                enabled: true,
            },
            QuickMenuButton {
                label: "Save",
                action: QuickMenuAction::QuickSave,
                is_active: false,
                enabled: true,
            },
            QuickMenuButton {
                label: "Menu",
                action: QuickMenuAction::OpenMenu,
                is_active: false,
                enabled: true,
            },
        ];

        let button_bounds = Arc::new(Mutex::new(vec![Bounds::default(); buttons.len()]));

        Self {
            id: ElementId::new(),
            layout_node: None,
            buttons,
            hovered_index: None,
            pending_action: None,
            dirty: true,
            button_bounds,
            animation_context: AnimationContext::default(),
            skip_active: false,
            auto_active: false,
        }
    }

    /// Set the animation context
    pub fn with_animation_context(mut self, context: AnimationContext) -> Self {
        self.animation_context = context;
        self
    }

    /// Update skip mode state
    pub fn set_skip_active(&mut self, active: bool) {
        if self.skip_active != active {
            self.skip_active = active;
            if let Some(button) = self.buttons.first_mut() {
                button.is_active = active;
            }
            self.dirty = true;
        }
    }

    /// Update auto mode state
    pub fn set_auto_active(&mut self, active: bool) {
        if self.auto_active != active {
            self.auto_active = active;
            if let Some(button) = self.buttons.get_mut(1) {
                button.is_active = active;
            }
            self.dirty = true;
        }
    }

    /// Get the pending action, if any
    pub fn pending_action(&self) -> Option<QuickMenuAction> {
        self.pending_action
    }

    /// Clear the pending action
    pub fn clear_pending_action(&mut self) {
        self.pending_action = None;
    }

    /// Calculate button bounds for layout (used in tests)
    #[cfg(test)]
    fn calculate_button_bounds(&mut self, container_bounds: Bounds) {
        let total_width = (Self::BUTTON_WIDTH * self.buttons.len() as f32)
            + (Self::BUTTON_SPACING * (self.buttons.len().saturating_sub(1)) as f32);

        // Right-align the menu
        let start_x = container_bounds.origin.x + container_bounds.size.width
            - total_width
            - Self::MENU_PADDING;
        let start_y = container_bounds.origin.y + Self::MENU_PADDING;

        if let Ok(mut bounds_vec) = self.button_bounds.lock() {
            bounds_vec.clear();
            for i in 0..self.buttons.len() {
                let x = start_x + (i as f32 * (Self::BUTTON_WIDTH + Self::BUTTON_SPACING));
                bounds_vec.push(Bounds {
                    origin: Point::new(x, start_y),
                    size: Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
                });
            }
        }
    }

    /// Get button color based on state
    fn get_button_color(&self, button: &QuickMenuButton, is_hovered: bool) -> Color {
        if button.is_active {
            // Active state - use accent color
            if is_hovered {
                // Slightly lighter version of accent color for hover
                Color::new(0.1, 0.9, 0.8, 1.0)
            } else {
                colors::ACCENT_PRIMARY
            }
        } else if is_hovered {
            // Hovered but not active
            colors::BG_HOVER
        } else {
            // Default state
            colors::CARD_BG
        }
    }

    /// Get button text color based on state
    fn get_text_color(&self, button: &QuickMenuButton) -> Color {
        if button.is_active {
            colors::BG_DARKEST
        } else {
            colors::TEXT_PRIMARY
        }
    }
}

impl Default for QuickMenuElement {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for QuickMenuElement {
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

        // Calculate total size needed
        let _total_width = (Self::BUTTON_WIDTH * self.buttons.len() as f32)
            + (Self::BUTTON_SPACING * (self.buttons.len().saturating_sub(1)) as f32)
            + (Self::MENU_PADDING * 2.0);
        let total_height = Self::BUTTON_HEIGHT + (Self::MENU_PADDING * 2.0);

        // Position above dialogue box
        // TODO: Consider moving these to DialogueBoxConfig or GameConfig for better maintainability
        // Currently hardcoded to match DialogueBoxConfig::default().height (200px)
        const DIALOGUE_BOX_HEIGHT: f32 = 200.0;
        const MENU_GAP: f32 = 8.0;

        taffy::Style {
            size: Size {
                width: Dimension::percent(1.0), // Full width for alignment
                height: Dimension::length(total_height),
            },
            position: Position::Absolute,
            inset: Rect {
                left: LengthPercentageAuto::length(0.0),
                right: LengthPercentageAuto::length(0.0),
                bottom: LengthPercentageAuto::length(DIALOGUE_BOX_HEIGHT + MENU_GAP),
                top: LengthPercentageAuto::auto(),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        // Draw semi-transparent background for the entire menu
        let bg_color = Color::new(
            colors::BG_PANEL.r,
            colors::BG_PANEL.g,
            colors::BG_PANEL.b,
            Self::BG_ALPHA,
        );
        cx.fill_rounded_rect(cx.bounds, bg_color, Self::CORNER_RADIUS);

        // Calculate button positions (absolute window coordinates)
        let total_width = (Self::BUTTON_WIDTH * self.buttons.len() as f32)
            + (Self::BUTTON_SPACING * (self.buttons.len().saturating_sub(1)) as f32);

        let start_x = cx.bounds.origin.x + cx.bounds.size.width - total_width - Self::MENU_PADDING;
        let start_y = cx.bounds.origin.y + Self::MENU_PADDING;

        // Store button bounds for event handling
        match self.button_bounds.lock() {
            Ok(mut bounds_vec) => {
                bounds_vec.clear();
                for i in 0..self.buttons.len() {
                    let x = start_x + (i as f32 * (Self::BUTTON_WIDTH + Self::BUTTON_SPACING));
                    bounds_vec.push(Bounds {
                        origin: Point::new(x, start_y),
                        size: Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
                    });
                }
            }
            Err(e) => {
                tracing::error!("Failed to lock button_bounds in paint: {:?}", e);
            }
        }

        // Draw each button
        for (i, button) in self.buttons.iter().enumerate() {
            if !button.enabled {
                continue;
            }

            let x = start_x + (i as f32 * (Self::BUTTON_WIDTH + Self::BUTTON_SPACING));
            let button_bounds = Bounds {
                origin: Point::new(x, start_y),
                size: Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
            };

            let is_hovered = self.hovered_index == Some(i);
            let bg_color = self.get_button_color(button, is_hovered);
            let text_color = self.get_text_color(button);

            // Draw button background
            cx.fill_rounded_rect(button_bounds, bg_color, Self::CORNER_RADIUS);

            // Draw button border
            let border_color = if button.is_active {
                colors::ACCENT_PRIMARY
            } else {
                colors::BORDER_LIGHT
            };
            cx.stroke_rect(button_bounds, border_color, 1.0);

            // Draw button text (centered)
            let text_width = button.label.len() as f32 * Self::FONT_SIZE * 0.6;
            let text_x = button_bounds.origin.x + (Self::BUTTON_WIDTH - text_width) / 2.0;
            let text_y =
                button_bounds.origin.y + (Self::BUTTON_HEIGHT + Self::FONT_SIZE * 0.7) / 2.0;

            cx.draw_text(
                button.label,
                Point::new(text_x, text_y),
                text_color,
                Self::FONT_SIZE,
            );
        }
    }

    fn handle_event(&mut self, event: &InputEvent, _bounds: Bounds) -> bool {
        // Use pre-calculated button bounds from paint (stored in Arc<Mutex<>>)
        // This avoids the issue of GameRoot passing wrong bounds

        match event {
            InputEvent::MouseMove { position, .. } => {
                // Check if mouse is over any button
                let mut found_hover = false;
                match self.button_bounds.lock() {
                    Ok(bounds_vec) => {
                        for (i, button_bound) in bounds_vec.iter().enumerate() {
                            if button_bound.contains(*position) {
                                if self.hovered_index != Some(i) {
                                    self.hovered_index = Some(i);
                                    self.dirty = true;
                                }
                                found_hover = true;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to lock button_bounds in MouseMove: {:?}", e);
                    }
                }

                if !found_hover && self.hovered_index.is_some() {
                    self.hovered_index = None;
                    self.dirty = true;
                }

                false // Don't consume the event
            }
            InputEvent::MouseDown { position, .. } => {
                // Check if click is on any button
                match self.button_bounds.lock() {
                    Ok(bounds_vec) => {
                        for (i, button_bound) in bounds_vec.iter().enumerate() {
                            if button_bound.contains(*position)
                                && let Some(button) = self.buttons.get(i)
                                && button.enabled
                            {
                                self.pending_action = Some(button.action);
                                self.dirty = true;
                                return true;
                            }
                        }
                        false
                    }
                    Err(e) => {
                        tracing::error!("Failed to lock button_bounds in MouseDown: {:?}", e);
                        false
                    }
                }
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
    fn test_quick_menu_creation() {
        let menu = QuickMenuElement::new();

        // Should have 5 buttons
        assert_eq!(menu.buttons.len(), 5);
        assert!(menu.pending_action().is_none());
        assert!(!menu.skip_active);
        assert!(!menu.auto_active);
    }

    #[test]
    fn test_skip_mode_toggle() {
        let mut menu = QuickMenuElement::new();

        // Initially inactive
        assert!(!menu.skip_active);
        assert!(!menu.buttons[0].is_active);

        // Activate skip mode
        menu.set_skip_active(true);
        assert!(menu.skip_active);
        assert!(menu.buttons[0].is_active);

        // Deactivate skip mode
        menu.set_skip_active(false);
        assert!(!menu.skip_active);
        assert!(!menu.buttons[0].is_active);
    }

    #[test]
    fn test_auto_mode_toggle() {
        let mut menu = QuickMenuElement::new();

        // Initially inactive
        assert!(!menu.auto_active);
        assert!(!menu.buttons[1].is_active);

        // Activate auto mode
        menu.set_auto_active(true);
        assert!(menu.auto_active);
        assert!(menu.buttons[1].is_active);

        // Deactivate auto mode
        menu.set_auto_active(false);
        assert!(!menu.auto_active);
        assert!(!menu.buttons[1].is_active);
    }

    #[test]
    fn test_pending_action() {
        let mut menu = QuickMenuElement::new();

        // Initially no pending action
        assert!(menu.pending_action().is_none());

        // Simulate button click by setting pending action
        menu.pending_action = Some(QuickMenuAction::QuickSave);
        assert_eq!(menu.pending_action(), Some(QuickMenuAction::QuickSave));

        // Clear pending action
        menu.clear_pending_action();
        assert!(menu.pending_action().is_none());
    }

    #[test]
    fn test_button_bounds_calculation() {
        let mut menu = QuickMenuElement::new();
        let container_bounds = Bounds::new(0.0, 0.0, 1280.0, 100.0);

        menu.calculate_button_bounds(container_bounds);

        // Check that all button bounds are calculated
        let bounds_vec = menu.button_bounds.lock().unwrap();
        assert_eq!(bounds_vec.len(), 5);

        // Buttons should be right-aligned
        let total_width =
            (QuickMenuElement::BUTTON_WIDTH * 5.0) + (QuickMenuElement::BUTTON_SPACING * 4.0);
        let expected_start_x = 1280.0 - total_width - QuickMenuElement::MENU_PADDING;

        assert!((bounds_vec[0].origin.x - expected_start_x).abs() < 0.1);
    }
}
