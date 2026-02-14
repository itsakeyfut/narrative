//! Save/Load menu element
//!
//! Main UI for saving and loading game progress.

use super::SaveSlotCard;
use narrative_engine::runtime::LayoutMode;
use narrative_engine::save::{SaveManager, SlotInfo, list_all_slots};
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use narrative_gui::framework::input::{InputEvent, KeyCode};
use narrative_gui::framework::layout::Bounds;
use narrative_gui::theme::{colors, font_size, spacing};
use narrative_gui::{Color, Point};
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use taffy::NodeId;

/// Save/Load menu action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveLoadMenuAction {
    /// Save to selected slot
    SaveToSlot(usize),
    /// Load from selected slot
    LoadFromSlot(usize),
    /// Delete selected slot
    DeleteSlot(usize),
    /// Go back to previous menu
    Back,
    /// Change to next page
    NextPage,
    /// Change to previous page
    PrevPage,
    /// Toggle layout mode (List ⇄ Grid)
    ToggleLayout,
}

/// Save/Load menu element
pub struct SaveLoadMenuElement {
    /// Unique element ID
    id: ElementId,
    /// Taffy layout node
    layout_node: Option<NodeId>,
    /// Save manager (unused in this element, operations are handled in GameRoot)
    #[allow(dead_code)]
    save_manager: Arc<Mutex<SaveManager>>,
    /// Current mode (Save or Load)
    is_save_mode: bool,
    /// Current page (0-indexed)
    current_page: usize,
    /// Layout mode
    layout_mode: LayoutMode,
    /// Slots per page
    slots_per_page: usize,
    /// All slot information
    all_slots: Vec<SlotInfo>,
    /// Total number of slots
    total_slots: usize,
    /// Selected slot index (global, across all pages)
    selected_slot: usize,
    /// Confirmed action
    action_confirmed: Option<SaveLoadMenuAction>,
    /// Dirty flag
    dirty: bool,
    /// Child elements (slot cards)
    children: Vec<Box<dyn Element>>,
    /// Animation context
    animation_context: AnimationContext,
}

impl SaveLoadMenuElement {
    /// Slots per page in List mode
    const SLOTS_PER_PAGE_LIST: usize = 6;
    /// Slots per page in Grid mode (3×3 grid)
    const SLOTS_PER_PAGE_GRID: usize = 9;
    /// Total slots supported
    const TOTAL_SLOTS: usize = 30;

    /// Create a new save/load menu element
    pub fn new(
        save_manager: Arc<Mutex<SaveManager>>,
        is_save_mode: bool,
        layout_mode: LayoutMode,
    ) -> Self {
        let slots_per_page = match layout_mode {
            LayoutMode::List => Self::SLOTS_PER_PAGE_LIST,
            LayoutMode::Grid => Self::SLOTS_PER_PAGE_GRID,
        };

        // Load all slot information
        let all_slots = match save_manager.lock() {
            Ok(manager) => list_all_slots(&manager, Self::TOTAL_SLOTS),
            Err(e) => {
                tracing::error!("Failed to lock save_manager during initialization: {:?}", e);
                // Return empty slots on error
                (0..Self::TOTAL_SLOTS).map(SlotInfo::empty).collect()
            }
        };

        Self {
            id: ElementId::new(),
            layout_node: None,
            save_manager,
            is_save_mode,
            current_page: 0,
            layout_mode,
            slots_per_page,
            all_slots,
            total_slots: Self::TOTAL_SLOTS,
            selected_slot: 0,
            action_confirmed: None,
            dirty: true,
            children: Vec::new(),
            animation_context: AnimationContext::default(),
        }
    }

    /// Set animation context
    pub fn with_animation_context(mut self, ctx: AnimationContext) -> Self {
        self.animation_context = ctx;
        self
    }

    /// Get confirmed action
    pub fn confirmed_action(&self) -> Option<SaveLoadMenuAction> {
        self.action_confirmed
    }

    /// Reset confirmation
    pub fn reset_confirmation(&mut self) {
        self.action_confirmed = None;
    }

    /// Get current page slot range
    fn current_page_slots(&self) -> Vec<&SlotInfo> {
        let start = self.current_page * self.slots_per_page;
        let end = (start + self.slots_per_page).min(self.total_slots);
        self.all_slots[start..end].iter().collect()
    }

    /// Calculate total pages
    fn total_pages(&self) -> usize {
        self.total_slots.div_ceil(self.slots_per_page)
    }

    /// Navigate to next page
    fn next_page(&mut self) {
        if self.current_page < self.total_pages() - 1 {
            self.current_page += 1;
            // Reset selection to first slot of new page
            self.selected_slot = self.current_page * self.slots_per_page;
            self.dirty = true;
        }
    }

    /// Navigate to previous page
    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            // Reset selection to first slot of new page
            self.selected_slot = self.current_page * self.slots_per_page;
            self.dirty = true;
        }
    }

    /// Toggle layout mode
    pub fn toggle_layout(&mut self) {
        self.layout_mode = match self.layout_mode {
            LayoutMode::List => LayoutMode::Grid,
            LayoutMode::Grid => LayoutMode::List,
        };
        self.slots_per_page = match self.layout_mode {
            LayoutMode::List => Self::SLOTS_PER_PAGE_LIST,
            LayoutMode::Grid => Self::SLOTS_PER_PAGE_GRID,
        };
        self.dirty = true;
    }

    /// Select next slot
    fn select_next(&mut self) {
        let slots_in_page = self.current_page_slots().len();
        let local_index = self.selected_slot % self.slots_per_page;
        if local_index < slots_in_page.saturating_sub(1) {
            self.selected_slot += 1;
            self.dirty = true;
        }
    }

    /// Select previous slot
    fn select_previous(&mut self) {
        let local_index = self.selected_slot % self.slots_per_page;
        if local_index > 0 {
            self.selected_slot -= 1;
            self.dirty = true;
        }
    }

    /// Confirm current selection
    fn confirm_selection(&mut self) {
        let global_slot =
            self.current_page * self.slots_per_page + (self.selected_slot % self.slots_per_page);

        if global_slot >= self.all_slots.len() {
            return;
        }

        let slot_info = &self.all_slots[global_slot];

        if self.is_save_mode {
            // Save mode: Always allow saving
            self.action_confirmed = Some(SaveLoadMenuAction::SaveToSlot(global_slot));
        } else {
            // Load mode: Only allow if slot exists
            if slot_info.exists {
                self.action_confirmed = Some(SaveLoadMenuAction::LoadFromSlot(global_slot));
            }
        }
        self.dirty = true;
    }

    /// Delete selected slot
    fn delete_slot(&mut self) {
        let global_slot =
            self.current_page * self.slots_per_page + (self.selected_slot % self.slots_per_page);

        if global_slot >= self.all_slots.len() {
            return;
        }

        let slot_info = &self.all_slots[global_slot];
        if slot_info.exists {
            self.action_confirmed = Some(SaveLoadMenuAction::DeleteSlot(global_slot));
            self.dirty = true;
        }
    }

    /// Rebuild children (slot cards)
    fn rebuild_children(&mut self) {
        self.children.clear();

        // Clone slot information to avoid borrow checker issues
        let start = self.current_page * self.slots_per_page;
        let end = (start + self.slots_per_page).min(self.total_slots);
        let slots: Vec<SlotInfo> = self.all_slots[start..end].to_vec();

        for (i, slot_info) in slots.iter().enumerate() {
            let global_slot = self.current_page * self.slots_per_page + i;
            let is_selected = global_slot == self.selected_slot;

            let card = SaveSlotCard::new(
                slot_info.clone(),
                is_selected,
                self.is_save_mode,
                self.layout_mode,
            )
            .with_animation_context(self.animation_context);

            self.children.push(Box::new(card));
        }
    }
}

impl Element for SaveLoadMenuElement {
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

        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: Some(AlignItems::Center),
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            padding: Rect::length(spacing::XL),
            gap: Size::length(spacing::MD),
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        // Background overlay
        let overlay_color = Color::new(0.0, 0.0, 0.0, 0.8);
        cx.fill_rect(cx.bounds, overlay_color);

        // Title
        let title = if self.is_save_mode {
            "Save Game"
        } else {
            "Load Game"
        };
        let title_y = cx.bounds.y() + spacing::XL;
        cx.draw_text(
            title,
            Point::new(cx.bounds.x() + spacing::XL, title_y),
            colors::TEXT_PRIMARY,
            font_size::TITLE,
        );

        // Layout mode toggle hint
        let layout_hint = match self.layout_mode {
            LayoutMode::List => "[Tab: Grid View]",
            LayoutMode::Grid => "[Tab: List View]",
        };
        cx.draw_text(
            layout_hint,
            Point::new(cx.bounds.x() + cx.bounds.width() - 200.0, title_y),
            colors::TEXT_SECONDARY,
            font_size::SM,
        );

        // Page info
        let page_info = format!("Page {} / {}", self.current_page + 1, self.total_pages());
        cx.draw_text(
            &page_info,
            Point::new(cx.bounds.x() + cx.bounds.width() / 2.0 - 50.0, title_y),
            colors::TEXT_SECONDARY,
            font_size::MD,
        );

        // Instructions at bottom
        let instructions_y = cx.bounds.y() + cx.bounds.height() - 60.0;
        cx.draw_text(
            "[↑↓] Select  [Enter] Confirm  [Delete] Delete  [←→] Page  [Esc] Back",
            Point::new(cx.bounds.x() + spacing::XL, instructions_y),
            colors::TEXT_SECONDARY,
            font_size::SM,
        );
    }

    fn handle_event(&mut self, event: &InputEvent, _bounds: Bounds) -> bool {
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
                KeyCode::Left => {
                    self.prev_page();
                    true
                }
                KeyCode::Right => {
                    self.next_page();
                    true
                }
                KeyCode::Enter | KeyCode::Space => {
                    self.confirm_selection();
                    true
                }
                KeyCode::Delete => {
                    self.delete_slot();
                    true
                }
                KeyCode::Escape => {
                    self.action_confirmed = Some(SaveLoadMenuAction::Back);
                    true
                }
                KeyCode::Tab => {
                    self.toggle_layout();
                    true
                }
                _ => false,
            },
            InputEvent::MouseDown { .. } => {
                // TODO: Handle mouse clicks on slot cards
                false
            }
            _ => false,
        }
    }

    fn children(&self) -> &[Box<dyn Element>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Element>] {
        &mut self.children
    }

    fn tick(&mut self, delta: Duration) -> bool {
        let was_dirty = self.dirty;

        // Rebuild children if dirty
        if self.dirty {
            self.rebuild_children();
            self.dirty = false;
        }

        // Tick children
        let mut needs_update = was_dirty;
        for child in &mut self.children {
            if child.tick(delta) {
                needs_update = true;
            }
        }

        needs_update
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
