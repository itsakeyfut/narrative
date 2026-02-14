//! Choice menu UI component using narrative-gui framework
//!
//! This component displays a list of clickable choice buttons with:
//! - Arrow key navigation (up/down)
//! - Enter key confirmation
//! - Mouse click support
//! - Visual highlight for selected choice

use narrative_gui::Point;
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use narrative_gui::framework::input::{InputEvent, KeyCode};
use narrative_gui::framework::layout::Bounds;
use narrative_gui::theme::colors;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use taffy::NodeId;

/// Choice menu element that displays a list of selectable options
pub struct ChoiceMenuElement {
    /// Unique element ID
    id: ElementId,
    /// Taffy layout node
    layout_node: Option<NodeId>,
    /// List of choice texts
    choices: Vec<Arc<str>>,
    /// Currently selected choice index
    selected_index: usize,
    /// Whether a choice has been confirmed (Enter or click)
    choice_confirmed: bool,
    /// Dirty flag to track if buttons need rebuilding
    dirty: bool,
    /// Cached button bounds for click detection
    button_bounds: Vec<Bounds>,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl ChoiceMenuElement {
    /// Default button width
    const BUTTON_WIDTH: f32 = 600.0;
    /// Default button height
    const BUTTON_HEIGHT: f32 = 60.0;
    /// Spacing between buttons
    const BUTTON_SPACING: f32 = 16.0;
    /// Button corner radius
    const CORNER_RADIUS: f32 = 8.0;
    /// Button font size
    const FONT_SIZE: f32 = 18.0;

    /// Create a new choice menu element
    pub fn new(choices: Vec<impl Into<Arc<str>>>) -> Self {
        let choices: Vec<Arc<str>> = choices.into_iter().map(|s| s.into()).collect();
        let button_bounds = vec![Bounds::default(); choices.len()];

        Self {
            id: ElementId::new(),
            layout_node: None,
            choices,
            selected_index: 0,
            choice_confirmed: false,
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

    /// Set the list of choices
    pub fn set_choices(&mut self, choices: Vec<Arc<str>>) {
        self.choices = choices;
        self.selected_index = 0;
        self.choice_confirmed = false;
        self.button_bounds = vec![Bounds::default(); self.choices.len()];
        self.dirty = true;
    }

    /// Set the selected choice index
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.choices.len() {
            self.selected_index = index;
            self.dirty = true;
        }
    }

    /// Get the currently selected choice index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Check if a choice has been confirmed
    pub fn is_choice_confirmed(&self) -> bool {
        self.choice_confirmed
    }

    /// Reset the confirmation state
    pub fn reset_confirmation(&mut self) {
        self.choice_confirmed = false;
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
        if self.selected_index < self.choices.len().saturating_sub(1) {
            self.selected_index = self.selected_index.saturating_add(1);
            self.dirty = true;
        }
    }

    /// Confirm the current selection
    fn confirm_selection(&mut self) {
        self.choice_confirmed = true;
    }

    /// Calculate button bounds for layout
    fn calculate_button_bounds(&mut self, container_bounds: Bounds) {
        let total_height = (Self::BUTTON_HEIGHT * self.choices.len() as f32)
            + (Self::BUTTON_SPACING * (self.choices.len().saturating_sub(1)) as f32);

        let start_y =
            container_bounds.origin.y + (container_bounds.size.height - total_height) / 2.0;
        let start_x =
            container_bounds.origin.x + (container_bounds.size.width - Self::BUTTON_WIDTH) / 2.0;

        for i in 0..self.choices.len() {
            let y = start_y + (i as f32 * (Self::BUTTON_HEIGHT + Self::BUTTON_SPACING));
            self.button_bounds[i] = Bounds {
                origin: Point::new(start_x, y),
                size: narrative_gui::Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
            };
        }
    }
}

impl Element for ChoiceMenuElement {
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
            gap: taffy::geometry::Size {
                width: LengthPercentage::length(0.0),
                height: LengthPercentage::length(Self::BUTTON_SPACING),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        let total_height = (Self::BUTTON_HEIGHT * self.choices.len() as f32)
            + (Self::BUTTON_SPACING * (self.choices.len().saturating_sub(1)) as f32);

        let start_y = cx.bounds.origin.y + (cx.bounds.size.height - total_height) / 2.0;
        let start_x = cx.bounds.origin.x + (cx.bounds.size.width - Self::BUTTON_WIDTH) / 2.0;

        // Draw each choice button
        for (i, choice) in self.choices.iter().enumerate() {
            let y = start_y + (i as f32 * (Self::BUTTON_HEIGHT + Self::BUTTON_SPACING));
            let button_bounds = Bounds {
                origin: Point::new(start_x, y),
                size: narrative_gui::Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
            };

            // Determine if this button should appear hovered (for selected item)
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

            // Draw choice text (centered)
            // TODO: Use cosmic-text's Buffer::shape() for accurate text width measurement
            // Current implementation assumes fixed-width characters, which is inaccurate
            // for proportional fonts and Japanese full-width/half-width character mixes.
            // For better centering accuracy, measure actual glyph widths.
            let text_width = choice.chars().count() as f32 * Self::FONT_SIZE * 0.6;
            let text_x = button_bounds.origin.x + (Self::BUTTON_WIDTH - text_width) / 2.0;
            let text_y =
                button_bounds.origin.y + (Self::BUTTON_HEIGHT + Self::FONT_SIZE * 0.8) / 2.0;

            cx.draw_text(
                choice.as_ref(),
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
                KeyCode::Enter => {
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
    fn test_choice_menu_creation() {
        let choices = vec!["Choice 1", "Choice 2", "Choice 3"];
        let menu = ChoiceMenuElement::new(choices);

        assert_eq!(menu.choices.len(), 3);
        assert_eq!(menu.selected_index(), 0);
        assert!(!menu.is_choice_confirmed());
    }

    #[test]
    fn test_selection_navigation() {
        let choices = vec!["Choice 1", "Choice 2", "Choice 3"];
        let mut menu = ChoiceMenuElement::new(choices);

        // Move down
        menu.select_next();
        assert_eq!(menu.selected_index(), 1);

        menu.select_next();
        assert_eq!(menu.selected_index(), 2);

        // Try to move past end
        menu.select_next();
        assert_eq!(menu.selected_index(), 2);

        // Move up
        menu.select_previous();
        assert_eq!(menu.selected_index(), 1);

        menu.select_previous();
        assert_eq!(menu.selected_index(), 0);

        // Try to move before start
        menu.select_previous();
        assert_eq!(menu.selected_index(), 0);
    }

    #[test]
    fn test_choice_confirmation() {
        let choices = vec!["Choice 1", "Choice 2"];
        let mut menu = ChoiceMenuElement::new(choices);

        assert!(!menu.is_choice_confirmed());

        menu.confirm_selection();
        assert!(menu.is_choice_confirmed());

        menu.reset_confirmation();
        assert!(!menu.is_choice_confirmed());
    }

    #[test]
    fn test_set_choices() {
        let choices = vec!["Choice 1", "Choice 2"];
        let mut menu = ChoiceMenuElement::new(choices);

        menu.set_selected_index(1);
        assert_eq!(menu.selected_index(), 1);

        let new_choices = vec!["New 1", "New 2", "New 3"];
        menu.set_choices(new_choices.into_iter().map(Arc::from).collect());

        assert_eq!(menu.choices.len(), 3);
        assert_eq!(menu.selected_index(), 0); // Reset to 0
        assert!(!menu.is_choice_confirmed());
    }

    #[test]
    fn test_set_selected_index() {
        let choices = vec!["Choice 1", "Choice 2", "Choice 3"];
        let mut menu = ChoiceMenuElement::new(choices);

        menu.set_selected_index(2);
        assert_eq!(menu.selected_index(), 2);

        // Try to set out of bounds
        menu.set_selected_index(10);
        assert_eq!(menu.selected_index(), 2); // Should not change
    }

    #[test]
    fn test_empty_choices() {
        let choices: Vec<&str> = vec![];
        let menu = ChoiceMenuElement::new(choices);

        assert_eq!(menu.choices.len(), 0);
        assert_eq!(menu.selected_index(), 0);
    }

    #[test]
    fn test_single_choice() {
        let choices = vec!["Only Choice"];
        let mut menu = ChoiceMenuElement::new(choices);

        assert_eq!(menu.selected_index(), 0);

        menu.select_next();
        assert_eq!(menu.selected_index(), 0); // Can't move

        menu.select_previous();
        assert_eq!(menu.selected_index(), 0); // Can't move

        menu.confirm_selection();
        assert!(menu.is_choice_confirmed());
    }

    #[test]
    fn test_button_bounds_calculation() {
        let choices = vec!["Choice 1", "Choice 2"];
        let mut menu = ChoiceMenuElement::new(choices);

        let bounds = Bounds::new(0.0, 0.0, 1280.0, 720.0);
        menu.calculate_button_bounds(bounds);

        assert_eq!(menu.button_bounds.len(), 2);

        // Buttons should be centered and spaced
        let first = &menu.button_bounds[0];
        let second = &menu.button_bounds[1];

        assert_eq!(first.size.width, ChoiceMenuElement::BUTTON_WIDTH);
        assert_eq!(first.size.height, ChoiceMenuElement::BUTTON_HEIGHT);
        assert_eq!(second.size.height, ChoiceMenuElement::BUTTON_HEIGHT);

        // Second button should be below first
        assert!(second.origin.y > first.origin.y);
    }
}
