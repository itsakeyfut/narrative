//! Batched draw call optimization
//!
//! Issue #250 Phase 2: Optimizes draw calls by sorting and grouping commands
//! to minimize GPU pipeline state changes.
//!
//! Key optimizations:
//! - Z-order sorting for correct layering
//! - Command type grouping to minimize pipeline switches
//! - Draw call counting for metrics

use super::{DrawCommand, TextDraw};
use crate::framework::Color;
use crate::framework::layout::{Bounds, Point};
use std::sync::Arc;

/// Layer identifier for z-ordering
///
/// Lower layers are rendered first (further back).
/// Elements within the same layer maintain their original order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ZLayer(pub i32);

impl ZLayer {
    /// Background layer (rendered first)
    pub const BACKGROUND: ZLayer = ZLayer(-1000);
    /// Default layer for most UI elements
    pub const DEFAULT: ZLayer = ZLayer(0);
    /// Overlay layer (rendered on top of most elements)
    pub const OVERLAY: ZLayer = ZLayer(1000);
    /// Tooltip/popup layer (rendered on top of overlays)
    pub const POPUP: ZLayer = ZLayer(2000);
    /// Debug/metrics overlay (always on top)
    pub const DEBUG: ZLayer = ZLayer(i32::MAX);
}

/// A draw command with z-order information
#[derive(Debug, Clone)]
pub struct LayeredCommand {
    /// The draw command
    pub command: DrawCommand,
    /// Z-layer for ordering
    pub layer: ZLayer,
    /// Original insertion order (for stable sorting within layers)
    pub order: u32,
}

/// Result of batch building with draw call statistics
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    /// Number of GPU draw calls that will be issued
    pub draw_calls: u32,
    /// Number of quad instances
    pub quad_count: u32,
    /// Number of text draws
    pub text_count: u32,
    // video_count removed - was video-editing specific
    // /// Number of video frames
    // pub video_count: u32,
    /// Number of pipeline state changes
    pub pipeline_switches: u32,
}

/// Command type for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CommandType {
    // Video removed - was video-editing specific
    Rect,
    Border,
    Text,
    Texture,
    Clip,
}

impl CommandType {
    fn from_command(cmd: &DrawCommand) -> Self {
        match cmd {
            // VideoFrame removed - was video-editing specific
            DrawCommand::Rect { .. } => CommandType::Rect,
            DrawCommand::Border { .. } => CommandType::Border,
            DrawCommand::Text { .. } => CommandType::Text,
            DrawCommand::Texture { .. } => CommandType::Texture,
            DrawCommand::PushClip { .. } | DrawCommand::PopClip => CommandType::Clip,
        }
    }
}

/// Builder for batching and optimizing draw commands
///
/// Collects commands with layer information and produces an optimized
/// draw order that minimizes pipeline state changes.
pub struct BatchBuilder {
    commands: Vec<LayeredCommand>,
    current_order: u32,
}

impl BatchBuilder {
    /// Create a new batch builder
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(256),
            current_order: 0,
        }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            commands: Vec::with_capacity(capacity),
            current_order: 0,
        }
    }

    /// Add a command at the default layer
    pub fn push(&mut self, command: DrawCommand) {
        self.push_at_layer(command, ZLayer::DEFAULT);
    }

    /// Add a command at a specific layer
    pub fn push_at_layer(&mut self, command: DrawCommand, layer: ZLayer) {
        self.commands.push(LayeredCommand {
            command,
            layer,
            order: self.current_order,
        });
        self.current_order += 1;
    }

    /// Add a rectangle at the default layer
    pub fn rect(&mut self, bounds: Bounds, color: Color, corner_radius: f32) {
        self.push(DrawCommand::Rect {
            bounds,
            color,
            corner_radius,
        });
    }

    /// Add a rectangle at a specific layer
    pub fn rect_at_layer(
        &mut self,
        bounds: Bounds,
        color: Color,
        corner_radius: f32,
        layer: ZLayer,
    ) {
        self.push_at_layer(
            DrawCommand::Rect {
                bounds,
                color,
                corner_radius,
            },
            layer,
        );
    }

    /// Add text at the default layer
    pub fn text(&mut self, text: String, position: Point, color: Color, font_size: f32) {
        self.push(DrawCommand::Text {
            text,
            position,
            color,
            font_size,
        });
    }

    /// Add text at a specific layer
    pub fn text_at_layer(
        &mut self,
        text: String,
        position: Point,
        color: Color,
        font_size: f32,
        layer: ZLayer,
    ) {
        self.push_at_layer(
            DrawCommand::Text {
                text,
                position,
                color,
                font_size,
            },
            layer,
        );
    }

    // Video frame method removed - was video-editing specific
    // /// Add a video frame at the background layer
    // pub fn video_frame(&mut self, data: Arc<Vec<u8>>, width: u32, height: u32, bounds: Bounds) {
    //     self.push_at_layer(
    //         DrawCommand::VideoFrame {
    //             data,
    //             width,
    //             height,
    //             bounds,
    //         },
    //         ZLayer::BACKGROUND,
    //     );
    // }

    /// Clear all commands
    pub fn clear(&mut self) {
        self.commands.clear();
        self.current_order = 0;
    }

    /// Get the number of commands
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Build the optimized command list and return statistics
    ///
    /// This sorts commands for optimal rendering order:
    /// 1. First by z-layer (lower layers first)
    /// 2. Within each layer, group by command type to minimize pipeline switches
    /// 3. Within each type group, maintain original order for deterministic rendering
    pub fn build(mut self) -> (Vec<DrawCommand>, BatchStats) {
        if self.commands.is_empty() {
            return (Vec::new(), BatchStats::default());
        }

        // Sort by layer, then by command type for batching, then by original order
        self.commands.sort_by(|a, b| {
            a.layer
                .cmp(&b.layer)
                .then_with(|| {
                    CommandType::from_command(&a.command)
                        .cmp(&CommandType::from_command(&b.command))
                })
                .then_with(|| a.order.cmp(&b.order))
        });

        let mut stats = BatchStats::default();
        let mut last_type: Option<CommandType> = None;

        for cmd in &self.commands {
            let cmd_type = CommandType::from_command(&cmd.command);

            // Count pipeline switches
            if last_type != Some(cmd_type) {
                if last_type.is_some() {
                    stats.pipeline_switches += 1;
                }
                last_type = Some(cmd_type);
            }

            // Count by type
            match &cmd.command {
                DrawCommand::Rect { .. } => stats.quad_count += 1,
                DrawCommand::Border { .. } => stats.quad_count += 4, // Border = 4 quads
                DrawCommand::Text { .. } => stats.text_count += 1,
                // VideoFrame removed - was video-editing specific
                _ => {}
            }
        }

        // Calculate actual draw calls
        // - All quads batch into 1 draw call (instanced rendering)
        // - All text batches into 1 draw call (atlas-based)
        // - Each video frame is 1 draw call (removed)
        if stats.quad_count > 0 {
            stats.draw_calls += 1;
        }
        if stats.text_count > 0 {
            stats.draw_calls += 1;
        }
        // video_count removed

        let commands = self.commands.into_iter().map(|lc| lc.command).collect();

        (commands, stats)
    }

    /// Build commands grouped by layer for proper z-order rendering
    ///
    /// Returns a Vec of (layer, commands) pairs sorted by layer.
    /// Each layer's commands are sorted by type then original order.
    pub fn build_by_layer(mut self) -> (Vec<(ZLayer, Vec<DrawCommand>)>, BatchStats) {
        if self.commands.is_empty() {
            return (Vec::new(), BatchStats::default());
        }

        // Sort by layer, then by command type for batching, then by original order
        self.commands.sort_by(|a, b| {
            a.layer
                .cmp(&b.layer)
                .then_with(|| {
                    CommandType::from_command(&a.command)
                        .cmp(&CommandType::from_command(&b.command))
                })
                .then_with(|| a.order.cmp(&b.order))
        });

        let mut stats = BatchStats::default();
        let mut layers: Vec<(ZLayer, Vec<DrawCommand>)> = Vec::new();
        let mut current_layer: Option<ZLayer> = None;

        for cmd in self.commands {
            // Count by type
            match &cmd.command {
                DrawCommand::Rect { .. } => stats.quad_count += 1,
                DrawCommand::Border { .. } => stats.quad_count += 4,
                DrawCommand::Text { .. } => stats.text_count += 1,
                // VideoFrame removed - was video-editing specific
                _ => {}
            }

            // Group by layer
            if current_layer != Some(cmd.layer) {
                current_layer = Some(cmd.layer);
                layers.push((cmd.layer, Vec::new()));
            }

            if let Some((_, cmds)) = layers.last_mut() {
                cmds.push(cmd.command);
            }
        }

        // Count draw calls per layer (each layer may have quads + text)
        for (_, cmds) in &layers {
            let has_quads = cmds
                .iter()
                .any(|c| matches!(c, DrawCommand::Rect { .. } | DrawCommand::Border { .. }));
            let has_text = cmds.iter().any(|c| matches!(c, DrawCommand::Text { .. }));
            // video_count removed - was video-editing specific

            if has_quads {
                stats.draw_calls += 1;
            }
            if has_text {
                stats.draw_calls += 1;
            }
            // video_count removed
        }

        (layers, stats)
    }

    /// Build without consuming self (for inspection)
    pub fn build_preview(&self) -> BatchStats {
        if self.commands.is_empty() {
            return BatchStats::default();
        }

        let mut stats = BatchStats::default();

        for cmd in &self.commands {
            match &cmd.command {
                DrawCommand::Rect { .. } => stats.quad_count += 1,
                DrawCommand::Border { .. } => stats.quad_count += 4,
                DrawCommand::Text { .. } => stats.text_count += 1,
                // VideoFrame removed - was video-editing specific
                _ => {}
            }
        }

        // Same draw call calculation as build()
        if stats.quad_count > 0 {
            stats.draw_calls += 1;
        }
        if stats.text_count > 0 {
            stats.draw_calls += 1;
        }
        // video_count removed

        stats
    }
}

impl Default for BatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_builder_empty() {
        let builder = BatchBuilder::new();
        let (commands, stats) = builder.build();
        assert!(commands.is_empty());
        assert_eq!(stats.draw_calls, 0);
    }

    #[test]
    fn test_batch_builder_single_rect() {
        let mut builder = BatchBuilder::new();
        builder.rect(Bounds::new(0.0, 0.0, 100.0, 100.0), Color::RED, 0.0);

        let (commands, stats) = builder.build();
        assert_eq!(commands.len(), 1);
        assert_eq!(stats.quad_count, 1);
        assert_eq!(stats.draw_calls, 1);
    }

    #[test]
    fn test_batch_builder_multiple_quads_single_draw_call() {
        let mut builder = BatchBuilder::new();
        for i in 0..10 {
            builder.rect(
                Bounds::new(i as f32 * 10.0, 0.0, 10.0, 10.0),
                Color::BLUE,
                0.0,
            );
        }

        let (commands, stats) = builder.build();
        assert_eq!(commands.len(), 10);
        assert_eq!(stats.quad_count, 10);
        // All quads batch into single draw call
        assert_eq!(stats.draw_calls, 1);
    }

    #[test]
    fn test_batch_builder_mixed_types() {
        let mut builder = BatchBuilder::new();
        builder.rect(Bounds::new(0.0, 0.0, 100.0, 100.0), Color::RED, 0.0);
        builder.text(
            "Hello".to_string(),
            Point::new(10.0, 10.0),
            Color::WHITE,
            12.0,
        );
        builder.rect(Bounds::new(50.0, 50.0, 100.0, 100.0), Color::GREEN, 0.0);

        let (commands, stats) = builder.build();
        assert_eq!(commands.len(), 3);
        assert_eq!(stats.quad_count, 2);
        assert_eq!(stats.text_count, 1);
        // 1 draw call for quads + 1 for text = 2 draw calls
        assert_eq!(stats.draw_calls, 2);
    }

    #[test]
    fn test_batch_builder_z_ordering() {
        let mut builder = BatchBuilder::new();

        // Add in reverse z-order
        builder.rect_at_layer(
            Bounds::new(0.0, 0.0, 100.0, 100.0),
            Color::RED,
            0.0,
            ZLayer::OVERLAY,
        );
        builder.rect_at_layer(
            Bounds::new(0.0, 0.0, 100.0, 100.0),
            Color::GREEN,
            0.0,
            ZLayer::DEFAULT,
        );
        builder.rect_at_layer(
            Bounds::new(0.0, 0.0, 100.0, 100.0),
            Color::BLUE,
            0.0,
            ZLayer::BACKGROUND,
        );

        let (commands, _stats) = builder.build();

        // Should be sorted: BACKGROUND (Blue) → DEFAULT (Green) → OVERLAY (Red)
        if let DrawCommand::Rect { color, .. } = &commands[0] {
            assert_eq!(*color, Color::BLUE);
        }
        if let DrawCommand::Rect { color, .. } = &commands[1] {
            assert_eq!(*color, Color::GREEN);
        }
        if let DrawCommand::Rect { color, .. } = &commands[2] {
            assert_eq!(*color, Color::RED);
        }
    }

    #[test]
    fn test_layer_constants() {
        assert!(ZLayer::BACKGROUND < ZLayer::DEFAULT);
        assert!(ZLayer::DEFAULT < ZLayer::OVERLAY);
        assert!(ZLayer::OVERLAY < ZLayer::POPUP);
        assert!(ZLayer::POPUP < ZLayer::DEBUG);
    }
}
