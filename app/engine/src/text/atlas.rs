//! Texture atlas for glyph caching

use crate::error::{EngineError, EngineResult};

/// Texture atlas for storing glyphs with GPU texture backing
pub struct TextureAtlas {
    width: u32,
    height: u32,
    current_x: u32,
    current_y: u32,
    row_height: u32,
    /// GPU texture for atlas
    texture: wgpu::Texture,
    /// Texture view for binding
    view: wgpu::TextureView,
    /// Sampler for texture sampling
    sampler: wgpu::Sampler,
}

impl std::fmt::Debug for TextureAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextureAtlas")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("current_x", &self.current_x)
            .field("current_y", &self.current_y)
            .field("row_height", &self.row_height)
            .field("texture", &"<wgpu::Texture>")
            .field("view", &"<wgpu::TextureView>")
            .field("sampler", &"<wgpu::Sampler>")
            .finish()
    }
}

impl TextureAtlas {
    /// Create a new texture atlas with GPU texture
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> EngineResult<Self> {
        // Create GPU texture for atlas
        // Format rationale:
        // - Rgba8UnormSrgb: Standard format for text rendering with gamma correction
        // - RGBA (4 channels): Supports both colored glyphs and alpha blending
        // - UnormSrgb: sRGB color space for correct color representation on screen
        // - 8 bits per channel: Sufficient precision for text rendering
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Create texture view
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler with linear filtering for smooth text
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Glyph Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            width,
            height,
            current_x: 0,
            current_y: 0,
            row_height: 0,
            texture,
            view,
            sampler,
        })
    }

    /// Allocate space in the atlas
    pub fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        // Check if allocation is too large for atlas
        if width > self.width || height > self.height {
            return None;
        }

        // Check if we need to move to next row
        if self.current_x + width > self.width {
            self.current_x = 0;
            self.current_y = self.current_y.saturating_add(self.row_height);
            self.row_height = 0;
        }

        // Check if we have space
        if self.current_y + height > self.height {
            return None;
        }

        let x = self.current_x;
        let y = self.current_y;

        self.current_x = self.current_x.saturating_add(width);
        self.row_height = self.row_height.max(height);

        Some((x, y))
    }

    /// Reset the atlas
    pub fn reset(&mut self) {
        self.current_x = 0;
        self.current_y = 0;
        self.row_height = 0;
    }

    /// Get atlas dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Upload glyph data to the atlas texture at the specified position
    pub fn upload(
        &self,
        queue: &wgpu::Queue,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        rgba_data: &[u8],
    ) -> EngineResult<()> {
        // Validate data size using saturating arithmetic to prevent overflow
        let expected_size = (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(4);

        if rgba_data.len() != expected_size {
            return Err(EngineError::GlyphCache(format!(
                "Invalid glyph data size: expected {} bytes, got {}",
                expected_size,
                rgba_data.len()
            )));
        }

        // Write texture data at the specified position
        // Note: write_texture (CPU-to-GPU) does not require COPY_BYTES_PER_ROW_ALIGNMENT (256 bytes)
        // unlike buffer-to-texture copies. 4 * width is sufficient for RGBA8 format.
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            rgba_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width), // 4 bytes per pixel (RGBA8)
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        Ok(())
    }

    /// Get a reference to the atlas texture
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Get a reference to the atlas texture view
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Get a reference to the atlas sampler
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::Arc;

    // Shared GPU device for all tests to avoid resource contention
    static TEST_DEVICE: Lazy<Arc<(wgpu::Device, wgpu::Queue)>> = Lazy::new(|| {
        Arc::new(pollster::block_on(async {
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
                .expect("Failed to find adapter");

            adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("Test Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                    experimental_features: Default::default(),
                    trace: Default::default(),
                })
                .await
                .expect("Failed to create device")
        }))
    });

    // Helper function to get the shared test device
    fn get_test_device() -> (&'static wgpu::Device, &'static wgpu::Queue) {
        let device_and_queue = &**TEST_DEVICE;
        (&device_and_queue.0, &device_and_queue.1)
    }

    #[test]
    fn test_atlas_allocation() {
        let (device, _queue) = get_test_device();
        let mut atlas = TextureAtlas::new(device, 256, 256).unwrap();

        let pos1 = atlas.allocate(64, 64);
        assert_eq!(pos1, Some((0, 0)));

        let pos2 = atlas.allocate(64, 64);
        assert_eq!(pos2, Some((64, 0)));
    }

    #[test]
    fn test_atlas_row_wrap() {
        let (device, _queue) = get_test_device();
        let mut atlas = TextureAtlas::new(device, 256, 256).unwrap();

        // Fill first row
        atlas.allocate(128, 64);
        atlas.allocate(128, 64);

        // Should move to next row
        let pos = atlas.allocate(64, 64);
        assert_eq!(pos, Some((0, 64)));
    }

    #[test]
    fn test_atlas_out_of_space() {
        let (device, _queue) = get_test_device();
        let mut atlas = TextureAtlas::new(device, 128, 128).unwrap();

        // Fill entire atlas vertically
        atlas.allocate(128, 128);

        // Should fail - no space left
        let pos = atlas.allocate(64, 64);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_atlas_reset() {
        let (device, _queue) = get_test_device();
        let mut atlas = TextureAtlas::new(device, 256, 256).unwrap();

        atlas.allocate(64, 64);
        atlas.allocate(64, 64);

        atlas.reset();

        // After reset, should start from (0, 0) again
        let pos = atlas.allocate(64, 64);
        assert_eq!(pos, Some((0, 0)));
    }

    #[test]
    fn test_atlas_dimensions() {
        let (device, _queue) = get_test_device();
        let atlas = TextureAtlas::new(device, 1024, 512).unwrap();
        assert_eq!(atlas.dimensions(), (1024, 512));
    }

    #[test]
    fn test_atlas_various_sizes() {
        let (device, _queue) = get_test_device();
        let mut atlas = TextureAtlas::new(device, 512, 512).unwrap();

        let pos1 = atlas.allocate(32, 32);
        let pos2 = atlas.allocate(64, 64);
        let pos3 = atlas.allocate(128, 128);

        assert_eq!(pos1, Some((0, 0)));
        assert_eq!(pos2, Some((32, 0)));
        assert_eq!(pos3, Some((96, 0)));
    }

    #[test]
    fn test_atlas_tight_packing() {
        let (device, _queue) = get_test_device();
        let mut atlas = TextureAtlas::new(device, 256, 256).unwrap();

        // Pack small items
        for _ in 0..16 {
            let pos = atlas.allocate(64, 64);
            assert!(pos.is_some());
        }
    }

    #[test]
    fn test_atlas_large_allocation_fails() {
        let (device, _queue) = get_test_device();
        let mut atlas = TextureAtlas::new(device, 256, 256).unwrap();

        // Too wide
        let pos = atlas.allocate(512, 64);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_atlas_exact_fit() {
        let (device, _queue) = get_test_device();
        let mut atlas = TextureAtlas::new(device, 256, 256).unwrap();

        let pos = atlas.allocate(256, 256);
        assert_eq!(pos, Some((0, 0)));

        // No more space
        assert_eq!(atlas.allocate(1, 1), None);
    }

    #[test]
    fn test_atlas_upload() {
        let (device, queue) = get_test_device();
        let atlas = TextureAtlas::new(device, 256, 256).unwrap();

        // Create test glyph data (4x4 white pixels)
        let glyph_data = vec![255u8; 4 * 4 * 4]; // 4x4 RGBA

        // Upload should succeed
        let result = atlas.upload(queue, 0, 0, 4, 4, &glyph_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_atlas_upload_invalid_size() {
        let (device, queue) = get_test_device();
        let atlas = TextureAtlas::new(device, 256, 256).unwrap();

        // Invalid data size (should be 4*4*4 = 64 bytes)
        let glyph_data = vec![255u8; 32];

        // Upload should fail
        let result = atlas.upload(queue, 0, 0, 4, 4, &glyph_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_atlas_accessors() {
        let (device, _queue) = get_test_device();
        let atlas = TextureAtlas::new(device, 256, 256).unwrap();

        // Test that accessors don't panic
        let _texture = atlas.texture();
        let _view = atlas.view();
        let _sampler = atlas.sampler();
    }
}
