//! Element trait and UI element abstractions
//!
//! Elements are the building blocks of the UI. They participate in layout and rendering.

use super::Color;
use super::input::InputEvent;
use super::layout::{Bounds, Point, Size};
use super::renderer::DrawCommand;
use crate::theme::{font_size, layout, timeline, typography};
use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use taffy::NodeId;

/// Requested window operations from elements
///
/// Elements can return these operations to request changes to the window,
/// such as resizing or maximizing. The app event loop will process these.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowOperation {
    /// Resize the window to the specified dimensions
    Resize { width: u32, height: u32 },
    /// Maximize the window
    Maximize,
    /// Restore window to normal size (un-maximize)
    Restore,
    /// Toggle between maximized and restored state
    ToggleMaximize,
    /// Minimize the window
    Minimize,
    /// Close the window
    Close,
    /// Center the window on screen
    Center,
    /// Enable or disable window decorations (title bar)
    SetDecorations(bool),
    /// Start dragging the window (for custom title bar)
    DragWindow,
}

/// Unique identifier for elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementId(u64);

impl ElementId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}

/// Context provided during layout phase
pub struct LayoutContext<'a> {
    pub available_size: Size,
    pub layout_engine: &'a mut super::layout::LayoutEngine,
}

/// Context provided during paint phase
pub struct PaintContext<'a> {
    pub bounds: Bounds,
    pub clip_bounds: Option<Bounds>,
    pub commands: &'a mut Vec<DrawCommand>,
}

impl<'a> PaintContext<'a> {
    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, bounds: Bounds, color: Color) {
        self.commands.push(DrawCommand::Rect {
            bounds,
            color,
            corner_radius: 0.0,
        });
    }

    /// Draw a filled rectangle with rounded corners
    pub fn fill_rounded_rect(&mut self, bounds: Bounds, color: Color, corner_radius: f32) {
        self.commands.push(DrawCommand::Rect {
            bounds,
            color,
            corner_radius,
        });
    }

    /// Draw a border rectangle
    pub fn stroke_rect(&mut self, bounds: Bounds, color: Color, width: f32) {
        self.commands.push(DrawCommand::Border {
            bounds,
            color,
            width,
            corner_radius: 0.0,
        });
    }

    /// Draw text at a position
    pub fn draw_text(&mut self, text: &str, position: Point, color: Color, font_size: f32) {
        self.commands.push(DrawCommand::Text {
            text: text.to_string(),
            position,
            color,
            font_size,
        });
    }

    /// Draw a texture with optional opacity
    pub fn draw_texture(&mut self, texture_id: u64, bounds: Bounds, opacity: f32) {
        self.commands.push(DrawCommand::Texture {
            texture_id,
            bounds,
            opacity,
        });
    }

    // Video frame drawing removed - was video-editing specific
    // /// Draw a video frame from RGBA data
    // /// Uses Arc to avoid cloning large frame buffers
    // pub fn draw_video_frame(
    //     &mut self,
    //     data: Arc<Vec<u8>>,
    //     width: u32,
    //     height: u32,
    //     bounds: Bounds,
    // ) {
    //     self.commands.push(DrawCommand::VideoFrame {
    //         data,
    //         width,
    //         height,
    //         bounds,
    //     });
    // }
}

/// Result of hit testing
#[derive(Debug, Clone)]
pub struct HitTestResult {
    pub element_id: ElementId,
    pub bounds: Bounds,
}

/// Trait for elements that can load background textures dynamically
///
/// This trait allows the window renderer to trigger texture loading
/// for elements that manage dynamic backgrounds (e.g., GameRootElement).
pub trait BackgroundTextureLoader {
    /// Load any pending background texture
    ///
    /// Returns: true if a texture was loaded (requires redraw)
    fn load_pending_background_texture(&mut self, renderer: &mut super::renderer::Renderer)
    -> bool;
}

/// The core Element trait that all UI elements implement
pub trait Element: Send + Sync {
    /// Get the element's unique ID
    fn id(&self) -> ElementId;

    /// Get the element's taffy node for layout
    fn layout_node(&self) -> Option<NodeId>;

    /// Set the layout node
    fn set_layout_node(&mut self, node: NodeId);

    /// Request layout and return preferred size constraints
    fn layout(&mut self, cx: &mut LayoutContext) -> taffy::Style;

    /// Paint the element
    fn paint(&self, cx: &mut PaintContext);

    /// Paint overlay content (popups, dropdowns, tooltips)
    ///
    /// This is called after all normal painting is complete, at a higher z-layer.
    /// Override this to render content that should appear on top of other elements.
    fn paint_overlay(&self, _cx: &mut PaintContext) {
        // Default: no overlay content
    }

    /// Handle input events. Returns true if the event was handled.
    fn handle_event(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        let _ = (event, bounds);
        false
    }

    /// Hit test at a point
    fn hit_test(&self, point: Point, bounds: Bounds) -> Option<HitTestResult> {
        if bounds.contains(point) {
            Some(HitTestResult {
                element_id: self.id(),
                bounds,
            })
        } else {
            None
        }
    }

    /// Get children elements
    fn children(&self) -> &[Box<dyn Element>] {
        &[]
    }

    /// Get mutable children elements
    fn children_mut(&mut self) -> &mut [Box<dyn Element>] {
        &mut []
    }

    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Cast to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Return pending window operations and clear them
    ///
    /// Elements can request window operations (resize, maximize, etc.)
    /// by returning them from this method. The app will process these
    /// after rendering.
    fn take_window_operations(&mut self) -> Vec<WindowOperation> {
        Vec::new()
    }

    /// Called every frame to update time-based state
    ///
    /// Override this method to perform continuous updates like animations,
    /// video playback, or other time-sensitive operations.
    /// Returns true if the element needs to be repainted.
    ///
    /// # Parameters
    /// * `delta` - Time elapsed since the last frame
    fn tick(&mut self, delta: Duration) -> bool {
        let _ = delta;
        false
    }

    /// Load any pending background texture
    ///
    /// Override this for elements that manage dynamic backgrounds.
    /// Returns true if a texture was loaded (requires redraw).
    fn load_pending_background_texture(
        &mut self,
        _renderer: &mut super::renderer::Renderer,
    ) -> bool {
        false // Default: no pending textures
    }
}

/// Flex direction for container layout
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FlexDirection {
    #[default]
    Column,
    Row,
}

/// Alignment options
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Alignment {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

/// A simple container element that holds children
pub struct Container {
    id: ElementId,
    layout_node: Option<NodeId>,
    children: Vec<Box<dyn Element>>,
    background: Option<Color>,
    corner_radius: f32,
    padding: f32,
    flex_direction: FlexDirection,
    gap: f32,
    flex_grow: f32,
    flex_shrink: f32,
    width: Option<f32>,
    height: Option<f32>,
    width_percent: Option<f32>,
    height_percent: Option<f32>,
    min_width: Option<f32>,
    min_height: Option<f32>,
    max_width: Option<f32>,
    max_height: Option<f32>,
    align_items: Alignment,
    justify_content: Alignment,
}

impl Container {
    pub fn new() -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            children: Vec::new(),
            background: None,
            corner_radius: 0.0,
            padding: 0.0,
            flex_direction: FlexDirection::Column,
            gap: 0.0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            width: None,
            height: None,
            width_percent: None,
            height_percent: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            align_items: Alignment::Stretch,
            justify_content: Alignment::Start,
        }
    }

    pub fn with_background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_flex_direction(mut self, direction: FlexDirection) -> Self {
        self.flex_direction = direction;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_flex_grow(mut self, grow: f32) -> Self {
        self.flex_grow = grow;
        self
    }

    pub fn with_flex_shrink(mut self, shrink: f32) -> Self {
        self.flex_shrink = shrink;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set width as a percentage (0.0 to 1.0, where 1.0 = 100%)
    pub fn with_width_percent(mut self, percent: f32) -> Self {
        self.width_percent = Some(percent);
        self
    }

    /// Set height as a percentage (0.0 to 1.0, where 1.0 = 100%)
    pub fn with_height_percent(mut self, percent: f32) -> Self {
        self.height_percent = Some(percent);
        self
    }

    /// Fill the parent container (100% width and height)
    pub fn with_fill(mut self) -> Self {
        self.width_percent = Some(1.0);
        self.height_percent = Some(1.0);
        self
    }

    pub fn with_min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn with_min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    pub fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn with_max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    pub fn with_align_items(mut self, align: Alignment) -> Self {
        self.align_items = align;
        self
    }

    pub fn with_justify_content(mut self, justify: Alignment) -> Self {
        self.justify_content = justify;
        self
    }

    pub fn with_child(mut self, child: Box<dyn Element>) -> Self {
        self.children.push(child);
        self
    }

    pub fn add_child(&mut self, child: Box<dyn Element>) {
        self.children.push(child);
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Container {
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

        let flex_dir = match self.flex_direction {
            self::FlexDirection::Column => taffy::FlexDirection::Column,
            self::FlexDirection::Row => taffy::FlexDirection::Row,
        };

        let align = |a: self::Alignment| match a {
            self::Alignment::Start => Some(taffy::AlignItems::Start),
            self::Alignment::Center => Some(taffy::AlignItems::Center),
            self::Alignment::End => Some(taffy::AlignItems::End),
            self::Alignment::Stretch => Some(taffy::AlignItems::Stretch),
        };

        let justify = |a: self::Alignment| match a {
            self::Alignment::Start => Some(taffy::JustifyContent::Start),
            self::Alignment::Center => Some(taffy::JustifyContent::Center),
            self::Alignment::End => Some(taffy::JustifyContent::End),
            self::Alignment::Stretch => Some(taffy::JustifyContent::Stretch),
        };

        Style {
            display: Display::Flex,
            flex_direction: flex_dir,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
            align_items: align(self.align_items),
            justify_content: justify(self.justify_content),
            gap: taffy::Size {
                width: LengthPercentage::length(self.gap),
                height: LengthPercentage::length(self.gap),
            },
            padding: taffy::Rect {
                top: LengthPercentage::length(self.padding),
                right: LengthPercentage::length(self.padding),
                bottom: LengthPercentage::length(self.padding),
                left: LengthPercentage::length(self.padding),
            },
            size: taffy::Size {
                width: if let Some(pct) = self.width_percent {
                    Dimension::percent(pct)
                } else {
                    self.width
                        .map(Dimension::length)
                        .unwrap_or(Dimension::auto())
                },
                height: if let Some(pct) = self.height_percent {
                    Dimension::percent(pct)
                } else {
                    self.height
                        .map(Dimension::length)
                        .unwrap_or(Dimension::auto())
                },
            },
            min_size: taffy::Size {
                width: self
                    .min_width
                    .map(Dimension::length)
                    .unwrap_or(Dimension::auto()),
                height: self
                    .min_height
                    .map(Dimension::length)
                    .unwrap_or(Dimension::auto()),
            },
            max_size: taffy::Size {
                width: self
                    .max_width
                    .map(Dimension::length)
                    .unwrap_or(Dimension::auto()),
                height: self
                    .max_height
                    .map(Dimension::length)
                    .unwrap_or(Dimension::auto()),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        if let Some(bg) = self.background {
            cx.fill_rounded_rect(cx.bounds, bg, self.corner_radius);
        }
    }

    fn children(&self) -> &[Box<dyn Element>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Element>] {
        &mut self.children
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A text element for displaying labels
pub struct Text {
    id: ElementId,
    layout_node: Option<NodeId>,
    text: String,
    color: Color,
    font_size: f32,
    width: Option<f32>,
    height: Option<f32>,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            text: text.into(),
            color: Color::WHITE,
            font_size: 14.0,
            width: None,
            height: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl Element for Text {
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

        // Approximate text size based on character count
        let approx_width = self.text.chars().count() as f32 * self.font_size * 0.6;
        let approx_height = self.font_size * 1.2;

        Style {
            display: Display::Block,
            size: taffy::Size {
                width: self
                    .width
                    .map(Dimension::length)
                    .unwrap_or(Dimension::length(approx_width)),
                height: self
                    .height
                    .map(Dimension::length)
                    .unwrap_or(Dimension::length(approx_height)),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        // Draw text at the top-left of the bounds
        cx.draw_text(
            &self.text,
            Point::new(cx.bounds.x(), cx.bounds.y() + self.font_size),
            self.color,
            self.font_size,
        );
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A video preview element that displays GPU textures directly
pub trait VideoElement: Element {
    /// Get the current texture view for rendering
    fn texture_view(&self) -> Option<&wgpu::TextureView>;

    /// Check if the element needs to be redrawn
    fn needs_redraw(&self) -> bool;

    /// Mark as redrawn
    fn mark_drawn(&mut self);
}
