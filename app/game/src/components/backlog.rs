//! Backlog UI component for viewing dialogue history
//!
//! This component displays a scrollable list of past dialogues, allowing
//! players to review previous conversations.

use narrative_core::BacklogEntry;
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use narrative_gui::framework::input::{InputEvent, KeyCode};
use narrative_gui::framework::layout::Bounds;
use narrative_gui::theme::colors;
use narrative_gui::{Point, Size};
use std::any::Any;
use taffy::NodeId;

/// Backlog element that displays dialogue history
pub struct BacklogElement {
    /// Unique element ID
    id: ElementId,
    /// Taffy layout node
    layout_node: Option<NodeId>,
    /// Backlog entries (newest first)
    entries: Vec<BacklogEntry>,
    /// Current scroll offset (in pixels)
    scroll_offset: f32,
    /// Maximum scroll offset
    max_scroll: f32,
    /// Dirty flag for repainting
    dirty: bool,
    /// Whether close was requested (Escape key)
    close_requested: bool,
    /// Whether the scrollbar is being dragged
    is_dragging_scrollbar: bool,
    /// Y offset when drag started
    drag_start_offset: f32,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl BacklogElement {
    /// Base entry height (speaker area)
    const BASE_ENTRY_HEIGHT: f32 = 58.0;
    /// Height per text line
    const TEXT_LINE_HEIGHT: f32 = 26.0;
    /// Spacing between entries
    const ENTRY_SPACING: f32 = 16.0;
    /// Padding inside the backlog container
    const PADDING: f32 = 24.0;
    /// Container margin from screen edges
    const CONTAINER_MARGIN: f32 = 40.0;
    /// Speaker name font size
    const SPEAKER_FONT_SIZE: f32 = 16.0;
    /// Dialogue text font size
    const TEXT_FONT_SIZE: f32 = 14.0;
    /// Scroll speed (pixels per wheel tick)
    const SCROLL_SPEED: f32 = 40.0;
    /// Background overlay opacity
    const OVERLAY_ALPHA: f32 = 0.92;
    /// Maximum characters per line (approximate)
    const MAX_CHARS_PER_LINE: usize = 60;
    /// Maximum number of visible entries at once
    const MAX_VISIBLE_ENTRIES: usize = 8;
    /// Scrollbar width in pixels
    const SCROLLBAR_WIDTH: f32 = 8.0;

    /// Create a new backlog element
    pub fn new(entries: Vec<BacklogEntry>) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            entries,
            scroll_offset: 0.0,
            max_scroll: 0.0, // Will be calculated in update_max_scroll
            dirty: true,
            close_requested: false,
            is_dragging_scrollbar: false,
            drag_start_offset: 0.0,
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

    /// Check if close was requested
    pub fn is_close_requested(&self) -> bool {
        self.close_requested
    }

    /// Calculate total content height based on actual entry heights
    fn calculate_total_content_height(&self) -> f32 {
        let mut total_height = Self::PADDING * 2.0;

        if self.entries.is_empty() {
            return total_height;
        }

        for entry in &self.entries {
            total_height += Self::calculate_entry_height(&entry.text) + Self::ENTRY_SPACING;
        }
        // Remove the last spacing
        total_height - Self::ENTRY_SPACING
    }

    /// Update max scroll based on viewport height
    fn update_max_scroll(&mut self, viewport_height: f32) {
        let content_height = self.calculate_total_content_height();
        self.max_scroll = (content_height - viewport_height).max(0.0);
    }

    /// Scroll by a delta amount
    fn scroll(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset + delta).clamp(0.0, self.max_scroll);
        self.dirty = true;
    }

    /// Scroll up (towards newer entries)
    fn scroll_up(&mut self) {
        self.scroll(-Self::SCROLL_SPEED);
    }

    /// Scroll down (towards older entries)
    fn scroll_down(&mut self) {
        self.scroll(Self::SCROLL_SPEED);
    }

    /// Split text into multiple lines based on character limit
    ///
    /// For space-separated languages (English, etc.), tries to break at word boundaries.
    /// For non-space-separated languages (Japanese, etc.), breaks at character limit.
    fn split_text_into_lines(text: &str, max_chars: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();

        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            current_line.clear();
            let mut line_len = 0;
            let mut last_space_pos = None;

            // Collect characters for current line
            while i < chars.len() && line_len < max_chars {
                let ch = chars[i];
                current_line.push(ch);
                line_len += 1;

                // Track last space position for word boundary breaking
                if ch.is_whitespace() {
                    last_space_pos = Some(line_len);
                }

                i += 1;
            }

            // If we reached max_chars and there's more text
            if i < chars.len() && line_len >= max_chars {
                // Try to break at word boundary for space-separated languages
                if let Some(space_pos) = last_space_pos {
                    // Only break at word boundary if it's not too early in the line
                    // (at least 50% of max_chars to avoid very short lines)
                    if space_pos > max_chars / 2 {
                        // Trim to last space
                        current_line.truncate(space_pos);
                        // Move back index to continue from after the space
                        i -= line_len - space_pos;
                    }
                }
            }

            // Trim trailing whitespace
            let trimmed = current_line.trim_end().to_string();
            if !trimmed.is_empty() {
                lines.push(trimmed);
            }
        }

        // If no lines were added, return at least one empty line
        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// Calculate the height of a single entry based on text content
    fn calculate_entry_height(text: &str) -> f32 {
        let lines = Self::split_text_into_lines(text, Self::MAX_CHARS_PER_LINE);
        let line_count = lines.len().min(4); // Max 4 lines
        Self::BASE_ENTRY_HEIGHT + (line_count as f32 * Self::TEXT_LINE_HEIGHT)
    }
}

impl Element for BacklogElement {
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

        // Take up full screen
        taffy::Style {
            size: taffy::geometry::Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        // Draw semi-transparent overlay background
        let overlay_color = narrative_gui::Color::new(0.1, 0.1, 0.1, Self::OVERLAY_ALPHA);
        cx.fill_rect(cx.bounds, overlay_color);

        // Draw backlog container
        let container_bounds = Bounds {
            origin: Point::new(
                cx.bounds.origin.x + Self::CONTAINER_MARGIN,
                cx.bounds.origin.y + Self::CONTAINER_MARGIN,
            ),
            size: Size::new(
                cx.bounds.size.width - (Self::CONTAINER_MARGIN * 2.0),
                cx.bounds.size.height - (Self::CONTAINER_MARGIN * 2.0),
            ),
        };

        // Draw container background
        cx.fill_rounded_rect(container_bounds, colors::CARD_BG, 12.0);
        cx.stroke_rect(container_bounds, colors::BORDER_LIGHT, 1.0);

        // Draw title
        let title = "Backlog";
        let title_x = container_bounds.origin.x + Self::PADDING;
        let title_y = container_bounds.origin.y + Self::PADDING;
        cx.draw_text(
            title,
            Point::new(title_x, title_y + 20.0),
            colors::TEXT_PRIMARY,
            24.0,
        );

        // Draw close hint
        let hint = "Press ESC to close";
        let hint_x = container_bounds.origin.x + container_bounds.size.width - 200.0;
        let hint_y = title_y;
        cx.draw_text(
            hint,
            Point::new(hint_x, hint_y + 20.0),
            colors::TEXT_SECONDARY,
            14.0,
        );

        // Calculate content area (below title)
        let content_start_y = title_y + 60.0;
        let content_height = container_bounds.size.height - 100.0;

        // Draw entries (newest first, scrollable)
        let mut current_y = content_start_y - self.scroll_offset;
        let mut visible_count = 0;
        let content_end_y = content_start_y + content_height;

        for entry in &self.entries {
            let entry_height = Self::calculate_entry_height(&entry.text);

            // Skip entries that are above the visible area
            if current_y + entry_height < content_start_y {
                current_y += entry_height + Self::ENTRY_SPACING;
                continue;
            }

            // Stop if we've reached the maximum number of visible entries
            if visible_count >= Self::MAX_VISIBLE_ENTRIES {
                break;
            }

            // Stop if the entry would extend beyond the visible area (prevents partial display)
            if current_y + entry_height > content_end_y {
                break;
            }

            // Draw entry background (subtle distinction)
            let entry_bounds = Bounds {
                origin: Point::new(container_bounds.origin.x + Self::PADDING, current_y),
                size: Size::new(
                    container_bounds.size.width - (Self::PADDING * 2.0),
                    entry_height,
                ),
            };

            // Draw entry background
            cx.fill_rounded_rect(entry_bounds, colors::BG_DARK, 4.0);

            // Draw speaker name
            let speaker_name = entry.speaker_name();
            let speaker_x = entry_bounds.origin.x + 12.0;
            let speaker_y = entry_bounds.origin.y + 20.0;
            cx.draw_text(
                speaker_name,
                Point::new(speaker_x, speaker_y),
                colors::ACCENT_PRIMARY,
                Self::SPEAKER_FONT_SIZE,
            );

            // Draw dialogue text (with line wrapping)
            let text_x = speaker_x;
            let mut text_y = speaker_y + 26.0;
            let lines = Self::split_text_into_lines(&entry.text, Self::MAX_CHARS_PER_LINE);

            for line in lines.iter().take(4) {
                // Limit to 4 lines to fit in entry height
                cx.draw_text(
                    line,
                    Point::new(text_x, text_y),
                    colors::TEXT_PRIMARY,
                    Self::TEXT_FONT_SIZE,
                );
                text_y += Self::TEXT_LINE_HEIGHT;
            }

            current_y += entry_height + Self::ENTRY_SPACING;
            visible_count += 1;
        }

        // Draw scrollbar if content is scrollable
        if self.max_scroll > 0.0 {
            let scrollbar_width = Self::SCROLLBAR_WIDTH;

            // Calculate scrollbar height based on viewport to total content ratio
            let total_content_height = content_height + self.max_scroll;
            let scrollbar_height = (content_height / total_content_height) * content_height;

            // Calculate scrollbar position within the available travel distance
            let scrollbar_travel_distance = content_height - scrollbar_height;
            let scrollbar_y = content_start_y
                + (self.scroll_offset / self.max_scroll) * scrollbar_travel_distance;

            let scrollbar_bounds = Bounds {
                origin: Point::new(
                    container_bounds.origin.x + container_bounds.size.width - 20.0,
                    scrollbar_y,
                ),
                size: Size::new(scrollbar_width, scrollbar_height),
            };

            cx.fill_rounded_rect(scrollbar_bounds, colors::ACCENT_PRIMARY, 4.0);
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        // Update max scroll based on current viewport
        // Calculate content height same as in paint():
        // container_bounds.height = bounds.height - (margin * 2)
        // content_height = container_bounds.height - 100
        let container_bounds_width = bounds.size.width - (Self::CONTAINER_MARGIN * 2.0);
        let content_height = bounds.size.height - (Self::CONTAINER_MARGIN * 2.0) - 100.0;
        let content_start_y = bounds.origin.y + Self::CONTAINER_MARGIN + 60.0 + Self::PADDING;
        self.update_max_scroll(content_height);

        // Calculate scrollbar bounds for hit testing
        let scrollbar_x = bounds.origin.x + Self::CONTAINER_MARGIN + container_bounds_width - 20.0;
        let scrollbar_width = Self::SCROLLBAR_WIDTH;

        match event {
            InputEvent::MouseDown { position, .. } => {
                // Check if clicking on scrollbar
                if self.max_scroll > 0.0 {
                    // Calculate scrollbar height based on viewport to total content ratio
                    let total_content_height = content_height + self.max_scroll;
                    let scrollbar_height = (content_height / total_content_height) * content_height;

                    // Calculate scrollbar position within the available travel distance
                    let scrollbar_travel_distance = content_height - scrollbar_height;
                    let scrollbar_y = content_start_y
                        + (self.scroll_offset / self.max_scroll) * scrollbar_travel_distance;

                    let scrollbar_bounds = Bounds {
                        origin: Point::new(scrollbar_x, scrollbar_y),
                        size: Size::new(scrollbar_width, scrollbar_height),
                    };

                    if scrollbar_bounds.contains(*position) {
                        self.is_dragging_scrollbar = true;
                        self.drag_start_offset = position.y - scrollbar_y;
                        self.dirty = true;
                        return true;
                    }
                }
                false
            }
            InputEvent::MouseUp { .. } => {
                if self.is_dragging_scrollbar {
                    self.is_dragging_scrollbar = false;
                    self.dirty = true;
                    return true;
                }
                false
            }
            InputEvent::MouseMove { position, .. } => {
                if self.is_dragging_scrollbar {
                    // Calculate scrollbar dimensions
                    let total_content_height = content_height + self.max_scroll;
                    let scrollbar_height = (content_height / total_content_height) * content_height;
                    let scrollbar_travel_distance = content_height - scrollbar_height;

                    // Calculate new scroll position based on mouse Y
                    let scrollbar_y = position.y - self.drag_start_offset;
                    let relative_y = scrollbar_y - content_start_y;
                    let scroll_ratio = relative_y / scrollbar_travel_distance;
                    self.scroll_offset =
                        (scroll_ratio * self.max_scroll).clamp(0.0, self.max_scroll);
                    self.dirty = true;
                    return true;
                }
                false
            }
            InputEvent::KeyDown { key, .. } => match key {
                KeyCode::Escape => {
                    self.close_requested = true;
                    true
                }
                KeyCode::Up => {
                    self.scroll_up();
                    true
                }
                KeyCode::Down => {
                    self.scroll_down();
                    true
                }
                KeyCode::PageUp => {
                    self.scroll(-300.0);
                    true
                }
                KeyCode::PageDown => {
                    self.scroll(300.0);
                    true
                }
                KeyCode::Home => {
                    self.scroll_offset = 0.0;
                    self.dirty = true;
                    true
                }
                KeyCode::End => {
                    self.scroll_offset = self.max_scroll;
                    self.dirty = true;
                    true
                }
                _ => false,
            },
            InputEvent::MouseScroll { delta, .. } => {
                // Scroll based on wheel delta (Y axis)
                // Positive delta = scroll up (towards newer entries)
                // Negative delta = scroll down (towards older entries)
                self.scroll(-delta.y * 20.0);
                true
            }
            _ => false,
        }
    }

    fn tick(&mut self, delta: std::time::Duration) -> bool {
        let _ = delta;
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
    use narrative_core::{SceneId, Speaker};

    fn create_test_entry(speaker: &str, text: &str, index: usize) -> BacklogEntry {
        BacklogEntry::new(
            SceneId::new("test_scene"),
            index,
            Speaker::character(speaker),
            text,
        )
    }

    #[test]
    fn test_backlog_creation() {
        let entries = vec![
            create_test_entry("alice", "Hello!", 0),
            create_test_entry("bob", "Hi there!", 1),
        ];

        let backlog = BacklogElement::new(entries);
        assert_eq!(backlog.entries.len(), 2);
        assert_eq!(backlog.scroll_offset, 0.0);
        assert!(!backlog.is_close_requested());
    }

    #[test]
    fn test_empty_backlog() {
        let backlog = BacklogElement::new(vec![]);
        assert_eq!(backlog.entries.len(), 0);
        assert_eq!(backlog.max_scroll, 0.0);
    }

    #[test]
    fn test_scroll_up() {
        let entries = vec![
            create_test_entry("alice", "Entry 1", 0),
            create_test_entry("bob", "Entry 2", 1),
            create_test_entry("alice", "Entry 3", 2),
        ];

        let mut backlog = BacklogElement::new(entries);
        backlog.max_scroll = 200.0;
        backlog.scroll_offset = 100.0;

        backlog.scroll_up();
        assert!(backlog.scroll_offset < 100.0);
    }

    #[test]
    fn test_scroll_down() {
        let entries = vec![
            create_test_entry("alice", "Entry 1", 0),
            create_test_entry("bob", "Entry 2", 1),
        ];

        let mut backlog = BacklogElement::new(entries);
        backlog.max_scroll = 200.0;

        backlog.scroll_down();
        assert!(backlog.scroll_offset > 0.0);
    }

    #[test]
    fn test_scroll_clamping() {
        let entries = vec![create_test_entry("alice", "Entry", 0)];
        let mut backlog = BacklogElement::new(entries);
        backlog.max_scroll = 100.0;

        // Scroll beyond max
        backlog.scroll(200.0);
        assert_eq!(backlog.scroll_offset, 100.0);

        // Scroll below min
        backlog.scroll(-200.0);
        assert_eq!(backlog.scroll_offset, 0.0);
    }

    #[test]
    fn test_content_height_calculation() {
        // Empty backlog should have minimal height (just padding)
        let backlog_0 = BacklogElement::new(vec![]);
        let height_0 = backlog_0.calculate_total_content_height();
        assert_eq!(height_0, BacklogElement::PADDING * 2.0);

        // Backlog with 1 entry should have height > 0
        let entry1 = create_test_entry("alice", "Hello!", 0);
        let backlog_1 = BacklogElement::new(vec![entry1]);
        let height_1 = backlog_1.calculate_total_content_height();
        assert!(height_1 > height_0);

        // Backlog with 2 entries should be taller
        let entry2 = create_test_entry("bob", "Hi there!", 1);
        let backlog_2 = BacklogElement::new(vec![create_test_entry("alice", "Hello!", 0), entry2]);
        let height_2 = backlog_2.calculate_total_content_height();
        assert!(height_2 > height_1);
    }

    #[test]
    fn test_close_requested() {
        let mut backlog = BacklogElement::new(vec![]);
        assert!(!backlog.is_close_requested());

        backlog.close_requested = true;
        assert!(backlog.is_close_requested());
    }
}
