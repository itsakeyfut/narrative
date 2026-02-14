//! Render batching

use super::{RenderLayer, SpriteVertex, TextureId};

/// Maximum number of sprites per batch
const MAX_BATCH_SPRITES: usize = 10000;

/// Render batch for efficient GPU usage
#[derive(Debug)]
pub struct RenderBatch {
    /// Texture ID for this batch
    texture_id: Option<TextureId>,
    /// Rendering layer
    layer: RenderLayer,
    /// Z-order within the layer
    z_order: i32,
    /// Vertex data
    vertices: Vec<SpriteVertex>,
    /// Index data
    indices: Vec<u16>,
}

impl RenderBatch {
    /// Create a new empty render batch
    pub fn new() -> Self {
        Self {
            texture_id: None,
            layer: RenderLayer::default(),
            z_order: 0,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Create a new render batch with specific texture, layer, and z-order
    pub fn with_params(texture_id: TextureId, layer: RenderLayer, z_order: i32) -> Self {
        Self {
            texture_id: Some(texture_id),
            layer,
            z_order,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Check if a sprite with the given texture can be added to this batch
    pub fn can_add(&self, texture_id: TextureId) -> bool {
        // Check if batch is not full and texture matches
        self.vertices.len() / 4 < MAX_BATCH_SPRITES
            && self.texture_id.is_none_or(|id| id == texture_id)
    }

    /// Add a sprite quad to the batch
    pub fn add_quad(&mut self, vertices: [SpriteVertex; 4]) {
        let base_index = self.vertices.len() as u16;
        self.vertices.extend_from_slice(&vertices);

        // Two triangles per quad
        self.indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index + 2,
            base_index + 3,
            base_index,
        ]);
    }

    /// Clear the batch completely (resets all parameters and data)
    pub fn clear(&mut self) {
        self.texture_id = None;
        self.layer = RenderLayer::default();
        self.z_order = 0;
        self.vertices.clear();
        self.indices.clear();
    }

    /// Clear only vertex and index data, preserving texture, layer, and z-order
    ///
    /// This is useful for reusing batches with the same parameters across frames
    pub fn clear_data(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Get the texture ID
    pub fn texture_id(&self) -> Option<TextureId> {
        self.texture_id
    }

    /// Set the texture ID
    pub fn set_texture_id(&mut self, texture_id: TextureId) {
        self.texture_id = Some(texture_id);
    }

    /// Get the rendering layer
    pub fn layer(&self) -> RenderLayer {
        self.layer
    }

    /// Set the rendering layer
    pub fn set_layer(&mut self, layer: RenderLayer) {
        self.layer = layer;
    }

    /// Get the z-order
    pub fn z_order(&self) -> i32 {
        self.z_order
    }

    /// Set the z-order
    pub fn set_z_order(&mut self, z_order: i32) {
        self.z_order = z_order;
    }

    /// Get vertices
    pub fn vertices(&self) -> &[SpriteVertex] {
        &self.vertices
    }

    /// Get indices
    pub fn indices(&self) -> &[u16] {
        &self.indices
    }

    /// Get the number of quads in this batch
    pub fn quad_count(&self) -> usize {
        self.indices.len() / 6
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_quad() -> [SpriteVertex; 4] {
        [
            SpriteVertex {
                position: [0.0, 0.0],
                tex_coords: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            SpriteVertex {
                position: [1.0, 0.0],
                tex_coords: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            SpriteVertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            SpriteVertex {
                position: [0.0, 1.0],
                tex_coords: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ]
    }

    #[test]
    fn test_batch_new() {
        let batch = RenderBatch::new();
        assert_eq!(batch.vertices().len(), 0);
        assert_eq!(batch.indices().len(), 0);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_batch_add_single_quad() {
        let mut batch = RenderBatch::new();
        batch.add_quad(create_test_quad());

        assert_eq!(batch.vertices().len(), 4);
        assert_eq!(batch.indices().len(), 6);
        assert_eq!(batch.quad_count(), 1);
        assert!(!batch.is_empty());

        // Verify indices form two triangles
        assert_eq!(batch.indices(), &[0, 1, 2, 2, 3, 0]);
    }

    #[test]
    fn test_batch_add_multiple_quads() {
        let mut batch = RenderBatch::new();
        batch.add_quad(create_test_quad());
        batch.add_quad(create_test_quad());
        batch.add_quad(create_test_quad());

        assert_eq!(batch.vertices().len(), 12);
        assert_eq!(batch.indices().len(), 18);
        assert_eq!(batch.quad_count(), 3);

        // Verify second quad indices
        assert_eq!(&batch.indices()[6..12], &[4, 5, 6, 6, 7, 4]);
    }

    #[test]
    fn test_batch_clear() {
        let mut batch = RenderBatch::new();
        batch.add_quad(create_test_quad());
        batch.add_quad(create_test_quad());

        assert!(!batch.is_empty());

        batch.clear();

        assert_eq!(batch.vertices().len(), 0);
        assert_eq!(batch.indices().len(), 0);
        assert!(batch.is_empty());
        assert_eq!(batch.quad_count(), 0);
    }

    #[test]
    fn test_batch_clear_and_reuse() {
        let mut batch = RenderBatch::new();
        batch.add_quad(create_test_quad());
        batch.clear();
        batch.add_quad(create_test_quad());

        assert_eq!(batch.vertices().len(), 4);
        assert_eq!(batch.indices().len(), 6);
        assert_eq!(batch.quad_count(), 1);

        // Indices should restart from 0
        assert_eq!(batch.indices(), &[0, 1, 2, 2, 3, 0]);
    }

    #[test]
    fn test_batch_many_quads() {
        let mut batch = RenderBatch::new();

        for _ in 0..100 {
            batch.add_quad(create_test_quad());
        }

        assert_eq!(batch.vertices().len(), 400);
        assert_eq!(batch.indices().len(), 600);
        assert_eq!(batch.quad_count(), 100);
    }

    #[test]
    fn test_batch_default() {
        let batch = RenderBatch::default();
        assert!(batch.is_empty());
        assert_eq!(batch.quad_count(), 0);
    }

    #[test]
    fn test_vertex_data_preserved() {
        let mut batch = RenderBatch::new();
        let quad = create_test_quad();
        batch.add_quad(quad);

        let vertices = batch.vertices();
        assert_eq!(vertices[0].position, [0.0, 0.0]);
        assert_eq!(vertices[1].position, [1.0, 0.0]);
        assert_eq!(vertices[2].position, [1.0, 1.0]);
        assert_eq!(vertices[3].position, [0.0, 1.0]);
    }

    #[test]
    fn test_batch_with_params() {
        let batch = RenderBatch::with_params(42, RenderLayer::Characters, 10);
        assert_eq!(batch.texture_id(), Some(42));
        assert_eq!(batch.layer(), RenderLayer::Characters);
        assert_eq!(batch.z_order(), 10);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_batch_set_texture_id() {
        let mut batch = RenderBatch::new();
        assert_eq!(batch.texture_id(), None);

        batch.set_texture_id(123);
        assert_eq!(batch.texture_id(), Some(123));
    }

    #[test]
    fn test_batch_set_layer() {
        let mut batch = RenderBatch::new();
        assert_eq!(batch.layer(), RenderLayer::Background);

        batch.set_layer(RenderLayer::UI);
        assert_eq!(batch.layer(), RenderLayer::UI);
    }

    #[test]
    fn test_batch_set_z_order() {
        let mut batch = RenderBatch::new();
        assert_eq!(batch.z_order(), 0);

        batch.set_z_order(42);
        assert_eq!(batch.z_order(), 42);
    }

    #[test]
    fn test_batch_can_add() {
        let mut batch = RenderBatch::new();

        // Empty batch can accept any texture
        assert!(batch.can_add(1));
        assert!(batch.can_add(2));

        // Set texture and add a quad
        batch.set_texture_id(1);
        batch.add_quad(create_test_quad());

        // Can add same texture
        assert!(batch.can_add(1));
        // Cannot add different texture
        assert!(!batch.can_add(2));
    }

    #[test]
    fn test_batch_can_add_max_sprites() {
        let mut batch = RenderBatch::with_params(1, RenderLayer::Background, 0);

        // Add MAX_BATCH_SPRITES quads
        for _ in 0..MAX_BATCH_SPRITES {
            assert!(batch.can_add(1));
            batch.add_quad(create_test_quad());
        }

        // Should not be able to add more
        assert!(!batch.can_add(1));
    }

    #[test]
    fn test_batch_clear_resets_params() {
        let mut batch = RenderBatch::with_params(42, RenderLayer::UI, 10);
        batch.add_quad(create_test_quad());

        batch.clear();

        assert_eq!(batch.texture_id(), None);
        assert_eq!(batch.layer(), RenderLayer::Background);
        assert_eq!(batch.z_order(), 0);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_batch_clear_data_preserves_params() {
        let mut batch = RenderBatch::with_params(42, RenderLayer::UI, 10);
        batch.add_quad(create_test_quad());
        batch.add_quad(create_test_quad());

        assert!(!batch.is_empty());
        assert_eq!(batch.quad_count(), 2);

        batch.clear_data();

        // Data should be cleared
        assert!(batch.is_empty());
        assert_eq!(batch.quad_count(), 0);

        // But parameters should be preserved
        assert_eq!(batch.texture_id(), Some(42));
        assert_eq!(batch.layer(), RenderLayer::UI);
        assert_eq!(batch.z_order(), 10);
    }
}
