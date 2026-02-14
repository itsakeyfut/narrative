//! Character sprite element - displays a single character sprite
//!
//! This component displays a character sprite with support for:
//! - Position specification (using CharacterPosition)
//! - Expression switching (via texture updates)
//! - Opacity control (for speaker highlighting/dimming)
//! - Z-order layering
//! - Optional tint color (for special effects)
//! - Transitions (fade in/out, slide in, crossfade)
//! - Emotion animations (shake, jump, tremble)

use super::character_animation::CharacterAnimationState;
use super::character_transition::CharacterTransitionState;
use narrative_core::character::{CharacterAnimation, CharacterPosition};
use narrative_core::{SlideDirection, Transition, TransitionKind};
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::{Bounds, Color, Element, ElementId, InputEvent, Point, Size};
use std::any::Any;
use std::time::Duration;
use taffy::NodeId;

/// Character sprite element that displays a character at a specific position
///
/// Supports:
/// - Positioning via CharacterPosition enum (Left, Center, Right, etc.)
/// - Opacity for highlighting active speaker
/// - Z-order for rendering order
/// - Tint color for visual effects (not yet supported by renderer)
/// - Transitions for smooth appearance/disappearance animations
#[allow(dead_code)]
pub struct CharacterSpriteElement {
    /// Unique element ID
    id: ElementId,
    /// Taffy layout node
    layout_node: Option<NodeId>,
    /// Character ID
    character_id: String,
    /// Current expression name
    expression: String,
    /// Texture ID for the current sprite
    texture_id: Option<u64>,
    /// Character position (Left, Center, Right, etc.)
    position: CharacterPosition,
    /// Visibility flag
    visible: bool,
    /// Opacity (0.0-1.0, used for speaker highlight)
    opacity: f32,
    /// Z-order (higher values render on top)
    z_order: i32,
    /// Optional tint color (default: white = no tint)
    tint: Color,
    /// Sprite dimensions (width, height)
    sprite_size: (f32, f32),
    /// Active transition state (if any)
    active_transition: Option<CharacterTransitionState>,
    /// Active animation state (if any)
    active_animation: Option<CharacterAnimationState>,
    /// Time accumulator for frame-independent animation
    time_accumulator: Duration,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
    /// Window size for positioning calculations (width, height)
    window_size: (f32, f32),
    /// Sprite position offset (x, y) for padding/margin adjustment
    /// Specified in pixels at reference resolution (1280x720), scales with screen size
    sprite_offset: (f32, f32),
    /// Sprite scale multiplier (1.0 = normal size)
    sprite_scale: f32,
}

impl CharacterSpriteElement {
    /// Default sprite dimensions (can be overridden)
    const DEFAULT_SPRITE_WIDTH: f32 = 400.0;
    const DEFAULT_SPRITE_HEIGHT: f32 = 600.0;

    /// Dimmed opacity for non-speaking characters
    pub const DIMMED_OPACITY: f32 = 0.5;
    /// Full opacity for speaking character
    pub const FULL_OPACITY: f32 = 1.0;

    /// Create a new character sprite element
    pub fn new(
        character_id: impl Into<String>,
        expression: impl Into<String>,
        position: CharacterPosition,
    ) -> Self {
        Self {
            id: ElementId::new(),
            layout_node: None,
            character_id: character_id.into(),
            expression: expression.into(),
            texture_id: None,
            position,
            visible: true,
            opacity: Self::FULL_OPACITY,
            z_order: 0,
            tint: Color::WHITE,
            sprite_size: (Self::DEFAULT_SPRITE_WIDTH, Self::DEFAULT_SPRITE_HEIGHT),
            active_transition: None,
            active_animation: None,
            time_accumulator: Duration::ZERO,
            animation_context: AnimationContext::default(),
            animations_enabled: None,
            window_size: (1280.0, 720.0), // Default window size
            sprite_offset: (0.0, 0.0),    // Default: no offset
            sprite_scale: 1.0,            // Default: normal size
        }
    }

    /// Set the texture ID for the sprite
    pub fn with_texture(mut self, texture_id: u64) -> Self {
        self.texture_id = Some(texture_id);
        self
    }

    /// Set the z-order
    pub fn with_z_order(mut self, z_order: i32) -> Self {
        self.z_order = z_order;
        self
    }

    /// Set the opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set the sprite size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.sprite_size = (width, height);
        self
    }

    /// Set visibility
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set the animation context
    ///
    /// This allows the sprite to respect global animation settings for transitions.
    pub fn with_animation_context(mut self, context: AnimationContext) -> Self {
        self.animation_context = context;
        self
    }

    /// Set component-specific animation override
    ///
    /// This allows disabling animations for this specific sprite
    /// even when global animations are enabled, or vice versa.
    pub fn with_animations_enabled(mut self, enabled: impl Into<Option<bool>>) -> Self {
        self.animations_enabled = enabled.into();
        self
    }

    /// Set the window size for positioning calculations
    pub fn with_window_size(mut self, width: f32, height: f32) -> Self {
        self.window_size = (width, height);
        self
    }

    /// Set the sprite offset for positioning adjustment
    /// Offset is specified in pixels at reference resolution (1280x720) and scales with screen size
    pub fn with_sprite_offset(mut self, x_offset: f32, y_offset: f32) -> Self {
        self.sprite_offset = (x_offset, y_offset);
        self
    }

    /// Set the sprite scale multiplier
    /// Values > 1.0 make sprite larger, < 1.0 make it smaller
    pub fn with_sprite_scale(mut self, scale: f32) -> Self {
        self.sprite_scale = scale;
        self
    }

    /// Update the texture ID (mutable)
    pub fn set_texture(&mut self, texture_id: Option<u64>) {
        self.texture_id = texture_id;
    }

    /// Update the expression (mutable)
    pub fn set_expression(&mut self, expression: impl Into<String>) {
        self.expression = expression.into();
    }

    /// Update the position (mutable)
    pub fn set_position(&mut self, position: CharacterPosition) {
        self.position = position;
    }

    /// Update visibility (mutable)
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Update opacity (mutable)
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Update z-order (mutable)
    pub fn set_z_order(&mut self, z_order: i32) {
        self.z_order = z_order;
    }

    /// Highlight this character (set to full opacity)
    pub fn highlight(&mut self) {
        self.opacity = Self::FULL_OPACITY;
    }

    /// Dim this character (set to dimmed opacity)
    pub fn dim(&mut self) {
        self.opacity = Self::DIMMED_OPACITY;
    }

    /// Get the character ID
    pub fn character_id(&self) -> &str {
        &self.character_id
    }

    /// Get the current expression
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Get the z-order (for sorting)
    pub fn z_order(&self) -> i32 {
        self.z_order
    }

    /// Start a fade in transition
    pub fn fade_in(&mut self, transition: Transition) {
        // Adjust transition duration based on animation context
        let base_duration = Duration::from_secs_f32(transition.duration);
        let effective_duration = self
            .animation_context
            .adjust_duration(base_duration, self.animations_enabled);
        let adjusted_transition =
            Transition::new(transition.kind, effective_duration.as_secs_f32());

        self.active_transition = Some(CharacterTransitionState::fade_in(adjusted_transition));
        self.visible = true;
    }

    /// Start a fade out transition
    pub fn fade_out(&mut self, transition: Transition) {
        // Adjust transition duration based on animation context
        let base_duration = Duration::from_secs_f32(transition.duration);
        let effective_duration = self
            .animation_context
            .adjust_duration(base_duration, self.animations_enabled);
        let adjusted_transition =
            Transition::new(transition.kind, effective_duration.as_secs_f32());

        self.active_transition = Some(CharacterTransitionState::fade_out(adjusted_transition));
    }

    /// Start a slide in transition
    pub fn slide_in(&mut self, transition: Transition, direction: SlideDirection) {
        // Adjust transition duration based on animation context
        let base_duration = Duration::from_secs_f32(transition.duration);
        let effective_duration = self
            .animation_context
            .adjust_duration(base_duration, self.animations_enabled);
        let adjusted_transition =
            Transition::new(transition.kind, effective_duration.as_secs_f32());

        self.active_transition = Some(CharacterTransitionState::slide_in(
            adjusted_transition,
            direction,
            self.position,
        ));
        self.visible = true;
    }

    /// Change expression with a crossfade transition
    pub fn change_expression_crossfade(
        &mut self,
        new_expression: impl Into<String>,
        new_texture_id: u64,
        transition: Transition,
    ) {
        // Adjust transition duration based on animation context
        let base_duration = Duration::from_secs_f32(transition.duration);
        let effective_duration = self
            .animation_context
            .adjust_duration(base_duration, self.animations_enabled);
        let adjusted_transition =
            Transition::new(transition.kind, effective_duration.as_secs_f32());

        if let Some(old_texture_id) = self.texture_id {
            self.active_transition = Some(CharacterTransitionState::crossfade(
                adjusted_transition,
                old_texture_id,
                new_texture_id,
            ));
        }
        self.expression = new_expression.into();
        self.texture_id = Some(new_texture_id);
    }

    /// Move character to a new position with smooth animation
    ///
    /// Smoothly animates the character from its current position to the target position.
    /// Uses EaseOut easing by default for natural movement.
    ///
    /// # Arguments
    /// * `to_position` - Target position to move to
    /// * `transition` - Transition configuration (duration and type)
    ///
    /// # Example
    /// ```ignore
    /// // Move character from current position to Right over 0.5 seconds
    /// sprite.move_to(CharacterPosition::Right, Transition::fade());
    /// ```
    pub fn move_to(&mut self, to_position: CharacterPosition, transition: Transition) {
        // Adjust transition duration based on animation context
        let base_duration = Duration::from_secs_f32(transition.duration);
        let effective_duration = self
            .animation_context
            .adjust_duration(base_duration, self.animations_enabled);
        let adjusted_transition =
            Transition::new(transition.kind, effective_duration.as_secs_f32());

        // Create move transition from current position to target position
        self.active_transition = Some(CharacterTransitionState::move_to(
            adjusted_transition,
            self.position,
            to_position,
        ));

        // Update position immediately so subsequent queries return the new position
        self.position = to_position;
    }

    /// Check if a transition is currently active
    pub fn is_transitioning(&self) -> bool {
        self.active_transition.is_some()
    }

    /// Start a character animation
    pub fn start_animation(&mut self, animation: CharacterAnimation) {
        if animation.is_active() {
            self.active_animation = Some(CharacterAnimationState::new(animation));
        }
    }

    /// Stop the current animation
    pub fn stop_animation(&mut self) {
        self.active_animation = None;
    }

    /// Check if an animation is currently active
    pub fn is_animating(&self) -> bool {
        self.active_animation.is_some()
    }

    /// Get the current animation (if any)
    pub fn current_animation(&self) -> Option<&CharacterAnimation> {
        self.active_animation.as_ref().map(|a| a.animation())
    }

    /// Calculate the sprite bounds based on position and screen size
    fn calculate_bounds(&self, screen_width: f32, screen_height: f32) -> Bounds {
        // Reference resolution for scaling (720p)
        const REFERENCE_WIDTH: f32 = 1280.0;
        const REFERENCE_HEIGHT: f32 = 720.0;

        // Scale sprite size based on screen height
        // Reference: 600px height at 720p (83.3% of screen height)
        const REFERENCE_SPRITE_HEIGHT: f32 = 600.0;
        let height_ratio = REFERENCE_SPRITE_HEIGHT / REFERENCE_HEIGHT;

        // Calculate scaled sprite size maintaining aspect ratio
        // Apply sprite_scale multiplier
        let sprite_height = screen_height * height_ratio * self.sprite_scale;
        let sprite_width = sprite_height * (self.sprite_size.0 / self.sprite_size.1);

        // Calculate x position based on CharacterPosition
        let x = match self.position {
            CharacterPosition::Fixed(fixed_x) => {
                // Scale fixed pixel position based on screen width
                // Fixed position is specified for reference resolution (1280x720)
                // and scales proportionally with screen size
                let x_scale = screen_width / REFERENCE_WIDTH;
                fixed_x * x_scale
            }
            _ => {
                // Use percentage-based positioning
                let x_percent = self.position.x_percent();
                screen_width * x_percent - sprite_width * 0.5 // Center sprite at position
            }
        };

        // Position sprite with bottom-left anchor at screen bottom
        // Sprite bottom edge is placed at screen_height (bottom of screen)
        let y = screen_height - sprite_height;

        // Apply sprite offset for padding/margin adjustments
        // Offset is specified at reference resolution and scales proportionally
        let x_scale = screen_width / REFERENCE_WIDTH;
        let y_scale = screen_height / REFERENCE_HEIGHT;
        let scaled_offset_x = self.sprite_offset.0 * x_scale;
        let scaled_offset_y = self.sprite_offset.1 * y_scale;

        let final_x = x + scaled_offset_x;
        let final_y = y + scaled_offset_y;

        tracing::debug!(
            "CharacterSprite calculate_bounds: position={:?}, screen=({}, {}), calculated_sprite_size=({:.1}, {:.1}), offset=({:.1}, {:.1}), calculated_pos=({:.1}, {:.1})",
            self.position,
            screen_width,
            screen_height,
            sprite_width,
            sprite_height,
            scaled_offset_x,
            scaled_offset_y,
            final_x,
            final_y
        );

        Bounds {
            origin: Point::new(final_x, final_y),
            size: Size::new(sprite_width, sprite_height),
        }
    }
}

impl Element for CharacterSpriteElement {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout_node(&self) -> Option<NodeId> {
        self.layout_node
    }

    fn set_layout_node(&mut self, node: NodeId) {
        self.layout_node = Some(node);
    }

    fn layout(
        &mut self,
        _cx: &mut narrative_gui::framework::element::LayoutContext,
    ) -> taffy::Style {
        use taffy::prelude::*;

        // Characters are positioned absolutely and don't participate in flexbox layout
        taffy::Style {
            position: Position::Absolute,
            size: taffy::geometry::Size {
                width: Dimension::length(self.sprite_size.0),
                height: Dimension::length(self.sprite_size.1),
            },
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut narrative_gui::framework::element::PaintContext) {
        // Only paint if visible and texture is loaded
        if !self.visible {
            return;
        }

        let Some(texture_id) = self.texture_id else {
            return;
        };

        // Calculate sprite position based on window size (not element bounds)
        let (window_width, window_height) = self.window_size;
        let mut sprite_bounds = self.calculate_bounds(window_width, window_height);

        tracing::debug!(
            "CharacterSprite paint: id='{}', position={:?}, window_size=({}, {}), bounds=({}, {}, {}, {}), texture_id={:?}, has_transition={}, has_animation={}",
            self.character_id,
            self.position,
            window_width,
            window_height,
            sprite_bounds.origin.x,
            sprite_bounds.origin.y,
            sprite_bounds.size.width,
            sprite_bounds.size.height,
            texture_id,
            self.active_transition.is_some(),
            self.active_animation.is_some()
        );

        // Apply transition effects
        let mut final_opacity = self.opacity;

        if let Some(ref transition) = self.active_transition {
            // Apply transition opacity
            final_opacity = transition.current_opacity(self.opacity);

            // Apply position offset for slide transitions
            let (x_offset, y_offset) = transition.position_offset(window_width, window_height);
            sprite_bounds.origin.x += x_offset;
            sprite_bounds.origin.y += y_offset;

            // Handle crossfade transitions (render both old and new textures)
            if let Some((old_texture_id, new_texture_id, progress)) =
                transition.crossfade_textures()
            {
                tracing::debug!(
                    "CharacterSprite crossfade: id='{}', old_tex={}, new_tex={}, progress={:.2}, old_opacity={:.2}, new_opacity={:.2}",
                    self.character_id,
                    old_texture_id,
                    new_texture_id,
                    progress,
                    self.opacity * (1.0 - progress),
                    self.opacity * progress
                );
                // Draw old texture fading out
                cx.draw_texture(
                    old_texture_id,
                    sprite_bounds,
                    self.opacity * (1.0 - progress),
                );
                // Draw new texture fading in
                cx.draw_texture(new_texture_id, sprite_bounds, self.opacity * progress);
                return; // Skip normal draw below
            }
        }

        // Apply animation effects (shake, jump, tremble)
        if let Some(ref animation) = self.active_animation {
            // Get animation offset at reference resolution (1280x720)
            let (anim_x_offset, anim_y_offset) = animation.current_offset();

            // Scale the offset based on current screen resolution
            // Reference resolution for scaling
            const REFERENCE_WIDTH: f32 = 1280.0;
            const REFERENCE_HEIGHT: f32 = 720.0;

            let x_scale = window_width / REFERENCE_WIDTH;
            let y_scale = window_height / REFERENCE_HEIGHT;

            let scaled_x_offset = anim_x_offset * x_scale;
            let scaled_y_offset = anim_y_offset * y_scale;

            sprite_bounds.origin.x += scaled_x_offset;
            sprite_bounds.origin.y += scaled_y_offset;

            tracing::debug!(
                "CharacterSprite animation: id='{}', offset=({:.1}, {:.1}), scaled=({:.1}, {:.1})",
                self.character_id,
                anim_x_offset,
                anim_y_offset,
                scaled_x_offset,
                scaled_y_offset
            );
        }

        // Draw the sprite texture with final opacity
        cx.draw_texture(texture_id, sprite_bounds, final_opacity);

        // TODO(tint): Tint color support requires additional shader changes
        // - Add tint field to TextureInstance struct
        // - Update texture.wgsl shader to multiply RGB by tint color
        // - Add tint parameter to DrawCommand::Texture and draw_texture()
        // - Currently only opacity is supported for speaker highlighting
        // - Tint would be useful for status effects (poisoned=green, frozen=blue, etc.)
    }

    fn tick(&mut self, delta: Duration) -> bool {
        // Use provided delta or fall back to assumed 60 FPS frame delta
        let frame_delta = if delta.as_millis() > 0 {
            delta
        } else {
            Duration::from_millis(16)
        };

        let mut needs_update = false;

        // Update transitions if active
        if let Some(ref mut transition) = self.active_transition {
            let is_complete = transition.update(frame_delta);
            let was_fade_out = if is_complete {
                // Check if this was a fade out by checking final opacity
                matches!(transition.kind(), TransitionKind::Fade)
                    && transition.current_opacity(1.0) < 0.1
            } else {
                false
            };

            if is_complete {
                // Transition completed, clean up
                tracing::debug!(
                    "Transition completed for character '{}', kind={:?}, was_fade_out={}",
                    self.character_id,
                    transition.kind(),
                    was_fade_out
                );
                self.active_transition = None;

                // Handle fade out completion (hide character)
                if was_fade_out {
                    self.visible = false;
                }
            }

            needs_update = true;
        }

        // Update animations if active
        if let Some(ref mut animation) = self.active_animation {
            let is_complete = animation.update(frame_delta);

            if is_complete {
                // Animation completed (auto or duration mode), clean up
                tracing::debug!(
                    "Animation completed for character '{}', animation={:?}",
                    self.character_id,
                    animation.animation()
                );
                self.active_animation = None;
            }

            needs_update = true;
        }

        needs_update
    }

    fn handle_event(&mut self, _event: &InputEvent, _bounds: Bounds) -> bool {
        // Characters don't handle input events
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for CharacterSpriteElement {
    fn default() -> Self {
        Self::new("", "", CharacterPosition::Center)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_sprite_new() {
        let sprite = CharacterSpriteElement::new("alice", "normal", CharacterPosition::Left);

        assert_eq!(sprite.character_id(), "alice");
        assert_eq!(sprite.expression(), "normal");
        assert_eq!(sprite.position, CharacterPosition::Left);
        assert!(sprite.visible);
        assert_eq!(sprite.opacity, CharacterSpriteElement::FULL_OPACITY);
        assert_eq!(sprite.z_order(), 0);
    }

    #[test]
    fn test_character_sprite_builder() {
        let sprite = CharacterSpriteElement::new("bob", "happy", CharacterPosition::Right)
            .with_texture(123)
            .with_z_order(5)
            .with_opacity(0.8)
            .with_size(500.0, 700.0)
            .with_visible(false);

        assert_eq!(sprite.texture_id, Some(123));
        assert_eq!(sprite.z_order(), 5);
        assert_eq!(sprite.opacity, 0.8);
        assert_eq!(sprite.sprite_size, (500.0, 700.0));
        assert!(!sprite.visible);
    }

    #[test]
    fn test_character_sprite_set_methods() {
        let mut sprite = CharacterSpriteElement::new("charlie", "sad", CharacterPosition::Center);

        sprite.set_texture(Some(456));
        sprite.set_expression("angry");
        sprite.set_position(CharacterPosition::FarLeft);
        sprite.set_visible(false);
        sprite.set_opacity(0.5);
        sprite.set_z_order(10);

        assert_eq!(sprite.texture_id, Some(456));
        assert_eq!(sprite.expression(), "angry");
        assert_eq!(sprite.position, CharacterPosition::FarLeft);
        assert!(!sprite.visible);
        assert_eq!(sprite.opacity, 0.5);
        assert_eq!(sprite.z_order(), 10);
    }

    #[test]
    fn test_character_sprite_highlight_dim() {
        let mut sprite = CharacterSpriteElement::new("dave", "normal", CharacterPosition::Center);

        sprite.dim();
        assert_eq!(sprite.opacity, CharacterSpriteElement::DIMMED_OPACITY);

        sprite.highlight();
        assert_eq!(sprite.opacity, CharacterSpriteElement::FULL_OPACITY);
    }

    #[test]
    fn test_character_sprite_opacity_clamping() {
        let mut sprite = CharacterSpriteElement::new("eve", "normal", CharacterPosition::Center);

        sprite.set_opacity(1.5);
        assert_eq!(sprite.opacity, 1.0);

        sprite.set_opacity(-0.5);
        assert_eq!(sprite.opacity, 0.0);
    }

    #[test]
    fn test_character_sprite_calculate_bounds() {
        let sprite = CharacterSpriteElement::new("frank", "normal", CharacterPosition::Center);

        let bounds = sprite.calculate_bounds(1280.0, 720.0);

        // Center position (0.5) with 400px width sprite
        // x = 1280 * 0.5 - 400 * 0.5 = 640 - 200 = 440
        assert_eq!(bounds.origin.x, 440.0);

        // Bottom-aligned: y = 720 - 600 = 120
        assert_eq!(bounds.origin.y, 120.0);

        assert_eq!(bounds.size.width, 400.0);
        assert_eq!(bounds.size.height, 600.0);
    }

    #[test]
    fn test_character_sprite_positions() {
        let screen_width = 1920.0;
        let screen_height = 1080.0;

        // Test different positions
        let left = CharacterSpriteElement::new("a", "normal", CharacterPosition::Left);
        let center = CharacterSpriteElement::new("b", "normal", CharacterPosition::Center);
        let right = CharacterSpriteElement::new("c", "normal", CharacterPosition::Right);

        let left_bounds = left.calculate_bounds(screen_width, screen_height);
        let center_bounds = center.calculate_bounds(screen_width, screen_height);
        let right_bounds = right.calculate_bounds(screen_width, screen_height);

        // Verify positions are different and ordered correctly
        assert!(left_bounds.origin.x < center_bounds.origin.x);
        assert!(center_bounds.origin.x < right_bounds.origin.x);

        // All should be bottom-aligned (y = screen_height - sprite_height)
        // With default sprite_scale=1.0, sprite_height = screen_height * 600/720 = 500
        let expected_y = screen_height - (screen_height * 600.0 / 720.0);
        assert_eq!(left_bounds.origin.y, expected_y);
        assert_eq!(center_bounds.origin.y, expected_y);
        assert_eq!(right_bounds.origin.y, expected_y);
    }

    #[test]
    fn test_character_sprite_element_trait() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        // Test Element trait methods
        assert!(!sprite.tick(Duration::from_millis(16))); // Characters don't need ticking by default
        assert!(sprite.layout_node().is_none());
    }

    #[test]
    fn test_character_sprite_fade_in() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        assert!(!sprite.is_transitioning());

        sprite.fade_in(Transition::fade());
        assert!(sprite.is_transitioning());
        assert!(sprite.visible);
    }

    #[test]
    fn test_character_sprite_fade_out() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        sprite.fade_out(Transition::fade());
        assert!(sprite.is_transitioning());
    }

    #[test]
    fn test_character_sprite_slide_in() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        sprite.slide_in(Transition::fade(), SlideDirection::Left);
        assert!(sprite.is_transitioning());
        assert!(sprite.visible);
    }

    #[test]
    fn test_character_sprite_crossfade() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center)
            .with_texture(100);

        sprite.change_expression_crossfade("happy", 200, Transition::crossfade());
        assert!(sprite.is_transitioning());
        assert_eq!(sprite.expression(), "happy");
        assert_eq!(sprite.texture_id, Some(200));
    }

    #[test]
    fn test_character_sprite_transition_tick() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        sprite.fade_in(Transition::instant());
        assert!(sprite.is_transitioning());

        // Tick should complete instant transition
        let needs_update = sprite.tick(Duration::from_millis(16));
        assert!(needs_update); // Should request update during transition

        // Tick again - transition should be complete
        let needs_update = sprite.tick(Duration::from_millis(16));
        assert!(!needs_update); // No more updates needed
        assert!(!sprite.is_transitioning());
    }

    #[test]
    fn test_character_sprite_move_to() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Left);

        assert_eq!(sprite.position, CharacterPosition::Left);
        assert!(!sprite.is_transitioning());

        // Start moving to Right
        sprite.move_to(CharacterPosition::Right, Transition::fade());

        assert!(sprite.is_transitioning());
        assert_eq!(sprite.position, CharacterPosition::Right); // Position updated immediately
    }

    #[test]
    fn test_character_sprite_move_to_instant() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        sprite.move_to(CharacterPosition::FarRight, Transition::instant());

        assert!(sprite.is_transitioning());
        assert_eq!(sprite.position, CharacterPosition::FarRight);

        // Complete instant transition
        sprite.tick(Duration::from_millis(16));
        assert!(!sprite.is_transitioning());
    }

    #[test]
    fn test_character_sprite_move_to_completion() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Left);

        // Move to Right with 0.5 second transition
        sprite.move_to(CharacterPosition::Right, Transition::fade());
        assert!(sprite.is_transitioning());

        // Tick through the transition
        sprite.tick(Duration::from_secs_f32(0.5));
        assert!(!sprite.is_transitioning()); // Should be complete

        assert_eq!(sprite.position, CharacterPosition::Right);
    }

    #[test]
    fn test_character_sprite_move_to_custom_position() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        let custom_pos = CharacterPosition::custom(0.33);
        sprite.move_to(custom_pos, Transition::fade());

        assert!(sprite.is_transitioning());
        assert_eq!(sprite.position, custom_pos);
    }

    #[test]
    fn test_character_sprite_move_to_fixed_position() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Left);

        let fixed_pos = CharacterPosition::Fixed(200.0);
        sprite.move_to(fixed_pos, Transition::fade());

        assert!(sprite.is_transitioning());
        assert_eq!(sprite.position, fixed_pos);
    }

    #[test]
    fn test_character_sprite_start_animation() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        assert!(!sprite.is_animating());

        sprite.start_animation(CharacterAnimation::shake());
        assert!(sprite.is_animating());
        assert!(sprite.current_animation().is_some());
    }

    #[test]
    fn test_character_sprite_stop_animation() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        sprite.start_animation(CharacterAnimation::shake());
        assert!(sprite.is_animating());

        sprite.stop_animation();
        assert!(!sprite.is_animating());
    }

    #[test]
    fn test_character_sprite_animation_tick() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        sprite.start_animation(CharacterAnimation::shake());
        assert!(sprite.is_animating());

        // Tick should update animation and request re-render
        let needs_update = sprite.tick(Duration::from_millis(16));
        assert!(needs_update);
        assert!(sprite.is_animating()); // Still animating

        // Complete animation (auto mode: 3 cycles * 0.15s = 0.45s)
        sprite.tick(Duration::from_secs_f32(0.45));
        assert!(!sprite.is_animating()); // Animation complete
    }

    #[test]
    fn test_character_sprite_animation_types() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        sprite.start_animation(CharacterAnimation::shake());
        assert!(sprite.is_animating());
        sprite.stop_animation();

        sprite.start_animation(CharacterAnimation::jump());
        assert!(sprite.is_animating());
        sprite.stop_animation();

        sprite.start_animation(CharacterAnimation::tremble());
        assert!(sprite.is_animating());
    }

    #[test]
    fn test_character_sprite_animation_continuous_mode() {
        use narrative_core::character::AnimationTiming;

        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        let continuous_shake = CharacterAnimation::shake()
            .with_timing(AnimationTiming::continuous());
        sprite.start_animation(continuous_shake);

        // Tick many times - continuous animation should never complete
        for _ in 0..100 {
            sprite.tick(Duration::from_millis(16));
        }

        assert!(sprite.is_animating()); // Still animating
    }

    #[test]
    fn test_character_sprite_animation_and_transition() {
        let mut sprite = CharacterSpriteElement::new("test", "normal", CharacterPosition::Center);

        // Start both animation and transition
        sprite.start_animation(CharacterAnimation::shake());
        sprite.fade_in(Transition::fade());

        assert!(sprite.is_animating());
        assert!(sprite.is_transitioning());

        // Both should update independently
        let needs_update = sprite.tick(Duration::from_millis(16));
        assert!(needs_update);
    }
}
