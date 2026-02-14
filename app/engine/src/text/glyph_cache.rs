//! Glyph cache with cosmic-text integration

use crate::error::{EngineError, EngineResult};
use crate::text::TextureAtlas;
use cosmic_text::{CacheKey, FontSystem};
use lru::LruCache;
use std::num::NonZeroUsize;
use swash::FontRef;
use swash::scale::{ScaleContext, Source, StrikeWith};
use swash::zeno::{Format, Vector};

/// Glyph cache key (wraps cosmic-text's CacheKey)
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GlyphKey {
    /// Cosmic-text cache key
    pub cache_key: CacheKey,
}

impl GlyphKey {
    /// Create a new glyph key
    pub fn new(cache_key: CacheKey) -> Self {
        Self { cache_key }
    }
}

/// Cached glyph information
#[derive(Debug, Clone)]
pub struct GlyphInfo {
    /// Atlas texture coordinates (x, y)
    pub atlas_pos: (u32, u32),
    /// Glyph dimensions
    pub width: u32,
    pub height: u32,
    /// Offset from baseline
    pub offset_x: i32,
    pub offset_y: i32,
    /// Horizontal advance
    pub advance: f32,
}

/// Rasterized glyph image data
pub struct RasterizedGlyph {
    /// RGBA image data
    pub data: Vec<u8>,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// X offset from baseline
    pub offset_x: i32,
    /// Y offset from baseline
    pub offset_y: i32,
    /// Horizontal advance
    pub advance: f32,
}

/// Glyph cache with swash integration
pub struct GlyphCache {
    /// LRU cache for glyph info
    cache: LruCache<GlyphKey, GlyphInfo>,
    /// Swash scale context for rasterization
    scale_context: ScaleContext,
    // TODO(Phase 0.4+): Add performance metrics tracking
    // #[cfg(feature = "dev")]
    // metrics: RenderMetrics,
}

impl std::fmt::Debug for GlyphCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphCache")
            .field("cache", &self.cache)
            .field("scale_context", &"<ScaleContext>")
            .finish()
    }
}

impl GlyphCache {
    /// Create a new glyph cache with specified capacity
    pub fn new(capacity: usize) -> EngineResult<Self> {
        let capacity = NonZeroUsize::new(capacity).ok_or(EngineError::InvalidCapacity(capacity))?;

        Ok(Self {
            cache: LruCache::new(capacity),
            scale_context: ScaleContext::new(),
        })
    }

    /// Get a cached glyph
    pub fn get(&mut self, key: &GlyphKey) -> Option<&GlyphInfo> {
        self.cache.get(key)
    }

    /// Insert a glyph into the cache
    pub fn insert(&mut self, key: GlyphKey, info: GlyphInfo) {
        self.cache.put(key, info);
    }

    /// Get or rasterize a glyph
    pub fn get_or_rasterize(
        &mut self,
        font_system: &mut FontSystem,
        atlas: &mut TextureAtlas,
        queue: &wgpu::Queue,
        cache_key: CacheKey,
    ) -> EngineResult<Option<&GlyphInfo>> {
        let key = GlyphKey::new(cache_key);

        // Check if already cached
        if self.cache.contains(&key) {
            return Ok(self.cache.get(&key));
        }

        // Rasterize the glyph
        if let Some(rasterized) = self.rasterize_glyph(font_system, cache_key)? {
            // Allocate space in atlas
            // TODO(Phase 0.4+): Implement atlas recovery strategy when full:
            // - LRU eviction: Remove least recently used glyphs to free space
            // - Multi-atlas support: Create additional atlases when primary is full
            // - Atlas compaction: Reorganize glyphs to reduce fragmentation
            // - Dynamic atlas resizing: Increase atlas size when needed
            // - Fallback fonts: Use simpler glyphs when atlas is full
            let atlas_pos = atlas
                .allocate(rasterized.width, rasterized.height)
                .ok_or_else(|| {
                    EngineError::GlyphCache(format!(
                        "Atlas is full, cannot allocate glyph (size: {}x{}). \
                         Current atlas dimensions: {:?}. \
                         Consider increasing atlas size or implementing dynamic expansion.",
                        rasterized.width,
                        rasterized.height,
                        atlas.dimensions()
                    ))
                })?;

            // Upload glyph data to GPU texture atlas
            atlas.upload(
                queue,
                atlas_pos.0,
                atlas_pos.1,
                rasterized.width,
                rasterized.height,
                &rasterized.data,
            )?;

            // Store glyph info
            let info = GlyphInfo {
                atlas_pos,
                width: rasterized.width,
                height: rasterized.height,
                offset_x: rasterized.offset_x,
                offset_y: rasterized.offset_y,
                advance: rasterized.advance,
            };

            self.cache.put(key, info);

            Ok(self.cache.get(&key))
        } else {
            Ok(None)
        }
    }

    /// Rasterize a glyph using swash
    fn rasterize_glyph(
        &mut self,
        font_system: &mut FontSystem,
        cache_key: CacheKey,
    ) -> EngineResult<Option<RasterizedGlyph>> {
        // Use the font ID from cache_key to get the exact font
        let font_id = cache_key.font_id;

        // Get glyph from cache key
        let glyph_id = cache_key.glyph_id;
        let font_size = cache_key.font_size_bits as f32 / 64.0; // Convert from 26.6 fixed point

        // Perform rasterization inside with_face_data callback to avoid lifetime issues
        let result = font_system.db().with_face_data(font_id, |data, index| {
            let font = FontRef::from_index(data, index as usize)
                .ok_or_else(|| EngineError::FontLoad("Failed to parse font data".to_string()))?;

            // Create scaler
            let mut scaler = self
                .scale_context
                .builder(font)
                .size(font_size)
                .hint(true)
                .build();

            // Get proper advance width from font metrics
            let advance = font
                .glyph_metrics(&[])
                .scale(font_size)
                .advance_width(glyph_id);

            // Render glyph
            let offset = Vector::new(0.0, 0.0);
            let rendered = swash::scale::Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
            ])
            .format(Format::Alpha)
            .offset(offset)
            .render(&mut scaler, glyph_id);

            if let Some(image) = rendered {
                // Convert alpha to RGBA
                let mut rgba_data = Vec::with_capacity(
                    image.placement.width as usize * image.placement.height as usize * 4,
                );

                for alpha in image.data.iter() {
                    rgba_data.extend_from_slice(&[255, 255, 255, *alpha]);
                }

                Ok(Some(RasterizedGlyph {
                    data: rgba_data,
                    width: image.placement.width,
                    height: image.placement.height,
                    offset_x: image.placement.left,
                    offset_y: image.placement.top,
                    // Use accurate advance width from font metrics
                    advance,
                }))
            } else {
                Ok(None)
            }
        });

        result.ok_or_else(|| EngineError::FontLoad("Font data not found".to_string()))?
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache capacity
    pub fn capacity(&self) -> usize {
        self.cache.cap().get()
    }
}

impl Default for GlyphCache {
    fn default() -> Self {
        // SAFETY: 1024 is a compile-time constant and guaranteed to be non-zero
        let capacity = unsafe { NonZeroUsize::new_unchecked(1024) };

        Self {
            cache: LruCache::new(capacity),
            scale_context: ScaleContext::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_cache_creation() {
        let cache = GlyphCache::new(512).unwrap();
        assert_eq!(cache.capacity(), 512);
    }

    #[test]
    fn test_glyph_cache_zero_capacity() {
        let result = GlyphCache::new(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EngineError::InvalidCapacity(0)
        ));
    }

    #[test]
    fn test_glyph_cache_default() {
        let cache = GlyphCache::default();
        assert_eq!(cache.capacity(), 1024);
    }

    // Note: We can't easily test insert/get with CacheKey
    // since it can't be safely constructed outside of cosmic-text
    // These tests verify the basic cache functionality works

    #[test]
    fn test_glyph_cache_capacity() {
        let cache = GlyphCache::new(512).unwrap();
        assert_eq!(cache.capacity(), 512);
    }

    #[test]
    fn test_glyph_cache_clear() {
        let mut cache = GlyphCache::new(128).unwrap();
        // Clear should not panic even on empty cache
        cache.clear();
    }

    #[test]
    fn test_glyph_info_fields() {
        let info = GlyphInfo {
            atlas_pos: (100, 200),
            width: 24,
            height: 32,
            offset_x: -2,
            offset_y: 4,
            advance: 20.5,
        };

        assert_eq!(info.atlas_pos, (100, 200));
        assert_eq!(info.width, 24);
        assert_eq!(info.height, 32);
        assert_eq!(info.offset_x, -2);
        assert_eq!(info.offset_y, 4);
        assert_eq!(info.advance, 20.5);
    }

    #[test]
    fn test_rasterized_glyph_creation() {
        let glyph = RasterizedGlyph {
            data: vec![255u8; 16 * 16 * 4], // 16x16 RGBA
            width: 16,
            height: 16,
            offset_x: 2,
            offset_y: -4,
            advance: 12.0,
        };

        assert_eq!(glyph.data.len(), 16 * 16 * 4);
        assert_eq!(glyph.width, 16);
        assert_eq!(glyph.height, 16);
        assert_eq!(glyph.offset_x, 2);
        assert_eq!(glyph.offset_y, -4);
        assert_eq!(glyph.advance, 12.0);
    }
}
