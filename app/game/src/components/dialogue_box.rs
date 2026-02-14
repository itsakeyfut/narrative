//! Dialogue box UI component using narrative-gui framework
//!
//! This component displays dialogue text with:
//! - Speaker name (optional)
//! - Typewriter effect (controlled by visible_chars)
//! - Blinking click indicator when text is complete
//! - Configurable styling via DialogueBoxConfig

use narrative_core::config::DialogueBoxConfig;
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::{Bounds, Color, Element, ElementId, InputEvent, Point, Size};
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use taffy::NodeId;

/// Dialogue box element that displays dialogue text with typewriter effect
pub struct DialogueBoxElement {
    /// Unique element ID
    id: ElementId,
    /// Taffy layout node
    layout_node: Option<NodeId>,
    /// Dialogue box configuration (colors, sizes, etc.)
    config: DialogueBoxConfig,
    /// Speaker name (optional)
    speaker: Option<Arc<str>>,
    /// Full dialogue text
    text: Arc<str>,
    /// Number of characters currently visible (for typewriter effect)
    visible_chars: usize,
    /// Whether all text has been displayed
    text_complete: bool,
    /// Elapsed time for animations (in seconds)
    elapsed: f32,
    /// Current phase of blink animation (0 to 2π)
    blink_phase: f32,
    /// Auto mode enabled
    auto_mode_enabled: bool,
    /// Skip mode enabled
    skip_mode_enabled: bool,
    /// Skip mode type (for display purposes)
    skip_mode: narrative_core::SkipMode,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
}

impl DialogueBoxElement {
    // Constants for UI layout and animation
    const CLICK_INDICATOR_SIZE: f32 = 16.0;
    const BLINK_ALPHA_MIN: f32 = 0.3;
    const BLINK_ALPHA_MAX: f32 = 1.0;
    const BLINK_ALPHA_SCALE: f32 = 0.5;
    /// Assumed frame delta time for 60 FPS
    const ASSUMED_FRAME_DELTA: f32 = 1.0 / 60.0;

    /// Create a new dialogue box element
    pub fn new(config: DialogueBoxConfig) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            config,
            speaker: None,
            text: Arc::from(""),
            visible_chars: 0,
            text_complete: false,
            elapsed: 0.0,
            blink_phase: 0.0,
            auto_mode_enabled: false,
            skip_mode_enabled: false,
            skip_mode: narrative_core::SkipMode::default(),
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

    /// Set the speaker name
    pub fn with_speaker(mut self, speaker: impl Into<Arc<str>>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }

    /// Set the dialogue text
    pub fn with_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.text = text.into();
        self.visible_chars = 0;
        self.text_complete = false;
        self
    }

    /// Set the number of visible characters (for typewriter effect)
    pub fn with_visible_chars(mut self, count: usize) -> Self {
        self.visible_chars = count;
        self.text_complete = count >= self.text.chars().count();
        self
    }

    /// Update the speaker name (mutable)
    pub fn set_speaker(&mut self, speaker: Option<Arc<str>>) {
        self.speaker = speaker;
    }

    /// Update the dialogue text (mutable)
    pub fn set_text(&mut self, text: Arc<str>) {
        self.text = text;
        self.visible_chars = 0;
        self.text_complete = false;
    }

    /// Update the number of visible characters (mutable)
    pub fn set_visible_chars(&mut self, count: usize) {
        self.visible_chars = count;
        self.text_complete = count >= self.text.chars().count();
    }

    /// Mark text as complete
    pub fn set_text_complete(&mut self, complete: bool) {
        self.text_complete = complete;
    }
    /// Set auto mode enabled
    pub fn set_auto_mode_enabled(&mut self, enabled: bool) {
        self.auto_mode_enabled = enabled;
    }

    /// Set skip mode enabled
    pub fn set_skip_mode_enabled(&mut self, enabled: bool, mode: narrative_core::SkipMode) {
        self.skip_mode_enabled = enabled;
        self.skip_mode = mode;
    }

    /// Get truncated text for typewriter effect
    fn get_visible_text(&self) -> String {
        if self.visible_chars >= self.text.chars().count() {
            self.text.to_string()
        } else {
            self.text.chars().take(self.visible_chars).collect()
        }
    }

    /// Calculate blink alpha for click indicator
    fn calculate_blink_alpha(&self) -> f32 {
        // Use sine wave for smooth blinking
        // Maps elapsed time to BLINK_ALPHA_MIN-BLINK_ALPHA_MAX range
        let sin_value = self.blink_phase.sin();
        (sin_value * Self::BLINK_ALPHA_SCALE + Self::BLINK_ALPHA_SCALE)
            .clamp(Self::BLINK_ALPHA_MIN, Self::BLINK_ALPHA_MAX)
    }

    /// Convert narrative_core::Color to narrative_gui::Color
    fn to_gui_color(color: &narrative_core::Color) -> Color {
        Color::new(color.r, color.g, color.b, color.a)
    }
}

impl Element for DialogueBoxElement {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout_node(&self) -> Option<NodeId> {
        self.layout_node
    }

    fn set_layout_node(&mut self, node: NodeId) {
        self.layout_node = Some(node);
    }

    fn layout(
        &mut self,
        _cx: &mut narrative_gui::framework::element::LayoutContext,
    ) -> taffy::Style {
        use taffy::prelude::*;

        // Fixed height, 100% width at bottom of screen
        taffy::Style {
            size: taffy::geometry::Size {
                width: Dimension::percent(1.0), // 100% width
                height: Dimension::length(self.config.height),
            },
            position: Position::Absolute,
            inset: taffy::geometry::Rect {
                left: LengthPercentageAuto::length(0.0),
                right: LengthPercentageAuto::length(0.0),
                bottom: LengthPercentageAuto::length(0.0), // Pin to bottom
                top: LengthPercentageAuto::auto(),
            },
            padding: taffy::geometry::Rect {
                left: LengthPercentage::length(self.config.padding),
                right: LengthPercentage::length(self.config.padding),
                top: LengthPercentage::length(self.config.padding),
                bottom: LengthPercentage::length(self.config.padding),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut narrative_gui::framework::element::PaintContext) {
        // 1. Draw background with rounded corners
        let bg_color = Self::to_gui_color(&self.config.background_color_with_opacity());
        cx.fill_rounded_rect(cx.bounds, bg_color, self.config.corner_radius);

        let mut current_y = cx.bounds.origin.y + self.config.padding;

        // 2. Draw speaker name if present
        if let Some(speaker) = &self.speaker {
            let speaker_color = Self::to_gui_color(&self.config.speaker_color);
            let speaker_pos = Point::new(cx.bounds.origin.x + self.config.padding, current_y);

            cx.draw_text(
                speaker.as_ref(),
                speaker_pos,
                speaker_color,
                self.config.speaker_font_size,
            );

            // Move down for dialogue text (speaker height + small gap)
            current_y += self.config.speaker_font_size + self.config.padding * 0.5;
        }

        // 3. Draw dialogue text (with typewriter effect)
        let visible_text = self.get_visible_text();
        let text_color = Self::to_gui_color(&self.config.text_color);
        let text_pos = Point::new(cx.bounds.origin.x + self.config.padding, current_y);

        cx.draw_text(
            &visible_text,
            text_pos,
            text_color,
            self.config.text_font_size,
        );

        // 4. Draw mode indicators (SKIP and AUTO can be shown simultaneously)
        let indicator_font_size = self.config.text_font_size * 0.8;
        let indicator_padding = self.config.padding * 0.5;
        let mut indicator_y = cx.bounds.origin.y + self.config.padding;

        // Draw skip mode indicator when skip mode is enabled
        if self.skip_mode_enabled {
            let skip_text = match self.skip_mode {
                narrative_core::SkipMode::ReadOnly => "SKIP (Read)",
                narrative_core::SkipMode::All => "SKIP (All)",
                narrative_core::SkipMode::Disabled => "SKIP",
            };

            // Approximate text width (conservative estimate to ensure text fits)
            let text_width = skip_text.len() as f32 * indicator_font_size * 0.7;

            let skip_x = cx.bounds.origin.x + cx.bounds.size.width
                - self.config.padding
                - text_width
                - indicator_padding * 2.0;

            // Draw background rectangle for SKIP indicator
            let skip_bg_bounds = Bounds {
                origin: Point::new(
                    skip_x - indicator_padding,
                    indicator_y - indicator_padding * 0.5,
                ),
                size: Size::new(
                    text_width + indicator_padding * 2.0,
                    indicator_font_size + indicator_padding,
                ),
            };

            // Semi-transparent background
            let skip_bg_color = Color::new(0.2, 0.2, 0.2, 0.7);
            cx.fill_rounded_rect(skip_bg_bounds, skip_bg_color, 4.0);

            // Draw SKIP text (green color to distinguish from AUTO)
            let skip_text_color = Color::new(0.3, 1.0, 0.3, 1.0); // Green color
            let skip_text_pos = Point::new(skip_x, indicator_y);
            cx.draw_text(
                skip_text,
                skip_text_pos,
                skip_text_color,
                indicator_font_size,
            );

            // Update Y position for next indicator
            indicator_y += indicator_font_size + indicator_padding * 2.0;
        }

        // Draw auto mode indicator when auto mode is enabled (below SKIP if both active)
        if self.auto_mode_enabled {
            let auto_text = "AUTO";

            // Approximate text width (conservative estimate to ensure text fits)
            let text_width = auto_text.len() as f32 * indicator_font_size * 0.7;

            let auto_x = cx.bounds.origin.x + cx.bounds.size.width
                - self.config.padding
                - text_width
                - indicator_padding * 2.0;

            // Draw background rectangle for AUTO indicator
            let auto_bg_bounds = Bounds {
                origin: Point::new(
                    auto_x - indicator_padding,
                    indicator_y - indicator_padding * 0.5,
                ),
                size: Size::new(
                    text_width + indicator_padding * 2.0,
                    indicator_font_size + indicator_padding,
                ),
            };

            // Semi-transparent background
            let auto_bg_color = Color::new(0.2, 0.2, 0.2, 0.7);
            cx.fill_rounded_rect(auto_bg_bounds, auto_bg_color, 4.0);

            // Draw AUTO text
            let auto_text_color = Color::new(1.0, 1.0, 0.3, 1.0); // Yellow color
            let auto_text_pos = Point::new(auto_x, indicator_y);
            cx.draw_text(
                auto_text,
                auto_text_pos,
                auto_text_color,
                indicator_font_size,
            );
        }

        // 5. Draw click indicator when text is complete
        if self.text_complete && self.config.show_click_indicator {
            let blink_alpha = self.calculate_blink_alpha();

            // Position at bottom-right corner
            let indicator_x = cx.bounds.origin.x + cx.bounds.size.width
                - self.config.padding
                - Self::CLICK_INDICATOR_SIZE;
            let indicator_y = cx.bounds.origin.y + cx.bounds.size.height
                - self.config.padding
                - Self::CLICK_INDICATOR_SIZE;

            let indicator_bounds = Bounds {
                origin: Point::new(indicator_x, indicator_y),
                size: Size::new(Self::CLICK_INDICATOR_SIZE, Self::CLICK_INDICATOR_SIZE),
            };

            // Draw as circular indicator
            let indicator_color = Color::new(
                self.config.text_color.r,
                self.config.text_color.g,
                self.config.text_color.b,
                blink_alpha,
            );

            cx.fill_rounded_rect(
                indicator_bounds,
                indicator_color,
                Self::CLICK_INDICATOR_SIZE * 0.5,
            );
        }
    }

    fn tick(&mut self, delta: Duration) -> bool {
        // Update blink animation with provided delta (convert Duration to f32 seconds)
        let frame_delta = if delta.as_millis() > 0 {
            delta.as_secs_f32()
        } else {
            Self::ASSUMED_FRAME_DELTA
        };
        self.elapsed += frame_delta;

        // Update blink phase using configured speed
        self.blink_phase =
            self.elapsed * self.config.click_indicator_blink_speed * std::f32::consts::TAU;

        // Request repaint if text is complete (for blinking indicator)
        self.text_complete && self.config.show_click_indicator
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        // Future: Handle click to advance dialogue
        let _ = (event, bounds);
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for DialogueBoxElement {
    fn default() -> Self {
        Self::new(DialogueBoxConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialogue_box_creation() {
        let config = DialogueBoxConfig::default();
        let dialogue_box = DialogueBoxElement::new(config);

        assert_eq!(dialogue_box.visible_chars, 0);
        assert!(!dialogue_box.text_complete);
        assert_eq!(dialogue_box.elapsed, 0.0);
    }

    #[test]
    fn test_with_speaker() {
        let config = DialogueBoxConfig::default();
        let dialogue_box = DialogueBoxElement::new(config).with_speaker("Alice");

        assert!(dialogue_box.speaker.is_some());
        assert_eq!(dialogue_box.speaker.unwrap().as_ref(), "Alice");
    }

    #[test]
    fn test_with_text() {
        let config = DialogueBoxConfig::default();
        let dialogue_box = DialogueBoxElement::new(config).with_text("Hello, world!");

        assert_eq!(dialogue_box.text.as_ref(), "Hello, world!");
        assert_eq!(dialogue_box.visible_chars, 0);
        assert!(!dialogue_box.text_complete);
    }

    #[test]
    fn test_typewriter_effect() {
        let config = DialogueBoxConfig::default();
        let text = "こんにちは";
        let dialogue_box = DialogueBoxElement::new(config)
            .with_text(text)
            .with_visible_chars(3);

        let visible = dialogue_box.get_visible_text();
        assert_eq!(visible.chars().count(), 3);
    }

    #[test]
    fn test_text_complete() {
        let config = DialogueBoxConfig::default();
        let text = "Complete";
        let char_count = text.chars().count();

        let dialogue_box = DialogueBoxElement::new(config)
            .with_text(text)
            .with_visible_chars(char_count);

        assert!(dialogue_box.text_complete);
    }

    #[test]
    fn test_blink_alpha() {
        let config = DialogueBoxConfig::default();
        let mut dialogue_box = DialogueBoxElement::new(config);

        dialogue_box.blink_phase = 0.0; // sin(0) = 0 -> alpha = 0.5
        let alpha1 = dialogue_box.calculate_blink_alpha();
        assert!((alpha1 - 0.5).abs() < 0.01);

        dialogue_box.blink_phase = std::f32::consts::PI / 2.0; // sin(π/2) = 1 -> alpha = 1.0
        let alpha2 = dialogue_box.calculate_blink_alpha();
        assert!((alpha2 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_tick_updates_animation() {
        let config = DialogueBoxConfig::default();
        let mut dialogue_box = DialogueBoxElement::new(config)
            .with_text("Text")
            .with_visible_chars(4); // Text complete

        let initial_elapsed = dialogue_box.elapsed;
        let should_repaint = dialogue_box.tick(Duration::from_millis(16));

        assert!(dialogue_box.elapsed > initial_elapsed);
        assert!(should_repaint); // Should repaint when text is complete
    }

    #[test]
    fn test_set_methods() {
        let config = DialogueBoxConfig::default();
        let mut dialogue_box = DialogueBoxElement::new(config);

        dialogue_box.set_speaker(Some(Arc::from("Bob")));
        assert_eq!(dialogue_box.speaker.as_ref().unwrap().as_ref(), "Bob");

        dialogue_box.set_text(Arc::from("New text"));
        assert_eq!(dialogue_box.text.as_ref(), "New text");
        assert_eq!(dialogue_box.visible_chars, 0); // Reset on new text

        dialogue_box.set_visible_chars(5);
        assert_eq!(dialogue_box.visible_chars, 5);
    }

    #[test]
    fn test_empty_text() {
        let config = DialogueBoxConfig::default();
        let dialogue_box = DialogueBoxElement::new(config).with_text("");

        assert_eq!(dialogue_box.get_visible_text(), "");
        assert_eq!(dialogue_box.visible_chars, 0);
        assert!(!dialogue_box.text_complete);
    }

    #[test]
    fn test_visible_chars_overflow() {
        let config = DialogueBoxConfig::default();
        let text = "Hello";
        let dialogue_box = DialogueBoxElement::new(config)
            .with_text(text)
            .with_visible_chars(100); // More than character count

        assert_eq!(dialogue_box.get_visible_text(), text);
        assert!(dialogue_box.text_complete);
    }

    #[test]
    fn test_long_text() {
        let config = DialogueBoxConfig::default();
        let long_text = "あ".repeat(1000); // 1000 Japanese characters
        let dialogue_box = DialogueBoxElement::new(config)
            .with_text(long_text.as_str())
            .with_visible_chars(500);

        let visible = dialogue_box.get_visible_text();
        assert_eq!(visible.chars().count(), 500);
        assert!(!dialogue_box.text_complete);
    }

    #[test]
    fn test_unicode_emoji_text() {
        let config = DialogueBoxConfig::default();
        let emoji_text = "Hello 👋 World 🌍";
        let dialogue_box = DialogueBoxElement::new(config)
            .with_text(emoji_text)
            .with_visible_chars(7); // "Hello 👋"

        let visible = dialogue_box.get_visible_text();
        assert_eq!(visible.chars().count(), 7);
    }

    #[test]
    fn test_constants_validity() {
        // Verify that constants are within valid ranges
        const {
            assert!(DialogueBoxElement::BLINK_ALPHA_MIN >= 0.0);
        }
        const {
            assert!(DialogueBoxElement::BLINK_ALPHA_MIN < DialogueBoxElement::BLINK_ALPHA_MAX);
        }
        const {
            assert!(DialogueBoxElement::BLINK_ALPHA_MAX <= 1.0);
        }
        const {
            assert!(DialogueBoxElement::CLICK_INDICATOR_SIZE > 0.0);
        }
        const {
            assert!(DialogueBoxElement::ASSUMED_FRAME_DELTA > 0.0);
        }
        const {
            assert!(DialogueBoxElement::ASSUMED_FRAME_DELTA < 1.0);
        }
    }

    #[test]
    fn test_color_conversion() {
        let core_color = narrative_core::Color::new(0.5, 0.6, 0.7, 0.8);
        let gui_color = DialogueBoxElement::to_gui_color(&core_color);

        assert_eq!(gui_color.r, 0.5);
        assert_eq!(gui_color.g, 0.6);
        assert_eq!(gui_color.b, 0.7);
        assert_eq!(gui_color.a, 0.8);
    }
}
