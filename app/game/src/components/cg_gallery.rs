//! CG Gallery UI Component
//!
//! Displays a grid of CG thumbnails with unlock status.
//! Features:
//! - 3x3 grid layout (9 CGs per page)
//! - Pagination with keyboard navigation
//! - Lock/unlock status display
//! - Unlock rate statistics

use narrative_core::{CgRegistry, UnlockData};
use narrative_engine::runtime::CgGalleryState;
use narrative_gui::Point;
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use narrative_gui::framework::input::{InputEvent, KeyCode};
use narrative_gui::framework::layout::Bounds;
use narrative_gui::theme::colors;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use taffy::{NodeId, Style};

/// Actions that can be confirmed by the CG gallery
#[derive(Debug, Clone, PartialEq)]
pub enum CgGalleryAction {
    /// Back to previous screen
    Back,
    /// View a specific CG (index in sorted CG list)
    ViewCg(usize),
}

/// CG Gallery UI element with grid layout
pub struct CgGalleryElement {
    id: ElementId,
    layout_node: Option<NodeId>,
    state: CgGalleryState,
    cg_registry: Arc<CgRegistry>,
    unlock_data: Arc<UnlockData>,
    confirmed_action: Option<CgGalleryAction>,
    /// Dirty flag for re-rendering
    dirty: bool,
    /// Animation context (reserved for future animation support)
    #[allow(dead_code)]
    animation_context: AnimationContext,
    /// Thumbnail textures (CgId -> TextureId)
    thumbnail_textures: HashMap<String, u64>,
}

impl CgGalleryElement {
    // Grid layout constants
    const GRID_COLS: usize = 3;
    const GRID_ROWS: usize = 3;
    const ITEMS_PER_PAGE: usize = Self::GRID_COLS * Self::GRID_ROWS;

    // Card dimensions
    const CARD_WIDTH: f32 = 320.0;
    const CARD_HEIGHT: f32 = 180.0;
    const CARD_SPACING: f32 = 20.0;
    const CORNER_RADIUS: f32 = 8.0;

    // UI constants
    const HEADER_HEIGHT: f32 = 100.0;
    const TITLE_FONT_SIZE: f32 = 36.0;
    const INFO_FONT_SIZE: f32 = 18.0;
    const HINT_FONT_SIZE: f32 = 16.0;
    const CG_TITLE_FONT_SIZE: f32 = 16.0;

    pub fn new(
        state: CgGalleryState,
        cg_registry: Arc<CgRegistry>,
        unlock_data: Arc<UnlockData>,
        thumbnail_textures: HashMap<String, u64>,
    ) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            state,
            cg_registry,
            unlock_data,
            confirmed_action: None,
            dirty: true,
            animation_context: AnimationContext::default(),
            thumbnail_textures,
        }
    }

    pub fn with_animation_context(mut self, context: AnimationContext) -> Self {
        self.animation_context = context;
        self
    }

    pub fn confirmed_action(&self) -> Option<CgGalleryAction> {
        self.confirmed_action.clone()
    }

    pub fn reset_confirmation(&mut self) {
        self.confirmed_action = None;
    }

    /// Calculate total number of pages
    fn total_pages(&self) -> usize {
        let total = self.state.total_cgs;
        total.div_ceil(Self::ITEMS_PER_PAGE)
    }

    /// Get grid position from selected CG index
    fn grid_position(&self) -> (usize, usize) {
        let index_in_page = self.state.selected_cg % Self::ITEMS_PER_PAGE;
        let row = index_in_page / Self::GRID_COLS;
        let col = index_in_page % Self::GRID_COLS;
        (row, col)
    }

    /// Move selection up
    fn select_up(&mut self) {
        let (row, _col) = self.grid_position();
        if row > 0 {
            self.state.selected_cg = self.state.selected_cg.saturating_sub(Self::GRID_COLS);
            self.dirty = true;
        }
    }

    /// Move selection down
    fn select_down(&mut self) {
        let (row, _col) = self.grid_position();
        if row < Self::GRID_ROWS - 1 {
            let new_index = self.state.selected_cg + Self::GRID_COLS;
            if new_index < self.state.total_cgs {
                self.state.selected_cg = new_index;
                self.dirty = true;
            }
        }
    }

    /// Move selection left
    fn select_left(&mut self) {
        let (_row, col) = self.grid_position();
        if col > 0 {
            self.state.selected_cg = self.state.selected_cg.saturating_sub(1);
            self.dirty = true;
        }
    }

    /// Move selection right
    fn select_right(&mut self) {
        let (_row, col) = self.grid_position();
        if col < Self::GRID_COLS - 1 {
            let new_index = self.state.selected_cg + 1;
            if new_index < self.state.total_cgs {
                self.state.selected_cg = new_index;
                self.dirty = true;
            }
        }
    }

    /// Go to previous page
    fn prev_page(&mut self) {
        if self.state.current_page > 0 {
            self.state.current_page -= 1;
            self.state.selected_cg = self.state.current_page * Self::ITEMS_PER_PAGE;
            self.dirty = true;
        }
    }

    /// Go to next page
    fn next_page(&mut self) {
        if self.state.current_page < self.total_pages().saturating_sub(1) {
            self.state.current_page += 1;
            self.state.selected_cg = self.state.current_page * Self::ITEMS_PER_PAGE;
            self.dirty = true;
        }
    }

    /// Confirm current selection
    fn confirm_selection(&mut self) {
        let sorted_cgs = self.cg_registry.get_all_sorted();
        if self.state.selected_cg < sorted_cgs.len() {
            let cg = &sorted_cgs[self.state.selected_cg];
            // Only allow viewing unlocked CGs
            if self.unlock_data.is_cg_unlocked(&cg.id) {
                self.confirmed_action = Some(CgGalleryAction::ViewCg(self.state.selected_cg));
                self.dirty = true;
            }
        }
    }
}

impl Element for CgGalleryElement {
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
        use taffy::prelude::*;

        Style {
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
        // Draw semi-transparent background overlay
        cx.fill_rect(cx.bounds, narrative_gui::Color::new(0.0, 0.0, 0.0, 0.85));

        // Calculate layout
        let content_width = (Self::CARD_WIDTH * Self::GRID_COLS as f32)
            + (Self::CARD_SPACING * (Self::GRID_COLS - 1) as f32);

        let start_x = cx.bounds.origin.x + (cx.bounds.size.width - content_width) / 2.0;
        let start_y = cx.bounds.origin.y + Self::HEADER_HEIGHT;

        // Draw header
        let title = "CG Gallery";
        let title_x = cx.bounds.origin.x + 50.0;
        let title_y = cx.bounds.origin.y + 40.0;
        cx.draw_text(
            title,
            Point::new(title_x, title_y),
            colors::TEXT_PRIMARY,
            Self::TITLE_FONT_SIZE,
        );

        // Draw unlock rate
        let unlock_rate = self
            .unlock_data
            .cg_unlock_rate(self.cg_registry.total_count());
        let unlock_text = format!(
            "Unlock Rate: {:.1}% ({}/{})",
            unlock_rate * 100.0,
            self.unlock_data.unlocked_cg_count(),
            self.cg_registry.total_count()
        );
        let info_x = cx.bounds.origin.x + 50.0;
        let info_y = title_y + Self::TITLE_FONT_SIZE + 10.0;
        cx.draw_text(
            &unlock_text,
            Point::new(info_x, info_y),
            colors::TEXT_SECONDARY,
            Self::INFO_FONT_SIZE,
        );

        // Draw page indicator
        let total_pages = self.total_pages();
        if total_pages > 1 {
            let page_text = format!("Page {}/{}", self.state.current_page + 1, total_pages);
            let page_x = cx.bounds.origin.x + cx.bounds.size.width - 150.0;
            cx.draw_text(
                &page_text,
                Point::new(page_x, info_y),
                colors::TEXT_SECONDARY,
                Self::INFO_FONT_SIZE,
            );
        }

        // Draw CG grid
        let sorted_cgs = self.cg_registry.get_all_sorted();
        let start_index = self.state.current_page * Self::ITEMS_PER_PAGE;

        for row in 0..Self::GRID_ROWS {
            for col in 0..Self::GRID_COLS {
                let cg_index = start_index + row * Self::GRID_COLS + col;
                if cg_index >= sorted_cgs.len() {
                    break;
                }

                let x = start_x + (col as f32 * (Self::CARD_WIDTH + Self::CARD_SPACING));
                let y = start_y + (row as f32 * (Self::CARD_HEIGHT + Self::CARD_SPACING));

                let card_bounds = Bounds {
                    origin: Point::new(x, y),
                    size: narrative_gui::Size::new(Self::CARD_WIDTH, Self::CARD_HEIGHT),
                };

                let cg = &sorted_cgs[cg_index];
                let is_unlocked = self.unlock_data.is_cg_unlocked(&cg.id);
                let is_selected = cg_index == self.state.selected_cg;

                // Card background
                let bg_color = if is_selected {
                    colors::ACCENT_PRIMARY
                } else if is_unlocked {
                    colors::CARD_BG
                } else {
                    narrative_gui::Color::new(0.2, 0.2, 0.2, 1.0)
                };

                cx.fill_rounded_rect(card_bounds, bg_color, Self::CORNER_RADIUS);

                // Card border
                if is_selected {
                    cx.stroke_rect(
                        card_bounds,
                        narrative_gui::Color::new(1.0, 1.0, 1.0, 1.0),
                        2.0,
                    );
                } else {
                    cx.stroke_rect(card_bounds, colors::BORDER_LIGHT, 1.0);
                }

                // Content
                if is_unlocked {
                    // Draw thumbnail texture if available
                    if let Some(&texture_id) = self.thumbnail_textures.get(&cg.id) {
                        cx.draw_texture(texture_id, card_bounds, 1.0);
                    } else {
                        // Fallback: Show CG title if thumbnail not loaded
                        let text_x = x + Self::CARD_WIDTH / 2.0 - 60.0;
                        let text_y = y + Self::CARD_HEIGHT / 2.0;
                        let text_color = if is_selected {
                            colors::BG_DARKEST
                        } else {
                            colors::TEXT_PRIMARY
                        };
                        cx.draw_text(
                            &cg.title,
                            Point::new(text_x, text_y),
                            text_color,
                            Self::CG_TITLE_FONT_SIZE,
                        );
                    }
                } else {
                    // Show lock icon (placeholder: "LOCKED" text)
                    let lock_x = x + Self::CARD_WIDTH / 2.0 - 30.0;
                    let lock_y = y + Self::CARD_HEIGHT / 2.0;
                    cx.draw_text(
                        "LOCKED",
                        Point::new(lock_x, lock_y),
                        narrative_gui::Color::new(0.4, 0.4, 0.4, 1.0),
                        Self::CG_TITLE_FONT_SIZE,
                    );
                }
            }
        }

        // Draw footer with hints
        let hint_text = "Arrow Keys: Select | Enter: View | ESC: Back | Q/E: Page";
        let hint_x = cx.bounds.origin.x + (cx.bounds.size.width / 2.0) - 250.0;
        let hint_y = cx.bounds.origin.y + cx.bounds.size.height - 30.0;
        cx.draw_text(
            hint_text,
            Point::new(hint_x, hint_y),
            colors::TEXT_SECONDARY,
            Self::HINT_FONT_SIZE,
        );
    }

    fn handle_event(&mut self, event: &InputEvent, _bounds: Bounds) -> bool {
        match event {
            InputEvent::KeyDown { key, .. } => match key {
                KeyCode::Escape => {
                    self.confirmed_action = Some(CgGalleryAction::Back);
                    self.dirty = true;
                    true
                }
                KeyCode::Up => {
                    self.select_up();
                    true
                }
                KeyCode::Down => {
                    self.select_down();
                    true
                }
                KeyCode::Left => {
                    self.select_left();
                    true
                }
                KeyCode::Right => {
                    self.select_right();
                    true
                }
                KeyCode::Enter | KeyCode::Space => {
                    self.confirm_selection();
                    true
                }
                KeyCode::Q | KeyCode::PageUp => {
                    self.prev_page();
                    true
                }
                KeyCode::E | KeyCode::PageDown => {
                    self.next_page();
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
