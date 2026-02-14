//! Confirmation dialog UI component
//!
//! This component displays a modal confirmation dialog with:
//! - Custom message
//! - Confirm button (Yes/OK)
//! - Cancel button (No/Cancel)
//!
//! Supports arrow key navigation and Enter/Escape for quick response.

use narrative_gui::Point;
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use narrative_gui::framework::input::{InputEvent, KeyCode};
use narrative_gui::framework::layout::Bounds;
use narrative_gui::theme::colors;
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

/// Confirmation dialog response
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DialogResponse {
    /// User confirmed (Yes/OK)
    Confirmed,
    /// User cancelled (No/Cancel)
    Cancelled,
    /// No response yet
    None,
}

/// Confirmation dialog element
pub struct ConfirmDialogElement {
    /// Unique element ID
    id: ElementId,
    /// Taffy layout node
    layout_node: Option<NodeId>,
    /// Dialog message to display
    message: String,
    /// Confirm button label
    confirm_label: &'static str,
    /// Cancel button label
    cancel_label: &'static str,
    /// Currently selected button index (0=Confirm, 1=Cancel)
    selected_index: usize,
    /// User's response
    response: DialogResponse,
    /// Dirty flag to track if rendering needs update
    dirty: bool,
    /// Cached button bounds for click detection
    button_bounds: [Bounds; 2],
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl ConfirmDialogElement {
    /// Dialog box width
    const DIALOG_WIDTH: f32 = 600.0;
    /// Dialog box height
    const DIALOG_HEIGHT: f32 = 250.0;
    /// Button width
    const BUTTON_WIDTH: f32 = 150.0;
    /// Button height
    const BUTTON_HEIGHT: f32 = 50.0;
    /// Spacing between buttons
    const BUTTON_SPACING: f32 = 20.0;
    /// Button corner radius
    const CORNER_RADIUS: f32 = 8.0;
    /// Message font size
    const MESSAGE_FONT_SIZE: f32 = 20.0;
    /// Button font size
    const BUTTON_FONT_SIZE: f32 = 18.0;
    /// Background overlay alpha
    const OVERLAY_ALPHA: f32 = 0.8;
    /// Message padding from top
    const MESSAGE_PADDING_TOP: f32 = 60.0;
    /// Message padding from left/right
    const MESSAGE_PADDING_X: f32 = 40.0;
    /// Buttons offset from bottom
    const BUTTONS_OFFSET_BOTTOM: f32 = 70.0;

    /// Create a new confirmation dialog
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            message: message.into(),
            confirm_label: "Yes",
            cancel_label: "No",
            selected_index: 1, // Default to Cancel for safety
            response: DialogResponse::None,
            dirty: true,
            button_bounds: [Bounds::default(); 2],
            animation_context: AnimationContext::default(),
            animations_enabled: None,
        }
    }

    /// Set custom button labels
    pub fn with_labels(mut self, confirm_label: &'static str, cancel_label: &'static str) -> Self {
        self.confirm_label = confirm_label;
        self.cancel_label = cancel_label;
        self
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

    /// Get the user's response
    pub fn response(&self) -> DialogResponse {
        self.response
    }

    /// Check if user confirmed
    pub fn is_confirmed(&self) -> bool {
        self.response == DialogResponse::Confirmed
    }

    /// Check if user cancelled
    pub fn is_cancelled(&self) -> bool {
        self.response == DialogResponse::Cancelled
    }

    /// Reset the dialog response
    pub fn reset(&mut self) {
        self.response = DialogResponse::None;
        self.selected_index = 1; // Default to Cancel
        self.dirty = true;
    }

    /// Move selection left
    fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.dirty = true;
        }
    }

    /// Move selection right
    fn select_next(&mut self) {
        if self.selected_index < 1 {
            self.selected_index = self.selected_index.saturating_add(1);
            self.dirty = true;
        }
    }

    /// Confirm the current selection
    fn confirm_selection(&mut self) {
        self.response = if self.selected_index == 0 {
            DialogResponse::Confirmed
        } else {
            DialogResponse::Cancelled
        };
        self.dirty = true;
    }

    /// Calculate button bounds for layout
    fn calculate_button_bounds(&mut self, container_bounds: Bounds) {
        // Center the dialog box
        let dialog_x =
            container_bounds.origin.x + (container_bounds.size.width - Self::DIALOG_WIDTH) / 2.0;
        let dialog_y =
            container_bounds.origin.y + (container_bounds.size.height - Self::DIALOG_HEIGHT) / 2.0;

        // Calculate button positions (centered horizontally at bottom of dialog)
        let total_button_width = Self::BUTTON_WIDTH * 2.0 + Self::BUTTON_SPACING;
        let buttons_start_x = dialog_x + (Self::DIALOG_WIDTH - total_button_width) / 2.0;
        let buttons_y = dialog_y + Self::DIALOG_HEIGHT - Self::BUTTONS_OFFSET_BOTTOM;

        // Confirm button (left)
        self.button_bounds[0] = Bounds {
            origin: Point::new(buttons_start_x, buttons_y),
            size: narrative_gui::Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
        };

        // Cancel button (right)
        self.button_bounds[1] = Bounds {
            origin: Point::new(
                buttons_start_x + Self::BUTTON_WIDTH + Self::BUTTON_SPACING,
                buttons_y,
            ),
            size: narrative_gui::Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
        };
    }
}

impl Element for ConfirmDialogElement {
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

        // Take up full available space (for overlay)
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

        // Calculate dialog box position (centered)
        let dialog_x = cx.bounds.origin.x + (cx.bounds.size.width - Self::DIALOG_WIDTH) / 2.0;
        let dialog_y = cx.bounds.origin.y + (cx.bounds.size.height - Self::DIALOG_HEIGHT) / 2.0;

        let dialog_bounds = Bounds {
            origin: Point::new(dialog_x, dialog_y),
            size: narrative_gui::Size::new(Self::DIALOG_WIDTH, Self::DIALOG_HEIGHT),
        };

        // Draw dialog background
        cx.fill_rounded_rect(dialog_bounds, colors::CARD_BG, Self::CORNER_RADIUS);
        cx.stroke_rect(dialog_bounds, colors::BORDER_LIGHT, 2.0);

        // Draw message text (with padding from left/right edges)
        // TODO: Support multi-line text wrapping
        let message_x = dialog_x + Self::MESSAGE_PADDING_X;
        let message_y = dialog_y + Self::MESSAGE_PADDING_TOP;

        cx.draw_text(
            &self.message,
            Point::new(message_x, message_y),
            colors::TEXT_PRIMARY,
            Self::MESSAGE_FONT_SIZE,
        );

        // Draw buttons
        let button_labels = [self.confirm_label, self.cancel_label];

        for (i, &label) in button_labels.iter().enumerate() {
            let button_bound = self.button_bounds[i];
            let is_selected = i == self.selected_index;

            // Determine button appearance
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
            cx.fill_rounded_rect(button_bound, bg_color, Self::CORNER_RADIUS);

            // Draw button border for non-selected items
            if !is_selected {
                cx.stroke_rect(button_bound, colors::BORDER_LIGHT, 1.0);
            }

            // Draw button text (centered)
            let text_width = label.len() as f32 * Self::BUTTON_FONT_SIZE * 0.6;
            let text_x = button_bound.origin.x + (Self::BUTTON_WIDTH - text_width) / 2.0;
            let text_y =
                button_bound.origin.y + (Self::BUTTON_HEIGHT + Self::BUTTON_FONT_SIZE * 0.8) / 2.0;

            cx.draw_text(
                label,
                Point::new(text_x, text_y),
                text_color,
                Self::BUTTON_FONT_SIZE,
            );
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        // Update button bounds for click detection
        self.calculate_button_bounds(bounds);

        match event {
            InputEvent::KeyDown { key, .. } => match key {
                KeyCode::Left => {
                    self.select_previous();
                    true
                }
                KeyCode::Right => {
                    self.select_next();
                    true
                }
                KeyCode::Enter | KeyCode::Space => {
                    self.confirm_selection();
                    true
                }
                KeyCode::Escape => {
                    // ESC key acts as Cancel
                    self.response = DialogResponse::Cancelled;
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
    fn test_confirm_dialog_creation() {
        let dialog = ConfirmDialogElement::new("Are you sure?");

        assert_eq!(dialog.message, "Are you sure?");
        assert_eq!(dialog.selected_index, 1); // Default to Cancel for safety
        assert_eq!(dialog.response(), DialogResponse::None);
    }

    #[test]
    fn test_custom_labels() {
        let dialog = ConfirmDialogElement::new("Delete file?").with_labels("Delete", "Keep");

        assert_eq!(dialog.confirm_label, "Delete");
        assert_eq!(dialog.cancel_label, "Keep");
    }

    #[test]
    fn test_selection_navigation() {
        let mut dialog = ConfirmDialogElement::new("Test");

        // Start at Cancel (index 1) for safety
        assert_eq!(dialog.selected_index, 1);

        // Move left to Confirm
        dialog.select_previous();
        assert_eq!(dialog.selected_index, 0);

        // Try to move left past Confirm (should stay at 0)
        dialog.select_previous();
        assert_eq!(dialog.selected_index, 0);

        // Move right back to Cancel
        dialog.select_next();
        assert_eq!(dialog.selected_index, 1);

        // Try to move right past Cancel (should stay at 1)
        dialog.select_next();
        assert_eq!(dialog.selected_index, 1);
    }

    #[test]
    fn test_confirm_response() {
        let mut dialog = ConfirmDialogElement::new("Test");

        // Confirm
        dialog.selected_index = 0;
        dialog.confirm_selection();
        assert_eq!(dialog.response(), DialogResponse::Confirmed);
        assert!(dialog.is_confirmed());
        assert!(!dialog.is_cancelled());
    }

    #[test]
    fn test_cancel_response() {
        let mut dialog = ConfirmDialogElement::new("Test");

        // Cancel
        dialog.selected_index = 1;
        dialog.confirm_selection();
        assert_eq!(dialog.response(), DialogResponse::Cancelled);
        assert!(!dialog.is_confirmed());
        assert!(dialog.is_cancelled());
    }

    #[test]
    fn test_reset() {
        let mut dialog = ConfirmDialogElement::new("Test");

        dialog.confirm_selection();
        assert_ne!(dialog.response(), DialogResponse::None);

        dialog.reset();
        assert_eq!(dialog.response(), DialogResponse::None);
        assert_eq!(dialog.selected_index, 1); // Reset to Cancel
    }

    #[test]
    fn test_escape_key_cancels() {
        use narrative_gui::framework::input::Modifiers;

        let mut dialog = ConfirmDialogElement::new("Test");
        let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);

        let event = InputEvent::KeyDown {
            key: KeyCode::Escape,
            modifiers: Modifiers::none(),
        };

        assert!(dialog.handle_event(&event, bounds));
        assert_eq!(dialog.response(), DialogResponse::Cancelled);
    }
}
