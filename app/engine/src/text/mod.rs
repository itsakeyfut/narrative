//! Text rendering module
//!
//! This module provides text layout and rendering using cosmic-text,
//! with glyph caching and typewriter effects.
//!
//! # Future Improvements
//!
//! - Vertical text support (Phase 0.5+)
//! - Performance metrics tracking (Phase 0.4+)
//! - Advanced font fallback strategies (Phase 0.5+)

mod atlas;
mod font_manager;
mod glyph_cache;
mod layout;
mod typewriter;

pub use atlas::TextureAtlas;
pub use font_manager::FontManager;
pub use glyph_cache::{GlyphCache, GlyphInfo, GlyphKey};
pub use layout::{LayoutGlyph, LayoutLine, TextLayout, TextStyle};
pub use typewriter::TypewriterEffect;
