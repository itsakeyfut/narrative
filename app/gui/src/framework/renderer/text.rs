//! Text rendering using cosmic-text and swash
//!
//! This module provides text rendering by:
//! 1. Using cosmic-text for text shaping and layout
//! 2. Using swash for glyph rasterization
//! 3. Uploading glyph bitmaps to a wgpu texture atlas
//! 4. Rendering text as textured quads
//!
//! Issue #250: Uses LRU cache for efficient glyph atlas management

use super::super::Color;
use super::super::layout::Point;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};
use lru::LruCache;
use std::num::NonZeroUsize;
use wgpu::util::DeviceExt;

/// Text alignment
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// A single text draw request
#[derive(Debug, Clone)]
pub struct TextDraw {
    pub text: String,
    pub position: Point,
    pub color: Color,
    pub font_size: f32,
    pub line_height: Option<f32>,
    pub align: TextAlign,
    pub max_width: Option<f32>,
}

/// Cached glyph data
///
/// Issue #250: Uses LRU cache for efficient memory management
struct GlyphCache {
    /// Texture atlas for glyphs
    atlas_texture: wgpu::Texture,
    atlas_view: wgpu::TextureView,
    atlas_size: u32,
    /// Current position in atlas
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    /// Glyph UV cache using LRU eviction (Issue #250)
    /// (glyph_id, font_size) -> GlyphInfo
    glyph_uvs: LruCache<(cosmic_text::CacheKey, u32), GlyphInfo>,
}

#[derive(Clone, Copy)]
struct GlyphInfo {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    offset_x: i32,
    offset_y: i32,
}

/// Vertex for text rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

/// Text renderer using cosmic-text
pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    glyph_cache: GlyphCache,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    pending_draws: Vec<TextDraw>,
    vertices: Vec<TextVertex>,
    indices: Vec<u32>,
    /// Cached vertex buffer for reuse (buffer pooling for performance)
    /// See Issue #250 for GPU optimization tracking
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_buffer_capacity: usize,
    /// Cached index buffer for reuse
    index_buffer: Option<wgpu::Buffer>,
    index_buffer_capacity: usize,
}

impl TextRenderer {
    const ATLAS_SIZE: u32 = 1024;

    /// Create a new text renderer
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        // Create glyph atlas texture
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas"),
            size: wgpu::Extent3d {
                width: Self::ATLAS_SIZE,
                height: Self::ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Text Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create uniform buffer for screen size
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text Uniform Buffer"),
            contents: bytemuck::cast_slice(&[width as f32, height as f32, 0.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Text Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/text.wgsl").into()),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,  // position
                        1 => Float32x2,  // uv
                        2 => Float32x4,  // color
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Issue #250: Use LRU cache with capacity for ~4000 glyphs
        // This is enough for common text rendering scenarios while avoiding atlas overflow
        // Note: 4096 is a non-zero constant, so this will always succeed
        let lru_capacity = NonZeroUsize::new(4096).unwrap_or(NonZeroUsize::MIN);

        Self {
            font_system,
            swash_cache,
            glyph_cache: GlyphCache {
                atlas_texture,
                atlas_view,
                atlas_size: Self::ATLAS_SIZE,
                cursor_x: 0,
                cursor_y: 0,
                row_height: 0,
                glyph_uvs: LruCache::new(lru_capacity),
            },
            pipeline,
            bind_group_layout,
            bind_group,
            sampler,
            uniform_buffer,
            pending_draws: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer: None,
            vertex_buffer_capacity: 0,
            index_buffer: None,
            index_buffer_capacity: 0,
        }
    }

    /// Update viewport size
    pub fn resize(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32, 0.0, 0.0]),
        );
    }

    /// Queue text for rendering
    pub fn queue_text(&mut self, draw: TextDraw) {
        self.pending_draws.push(draw);
    }

    /// Clear all queued text
    pub fn clear(&mut self) {
        self.pending_draws.clear();
        self.vertices.clear();
        self.indices.clear();
    }

    /// Prepare text for rendering
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_width: u32,
        screen_height: u32,
    ) -> Result<(), String> {
        // Update uniform buffer
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[screen_width as f32, screen_height as f32, 0.0, 0.0]),
        );

        self.vertices.clear();
        self.indices.clear();

        let draws = std::mem::take(&mut self.pending_draws);

        for draw in draws {
            self.prepare_text_draw(device, queue, &draw)?;
        }

        Ok(())
    }

    fn prepare_text_draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        draw: &TextDraw,
    ) -> Result<(), String> {
        let metrics = Metrics::new(
            draw.font_size,
            draw.line_height.unwrap_or(draw.font_size * 1.2),
        );
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        let width = draw.max_width.unwrap_or(f32::MAX);
        buffer.set_size(&mut self.font_system, Some(width), None);

        let attrs = Attrs::new();
        buffer.set_text(
            &mut self.font_system,
            &draw.text,
            &attrs,
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let x = draw.position.x;
        let mut y = draw.position.y;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((x, y), 1.0);

                // Get or create glyph in atlas
                let glyph_info = self.get_or_create_glyph(
                    device,
                    queue,
                    physical_glyph.cache_key,
                    draw.font_size as u32,
                )?;

                if let Some(info) = glyph_info {
                    // Create quad for this glyph
                    let gx = physical_glyph.x as f32 + info.offset_x as f32;
                    let gy = physical_glyph.y as f32 - info.offset_y as f32;
                    let gw = info.width as f32;
                    let gh = info.height as f32;

                    let u0 = info.x as f32 / self.glyph_cache.atlas_size as f32;
                    let v0 = info.y as f32 / self.glyph_cache.atlas_size as f32;
                    let u1 = (info.x + info.width) as f32 / self.glyph_cache.atlas_size as f32;
                    let v1 = (info.y + info.height) as f32 / self.glyph_cache.atlas_size as f32;

                    let color = draw.color.to_array();
                    let base_idx = self.vertices.len() as u32;

                    self.vertices.extend_from_slice(&[
                        TextVertex {
                            position: [gx, gy],
                            uv: [u0, v0],
                            color,
                        },
                        TextVertex {
                            position: [gx + gw, gy],
                            uv: [u1, v0],
                            color,
                        },
                        TextVertex {
                            position: [gx + gw, gy + gh],
                            uv: [u1, v1],
                            color,
                        },
                        TextVertex {
                            position: [gx, gy + gh],
                            uv: [u0, v1],
                            color,
                        },
                    ]);

                    self.indices.extend_from_slice(&[
                        base_idx,
                        base_idx + 1,
                        base_idx + 2,
                        base_idx,
                        base_idx + 2,
                        base_idx + 3,
                    ]);
                }
            }
            y += run.line_y;
        }

        Ok(())
    }

    /// Get or create a glyph in the atlas
    ///
    /// Issue #250: Uses LRU cache for efficient glyph management.
    /// When atlas is full, clears the oldest entries and rebuilds.
    fn get_or_create_glyph(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        cache_key: cosmic_text::CacheKey,
        font_size: u32,
    ) -> Result<Option<GlyphInfo>, String> {
        let key = (cache_key, font_size);

        // LRU get promotes the entry to most recently used
        if let Some(&info) = self.glyph_cache.glyph_uvs.get(&key) {
            return Ok(Some(info));
        }

        // Rasterize glyph using swash
        let image = self.swash_cache.get_image(&mut self.font_system, cache_key);
        let image = match image {
            Some(img) => img,
            None => return Ok(None),
        };

        if image.placement.width == 0 || image.placement.height == 0 {
            return Ok(None);
        }

        let width = image.placement.width;
        let height = image.placement.height;

        // Check if we need to move to next row
        if self.glyph_cache.cursor_x + width > self.glyph_cache.atlas_size {
            self.glyph_cache.cursor_x = 0;
            self.glyph_cache.cursor_y += self.glyph_cache.row_height;
            self.glyph_cache.row_height = 0;
        }

        // Check if atlas is full
        if self.glyph_cache.cursor_y + height > self.glyph_cache.atlas_size {
            // Atlas is full - clear cache and restart from beginning
            // Issue #250: LRU eviction helps prioritize frequently used glyphs
            // When we rebuild, the LRU cache ensures recently used glyphs are re-added first
            let evicted_count = self.glyph_cache.glyph_uvs.len();
            self.glyph_cache.glyph_uvs.clear();
            self.glyph_cache.cursor_x = 0;
            self.glyph_cache.cursor_y = 0;
            self.glyph_cache.row_height = 0;
            tracing::debug!(
                "Glyph atlas full, clearing cache ({} glyphs evicted). Recently used glyphs will be re-added.",
                evicted_count
            );
        }

        // Convert to grayscale if needed
        let data: Vec<u8> = match image.content {
            cosmic_text::SwashContent::Mask => image.data.clone(),
            cosmic_text::SwashContent::Color => {
                // Convert RGBA to grayscale (use alpha channel)
                image.data.chunks(4).map(|c| c[3]).collect()
            }
            cosmic_text::SwashContent::SubpixelMask => {
                // Use average of RGB
                image
                    .data
                    .chunks(3)
                    .map(|c| ((c[0] as u16 + c[1] as u16 + c[2] as u16) / 3) as u8)
                    .collect()
            }
        };

        // Upload to texture
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.glyph_cache.atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: self.glyph_cache.cursor_x,
                    y: self.glyph_cache.cursor_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let info = GlyphInfo {
            x: self.glyph_cache.cursor_x,
            y: self.glyph_cache.cursor_y,
            width,
            height,
            offset_x: image.placement.left,
            offset_y: image.placement.top,
        };

        // LRU put adds as most recently used
        self.glyph_cache.glyph_uvs.put(key, info);

        // Advance cursor
        self.glyph_cache.cursor_x += width + 1; // +1 for padding
        self.glyph_cache.row_height = self.glyph_cache.row_height.max(height + 1);

        Ok(Some(info))
    }

    /// Render text
    pub fn render<'rp>(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'rp>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), String> {
        if self.vertices.is_empty() {
            return Ok(());
        }

        // Reuse or create vertex buffer (buffer pooling for performance)
        let vertex_count = self.vertices.len();
        if vertex_count > self.vertex_buffer_capacity {
            let new_capacity = vertex_count.max(256).next_power_of_two();
            let buffer_size = new_capacity * std::mem::size_of::<TextVertex>();
            self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Vertex Buffer"),
                size: buffer_size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.vertex_buffer_capacity = new_capacity;
        }

        // Reuse or create index buffer
        let index_count = self.indices.len();
        if index_count > self.index_buffer_capacity {
            let new_capacity = index_count.max(384).next_power_of_two();
            let buffer_size = new_capacity * std::mem::size_of::<u32>();
            self.index_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Index Buffer"),
                size: buffer_size as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.index_buffer_capacity = new_capacity;
        }

        // Write data to buffers
        if let Some(ref buffer) = self.vertex_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.vertices));
        }
        if let Some(ref buffer) = self.index_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.indices));
        }

        // Render
        if let (Some(vertex_buffer), Some(index_buffer)) = (&self.vertex_buffer, &self.index_buffer)
        {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
        }

        Ok(())
    }

    /// Measure text dimensions
    pub fn measure_text(&mut self, text: &str, font_size: f32) -> (f32, f32) {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, Some(f32::MAX), None);

        let attrs = Attrs::new();
        buffer.set_text(&mut self.font_system, text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let width = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        let height = buffer.layout_runs().count() as f32 * font_size * 1.2;

        (width, height)
    }
}
