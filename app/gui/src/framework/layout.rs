//! Layout primitives and taffy integration

use taffy::prelude::*;

use super::error::{FrameworkError, FrameworkResult};

/// A 2D point
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(self, other: Point) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// A 2D size
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn area(self) -> f32 {
        self.width * self.height
    }
}

/// A rectangle defined by position and size
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Bounds {
    pub origin: Point,
    pub size: Size,
}

impl Bounds {
    pub const ZERO: Self = Self {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn from_points(top_left: Point, bottom_right: Point) -> Self {
        Self {
            origin: top_left,
            size: Size::new(bottom_right.x - top_left.x, bottom_right.y - top_left.y),
        }
    }

    pub fn x(&self) -> f32 {
        self.origin.x
    }

    pub fn y(&self) -> f32 {
        self.origin.y
    }

    pub fn width(&self) -> f32 {
        self.size.width
    }

    pub fn height(&self) -> f32 {
        self.size.height
    }

    pub fn right(&self) -> f32 {
        self.origin.x + self.size.width
    }

    pub fn bottom(&self) -> f32 {
        self.origin.y + self.size.height
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.origin.x + self.size.width / 2.0,
            self.origin.y + self.size.height / 2.0,
        )
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.x <= self.right()
            && point.y >= self.origin.y
            && point.y <= self.bottom()
    }

    pub fn intersects(&self, other: &Bounds) -> bool {
        self.origin.x < other.right()
            && self.right() > other.origin.x
            && self.origin.y < other.bottom()
            && self.bottom() > other.origin.y
    }

    pub fn intersection(&self, other: &Bounds) -> Option<Bounds> {
        if !self.intersects(other) {
            return None;
        }

        let x = self.origin.x.max(other.origin.x);
        let y = self.origin.y.max(other.origin.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        Some(Bounds::new(x, y, right - x, bottom - y))
    }

    /// Expand bounds by a uniform amount in all directions
    pub fn inset(&self, amount: f32) -> Bounds {
        Bounds::new(
            self.origin.x + amount,
            self.origin.y + amount,
            (self.size.width - 2.0 * amount).max(0.0),
            (self.size.height - 2.0 * amount).max(0.0),
        )
    }

    /// Expand bounds by a uniform amount in all directions
    pub fn expand(&self, amount: f32) -> Bounds {
        self.inset(-amount)
    }
}

/// Edge insets (padding/margin)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

impl From<EdgeInsets> for taffy::Rect<taffy::LengthPercentage> {
    fn from(insets: EdgeInsets) -> Self {
        taffy::Rect {
            top: taffy::LengthPercentage::length(insets.top),
            right: taffy::LengthPercentage::length(insets.right),
            bottom: taffy::LengthPercentage::length(insets.bottom),
            left: taffy::LengthPercentage::length(insets.left),
        }
    }
}

/// Layout engine wrapping taffy
pub struct LayoutEngine {
    taffy: TaffyTree,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
        }
    }

    /// Create a new layout node with the given style
    pub fn new_node(&mut self, style: Style) -> FrameworkResult<NodeId> {
        self.taffy
            .new_leaf(style)
            .map_err(|e| FrameworkError::Layout(format!("Failed to create node: {e}")))
    }

    /// Create a new layout node with children
    pub fn new_node_with_children(
        &mut self,
        style: Style,
        children: &[NodeId],
    ) -> FrameworkResult<NodeId> {
        self.taffy.new_with_children(style, children).map_err(|e| {
            FrameworkError::Layout(format!("Failed to create node with children: {e}"))
        })
    }

    /// Set children for a node
    pub fn set_children(&mut self, node: NodeId, children: &[NodeId]) -> FrameworkResult<()> {
        self.taffy
            .set_children(node, children)
            .map_err(|e| FrameworkError::Layout(format!("Failed to set children: {e}")))
    }

    /// Update node style
    pub fn set_style(&mut self, node: NodeId, style: Style) -> FrameworkResult<()> {
        self.taffy
            .set_style(node, style)
            .map_err(|e| FrameworkError::Layout(format!("Failed to set style: {e}")))
    }

    /// Compute layout for the tree rooted at the given node
    pub fn compute_layout(&mut self, root: NodeId, available_space: Size) -> FrameworkResult<()> {
        self.taffy
            .compute_layout(
                root,
                taffy::Size {
                    width: AvailableSpace::Definite(available_space.width),
                    height: AvailableSpace::Definite(available_space.height),
                },
            )
            .map_err(|e| FrameworkError::Layout(format!("Failed to compute layout: {e}")))
    }

    /// Get the computed bounds for a node
    pub fn get_bounds(&self, node: NodeId) -> FrameworkResult<Bounds> {
        let layout = self
            .taffy
            .layout(node)
            .map_err(|e| FrameworkError::Layout(format!("Node not found: {e}")))?;
        Ok(Bounds::new(
            layout.location.x,
            layout.location.y,
            layout.size.width,
            layout.size.height,
        ))
    }

    /// Remove a node from the tree
    pub fn remove_node(&mut self, node: NodeId) {
        self.taffy.remove(node).ok();
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_new() {
        let p = Point::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert!((p1.distance_to(p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_size_area() {
        let s = Size::new(10.0, 5.0);
        assert_eq!(s.area(), 50.0);
    }

    #[test]
    fn test_bounds_new() {
        let b = Bounds::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(b.x(), 10.0);
        assert_eq!(b.y(), 20.0);
        assert_eq!(b.width(), 100.0);
        assert_eq!(b.height(), 50.0);
        assert_eq!(b.right(), 110.0);
        assert_eq!(b.bottom(), 70.0);
    }

    #[test]
    fn test_bounds_center() {
        let b = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let c = b.center();
        assert_eq!(c.x, 50.0);
        assert_eq!(c.y, 50.0);
    }

    #[test]
    fn test_bounds_contains() {
        let b = Bounds::new(0.0, 0.0, 100.0, 100.0);
        assert!(b.contains(Point::new(50.0, 50.0)));
        assert!(b.contains(Point::new(0.0, 0.0)));
        assert!(b.contains(Point::new(100.0, 100.0)));
        assert!(!b.contains(Point::new(-1.0, 50.0)));
        assert!(!b.contains(Point::new(101.0, 50.0)));
    }

    #[test]
    fn test_bounds_intersects() {
        let b1 = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let b2 = Bounds::new(50.0, 50.0, 100.0, 100.0);
        let b3 = Bounds::new(200.0, 200.0, 50.0, 50.0);
        assert!(b1.intersects(&b2));
        assert!(!b1.intersects(&b3));
    }

    #[test]
    fn test_bounds_intersection() {
        let b1 = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let b2 = Bounds::new(50.0, 50.0, 100.0, 100.0);
        let intersection = b1.intersection(&b2).unwrap();
        assert_eq!(intersection.x(), 50.0);
        assert_eq!(intersection.y(), 50.0);
        assert_eq!(intersection.width(), 50.0);
        assert_eq!(intersection.height(), 50.0);
    }

    #[test]
    fn test_bounds_inset() {
        let b = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let inset = b.inset(10.0);
        assert_eq!(inset.x(), 10.0);
        assert_eq!(inset.y(), 10.0);
        assert_eq!(inset.width(), 80.0);
        assert_eq!(inset.height(), 80.0);
    }

    #[test]
    fn test_layout_engine_new_node() {
        let mut engine = LayoutEngine::new();
        let node = engine.new_node(taffy::Style::default());
        assert!(node.is_ok());
    }

    #[test]
    fn test_layout_engine_with_children() {
        let mut engine = LayoutEngine::new();
        let child1 = engine.new_node(taffy::Style::default()).unwrap();
        let child2 = engine.new_node(taffy::Style::default()).unwrap();
        let parent = engine.new_node_with_children(taffy::Style::default(), &[child1, child2]);
        assert!(parent.is_ok());
    }

    #[test]
    fn test_layout_engine_compute_layout() {
        let mut engine = LayoutEngine::new();
        let node = engine
            .new_node(taffy::Style {
                size: taffy::Size {
                    width: taffy::Dimension::length(100.0),
                    height: taffy::Dimension::length(50.0),
                },
                ..Default::default()
            })
            .unwrap();

        let result = engine.compute_layout(node, Size::new(800.0, 600.0));
        assert!(result.is_ok());

        let bounds = engine.get_bounds(node).unwrap();
        assert_eq!(bounds.width(), 100.0);
        assert_eq!(bounds.height(), 50.0);
    }
}
