//! GPU renderer using wgpu
//!
//! Handles all GPU rendering operations including quads, text, and textures.
//!
//! Issue #250 Phase 2: Batched draw calls optimization
//! - `batch` module provides command sorting and grouping
//! - Minimizes GPU pipeline state changes
//! - Accurate draw call counting for metrics

mod batch;
mod quad;
mod text;
mod texture;
// Video rendering removed - was video-editing specific

pub use batch::{BatchBuilder, BatchStats, LayeredCommand, ZLayer};
pub use quad::QuadRenderer;
pub use text::{TextAlign, TextDraw, TextRenderer};
pub use texture::{TextureInstance, TextureRenderer};
// Video rendering removed - was video-editing specific
// pub use video::{VideoRenderer, VideoTexture};

use super::Color;
use super::layout::{Bounds, Point};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Draw commands that can be batched and rendered
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw a filled rectangle
    Rect {
        bounds: Bounds,
        color: Color,
        corner_radius: f32,
    },

    /// Draw a border
    Border {
        bounds: Bounds,
        color: Color,
        width: f32,
        corner_radius: f32,
    },

    /// Draw text
    Text {
        text: String,
        position: Point,
        color: Color,
        font_size: f32,
    },

    /// Draw a texture with opacity
    Texture {
        texture_id: u64,
        bounds: Bounds,
        opacity: f32,
    },

    // VideoFrame removed - was video-editing specific
    // /// Draw a video frame (RGBA data)
    // /// Uses Arc to avoid cloning large frame buffers
    // VideoFrame {
    //     data: Arc<Vec<u8>>,
    //     width: u32,
    //     height: u32,
    //     bounds: Bounds,
    // },
    /// Push a clip region
    PushClip { bounds: Bounds },

    /// Pop a clip region
    PopClip,
}

/// Loaded texture resource
pub struct LoadedTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub size: (u32, u32),
}

/// Error type for renderer operations
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Image loading error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid texture data: {0}")]
    InvalidTextureData(String),

    #[error("Texture ID pool exhausted")]
    TextureIdPoolExhausted,
}

/// The main renderer that coordinates all rendering operations
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_format: wgpu::TextureFormat,
    quad_renderer: QuadRenderer,
    text_renderer: TextRenderer,
    texture_renderer: TextureRenderer,
    // video_renderer removed - was video-editing specific
    // video_renderer: VideoRenderer,
    screen_size: (u32, u32),
    // Texture cache for loaded images
    textures: HashMap<u64, LoadedTexture>,
    next_texture_id: u64,
    // Cached video texture for preview - removed (was video-editing specific)
    // video_texture_cache: Option<(VideoTexture, wgpu::BindGroup)>,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let quad_renderer = QuadRenderer::new(&device, surface_format);
        let text_renderer = TextRenderer::new(&device, &queue, surface_format, width, height);
        let texture_renderer = TextureRenderer::new(&device, surface_format);
        // Video renderer removed - was video-editing specific
        // let video_renderer = VideoRenderer::new(&device, surface_format);

        Self {
            device,
            queue,
            surface_format,
            quad_renderer,
            text_renderer,
            texture_renderer,
            // video_renderer removed
            screen_size: (width, height),
            textures: HashMap::new(),
            next_texture_id: 0,
            // video_texture_cache removed
        }
    }

    /// Update the screen size
    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_size = (width, height);
        self.text_renderer
            .resize(&self.device, &self.queue, width, height);
    }

    /// Render a frame
    pub fn render(
        &mut self,
        surface_view: &wgpu::TextureView,
        commands: &[DrawCommand],
        clear_color: Color,
    ) {
        // Convert draw commands to quad instances and texture instances
        let mut quad_instances = Vec::new();
        let mut texture_instances: HashMap<u64, Vec<TextureInstance>> = HashMap::new();
        // video_frame_bounds removed - was video-editing specific

        for cmd in commands {
            match cmd {
                DrawCommand::Rect {
                    bounds,
                    color,
                    corner_radius,
                } => {
                    quad_instances.push(quad::QuadInstance {
                        position: [bounds.x(), bounds.y()],
                        size: [bounds.width(), bounds.height()],
                        color: color.to_array(),
                        corner_radius: *corner_radius,
                        border_width: 0.0,
                        border_color: [0.0, 0.0, 0.0, 0.0],
                    });
                }
                DrawCommand::Border {
                    bounds,
                    color,
                    width,
                    corner_radius: _,
                } => {
                    // For now, render border as a rect outline (4 rects)
                    // Top
                    quad_instances.push(quad::QuadInstance {
                        position: [bounds.x(), bounds.y()],
                        size: [bounds.width(), *width],
                        color: color.to_array(),
                        corner_radius: 0.0,
                        border_width: 0.0,
                        border_color: [0.0, 0.0, 0.0, 0.0],
                    });
                    // Bottom
                    quad_instances.push(quad::QuadInstance {
                        position: [bounds.x(), bounds.bottom() - width],
                        size: [bounds.width(), *width],
                        color: color.to_array(),
                        corner_radius: 0.0,
                        border_width: 0.0,
                        border_color: [0.0, 0.0, 0.0, 0.0],
                    });
                    // Left
                    quad_instances.push(quad::QuadInstance {
                        position: [bounds.x(), bounds.y() + width],
                        size: [*width, bounds.height() - 2.0 * width],
                        color: color.to_array(),
                        corner_radius: 0.0,
                        border_width: 0.0,
                        border_color: [0.0, 0.0, 0.0, 0.0],
                    });
                    // Right
                    quad_instances.push(quad::QuadInstance {
                        position: [bounds.right() - width, bounds.y() + width],
                        size: [*width, bounds.height() - 2.0 * width],
                        color: color.to_array(),
                        corner_radius: 0.0,
                        border_width: 0.0,
                        border_color: [0.0, 0.0, 0.0, 0.0],
                    });
                }
                DrawCommand::Text {
                    text,
                    position,
                    color,
                    font_size,
                } => {
                    self.text_renderer.queue_text(TextDraw {
                        text: text.clone(),
                        position: *position,
                        color: *color,
                        font_size: *font_size,
                        line_height: None,
                        align: TextAlign::Left,
                        max_width: None,
                    });
                }
                DrawCommand::Texture {
                    texture_id,
                    bounds,
                    opacity,
                } => {
                    // Collect texture instances grouped by texture_id
                    texture_instances
                        .entry(*texture_id)
                        .or_default()
                        .push(TextureInstance {
                            position: [bounds.x(), bounds.y()],
                            size: [bounds.width(), bounds.height()],
                            opacity: *opacity,
                            _padding: [0.0, 0.0, 0.0],
                        });
                }
                // VideoFrame removed - was video-editing specific
                DrawCommand::PushClip { bounds } => {
                    // TODO(#250): Implement scissor rect clipping for UI elements
                    tracing::trace!("Clipping not yet implemented: bounds={:?}", bounds);
                }
                DrawCommand::PopClip => {
                    // TODO(#250): Implement scissor rect clipping for UI elements
                    tracing::trace!("Clipping not yet implemented: PopClip");
                }
            }
        }

        // Prepare text rendering
        if let Err(e) = self.text_renderer.prepare(
            &self.device,
            &self.queue,
            self.screen_size.0,
            self.screen_size.1,
        ) {
            tracing::error!("Failed to prepare text: {}", e);
        }

        // Begin rendering frame
        self.quad_renderer.begin_frame();
        self.texture_renderer.begin_frame();

        // CRITICAL: Prepare ALL buffers BEFORE the render pass
        // This avoids GPU synchronization issues that cause flickering

        // Prepare quad buffers
        self.quad_renderer
            .prepare(&self.device, &self.queue, &quad_instances, self.screen_size);

        // Prepare texture buffers
        for (texture_id, instances) in &texture_instances {
            if self.textures.contains_key(texture_id) {
                self.texture_renderer.prepare(
                    &self.device,
                    &self.queue,
                    *texture_id,
                    instances,
                    self.screen_size,
                );
            }
        }

        // Video rendering removed - was video-editing specific

        // CRITICAL: Ensure all buffer writes are queued before creating the encoder
        // In wgpu 28.0, queue.write_buffer() is guaranteed to complete before
        // the next queue.submit(), so explicit synchronization is not needed
        // (Issue #120 high-priority fix)

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("GUI Render Encoder"),
            });

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GUI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: surface_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Video rendering removed - was video-editing specific

            // Render in proper Z-order: textures (background) → quads (UI) → text (foreground)

            // Textures first (backgrounds and characters)
            for texture_id in texture_instances.keys() {
                if let Some(loaded_texture) = self.textures.get(texture_id) {
                    self.texture_renderer.render(
                        &mut render_pass,
                        *texture_id,
                        &loaded_texture.bind_group,
                    );
                }
            }

            // Quads second (UI elements like dialogue boxes)
            if !quad_instances.is_empty() {
                self.quad_renderer.render(&mut render_pass);
            }

            // Text last (always on top)
            if let Err(e) = self
                .text_renderer
                .render(&mut render_pass, &self.device, &self.queue)
            {
                tracing::error!("Failed to render text: {}", e);
            }
        }

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Measure text dimensions
    pub fn measure_text(&mut self, text: &str, font_size: f32) -> (f32, f32) {
        self.text_renderer.measure_text(text, font_size)
    }

    /// Get the wgpu device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get the wgpu queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get the surface format
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    /// Load a texture from a file path
    ///
    /// Supports PNG and JPEG formats. Returns a texture ID that can be used
    /// with DrawCommand::Texture.
    pub fn load_texture_from_path(&mut self, path: &Path) -> Result<u64, RendererError> {
        use image::GenericImageView;

        // Load image using image crate
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let (width, height) = img.dimensions();

        self.load_texture_from_bytes(&rgba, width, height)
    }

    /// Create a placeholder texture for graceful degradation
    ///
    /// Creates a simple checkerboard pattern texture when assets are missing.
    /// Returns a texture ID that can be used with DrawCommand::Texture.
    pub fn create_placeholder_texture(
        &mut self,
        width: u32,
        height: u32,
        color1: [u8; 4],
        color2: [u8; 4],
    ) -> Result<u64, RendererError> {
        let mut rgba_data = vec![0u8; (width * height * 4) as usize];

        // Create checkerboard pattern (16x16 pixel squares)
        let square_size = 16;
        for y in 0..height {
            for x in 0..width {
                let square_x = x / square_size;
                let square_y = y / square_size;
                let is_even = (square_x + square_y) % 2 == 0;

                let color = if is_even { color1 } else { color2 };
                let idx = ((y * width + x) * 4) as usize;
                rgba_data[idx..idx + 4].copy_from_slice(&color);
            }
        }

        self.load_texture_from_bytes(&rgba_data, width, height)
    }

    /// Load a texture from raw RGBA bytes
    ///
    /// Returns a texture ID that can be used with DrawCommand::Texture.
    pub fn load_texture_from_bytes(
        &mut self,
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<u64, RendererError> {
        // Validate data size
        let expected_size = (width * height * 4) as usize;
        if rgba_data.len() != expected_size {
            return Err(RendererError::InvalidTextureData(format!(
                "Expected {} bytes for {}x{} RGBA texture, got {} bytes",
                expected_size,
                width,
                height,
                rgba_data.len()
            )));
        }

        // Create texture
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Loaded Texture"),
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

        // Write texture data
        self.queue.write_texture(
            texture.as_image_copy(),
            rgba_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Create texture view
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: self.texture_renderer.texture_bind_group_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Get next texture ID
        let id = self.next_texture_id;
        if self.next_texture_id == u64::MAX {
            return Err(RendererError::TextureIdPoolExhausted);
        }
        self.next_texture_id = self.next_texture_id.saturating_add(1);

        // Store texture in cache
        self.textures.insert(
            id,
            LoadedTexture {
                texture,
                view,
                sampler,
                bind_group,
                size: (width, height),
            },
        );

        Ok(id)
    }

    /// Get the size of a loaded texture by ID
    ///
    /// Returns the (width, height) of the texture, or None if the texture is not found.
    pub fn get_texture_size(&self, id: u64) -> Option<(u32, u32)> {
        self.textures.get(&id).map(|texture| texture.size)
    }

    /// Remove a texture from the cache
    ///
    /// This frees GPU memory for the texture. Any subsequent DrawCommand::Texture
    /// using this texture_id will be ignored.
    pub fn remove_texture(&mut self, texture_id: u64) -> Option<LoadedTexture> {
        self.textures.remove(&texture_id)
    }

    // Video renderer methods removed - were video-editing specific
    // /// Get the video renderer
    // pub fn video_renderer(&self) -> &VideoRenderer {
    //     &self.video_renderer
    // }

    // /// Create a video texture
    // pub fn create_video_texture(&self, width: u32, height: u32) -> VideoTexture {
    //     VideoTexture::new(&self.device, width, height)
    // }

    /// Render using a BatchBuilder for optimized draw call ordering
    ///
    /// Issue #250 Phase 2: Returns BatchStats for accurate draw call metrics.
    /// Commands are sorted by z-layer and grouped by type to minimize pipeline switches.
    /// Now uses layered rendering to ensure proper z-order (overlays render on top).
    pub fn render_batched(
        &mut self,
        surface_view: &wgpu::TextureView,
        batch: BatchBuilder,
        clear_color: Color,
    ) -> BatchStats {
        let (layers, stats) = batch.build_by_layer();
        self.render_layered(surface_view, &layers, clear_color);
        stats
    }

    /// Render commands grouped by layer for proper z-ordering
    ///
    /// Each layer is rendered completely (quads then text) before moving to the next.
    /// This ensures overlays (popups, dropdowns) render on top of lower layers.
    fn render_layered(
        &mut self,
        surface_view: &wgpu::TextureView,
        layers: &[(ZLayer, Vec<DrawCommand>)],
        clear_color: Color,
    ) {
        // Type alias for layer data to satisfy clippy::type_complexity
        // Use Vec instead of HashMap to preserve texture insertion order (Issue #120)
        type LayerData = (Vec<quad::QuadInstance>, Vec<(u64, Vec<TextureInstance>)>);

        // Create a single command encoder for all layers
        // This prevents flickering caused by multiple queue.submit() calls per frame
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("GUI Layered Render Encoder"),
            });

        // First, collect all layers' data and queue all text
        let mut layers_data: Vec<LayerData> = Vec::new();

        for (_layer, commands) in layers {
            // Collect quads and textures for this layer
            let mut quad_instances = Vec::new();
            let mut texture_instances: Vec<(u64, Vec<TextureInstance>)> = Vec::new();

            // Collect quads, textures, and queue text
            for cmd in commands {
                match cmd {
                    DrawCommand::Rect {
                        bounds,
                        color,
                        corner_radius,
                    } => {
                        quad_instances.push(quad::QuadInstance {
                            position: [bounds.x(), bounds.y()],
                            size: [bounds.width(), bounds.height()],
                            color: color.to_array(),
                            corner_radius: *corner_radius,
                            border_width: 0.0,
                            border_color: [0.0, 0.0, 0.0, 0.0],
                        });
                    }
                    DrawCommand::Border {
                        bounds,
                        color,
                        width,
                        corner_radius: _,
                    } => {
                        // Border as 4 rects
                        quad_instances.push(quad::QuadInstance {
                            position: [bounds.x(), bounds.y()],
                            size: [bounds.width(), *width],
                            color: color.to_array(),
                            corner_radius: 0.0,
                            border_width: 0.0,
                            border_color: [0.0, 0.0, 0.0, 0.0],
                        });
                        quad_instances.push(quad::QuadInstance {
                            position: [bounds.x(), bounds.bottom() - width],
                            size: [bounds.width(), *width],
                            color: color.to_array(),
                            corner_radius: 0.0,
                            border_width: 0.0,
                            border_color: [0.0, 0.0, 0.0, 0.0],
                        });
                        quad_instances.push(quad::QuadInstance {
                            position: [bounds.x(), bounds.y() + width],
                            size: [*width, bounds.height() - 2.0 * width],
                            color: color.to_array(),
                            corner_radius: 0.0,
                            border_width: 0.0,
                            border_color: [0.0, 0.0, 0.0, 0.0],
                        });
                        quad_instances.push(quad::QuadInstance {
                            position: [bounds.right() - width, bounds.y() + width],
                            size: [*width, bounds.height() - 2.0 * width],
                            color: color.to_array(),
                            corner_radius: 0.0,
                            border_width: 0.0,
                            border_color: [0.0, 0.0, 0.0, 0.0],
                        });
                    }
                    DrawCommand::Text {
                        text,
                        position,
                        color,
                        font_size,
                    } => {
                        self.text_renderer.queue_text(TextDraw {
                            text: text.clone(),
                            position: *position,
                            color: *color,
                            font_size: *font_size,
                            line_height: None,
                            align: TextAlign::Left,
                            max_width: None,
                        });
                    }
                    DrawCommand::Texture {
                        texture_id,
                        bounds,
                        opacity,
                    } => {
                        // Collect texture instances grouped by texture_id, preserving insertion order
                        let instance = TextureInstance {
                            position: [bounds.x(), bounds.y()],
                            size: [bounds.width(), bounds.height()],
                            opacity: *opacity,
                            _padding: [0.0, 0.0, 0.0],
                        };

                        // Find existing entry or create new one
                        if let Some((_id, instances)) = texture_instances
                            .iter_mut()
                            .find(|(id, _)| id == texture_id)
                        {
                            instances.push(instance);
                        } else {
                            texture_instances.push((*texture_id, vec![instance]));
                        }
                    }
                    // VideoFrame removed - was video-editing specific
                    DrawCommand::PushClip { .. } | DrawCommand::PopClip => {
                        // Not yet implemented
                    }
                }
            }

            layers_data.push((quad_instances, texture_instances));
        }

        // Prepare text once for all layers (after all text has been queued)
        if let Err(e) = self.text_renderer.prepare(
            &self.device,
            &self.queue,
            self.screen_size.0,
            self.screen_size.1,
        ) {
            tracing::error!("Failed to prepare text: {}", e);
        }

        // Begin rendering frame
        self.quad_renderer.begin_frame();
        self.texture_renderer.begin_frame();

        // CRITICAL: Prepare ALL buffers BEFORE the render pass
        // Merge all layers' data to avoid buffer overwrites
        // This avoids GPU synchronization issues that cause flickering

        // Merge all quads from all layers (maintains Z-order as layers are sorted)
        let all_quads: Vec<quad::QuadInstance> = layers_data
            .iter()
            .flat_map(|(quads, _)| quads.iter())
            .copied()
            .collect();

        // Merge all textures from all layers while preserving insertion order
        // CRITICAL: Use Vec to maintain Z-order (HashMap randomizes iteration order!)
        let mut all_textures: Vec<(u64, Vec<TextureInstance>)> = Vec::new();
        for (_quads, texture_instances) in &layers_data {
            for (texture_id, instances) in texture_instances {
                // Find existing entry or create new one
                if let Some((_id, existing_instances)) =
                    all_textures.iter_mut().find(|(id, _)| id == texture_id)
                {
                    existing_instances.extend(instances.iter().copied());
                } else {
                    all_textures.push((*texture_id, instances.clone()));
                }
            }
        }

        // Prepare quad buffers once for all layers
        self.quad_renderer
            .prepare(&self.device, &self.queue, &all_quads, self.screen_size);

        // Prepare texture buffers once for all layers (in insertion order)
        for (texture_id, instances) in &all_textures {
            if self.textures.contains_key(texture_id) {
                self.texture_renderer.prepare(
                    &self.device,
                    &self.queue,
                    *texture_id,
                    instances,
                    self.screen_size,
                );
            }
        }

        // Video preparation removed - was video-editing specific

        // CRITICAL: Ensure all buffer writes are queued before creating the encoder
        // In wgpu 28.0, queue.write_buffer() is guaranteed to complete before
        // the next queue.submit(), so explicit synchronization is not needed
        // (Issue #120 high-priority fix)

        // Use a single render pass for all layers (prevents flickering)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GUI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: surface_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Video rendering removed - was video-editing specific

            // Render in proper Z-order: textures (background) → quads (UI) → text (foreground)
            // Textures first (backgrounds and characters) - in insertion order!
            for (texture_id, _instances) in &all_textures {
                if let Some(loaded_texture) = self.textures.get(texture_id) {
                    self.texture_renderer.render(
                        &mut render_pass,
                        *texture_id,
                        &loaded_texture.bind_group,
                    );
                }
            }

            // Quads second (UI elements like dialogue boxes)
            if !all_quads.is_empty() {
                self.quad_renderer.render(&mut render_pass);
            }

            // Render all text at the end (on top of all layers)
            if let Err(e) = self
                .text_renderer
                .render(&mut render_pass, &self.device, &self.queue)
            {
                tracing::error!("Failed to render text: {}", e);
            }
        }

        // Submit all commands at once (single submit per frame prevents flickering)
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // GPU tests are heavy and require a graphics device
    // Run with: cargo test -- --ignored
    #[test]
    #[ignore]
    fn test_load_texture_from_bytes_valid_data() {
        // Create a minimal wgpu instance for testing
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .expect("Failed to find adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Failed to create device");

        let mut renderer = Renderer::new_with_device_and_queue(
            device,
            queue,
            (800, 600),
            wgpu::TextureFormat::Bgra8UnormSrgb,
        );

        // Create 2x2 RGBA test texture (red, green, blue, white)
        let rgba_data = vec![
            255, 0, 0, 255, // Red
            0, 255, 0, 255, // Green
            0, 0, 255, 255, // Blue
            255, 255, 255, 255, // White
        ];

        let result = renderer.load_texture_from_bytes(&rgba_data, 2, 2);
        assert!(result.is_ok());

        let texture_id = result.unwrap();
        assert!(renderer.textures.contains_key(&texture_id));
    }

    #[test]
    #[ignore]
    fn test_load_texture_from_bytes_invalid_size() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("Failed to find adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Failed to create device");

        let mut renderer = Renderer::new_with_device_and_queue(
            device,
            queue,
            (800, 600),
            wgpu::TextureFormat::Bgra8UnormSrgb,
        );

        // Invalid data size (should be 2*2*4 = 16 bytes)
        let rgba_data = vec![255, 0, 0, 255]; // Only 4 bytes

        let result = renderer.load_texture_from_bytes(&rgba_data, 2, 2);
        assert!(result.is_err());

        if let Err(RendererError::InvalidTextureData(msg)) = result {
            assert!(msg.contains("Expected 16 bytes"));
        } else {
            panic!("Expected InvalidTextureData error");
        }
    }

    #[test]
    #[ignore]
    fn test_create_placeholder_texture() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("Failed to find adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Failed to create device");

        let mut renderer = Renderer::new_with_device_and_queue(
            device,
            queue,
            (800, 600),
            wgpu::TextureFormat::Bgra8UnormSrgb,
        );

        // Create placeholder with red/blue checkerboard
        let result = renderer.create_placeholder_texture(
            64,
            64,
            [255, 0, 0, 255], // Red
            [0, 0, 255, 255], // Blue
        );

        assert!(result.is_ok());

        let texture_id = result.unwrap();
        assert!(renderer.textures.contains_key(&texture_id));
    }

    #[test]
    #[ignore]
    fn test_texture_cache() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("Failed to find adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Failed to create device");

        let mut renderer = Renderer::new_with_device_and_queue(
            device,
            queue,
            (800, 600),
            wgpu::TextureFormat::Bgra8UnormSrgb,
        );

        // Load first texture
        let rgba_data1 = vec![255; 16]; // 2x2 white texture
        let id1 = renderer.load_texture_from_bytes(&rgba_data1, 2, 2).unwrap();

        // Load second texture
        let rgba_data2 = vec![0; 16]; // 2x2 black texture
        let id2 = renderer.load_texture_from_bytes(&rgba_data2, 2, 2).unwrap();

        // IDs should be different
        assert_ne!(id1, id2);

        // Both should be in cache
        assert!(renderer.textures.contains_key(&id1));
        assert!(renderer.textures.contains_key(&id2));

        // Remove first texture
        let removed = renderer.remove_texture(id1);
        assert!(removed.is_some());
        assert!(!renderer.textures.contains_key(&id1));
        assert!(renderer.textures.contains_key(&id2));
    }

    #[test]
    #[ignore]
    fn test_texture_id_increment() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("Failed to find adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Failed to create device");

        let mut renderer = Renderer::new_with_device_and_queue(
            device,
            queue,
            (800, 600),
            wgpu::TextureFormat::Bgra8UnormSrgb,
        );

        let rgba_data = vec![255; 16];

        // Load multiple textures and verify ID increment
        let id1 = renderer.load_texture_from_bytes(&rgba_data, 2, 2).unwrap();
        let id2 = renderer.load_texture_from_bytes(&rgba_data, 2, 2).unwrap();
        let id3 = renderer.load_texture_from_bytes(&rgba_data, 2, 2).unwrap();

        assert_eq!(id2, id1 + 1);
        assert_eq!(id3, id2 + 1);
    }

    impl Renderer {
        // Helper method for tests to create Renderer without a surface
        #[cfg(test)]
        fn new_with_device_and_queue(
            device: wgpu::Device,
            queue: wgpu::Queue,
            screen_size: (u32, u32),
            surface_format: wgpu::TextureFormat,
        ) -> Self {
            let quad_renderer = quad::QuadRenderer::new(&device, surface_format);
            let text_renderer = text::TextRenderer::new(
                &device,
                &queue,
                surface_format,
                screen_size.0,
                screen_size.1,
            );
            let texture_renderer = texture::TextureRenderer::new(&device, surface_format);

            Self {
                device,
                queue,
                quad_renderer,
                text_renderer,
                texture_renderer,
                surface_format,
                screen_size,
                textures: HashMap::new(),
                next_texture_id: 0,
            }
        }
    }
}
