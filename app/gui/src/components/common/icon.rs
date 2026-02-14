//! Icon component for displaying simple icons
//!
//! Uses simple geometric shapes to represent icons since we don't have
//! actual icon fonts loaded yet.
//!
//! # TODO: Future Improvements
//!
//! - Replace geometric shapes with proper icon font (e.g., Material Icons, Lucide)
//! - Add SVG icon support for scalable vector graphics
//! - Extract magic numbers in draw_icon() to named constants
//! - Implement icon caching for frequently used icons

use crate::framework::Color;
use crate::framework::element::{Element, ElementId, LayoutContext, PaintContext};
use crate::framework::layout::{Bounds, Point};
use crate::theme::{colors, icon_size};
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

/// Available icon types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconType {
    // Navigation
    Home,
    Folder,
    Cloud,
    Settings,

    // Actions
    Plus,
    Play,
    Pause,
    Stop,

    // Media types
    Video,
    Audio,
    Image,

    // Tools
    Cut,
    Trim,
    Effects,
    Text,
    Transitions,

    // File operations
    Import,
    Export,
    Save,

    // Misc
    Search,
    User,
    Star,
    Clock,
    Grid,
    List,
}

/// Icon component
pub struct Icon {
    id: ElementId,
    layout_node: Option<NodeId>,
    icon_type: IconType,
    size: f32,
    color: Color,
}

impl Icon {
    pub fn new(icon_type: IconType) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            icon_type,
            size: icon_size::MD,
            color: colors::TEXT_SECONDARY,
        }
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Draw simple geometric representation of the icon
    fn draw_icon(&self, cx: &mut PaintContext) {
        let center_x = cx.bounds.x() + cx.bounds.width() / 2.0;
        let center_y = cx.bounds.y() + cx.bounds.height() / 2.0;
        let half = self.size / 2.0;

        match self.icon_type {
            IconType::Plus => {
                // Horizontal line
                cx.fill_rect(
                    Bounds::new(center_x - half * 0.6, center_y - 1.5, self.size * 0.6, 3.0),
                    self.color,
                );
                // Vertical line
                cx.fill_rect(
                    Bounds::new(center_x - 1.5, center_y - half * 0.6, 3.0, self.size * 0.6),
                    self.color,
                );
            }
            IconType::Play => {
                // Triangle (simplified as rect for now)
                cx.fill_rect(
                    Bounds::new(
                        center_x - half * 0.3,
                        center_y - half * 0.5,
                        self.size * 0.5,
                        self.size * 0.5,
                    ),
                    self.color,
                );
            }
            IconType::Folder => {
                // Folder body
                cx.fill_rounded_rect(
                    Bounds::new(
                        cx.bounds.x() + 2.0,
                        center_y - half * 0.3,
                        self.size - 4.0,
                        self.size * 0.5,
                    ),
                    self.color,
                    2.0,
                );
                // Folder tab
                cx.fill_rect(
                    Bounds::new(
                        cx.bounds.x() + 2.0,
                        center_y - half * 0.5,
                        self.size * 0.4,
                        4.0,
                    ),
                    self.color,
                );
            }
            IconType::Cloud => {
                // Cloud (simplified)
                cx.fill_rounded_rect(
                    Bounds::new(
                        center_x - half * 0.6,
                        center_y - half * 0.2,
                        self.size * 0.8,
                        self.size * 0.4,
                    ),
                    self.color,
                    self.size * 0.2,
                );
            }
            IconType::Video => {
                // Video rectangle
                cx.fill_rounded_rect(
                    Bounds::new(
                        cx.bounds.x() + 3.0,
                        center_y - half * 0.35,
                        self.size - 6.0,
                        self.size * 0.5,
                    ),
                    self.color,
                    2.0,
                );
            }
            IconType::Audio => {
                // Audio bars
                for i in 0..4 {
                    let bar_height = match i {
                        0 => 0.4,
                        1 => 0.7,
                        2 => 0.5,
                        _ => 0.6,
                    };
                    cx.fill_rect(
                        Bounds::new(
                            cx.bounds.x() + 4.0 + (i as f32 * 5.0),
                            center_y - half * bar_height,
                            3.0,
                            self.size * bar_height,
                        ),
                        self.color,
                    );
                }
            }
            IconType::Search => {
                // Search circle
                cx.stroke_rect(
                    Bounds::new(
                        center_x - half * 0.4,
                        center_y - half * 0.4,
                        self.size * 0.5,
                        self.size * 0.5,
                    ),
                    self.color,
                    2.0,
                );
                // Search handle
                cx.fill_rect(
                    Bounds::new(
                        center_x + half * 0.15,
                        center_y + half * 0.15,
                        self.size * 0.25,
                        3.0,
                    ),
                    self.color,
                );
            }
            IconType::User => {
                // Head
                cx.fill_rounded_rect(
                    Bounds::new(
                        center_x - half * 0.25,
                        center_y - half * 0.5,
                        self.size * 0.35,
                        self.size * 0.35,
                    ),
                    self.color,
                    self.size * 0.2,
                );
                // Body
                cx.fill_rounded_rect(
                    Bounds::new(
                        center_x - half * 0.4,
                        center_y + half * 0.05,
                        self.size * 0.55,
                        self.size * 0.35,
                    ),
                    self.color,
                    4.0,
                );
            }
            IconType::Star => {
                // Simplified star as diamond
                cx.fill_rect(
                    Bounds::new(center_x - 2.0, center_y - half * 0.5, 4.0, self.size * 0.5),
                    self.color,
                );
                cx.fill_rect(
                    Bounds::new(center_x - half * 0.5, center_y - 2.0, self.size * 0.5, 4.0),
                    self.color,
                );
            }
            IconType::Clock => {
                // Clock face
                cx.stroke_rect(
                    Bounds::new(
                        center_x - half * 0.45,
                        center_y - half * 0.45,
                        self.size * 0.6,
                        self.size * 0.6,
                    ),
                    self.color,
                    2.0,
                );
                // Clock hands
                cx.fill_rect(
                    Bounds::new(center_x - 1.0, center_y - half * 0.3, 2.0, half * 0.35),
                    self.color,
                );
                cx.fill_rect(
                    Bounds::new(center_x, center_y - 1.0, half * 0.25, 2.0),
                    self.color,
                );
            }
            IconType::Grid => {
                // Grid of 4 squares
                let square_size = self.size * 0.35;
                let gap = 3.0;
                for row in 0..2 {
                    for col in 0..2 {
                        cx.fill_rounded_rect(
                            Bounds::new(
                                cx.bounds.x() + 3.0 + (col as f32 * (square_size + gap)),
                                cx.bounds.y() + 3.0 + (row as f32 * (square_size + gap)),
                                square_size,
                                square_size,
                            ),
                            self.color,
                            2.0,
                        );
                    }
                }
            }
            _ => {
                // Default: simple square
                cx.fill_rounded_rect(
                    Bounds::new(
                        cx.bounds.x() + 3.0,
                        cx.bounds.y() + 3.0,
                        self.size - 6.0,
                        self.size - 6.0,
                    ),
                    self.color,
                    3.0,
                );
            }
        }
    }
}

impl Element for Icon {
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
            size: taffy::Size {
                width: Dimension::length(self.size),
                height: Dimension::length(self.size),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        self.draw_icon(cx);
    }

    fn tick(&mut self, delta: Duration) -> bool {
        let _ = delta;
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
