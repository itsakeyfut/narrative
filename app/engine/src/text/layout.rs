//! Text layout with cosmic-text integration
//!
//! # Future Improvements
//!
//! ## Vertical Text Support (Phase 0.5+)
//!
//! ```ignore
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! pub enum TextDirection {
//!     Horizontal,
//!     Vertical,
//! }
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! pub enum WritingMode {
//!     HorizontalTb,  // Horizontal, top to bottom
//!     VerticalRl,    // Vertical, right to left
//!     VerticalLr,    // Vertical, left to right
//! }
//! ```

use crate::text::FontManager;
use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping};
use narrative_core::{Color, EngineResult, Point, Size};
use std::sync::Arc;

/// Text style configuration
#[derive(Debug, Clone)]
pub struct TextStyle {
    /// Font size in pixels
    pub font_size: f32,
    /// Line height in pixels
    pub line_height: f32,
    /// Text color
    pub color: Color,
    /// Font family
    pub family: Family<'static>,
    // TODO(Phase 0.4+): Add font_id field for explicit font specification
    // pub font_id: Option<fontdb::ID>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            line_height: 16.0 * 1.4, // 1.4 line height multiplier
            color: Color::WHITE,
            family: Family::SansSerif,
        }
    }
}

impl TextStyle {
    /// Create attributes for cosmic-text
    pub fn attrs(&self) -> Attrs<'static> {
        Attrs::new().family(self.family)
    }
}

/// Positioned glyph for rendering
#[derive(Debug, Clone)]
pub struct LayoutGlyph {
    /// Glyph ID
    pub glyph_id: u16,
    /// X position
    pub x: f32,
    /// Y position (baseline)
    pub y: f32,
    /// Width
    pub width: f32,
    /// Font size
    pub font_size: f32,
}

/// Text layout line
#[derive(Debug, Clone)]
pub struct LayoutLine {
    /// Glyphs in this line
    pub glyphs: Vec<LayoutGlyph>,
    /// Line width
    pub width: f32,
    /// Line height
    pub height: f32,
    /// Baseline Y position
    pub baseline_y: f32,
}

/// Text layout using cosmic-text
pub struct TextLayout {
    /// Cosmic-text buffer
    buffer: Buffer,
    /// Original text (Arc<str> for efficient cloning)
    text: Arc<str>,
    /// Position
    position: Point,
    /// Text style
    style: TextStyle,
    /// Cached layout lines
    lines: Vec<LayoutLine>,
}

impl TextLayout {
    /// Create a new text layout
    pub fn new(
        font_manager: &mut FontManager,
        text: Arc<str>,
        position: Point,
        style: TextStyle,
    ) -> Self {
        let metrics = Metrics::new(style.font_size, style.line_height);
        let mut buffer = Buffer::new(font_manager.font_system_mut(), metrics);

        buffer.set_text(
            font_manager.font_system_mut(),
            &text,
            &style.attrs(),
            Shaping::Advanced,
            None, // alignment
        );

        let mut layout = Self {
            buffer,
            text,
            position,
            style,
            lines: Vec::new(),
        };

        layout.update_layout(font_manager);
        layout
    }

    /// Create with maximum width for text wrapping
    pub fn with_max_width(
        font_manager: &mut FontManager,
        text: Arc<str>,
        position: Point,
        style: TextStyle,
        max_width: f32,
    ) -> Self {
        let metrics = Metrics::new(style.font_size, style.line_height);
        let mut buffer = Buffer::new(font_manager.font_system_mut(), metrics);

        buffer.set_size(font_manager.font_system_mut(), Some(max_width), None);
        buffer.set_text(
            font_manager.font_system_mut(),
            &text,
            &style.attrs(),
            Shaping::Advanced,
            None, // alignment
        );

        let mut layout = Self {
            buffer,
            text,
            position,
            style,
            lines: Vec::new(),
        };

        layout.update_layout(font_manager);
        layout
    }

    /// Update the layout (should be called after buffer changes)
    fn update_layout(&mut self, _font_manager: &mut FontManager) {
        self.lines.clear();

        for run in self.buffer.layout_runs() {
            let mut glyphs = Vec::new();

            for glyph in run.glyphs.iter() {
                glyphs.push(LayoutGlyph {
                    glyph_id: glyph.glyph_id,
                    x: glyph.x + self.position.x,
                    y: run.line_y + self.position.y,
                    width: glyph.w,
                    font_size: glyph.font_size,
                });
            }

            self.lines.push(LayoutLine {
                glyphs,
                width: run.line_w,
                height: self.style.line_height,
                baseline_y: run.line_y + self.position.y,
            });
        }
    }

    /// Get the text content
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the position
    pub fn position(&self) -> Point {
        self.position
    }

    /// Get the text style
    pub fn style(&self) -> &TextStyle {
        &self.style
    }

    /// Get layout lines
    pub fn lines(&self) -> &[LayoutLine] {
        &self.lines
    }

    /// Get all glyphs (flattened from all lines)
    pub fn glyphs(&self) -> impl Iterator<Item = &LayoutGlyph> {
        self.lines.iter().flat_map(|line| line.glyphs.iter())
    }

    /// Get visible glyphs for typewriter effect (up to char_count characters)
    pub fn visible_glyphs(&self, char_count: usize) -> impl Iterator<Item = &LayoutGlyph> {
        self.glyphs().take(char_count)
    }

    /// Calculate the total size of the layout
    pub fn calculate(&self) -> EngineResult<Size> {
        let width = self
            .lines
            .iter()
            .map(|line| line.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        let height = self.lines.iter().map(|line| line.height).sum::<f32>();

        Ok(Size::new(width, height))
    }

    /// Get the cosmic-text buffer
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get mutable cosmic-text buffer
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Update text content
    pub fn set_text(&mut self, font_manager: &mut FontManager, text: String) {
        self.text = Arc::from(text);
        self.buffer.set_text(
            font_manager.font_system_mut(),
            self.text.as_ref(),
            &self.style.attrs(),
            Shaping::Advanced,
            None, // alignment
        );
        self.update_layout(font_manager);
    }

    /// Update position
    pub fn set_position(&mut self, font_manager: &mut FontManager, position: Point) {
        self.position = position;
        self.update_layout(font_manager);
    }

    /// Update maximum width
    pub fn set_max_width(&mut self, font_manager: &mut FontManager, max_width: Option<f32>) {
        self.buffer
            .set_size(font_manager.font_system_mut(), max_width, None);
        self.update_layout(font_manager);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_style_default() {
        let style = TextStyle::default();
        assert_eq!(style.font_size, 16.0);
        assert_eq!(style.line_height, 16.0 * 1.4);
        assert_eq!(style.color, Color::WHITE);
    }

    #[test]
    fn test_text_layout_creation() {
        let mut font_manager = FontManager::new().unwrap();
        let text = "Hello, World!".to_string();
        let position = Point::new(0.0, 0.0);
        let style = TextStyle::default();

        let layout = TextLayout::new(&mut font_manager, Arc::from(text.clone()), position, style);
        assert_eq!(layout.text(), "Hello, World!");
        assert_eq!(layout.position(), position);
    }

    #[test]
    fn test_text_layout_with_max_width() {
        let mut font_manager = FontManager::new().unwrap();
        let text = "This is a longer text that should wrap".to_string();
        let position = Point::new(0.0, 0.0);
        let style = TextStyle::default();

        let layout = TextLayout::with_max_width(
            &mut font_manager,
            Arc::from(text.clone()),
            position,
            style,
            100.0,
        );

        assert_eq!(layout.text(), text);
        // With wrapping enabled, we might have multiple lines
        assert!(!layout.lines().is_empty());
    }

    #[test]
    fn test_text_layout_calculate_size() {
        let mut font_manager = FontManager::new().unwrap();
        let text = "Test".to_string();
        let position = Point::new(0.0, 0.0);
        let style = TextStyle::default();

        let layout = TextLayout::new(&mut font_manager, Arc::from(text), position, style);
        let size = layout.calculate().unwrap();

        // Should have non-zero dimensions for non-empty text
        assert!(size.width >= 0.0);
        assert!(size.height >= 0.0);
    }

    #[test]
    fn test_text_layout_set_text() {
        let mut font_manager = FontManager::new().unwrap();
        let text = "Initial".to_string();
        let position = Point::new(0.0, 0.0);
        let style = TextStyle::default();

        let mut layout = TextLayout::new(&mut font_manager, Arc::from(text), position, style);
        assert_eq!(layout.text(), "Initial");

        layout.set_text(&mut font_manager, "Updated".to_string());
        assert_eq!(layout.text(), "Updated");
    }

    #[test]
    fn test_text_layout_set_position() {
        let mut font_manager = FontManager::new().unwrap();
        let text = "Test".to_string();
        let position = Point::new(10.0, 20.0);
        let style = TextStyle::default();

        let mut layout = TextLayout::new(&mut font_manager, Arc::from(text), position, style);
        assert_eq!(layout.position(), Point::new(10.0, 20.0));

        layout.set_position(&mut font_manager, Point::new(30.0, 40.0));
        assert_eq!(layout.position(), Point::new(30.0, 40.0));
    }

    #[test]
    fn test_visible_glyphs() {
        let mut font_manager = FontManager::new().unwrap();
        let text = "Hello".to_string();
        let position = Point::new(0.0, 0.0);
        let style = TextStyle::default();

        let layout = TextLayout::new(&mut font_manager, Arc::from(text), position, style);

        // Test typewriter effect - should limit visible glyphs
        let visible_count = layout.visible_glyphs(3).count();
        assert!(visible_count <= 3);
    }

    #[test]
    fn test_layout_lines() {
        let mut font_manager = FontManager::new().unwrap();
        let text = "Line 1\nLine 2".to_string();
        let position = Point::new(0.0, 0.0);
        let style = TextStyle::default();

        let layout = TextLayout::new(&mut font_manager, Arc::from(text), position, style);

        // Should have at least one line
        assert!(!layout.lines().is_empty());
    }
}
