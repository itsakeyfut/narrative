//! Texture handle

/// Texture handle (opaque reference to GPU texture)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle {
    id: u64,
}

impl TextureHandle {
    /// Create a new texture handle
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// Get the handle ID
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl Default for TextureHandle {
    fn default() -> Self {
        Self::new(0)
    }
}
