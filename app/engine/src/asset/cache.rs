//! Texture cache

use super::TextureHandle;
use crate::error::{EngineError, EngineResult};
use lru::LruCache;
use narrative_core::AssetRef;
use std::num::NonZeroUsize;

/// Texture cache with LRU eviction
#[derive(Debug)]
pub struct TextureCache {
    cache: LruCache<AssetRef, TextureHandle>,
}

impl TextureCache {
    /// Create a new texture cache with default capacity (128)
    pub fn new() -> EngineResult<Self> {
        Self::with_capacity(128)
    }

    /// Create a new texture cache with specified capacity
    pub fn with_capacity(capacity: usize) -> EngineResult<Self> {
        let capacity = NonZeroUsize::new(capacity).ok_or(EngineError::InvalidCapacity(capacity))?;
        Ok(Self {
            cache: LruCache::new(capacity),
        })
    }

    /// Get a cached texture
    pub fn get(&mut self, asset_ref: &AssetRef) -> Option<&TextureHandle> {
        self.cache.get(asset_ref)
    }

    /// Insert a texture into the cache
    pub fn insert(&mut self, asset_ref: AssetRef, handle: TextureHandle) {
        self.cache.put(asset_ref, handle);
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

impl Default for TextureCache {
    fn default() -> Self {
        // Safe: 128 is a valid non-zero capacity
        Self::with_capacity(128).expect("Default capacity is valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = TextureCache::new().unwrap();
        assert_eq!(cache.capacity(), 128);
    }

    #[test]
    fn test_cache_with_capacity() {
        let cache = TextureCache::with_capacity(64).unwrap();
        assert_eq!(cache.capacity(), 64);
    }

    #[test]
    fn test_cache_with_zero_capacity() {
        let result = TextureCache::with_capacity(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EngineError::InvalidCapacity(0)
        ));
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = TextureCache::new().unwrap();
        let asset = AssetRef::new("texture.png");
        let handle = TextureHandle::new(42);

        cache.insert(asset.clone(), handle);

        let retrieved = cache.get(&asset);
        assert_eq!(retrieved, Some(&handle));
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = TextureCache::new().unwrap();
        let asset = AssetRef::new("nonexistent.png");

        assert_eq!(cache.get(&asset), None);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = TextureCache::new().unwrap();
        let asset1 = AssetRef::new("tex1.png");
        let asset2 = AssetRef::new("tex2.png");

        cache.insert(asset1.clone(), TextureHandle::new(1));
        cache.insert(asset2.clone(), TextureHandle::new(2));

        cache.clear();

        assert_eq!(cache.get(&asset1), None);
        assert_eq!(cache.get(&asset2), None);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = TextureCache::with_capacity(2).unwrap();

        let asset1 = AssetRef::new("tex1.png");
        let asset2 = AssetRef::new("tex2.png");
        let asset3 = AssetRef::new("tex3.png");

        cache.insert(asset1.clone(), TextureHandle::new(1));
        cache.insert(asset2.clone(), TextureHandle::new(2));

        // Insert third item - should evict first
        cache.insert(asset3.clone(), TextureHandle::new(3));

        assert_eq!(cache.get(&asset1), None); // Evicted
        assert!(cache.get(&asset2).is_some());
        assert!(cache.get(&asset3).is_some());
    }

    #[test]
    fn test_cache_update_existing() {
        let mut cache = TextureCache::new().unwrap();
        let asset = AssetRef::new("texture.png");

        cache.insert(asset.clone(), TextureHandle::new(1));
        cache.insert(asset.clone(), TextureHandle::new(2));

        assert_eq!(cache.get(&asset), Some(&TextureHandle::new(2)));
    }

    #[test]
    fn test_cache_default() {
        let cache = TextureCache::default();
        assert_eq!(cache.capacity(), 128);
    }
}
