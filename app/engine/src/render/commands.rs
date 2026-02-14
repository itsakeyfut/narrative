//! Render commands

use narrative_core::{AssetRef, CharacterPosition, Color, Point, Rect};
use std::sync::Arc;

/// Rendering layer for Z-order management
///
/// Explicit numeric values are assigned for clarity and stability.
/// While Ord trait provides ordering, explicit values make the render order
/// immediately clear and prevent unintended changes from enum reordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum RenderLayer {
    /// Background layer (lowest priority)
    #[default]
    Background = 0,
    /// CG (event graphics) layer (rendered above background)
    CG = 1,
    /// Character layer (rendered above CG)
    Characters = 2,
    /// UI layer (dialogue box, choices, rendered above characters)
    UI = 3,
    /// Overlay layer (transitions, effects, highest priority)
    Overlay = 4,
}

/// Rendering commands
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Set the current rendering layer
    SetLayer(RenderLayer),

    /// Draw a background
    DrawBackground {
        /// Asset reference for the background texture
        texture: AssetRef,
        /// Opacity (0.0 - 1.0)
        opacity: f32,
        /// Optional tint color
        tint: Option<Color>,
    },

    /// Draw a CG (event graphics) with aspect ratio preservation
    DrawCG {
        /// Asset reference for the CG texture
        texture: AssetRef,
        /// Opacity (0.0 - 1.0)
        opacity: f32,
        /// Optional tint color
        tint: Option<Color>,
    },

    /// Draw a sprite with Z-order
    DrawSprite {
        /// Asset reference for the sprite texture
        texture: AssetRef,
        /// Destination rectangle
        dest: Rect,
        /// Source rectangle (UV coordinates, None = full texture)
        source: Option<Rect>,
        /// Opacity (0.0 - 1.0)
        opacity: f32,
        /// Tint color
        tint: Option<Color>,
        /// Z-order within the layer (higher = closer to camera)
        z_order: i32,
    },

    /// Draw a character sprite
    DrawCharacter {
        /// Asset reference for the character texture
        texture: AssetRef,
        /// Character position on screen
        position: CharacterPosition,
        /// Scale factor (1.0 = normal size)
        scale: f32,
        /// Opacity (0.0 - 1.0)
        opacity: f32,
        /// Flip horizontally
        flip_x: bool,
        /// Z-order within the layer
        z_order: i32,
    },

    /// Draw text (will be rendered by text module)
    DrawText {
        /// Text content (Arc<str> for efficient cloning across rendering pipeline)
        text: Arc<str>,
        /// Position
        position: Point,
        /// Font size in pixels
        font_size: f32,
        /// Line height in pixels
        line_height: f32,
        /// Text color
        color: Color,
        /// Number of visible characters for typewriter effect (None = show all)
        visible_chars: Option<usize>,
    },

    /// Draw a dialogue box
    DrawDialogueBox {
        /// Box rectangle
        rect: Rect,
        /// Background color
        background: Color,
        /// Optional border color
        border: Option<Color>,
        /// Border width
        border_width: f32,
    },

    /// Draw a colored rectangle (for UI)
    DrawRect {
        /// Rectangle bounds
        rect: Rect,
        /// Fill color
        color: Color,
        /// Corner radius for rounded rectangles
        corner_radius: f32,
    },

    /// Draw a border rectangle
    DrawBorder {
        /// Rectangle bounds
        rect: Rect,
        /// Border color
        color: Color,
        /// Border width
        width: f32,
        /// Corner radius for rounded rectangles
        corner_radius: f32,
    },

    /// Apply a transition effect (overlay layer)
    DrawTransition {
        /// Type of transition
        kind: TransitionKind,
        /// Transition progress (0.0 = start, 1.0 = end)
        progress: f32,
        /// Optional custom fade color (for fade transitions)
        fade_color: Option<Color>,
        /// Optional source texture (for cross-dissolve, None = use current framebuffer)
        from_texture: Option<AssetRef>,
        /// Optional destination texture (for cross-dissolve)
        to_texture: Option<AssetRef>,
    },
}

/// Transition effect kind
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionKind {
    /// Fade to black
    FadeBlack,
    /// Fade to white
    FadeWhite,
    /// Fade to custom color
    FadeColor,
    /// Cross-dissolve between two images
    CrossDissolve,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_kind_equality() {
        assert_eq!(TransitionKind::FadeBlack, TransitionKind::FadeBlack);
        assert_ne!(TransitionKind::FadeBlack, TransitionKind::FadeWhite);
        assert_ne!(TransitionKind::FadeWhite, TransitionKind::FadeColor);
        assert_ne!(TransitionKind::CrossDissolve, TransitionKind::FadeBlack);
    }

    #[test]
    fn test_render_layer_ordering() {
        assert!(RenderLayer::Background < RenderLayer::CG);
        assert!(RenderLayer::CG < RenderLayer::Characters);
        assert!(RenderLayer::Characters < RenderLayer::UI);
        assert!(RenderLayer::UI < RenderLayer::Overlay);
    }

    #[test]
    fn test_render_layer_values() {
        assert_eq!(RenderLayer::Background as i32, 0);
        assert_eq!(RenderLayer::CG as i32, 1);
        assert_eq!(RenderLayer::Characters as i32, 2);
        assert_eq!(RenderLayer::UI as i32, 3);
        assert_eq!(RenderLayer::Overlay as i32, 4);
    }

    #[test]
    fn test_render_layer_default() {
        let layer = RenderLayer::default();
        assert_eq!(layer, RenderLayer::Background);
    }
}
