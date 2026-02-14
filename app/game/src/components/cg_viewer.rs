//! CG Viewer UI Component (Simplified Placeholder)
//!
//! Displays a placeholder for the CG viewer.
//! TODO: Full implementation with actual CG display and variation navigation.

use narrative_core::CgRegistry;
use narrative_engine::runtime::CgViewerState;
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use narrative_gui::framework::input::{InputEvent, KeyCode};
use narrative_gui::framework::layout::Bounds;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use taffy::{NodeId, Style};

/// Actions that can be confirmed by the CG viewer
#[derive(Debug, Clone, PartialEq)]
pub enum CgViewerAction {
    /// Close the viewer and return to gallery
    Close,
}

/// CG Viewer UI element
pub struct CgViewerElement {
    id: ElementId,
    layout_node: Option<NodeId>,
    state: CgViewerState,
    #[allow(dead_code)] // May be used for showing CG titles/metadata in future
    cg_registry: Arc<CgRegistry>,
    confirmed_action: Option<CgViewerAction>,
    /// Current CG texture ID (full-size)
    cg_texture_id: Option<u64>,
    /// Current CG texture size for aspect ratio
    cg_texture_size: Option<(u32, u32)>,
    /// Dirty flag for re-rendering
    dirty: bool,
}

impl CgViewerElement {
    // UI layout constants
    const INDICATOR_MARGIN: f32 = 80.0;
    const INDICATOR_TOP_MARGIN: f32 = 30.0;
    const HINT_BOTTOM_MARGIN: f32 = 30.0;
    const HINT_OFFSET: f32 = 100.0;

    pub fn new(
        state: CgViewerState,
        cg_registry: Arc<CgRegistry>,
        cg_texture_id: Option<u64>,
        cg_texture_size: Option<(u32, u32)>,
    ) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            state,
            cg_registry,
            confirmed_action: None,
            cg_texture_id,
            cg_texture_size,
            dirty: true,
        }
    }

    pub fn set_texture(&mut self, texture_id: u64, texture_size: (u32, u32)) {
        self.cg_texture_id = Some(texture_id);
        self.cg_texture_size = Some(texture_size);
        self.dirty = true;
    }

    /// Navigate to previous variation
    fn prev_variation(&mut self) {
        if self.state.variation_index > 0 {
            self.state.variation_index -= 1;
            self.dirty = true;
        }
    }

    /// Navigate to next variation
    fn next_variation(&mut self) {
        if self.state.variation_index + 1 < self.state.total_variations {
            self.state.variation_index += 1;
            self.dirty = true;
        }
    }

    /// Get current state (for variation change detection)
    pub fn get_variation_index(&self) -> usize {
        self.state.variation_index
    }

    pub fn with_animation_context(self, _context: AnimationContext) -> Self {
        self
    }

    pub fn confirmed_action(&self) -> Option<CgViewerAction> {
        self.confirmed_action.clone()
    }

    pub fn reset_confirmation(&mut self) {
        self.confirmed_action = None;
    }

    /// Calculate bounds that fit texture with preserved aspect ratio
    fn calculate_aspect_ratio_fit(
        &self,
        container: narrative_gui::Bounds,
        texture_width: f32,
        texture_height: f32,
    ) -> narrative_gui::Bounds {
        let container_width = container.size.width;
        let container_height = container.size.height;

        // Guard against zero or invalid dimensions
        if container_width <= 0.0
            || container_height <= 0.0
            || texture_width <= 0.0
            || texture_height <= 0.0
            || !container_width.is_finite()
            || !container_height.is_finite()
            || !texture_width.is_finite()
            || !texture_height.is_finite()
        {
            tracing::warn!("Invalid dimensions for aspect ratio calculation");
            return container;
        }

        // Calculate aspect ratios
        let container_aspect = container_width / container_height;
        let texture_aspect = texture_width / texture_height;

        // Calculate fitted size
        let (fitted_width, fitted_height) = if texture_aspect > container_aspect {
            // Texture is wider - fit to width
            (container_width, container_width / texture_aspect)
        } else {
            // Texture is taller - fit to height
            (container_height * texture_aspect, container_height)
        };

        // Center the fitted bounds
        let x = container.origin.x + (container_width - fitted_width) / 2.0;
        let y = container.origin.y + (container_height - fitted_height) / 2.0;

        narrative_gui::Bounds {
            origin: narrative_gui::Point::new(x, y),
            size: narrative_gui::Size::new(fitted_width, fitted_height),
        }
    }
}

impl Element for CgViewerElement {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout_node(&self) -> Option<NodeId> {
        self.layout_node
    }

    fn set_layout_node(&mut self, node: NodeId) {
        self.layout_node = Some(node);
    }

    fn layout(&mut self, _cx: &mut LayoutContext) -> Style {
        Style::default()
    }

    fn paint(&self, cx: &mut PaintContext) {
        // Draw semi-transparent black background
        cx.fill_rect(cx.bounds, narrative_gui::Color::new(0.0, 0.0, 0.0, 0.95));

        // Draw CG texture if available
        if let Some(texture_id) = self.cg_texture_id {
            if let Some((width, height)) = self.cg_texture_size {
                // Calculate aspect-ratio-fitted bounds
                let cg_bounds =
                    self.calculate_aspect_ratio_fit(cx.bounds, width as f32, height as f32);
                cx.draw_texture(texture_id, cg_bounds, 1.0);
            } else {
                // Fallback: draw fullscreen if size is unknown
                cx.draw_texture(texture_id, cx.bounds, 1.0);
            }
        } else {
            // Show placeholder text if texture not loaded
            let text = format!("Loading CG: {}...", self.state.cg_id);
            cx.draw_text(
                &text,
                narrative_gui::Point::new(100.0, 100.0),
                narrative_gui::Color::WHITE,
                24.0,
            );
        }

        // Draw variation indicator if there are multiple variations
        if self.state.total_variations > 1 {
            let indicator_text = format!(
                "{}/{}",
                self.state.variation_index + 1,
                self.state.total_variations
            );
            let indicator_x = cx.bounds.origin.x + cx.bounds.size.width - Self::INDICATOR_MARGIN;
            let indicator_y = cx.bounds.origin.y + Self::INDICATOR_TOP_MARGIN;
            cx.draw_text(
                &indicator_text,
                narrative_gui::Point::new(indicator_x, indicator_y),
                narrative_gui::Color::WHITE,
                20.0,
            );
        }

        // Draw hint text
        let hint_text = if self.state.total_variations > 1 {
            "ESC: Close | ←→: Switch Variation"
        } else {
            "ESC: Close"
        };
        let hint_x = cx.bounds.origin.x + (cx.bounds.size.width / 2.0) - Self::HINT_OFFSET;
        let hint_y = cx.bounds.origin.y + cx.bounds.size.height - Self::HINT_BOTTOM_MARGIN;
        cx.draw_text(
            hint_text,
            narrative_gui::Point::new(hint_x, hint_y),
            narrative_gui::Color::new(0.7, 0.7, 0.7, 1.0),
            16.0,
        );
    }

    fn handle_event(&mut self, event: &InputEvent, _bounds: Bounds) -> bool {
        match event {
            InputEvent::KeyDown { key, .. } => match key {
                KeyCode::Escape | KeyCode::Enter => {
                    self.confirmed_action = Some(CgViewerAction::Close);
                    true
                }
                KeyCode::Left => {
                    self.prev_variation();
                    true
                }
                KeyCode::Right => {
                    self.next_variation();
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn children(&self) -> &[Box<dyn Element>] {
        &[]
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Element>] {
        &mut []
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn tick(&mut self, _delta: Duration) -> bool {
        let was_dirty = self.dirty;
        self.dirty = false;
        was_dirty
    }
}
