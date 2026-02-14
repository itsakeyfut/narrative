//! Dirty tracking system for elements
//!
//! Tracks which elements have changed and need to be repainted,
//! enabling partial redraws and reducing GPU work.
//!
//! Issue #250: Optimize GUI framework for 60+ FPS (target: 120 FPS)

use super::element::ElementId;
use super::layout::Bounds;
use std::collections::HashSet;

/// State of an element's dirtiness
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyState {
    /// Element is clean, no repaint needed
    Clean,
    /// Element's visual properties changed, needs repaint
    Visual,
    /// Element's layout changed, needs relayout and repaint
    Layout,
    /// Element is newly created
    New,
}

impl DirtyState {
    /// Check if element needs repaint
    pub fn needs_repaint(&self) -> bool {
        matches!(self, Self::Visual | Self::Layout | Self::New)
    }

    /// Check if element needs relayout
    pub fn needs_relayout(&self) -> bool {
        matches!(self, Self::Layout | Self::New)
    }
}

/// Tracks dirty state across the element tree
pub struct DirtyTracker {
    /// Whether a full redraw is needed (e.g., on resize)
    full_redraw: bool,
    /// Whether a full relayout is needed
    full_relayout: bool,
    /// Set of elements that need repaint
    dirty_elements: HashSet<ElementId>,
    /// Set of elements that need relayout
    relayout_elements: HashSet<ElementId>,
    /// Dirty bounds (union of all dirty element bounds)
    dirty_bounds: Vec<Bounds>,
}

impl DirtyTracker {
    /// Create a new dirty tracker
    pub fn new() -> Self {
        Self {
            full_redraw: true, // First frame needs full redraw
            full_relayout: true,
            dirty_elements: HashSet::new(),
            relayout_elements: HashSet::new(),
            dirty_bounds: Vec::new(),
        }
    }

    /// Request a full redraw of the entire window
    pub fn request_full_redraw(&mut self) {
        self.full_redraw = true;
    }

    /// Request a full relayout
    pub fn request_full_relayout(&mut self) {
        self.full_relayout = true;
    }

    /// Mark an element as needing repaint
    pub fn mark_dirty(&mut self, id: ElementId, bounds: Bounds) {
        self.dirty_elements.insert(id);
        self.dirty_bounds.push(bounds);
    }

    /// Mark an element as needing relayout
    pub fn mark_relayout(&mut self, id: ElementId) {
        self.relayout_elements.insert(id);
    }

    /// Check if any repaint is needed
    pub fn needs_redraw(&self) -> bool {
        self.full_redraw || !self.dirty_elements.is_empty()
    }

    /// Check if full redraw is needed
    pub fn needs_full_redraw(&self) -> bool {
        self.full_redraw
    }

    /// Check if relayout is needed
    pub fn needs_relayout(&self) -> bool {
        self.full_relayout || !self.relayout_elements.is_empty()
    }

    /// Check if full relayout is needed
    pub fn needs_full_relayout(&self) -> bool {
        self.full_relayout
    }

    /// Check if a specific element needs repaint
    pub fn is_dirty(&self, id: ElementId) -> bool {
        self.full_redraw || self.dirty_elements.contains(&id)
    }

    /// Check if a specific element needs relayout
    pub fn needs_element_relayout(&self, id: ElementId) -> bool {
        self.full_relayout || self.relayout_elements.contains(&id)
    }

    /// Get the number of dirty elements
    pub fn dirty_count(&self) -> usize {
        if self.full_redraw {
            0 // Full redraw means all elements are dirty
        } else {
            self.dirty_elements.len()
        }
    }

    /// Get the damage region (union of all dirty bounds)
    ///
    /// Returns `None` if full redraw is needed (entire window is damaged)
    pub fn damage_region(&self) -> Option<Bounds> {
        if self.full_redraw || self.dirty_bounds.is_empty() {
            return None;
        }

        // Compute union of all dirty bounds
        let mut result = self.dirty_bounds[0];
        for bounds in &self.dirty_bounds[1..] {
            result = union_bounds(result, *bounds);
        }

        Some(result)
    }

    /// Clear dirty state after frame render
    pub fn clear(&mut self) {
        self.full_redraw = false;
        self.full_relayout = false;
        self.dirty_elements.clear();
        self.relayout_elements.clear();
        self.dirty_bounds.clear();
    }
}

impl Default for DirtyTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the union of two bounds
fn union_bounds(a: Bounds, b: Bounds) -> Bounds {
    let x = a.x().min(b.x());
    let y = a.y().min(b.y());
    let right = a.right().max(b.right());
    let bottom = a.bottom().max(b.bottom());
    Bounds::new(x, y, right - x, bottom - y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_state() {
        assert!(DirtyState::Visual.needs_repaint());
        assert!(DirtyState::Layout.needs_repaint());
        assert!(DirtyState::New.needs_repaint());
        assert!(!DirtyState::Clean.needs_repaint());

        assert!(!DirtyState::Visual.needs_relayout());
        assert!(DirtyState::Layout.needs_relayout());
        assert!(DirtyState::New.needs_relayout());
    }

    #[test]
    fn test_dirty_tracker_new() {
        let tracker = DirtyTracker::new();
        // First frame needs full redraw
        assert!(tracker.needs_redraw());
        assert!(tracker.needs_full_redraw());
    }

    #[test]
    fn test_dirty_tracker_mark_dirty() {
        let mut tracker = DirtyTracker::new();
        tracker.clear();

        let id = ElementId::new();
        let bounds = Bounds::new(10.0, 20.0, 100.0, 50.0);

        tracker.mark_dirty(id, bounds);

        assert!(tracker.needs_redraw());
        assert!(!tracker.needs_full_redraw());
        assert!(tracker.is_dirty(id));
        assert_eq!(tracker.dirty_count(), 1);

        let damage = tracker.damage_region().unwrap();
        assert_eq!(damage.x(), 10.0);
        assert_eq!(damage.y(), 20.0);
    }

    #[test]
    fn test_damage_region_union() {
        let mut tracker = DirtyTracker::new();
        tracker.clear();

        let id1 = ElementId::new();
        let id2 = ElementId::new();

        tracker.mark_dirty(id1, Bounds::new(0.0, 0.0, 50.0, 50.0));
        tracker.mark_dirty(id2, Bounds::new(25.0, 25.0, 50.0, 50.0));

        let damage = tracker.damage_region().unwrap();
        assert_eq!(damage.x(), 0.0);
        assert_eq!(damage.y(), 0.0);
        assert_eq!(damage.width(), 75.0);
        assert_eq!(damage.height(), 75.0);
    }

    #[test]
    fn test_dirty_tracker_clear() {
        let mut tracker = DirtyTracker::new();
        tracker.clear();

        let id = ElementId::new();
        tracker.mark_dirty(id, Bounds::ZERO);

        tracker.clear();

        assert!(!tracker.needs_redraw());
        assert!(!tracker.is_dirty(id));
        assert_eq!(tracker.dirty_count(), 0);
    }
}
