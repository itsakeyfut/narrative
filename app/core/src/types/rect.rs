use serde::{Deserialize, Serialize};

/// 2D point
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Create a new point
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero point (0, 0)
    pub const ZERO: Self = Self::new(0.0, 0.0);
}

impl Default for Point {
    fn default() -> Self {
        Self::ZERO
    }
}

/// 2D size
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Create a new size
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Zero size (0, 0)
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Get the area (width * height)
    pub fn area(self) -> f32 {
        self.width * self.height
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::ZERO
    }
}

/// 2D rectangle
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rectangle from position and size
    pub const fn from_pos_size(pos: Point, size: Size) -> Self {
        Self::new(pos.x, pos.y, size.width, size.height)
    }

    /// Get the left edge
    pub fn left(self) -> f32 {
        self.x
    }

    /// Get the right edge
    pub fn right(self) -> f32 {
        self.x + self.width
    }

    /// Get the top edge
    pub fn top(self) -> f32 {
        self.y
    }

    /// Get the bottom edge
    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    /// Get the center point
    pub fn center(self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Get the position (top-left corner)
    pub fn position(self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Get the size
    pub fn size(self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Check if a point is inside the rectangle
    pub fn contains(self, point: Point) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(self, other: Rect) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
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
    fn test_point_zero() {
        let p = Point::ZERO;
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, 0.0);
    }

    #[test]
    fn test_point_default() {
        let p = Point::default();
        assert_eq!(p, Point::ZERO);
    }

    #[test]
    fn test_point_equality() {
        let p1 = Point::new(5.0, 10.0);
        let p2 = Point::new(5.0, 10.0);
        let p3 = Point::new(5.0, 11.0);
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_point_serialization() {
        let p = Point::new(3.5, 7.2);
        let serialized = serde_json::to_string(&p).unwrap();
        let deserialized: Point = serde_json::from_str(&serialized).unwrap();
        assert_eq!(p, deserialized);
    }

    #[test]
    fn test_size_new() {
        let s = Size::new(100.0, 200.0);
        assert_eq!(s.width, 100.0);
        assert_eq!(s.height, 200.0);
    }

    #[test]
    fn test_size_zero() {
        let s = Size::ZERO;
        assert_eq!(s.width, 0.0);
        assert_eq!(s.height, 0.0);
    }

    #[test]
    fn test_size_default() {
        let s = Size::default();
        assert_eq!(s, Size::ZERO);
    }

    #[test]
    fn test_size_area() {
        let s = Size::new(10.0, 20.0);
        assert_eq!(s.area(), 200.0);

        let zero = Size::ZERO;
        assert_eq!(zero.area(), 0.0);
    }

    #[test]
    fn test_size_serialization() {
        let s = Size::new(50.0, 100.0);
        let serialized = serde_json::to_string(&s).unwrap();
        let deserialized: Size = serde_json::from_str(&serialized).unwrap();
        assert_eq!(s, deserialized);
    }

    #[test]
    fn test_rect_new() {
        let r = Rect::new(10.0, 20.0, 100.0, 200.0);
        assert_eq!(r.x, 10.0);
        assert_eq!(r.y, 20.0);
        assert_eq!(r.width, 100.0);
        assert_eq!(r.height, 200.0);
    }

    #[test]
    fn test_rect_from_pos_size() {
        let pos = Point::new(5.0, 10.0);
        let size = Size::new(50.0, 100.0);
        let r = Rect::from_pos_size(pos, size);
        assert_eq!(r.x, 5.0);
        assert_eq!(r.y, 10.0);
        assert_eq!(r.width, 50.0);
        assert_eq!(r.height, 100.0);
    }

    #[test]
    fn test_rect_edges() {
        let r = Rect::new(10.0, 20.0, 100.0, 200.0);
        assert_eq!(r.left(), 10.0);
        assert_eq!(r.right(), 110.0);
        assert_eq!(r.top(), 20.0);
        assert_eq!(r.bottom(), 220.0);
    }

    #[test]
    fn test_rect_center() {
        let r = Rect::new(0.0, 0.0, 100.0, 200.0);
        let center = r.center();
        assert_eq!(center.x, 50.0);
        assert_eq!(center.y, 100.0);
    }

    #[test]
    fn test_rect_position() {
        let r = Rect::new(10.0, 20.0, 100.0, 200.0);
        let pos = r.position();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }

    #[test]
    fn test_rect_size() {
        let r = Rect::new(10.0, 20.0, 100.0, 200.0);
        let size = r.size();
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 200.0);
    }

    #[test]
    fn test_rect_contains_point_inside() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(Point::new(50.0, 50.0)));
        assert!(r.contains(Point::new(0.0, 0.0))); // edge
        assert!(r.contains(Point::new(100.0, 100.0))); // edge
    }

    #[test]
    fn test_rect_contains_point_outside() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(!r.contains(Point::new(-10.0, 50.0)));
        assert!(!r.contains(Point::new(150.0, 50.0)));
        assert!(!r.contains(Point::new(50.0, -10.0)));
        assert!(!r.contains(Point::new(50.0, 150.0)));
    }

    #[test]
    fn test_rect_intersects_overlapping() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        assert!(r1.intersects(r2));
        assert!(r2.intersects(r1));
    }

    #[test]
    fn test_rect_intersects_separate() {
        let r1 = Rect::new(0.0, 0.0, 50.0, 50.0);
        let r2 = Rect::new(100.0, 100.0, 50.0, 50.0);
        assert!(!r1.intersects(r2));
        assert!(!r2.intersects(r1));
    }

    #[test]
    fn test_rect_intersects_touching() {
        let r1 = Rect::new(0.0, 0.0, 50.0, 50.0);
        let r2 = Rect::new(50.0, 0.0, 50.0, 50.0);
        // Touching edges don't intersect (strict inequality)
        assert!(!r1.intersects(r2));
    }

    #[test]
    fn test_rect_default() {
        let r = Rect::default();
        assert_eq!(r.x, 0.0);
        assert_eq!(r.y, 0.0);
        assert_eq!(r.width, 0.0);
        assert_eq!(r.height, 0.0);
    }

    #[test]
    fn test_rect_serialization() {
        let r = Rect::new(10.0, 20.0, 30.0, 40.0);
        let serialized = serde_json::to_string(&r).unwrap();
        let deserialized: Rect = serde_json::from_str(&serialized).unwrap();
        assert_eq!(r, deserialized);
    }
}
