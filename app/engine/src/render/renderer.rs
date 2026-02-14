//! Core renderer

use crate::error::EngineResult;
use crate::render::{
    RenderBatch, RenderCommand, RenderLayer, SpritePipeline, SpriteVertex, TransitionKind,
    TransitionPipeline,
};
use crate::text::{FontManager, GlyphCache, TextLayout, TextStyle, TextureAtlas};
use narrative_core::{AssetRef, Rect};
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use winit::window::Window;

/// Texture handle for referencing loaded textures
pub type TextureId = u32;

/// Loaded texture resource
pub struct LoadedTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub size: (u32, u32),
}

/// Main renderer struct
pub struct Renderer {
    /// wgpu instance
    /// Kept alive to maintain GPU context - dropping would invalidate adapter/device
    #[allow(dead_code)]
    instance: wgpu::Instance,
    /// wgpu surface for rendering to window
    surface: wgpu::Surface<'static>,
    /// wgpu adapter (physical GPU)
    /// Kept alive for future features (e.g., adapter info, capabilities query)
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    /// wgpu device (logical GPU)
    device: wgpu::Device,
    /// wgpu command queue
    queue: wgpu::Queue,
    /// Surface configuration
    surface_config: wgpu::SurfaceConfiguration,
    /// Window size
    size: winit::dpi::PhysicalSize<u32>,
    /// Sprite rendering pipeline
    sprite_pipeline: SpritePipeline,
    /// Transition rendering pipeline
    transition_pipeline: TransitionPipeline,
    /// Projection matrix uniform buffer
    projection_buffer: wgpu::Buffer,
    /// Projection bind group
    projection_bind_group: wgpu::BindGroup,
    /// Loaded textures cache
    textures: HashMap<TextureId, LoadedTexture>,
    /// Next texture ID
    next_texture_id: TextureId,
    /// Asset reference to texture ID mapping
    asset_to_texture: HashMap<AssetRef, TextureId>,
    /// Render batches for command processing
    batches: Vec<RenderBatch>,
    /// White 1x1 texture for solid color rendering
    white_texture_id: TextureId,
    /// Font manager for text rendering
    font_manager: FontManager,
    /// Glyph cache for text rendering
    glyph_cache: GlyphCache,
    /// Texture atlas for glyph storage
    text_atlas: TextureAtlas,
    /// Texture ID for the text atlas
    text_atlas_id: Option<TextureId>,
}

impl Renderer {
    /// Create a new renderer with wgpu initialization
    pub async fn new(window: std::sync::Arc<Window>) -> EngineResult<Self> {
        let size = window.inner_size();

        // 1. Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // 2. Create surface
        let surface = instance.create_surface(window.clone())?;

        // 3. Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| {
                crate::error::EngineError::Rendering(format!("Failed to find adapter: {:?}", e))
            })?;

        // 4. Request device and queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Narrative Render Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                experimental_features: Default::default(),
                trace: Default::default(),
            })
            .await?;

        // 5. Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .or_else(|| surface_caps.formats.first().copied())
            .ok_or_else(|| {
                crate::error::EngineError::Rendering(
                    "No supported surface formats found".to_string(),
                )
            })?;

        let alpha_mode = surface_caps.alpha_modes.first().copied().ok_or_else(|| {
            crate::error::EngineError::Rendering("No supported alpha modes found".to_string())
        })?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // VSync
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        // Create sprite pipeline
        let sprite_pipeline = SpritePipeline::new(&device, surface_format);

        // Create transition pipeline
        let transition_pipeline = TransitionPipeline::new(&device, surface_format)?;

        // Create orthographic projection matrix for 2D rendering
        // Using screen coordinates: (0, 0) at top-left, (width, height) at bottom-right
        let projection = create_orthographic_projection(size.width, size.height);
        let projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Projection Buffer"),
            size: std::mem::size_of::<[[f32; 4]; 4]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Write initial projection matrix
        queue.write_buffer(&projection_buffer, 0, bytemuck::cast_slice(&projection));

        // Create projection bind group
        let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Projection Bind Group"),
            layout: sprite_pipeline.uniform_bind_group_layout(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
        });

        // Create font manager for text rendering
        let font_manager = FontManager::new()?;

        // Create glyph cache (capacity: 1024 glyphs)
        // This should be sufficient for typical VN scenes with mixed Japanese/English text
        let glyph_cache = GlyphCache::new(1024)?;

        // Create texture atlas for glyph storage (2048x2048 pixels)
        // This size supports approximately 1000-2000 glyphs depending on their size
        // TODO(Phase 0.4+): Consider dynamic atlas expansion or multi-atlas support
        let text_atlas = TextureAtlas::new(&device, 2048, 2048)?;

        let mut renderer = Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_config,
            size,
            sprite_pipeline,
            transition_pipeline,
            projection_buffer,
            projection_bind_group,
            textures: HashMap::new(),
            next_texture_id: 0,
            asset_to_texture: HashMap::new(),
            batches: Vec::new(),
            white_texture_id: 0, // Temporary, will be replaced
            font_manager,
            glyph_cache,
            text_atlas,
            text_atlas_id: None,
        };

        // Create 1x1 white texture for solid color rendering
        let white_pixel = [255u8, 255, 255, 255]; // RGBA white
        let white_texture_id = renderer.load_texture_from_bytes(1, 1, &white_pixel)?;
        renderer.white_texture_id = white_texture_id;

        Ok(renderer)
    }

    /// Resize the surface
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // Avoid zero-sized surface which causes wgpu validation errors
        // Window minimization can trigger zero-size events
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);

            // Update projection matrix for new size
            let projection = create_orthographic_projection(new_size.width, new_size.height);
            self.queue.write_buffer(
                &self.projection_buffer,
                0,
                bytemuck::cast_slice(&projection),
            );
        }
    }

    /// Render a frame with clear color
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Get current surface texture
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Render pass with clear color (dark blue)
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Render sprites using the sprite pipeline
    ///
    /// This method renders a batch of sprites to the screen
    pub fn render_sprites(
        &mut self,
        vertices: &[super::SpriteVertex],
        indices: &[u16],
        texture_id: TextureId,
    ) -> EngineResult<()> {
        // Get texture
        let texture = self.textures.get(&texture_id).ok_or_else(|| {
            crate::error::EngineError::Rendering(format!("Texture not found: {}", texture_id))
        })?;

        // Get current surface texture
        let output = self.surface.get_current_texture().map_err(|e| {
            crate::error::EngineError::Rendering(format!("Failed to get surface texture: {:?}", e))
        })?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create vertex buffer
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sprite Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // Create index buffer
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sprite Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Sprite Render Encoder"),
            });

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sprite Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(self.sprite_pipeline.pipeline());
            render_pass.set_bind_group(0, &self.projection_bind_group, &[]);
            render_pass.set_bind_group(1, &texture.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Get reference to device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get reference to queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get surface configuration
    pub fn surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.surface_config
    }

    /// Load a Japanese font from file path
    ///
    /// This loads the font into the renderer's font manager for use in text rendering.
    pub fn load_font(&mut self, path: &std::path::Path) -> EngineResult<()> {
        self.font_manager.load_japanese_font(path)
    }

    /// Get reference to font manager (for advanced usage)
    pub fn font_manager(&self) -> &crate::text::FontManager {
        &self.font_manager
    }

    /// Get mutable reference to font manager (for advanced usage)
    pub fn font_manager_mut(&mut self) -> &mut crate::text::FontManager {
        &mut self.font_manager
    }

    /// Load a texture from raw RGBA bytes
    pub fn load_texture_from_bytes(
        &mut self,
        width: u32,
        height: u32,
        rgba_data: &[u8],
    ) -> EngineResult<TextureId> {
        // Validate data size
        let expected_size = (width * height * 4) as usize;
        if rgba_data.len() != expected_size {
            return Err(crate::error::EngineError::Rendering(format!(
                "Invalid texture data size: expected {} bytes, got {}",
                expected_size,
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
            layout: self.sprite_pipeline.texture_bind_group_layout(),
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

        // Store texture and return ID
        let id = self.next_texture_id;

        // Check for ID exhaustion
        if self.next_texture_id == TextureId::MAX {
            return Err(crate::error::EngineError::Rendering(
                "Texture ID pool exhausted".to_string(),
            ));
        }

        self.next_texture_id = self.next_texture_id.saturating_add(1);
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

    /// Load a texture from an image file
    pub fn load_texture_from_file(&mut self, path: &str) -> EngineResult<TextureId> {
        use image::GenericImageView;

        // Load image using image crate
        let img = image::open(path).map_err(|e| {
            crate::error::EngineError::Rendering(format!("Failed to load image {}: {}", path, e))
        })?;

        let rgba = img.to_rgba8();
        let (width, height) = img.dimensions();

        self.load_texture_from_bytes(width, height, &rgba)
    }

    /// Get a loaded texture by ID
    pub fn get_texture(&self, id: TextureId) -> Option<&LoadedTexture> {
        self.textures.get(&id)
    }

    /// Get the size of a loaded texture by ID
    pub fn get_texture_size(&self, id: TextureId) -> Option<(u32, u32)> {
        self.textures.get(&id).map(|texture| texture.size)
    }

    /// Remove a texture from cache
    pub fn remove_texture(&mut self, id: TextureId) -> Option<LoadedTexture> {
        self.textures.remove(&id)
    }

    /// Get or load a texture from an asset reference
    ///
    /// This method caches textures by AssetRef to avoid reloading
    pub fn get_or_load_texture(&mut self, asset: &AssetRef) -> EngineResult<TextureId> {
        // Check if already loaded
        if let Some(&texture_id) = self.asset_to_texture.get(asset) {
            return Ok(texture_id);
        }

        // Load the texture from file
        let texture_id = self.load_texture_from_file(asset.path())?;

        // Cache the mapping
        self.asset_to_texture.insert(asset.clone(), texture_id);

        Ok(texture_id)
    }

    /// Build render batches from render commands
    ///
    /// This processes commands, groups sprites by texture and layer,
    /// and sorts by z-order for efficient rendering
    fn build_batches(&mut self, commands: &[RenderCommand]) -> EngineResult<()> {
        // Clear previous batches
        self.batches.clear();

        let mut current_layer = RenderLayer::Background;
        let mut current_batch: Option<RenderBatch> = None;

        for command in commands {
            match command {
                RenderCommand::SetLayer(layer) => {
                    // Flush current batch when changing layers
                    if let Some(batch) = current_batch.take()
                        && !batch.is_empty()
                    {
                        self.batches.push(batch);
                    }
                    current_layer = *layer;
                }

                RenderCommand::DrawSprite {
                    texture,
                    dest,
                    source,
                    opacity,
                    tint,
                    z_order,
                } => {
                    // Load texture if needed
                    let texture_id = self.get_or_load_texture(texture)?;

                    // Create sprite vertices
                    let vertices = self.create_sprite_vertices(*dest, *source, *opacity, *tint);

                    // Check if we can add to current batch
                    let can_batch = current_batch.as_ref().is_some_and(|b| {
                        b.layer() == current_layer
                            && b.z_order() == *z_order
                            && b.can_add(texture_id)
                    });

                    if !can_batch {
                        // Flush current batch
                        if let Some(batch) = current_batch.take()
                            && !batch.is_empty()
                        {
                            self.batches.push(batch);
                        }

                        // Start new batch
                        current_batch = Some(RenderBatch::with_params(
                            texture_id,
                            current_layer,
                            *z_order,
                        ));
                    }

                    // Add to batch
                    if let Some(ref mut batch) = current_batch {
                        batch.add_quad(vertices);
                    }
                }

                RenderCommand::DrawBackground {
                    texture,
                    opacity,
                    tint,
                } => {
                    // Flush current batch
                    if let Some(batch) = current_batch.take()
                        && !batch.is_empty()
                    {
                        self.batches.push(batch);
                    }

                    // Load texture
                    let texture_id = self.get_or_load_texture(texture)?;

                    // Full screen quad
                    let dest = Rect::new(0.0, 0.0, self.size.width as f32, self.size.height as f32);
                    let vertices = self.create_sprite_vertices(dest, None, *opacity, *tint);

                    // Create batch for background
                    let mut batch =
                        RenderBatch::with_params(texture_id, RenderLayer::Background, 0);
                    batch.add_quad(vertices);
                    self.batches.push(batch);
                }

                RenderCommand::DrawCG {
                    texture,
                    opacity,
                    tint,
                } => {
                    // Flush current batch
                    if let Some(batch) = current_batch.take()
                        && !batch.is_empty()
                    {
                        self.batches.push(batch);
                    }

                    // Load texture
                    let texture_id = self.get_or_load_texture(texture)?;

                    // Full screen quad (same as background)
                    let dest = Rect::new(0.0, 0.0, self.size.width as f32, self.size.height as f32);
                    let vertices = self.create_sprite_vertices(dest, None, *opacity, *tint);

                    // Create batch for CG (layer CG, z_order 0)
                    let mut batch = RenderBatch::with_params(texture_id, RenderLayer::CG, 0);
                    batch.add_quad(vertices);
                    self.batches.push(batch);
                }

                RenderCommand::DrawCharacter {
                    texture,
                    position,
                    scale,
                    opacity,
                    flip_x,
                    z_order,
                } => {
                    // Load texture
                    let texture_id = self.get_or_load_texture(texture)?;

                    // Get texture size for proper scaling
                    let tex_size =
                        self.textures
                            .get(&texture_id)
                            .map(|t| t.size)
                            .ok_or_else(|| {
                                crate::error::EngineError::Rendering(format!(
                                    "Texture not found after loading: {}",
                                    texture_id
                                ))
                            })?;

                    // Calculate screen position
                    // Characters are positioned at the bottom of the screen
                    let x_percent = position.x_percent();
                    let char_width = tex_size.0 as f32 * scale;
                    let char_height = tex_size.1 as f32 * scale;

                    // Center character horizontally at the position
                    let x = (self.size.width as f32 * x_percent) - (char_width / 2.0);
                    // Position at bottom of screen
                    let y = self.size.height as f32 - char_height;

                    let dest = Rect::new(x, y, char_width, char_height);

                    // Handle horizontal flip by swapping UV coordinates
                    let source = if *flip_x {
                        Some(Rect::new(1.0, 0.0, -1.0, 1.0)) // Flipped UVs
                    } else {
                        None // Normal UVs
                    };

                    // Create sprite vertices
                    let vertices = self.create_sprite_vertices(dest, source, *opacity, None);

                    // Check if we can add to current batch
                    let can_batch = current_batch.as_ref().is_some_and(|b| {
                        b.layer() == current_layer
                            && b.z_order() == *z_order
                            && b.can_add(texture_id)
                    });

                    if !can_batch {
                        // Flush current batch
                        if let Some(batch) = current_batch.take()
                            && !batch.is_empty()
                        {
                            self.batches.push(batch);
                        }

                        // Start new batch
                        current_batch = Some(RenderBatch::with_params(
                            texture_id,
                            current_layer,
                            *z_order,
                        ));
                    }

                    // Add to batch
                    if let Some(ref mut batch) = current_batch {
                        batch.add_quad(vertices);
                    }
                }

                RenderCommand::DrawText {
                    text,
                    position,
                    font_size,
                    line_height,
                    color,
                    visible_chars,
                } => {
                    // Flush current batch before text rendering
                    if let Some(batch) = current_batch.take()
                        && !batch.is_empty()
                    {
                        self.batches.push(batch);
                    }

                    // Create text style from command parameters
                    let style = TextStyle {
                        font_size: *font_size,
                        line_height: *line_height,
                        color: *color,
                        family: cosmic_text::Family::SansSerif,
                    };

                    // Create text layout
                    let layout =
                        TextLayout::new(&mut self.font_manager, text.clone(), *position, style);

                    // Ensure text atlas is registered as a texture
                    if self.text_atlas_id.is_none() {
                        // Create bind group for text atlas
                        let bind_group =
                            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("Text Atlas Bind Group"),
                                layout: self.sprite_pipeline.texture_bind_group_layout(),
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::TextureView(
                                            self.text_atlas.view(),
                                        ),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::Sampler(
                                            self.text_atlas.sampler(),
                                        ),
                                    },
                                ],
                            });

                        // Store the atlas texture
                        let atlas_id = self.next_texture_id;
                        if self.next_texture_id == TextureId::MAX {
                            return Err(crate::error::EngineError::Rendering(
                                "Texture ID pool exhausted".to_string(),
                            ));
                        }
                        self.next_texture_id = self.next_texture_id.saturating_add(1);

                        let (width, height) = self.text_atlas.dimensions();
                        self.textures.insert(
                            atlas_id,
                            LoadedTexture {
                                texture: self.text_atlas.texture().clone(),
                                view: self.text_atlas.view().clone(),
                                sampler: self.text_atlas.sampler().clone(),
                                bind_group,
                                size: (width, height),
                            },
                        );

                        self.text_atlas_id = Some(atlas_id);
                    }

                    let atlas_id = self.text_atlas_id.ok_or_else(|| {
                        crate::error::EngineError::Rendering(
                            "Text atlas not initialized".to_string(),
                        )
                    })?;

                    // Render each glyph (limited to visible_chars for typewriter effect)
                    let max_chars = visible_chars.unwrap_or(usize::MAX);
                    let mut rendered_chars = 0;

                    'outer: for run in layout.buffer().layout_runs() {
                        for glyph in run.glyphs.iter() {
                            // Stop if we've reached the character limit (typewriter effect)
                            if rendered_chars >= max_chars {
                                break 'outer;
                            }

                            // Construct cache key from glyph metadata
                            let cache_key = cosmic_text::CacheKey {
                                font_id: glyph.font_id,
                                glyph_id: glyph.glyph_id,
                                font_size_bits: (glyph.font_size * 64.0) as u32,
                                font_weight: cosmic_text::Weight::NORMAL,
                                x_bin: cosmic_text::SubpixelBin::Zero,
                                y_bin: cosmic_text::SubpixelBin::Zero,
                                flags: cosmic_text::CacheKeyFlags::empty(),
                            };

                            // Get or rasterize glyph
                            if let Some(glyph_info) = self.glyph_cache.get_or_rasterize(
                                self.font_manager.font_system_mut(),
                                &mut self.text_atlas,
                                &self.queue,
                                cache_key,
                            )? {
                                // Calculate glyph position
                                let glyph_x = position.x + glyph.x;
                                let glyph_y = position.y + run.line_y;
                                // Apply glyph offsets
                                let dest_x = glyph_x + glyph_info.offset_x as f32;
                                let dest_y = glyph_y + glyph_info.offset_y as f32;

                                let dest = Rect::new(
                                    dest_x,
                                    dest_y,
                                    glyph_info.width as f32,
                                    glyph_info.height as f32,
                                );

                                // Calculate UV coordinates in the atlas
                                let (atlas_w, atlas_h) = self.text_atlas.dimensions();
                                let u = glyph_info.atlas_pos.0 as f32 / atlas_w as f32;
                                let v = glyph_info.atlas_pos.1 as f32 / atlas_h as f32;
                                let u_width = glyph_info.width as f32 / atlas_w as f32;
                                let v_height = glyph_info.height as f32 / atlas_h as f32;

                                let source = Some(Rect::new(u, v, u_width, v_height));

                                // Create sprite vertices for this glyph
                                let vertices =
                                    self.create_sprite_vertices(dest, source, 1.0, Some(*color));

                                // Check if we can add to current batch
                                let can_batch = current_batch.as_ref().is_some_and(|b| {
                                    b.layer() == current_layer && b.can_add(atlas_id)
                                });

                                if !can_batch {
                                    // Flush current batch
                                    if let Some(batch) = current_batch.take()
                                        && !batch.is_empty()
                                    {
                                        self.batches.push(batch);
                                    }

                                    // Start new batch for text
                                    current_batch =
                                        Some(RenderBatch::with_params(atlas_id, current_layer, 0));
                                }

                                // Add glyph to batch
                                if let Some(ref mut batch) = current_batch {
                                    batch.add_quad(vertices);
                                }
                            }

                            // Increment character count after processing each glyph
                            rendered_chars = rendered_chars.saturating_add(1);
                        }
                    }
                }

                RenderCommand::DrawDialogueBox {
                    rect,
                    background,
                    border,
                    border_width,
                } => {
                    // Flush current batch
                    if let Some(batch) = current_batch.take()
                        && !batch.is_empty()
                    {
                        self.batches.push(batch);
                    }

                    // Draw background rectangle
                    let bg_vertices =
                        self.create_sprite_vertices(*rect, None, 1.0, Some(*background));
                    let mut bg_batch =
                        RenderBatch::with_params(self.white_texture_id, current_layer, 0);
                    bg_batch.add_quad(bg_vertices);
                    self.batches.push(bg_batch);

                    // Draw border if specified
                    if let Some(border_color) = border {
                        let w = *border_width;
                        // Top border
                        let top = Rect::new(rect.x, rect.y, rect.width, w);
                        // Bottom border
                        let bottom = Rect::new(rect.x, rect.y + rect.height - w, rect.width, w);
                        // Left border
                        let left = Rect::new(rect.x, rect.y, w, rect.height);
                        // Right border
                        let right = Rect::new(rect.x + rect.width - w, rect.y, w, rect.height);

                        for border_rect in [top, bottom, left, right] {
                            let border_vertices = self.create_sprite_vertices(
                                border_rect,
                                None,
                                1.0,
                                Some(*border_color),
                            );
                            let mut border_batch =
                                RenderBatch::with_params(self.white_texture_id, current_layer, 1);
                            border_batch.add_quad(border_vertices);
                            self.batches.push(border_batch);
                        }
                    }
                }

                RenderCommand::DrawRect {
                    rect,
                    color,
                    corner_radius: _, // TODO: Implement rounded corners
                } => {
                    // Flush current batch
                    if let Some(batch) = current_batch.take()
                        && !batch.is_empty()
                    {
                        self.batches.push(batch);
                    }

                    // Draw solid color rectangle using white texture with color tint
                    let vertices = self.create_sprite_vertices(*rect, None, 1.0, Some(*color));
                    let mut rect_batch =
                        RenderBatch::with_params(self.white_texture_id, current_layer, 0);
                    rect_batch.add_quad(vertices);
                    self.batches.push(rect_batch);
                }

                RenderCommand::DrawBorder {
                    rect,
                    color,
                    width,
                    corner_radius: _, // TODO: Implement rounded corners
                } => {
                    // Flush current batch
                    if let Some(batch) = current_batch.take()
                        && !batch.is_empty()
                    {
                        self.batches.push(batch);
                    }

                    // Draw border as four rectangles (top, bottom, left, right)
                    let w = *width;
                    // Top border
                    let top = Rect::new(rect.x, rect.y, rect.width, w);
                    // Bottom border
                    let bottom = Rect::new(rect.x, rect.y + rect.height - w, rect.width, w);
                    // Left border
                    let left = Rect::new(rect.x, rect.y, w, rect.height);
                    // Right border
                    let right = Rect::new(rect.x + rect.width - w, rect.y, w, rect.height);

                    for border_rect in [top, bottom, left, right] {
                        let vertices =
                            self.create_sprite_vertices(border_rect, None, 1.0, Some(*color));
                        let mut border_batch =
                            RenderBatch::with_params(self.white_texture_id, current_layer, 0);
                        border_batch.add_quad(vertices);
                        self.batches.push(border_batch);
                    }
                }

                RenderCommand::DrawTransition {
                    kind: _,
                    progress: _,
                    fade_color: _,
                    from_texture: _,
                    to_texture: _,
                } => {
                    // Flush current batch before rendering transition
                    if let Some(batch) = current_batch.take()
                        && !batch.is_empty()
                    {
                        self.batches.push(batch);
                    }

                    // Transitions are handled separately in render pass
                    // We'll store transition commands and render them last
                    // For now, we'll handle them in the render_commands method directly
                    // This is a placeholder - actual transition rendering happens in render pass
                }
            }
        }

        // Flush final batch
        if let Some(batch) = current_batch
            && !batch.is_empty()
        {
            self.batches.push(batch);
        }

        // Sort batches by layer then z-order
        self.batches
            .sort_by_key(|b| (b.layer() as i32, b.z_order()));

        Ok(())
    }

    /// Create sprite vertices from rectangle and parameters
    ///
    /// Creates a quad with CCW (Counter-Clockwise) winding order:
    /// ```text
    /// 0-------1
    /// |     / |
    /// |   /   |
    /// | /     |
    /// 3-------2
    /// ```
    /// Triangle 1: 0→1→2 (top-left to top-right to bottom-right)
    /// Triangle 2: 2→3→0 (bottom-right to bottom-left to top-left)
    ///
    /// # Arguments
    /// * `dest` - Destination rectangle in screen coordinates
    /// * `source` - Optional source UV coordinates (None = full texture 0.0-1.0)
    /// * `opacity` - Alpha multiplier (0.0 = transparent, 1.0 = opaque)
    /// * `tint` - Optional color tint (None = white, no tinting)
    fn create_sprite_vertices(
        &self,
        dest: Rect,
        source: Option<Rect>,
        opacity: f32,
        tint: Option<narrative_core::Color>,
    ) -> [SpriteVertex; 4] {
        // UV coordinates (default to full texture)
        let uv = source.unwrap_or_else(|| Rect::new(0.0, 0.0, 1.0, 1.0));

        // Calculate color with opacity and tint
        let base_color = tint.unwrap_or_else(|| narrative_core::Color::new(1.0, 1.0, 1.0, 1.0));
        let color = [
            base_color.r,
            base_color.g,
            base_color.b,
            base_color.a * opacity,
        ];

        // Create quad vertices (CCW winding)
        [
            SpriteVertex {
                position: [dest.x, dest.y],
                tex_coords: [uv.x, uv.y],
                color,
            },
            SpriteVertex {
                position: [dest.x + dest.width, dest.y],
                tex_coords: [uv.x + uv.width, uv.y],
                color,
            },
            SpriteVertex {
                position: [dest.x + dest.width, dest.y + dest.height],
                tex_coords: [uv.x + uv.width, uv.y + uv.height],
                color,
            },
            SpriteVertex {
                position: [dest.x, dest.y + dest.height],
                tex_coords: [uv.x, uv.y + uv.height],
                color,
            },
        ]
    }

    /// Render using render commands
    ///
    /// This is the main rendering method that processes commands and renders batches
    pub fn render_commands(&mut self, commands: &[RenderCommand]) -> EngineResult<()> {
        // Build batches from commands
        self.build_batches(commands)?;

        // Extract transition commands (overlay layer)
        let transition_commands: Vec<_> = commands
            .iter()
            .filter_map(|cmd| match cmd {
                RenderCommand::DrawTransition {
                    kind,
                    progress,
                    fade_color,
                    from_texture,
                    to_texture,
                } => Some((*kind, *progress, *fade_color, from_texture, to_texture)),
                _ => None,
            })
            .collect();

        // Get current surface texture
        let output = self.surface.get_current_texture().map_err(|e| {
            crate::error::EngineError::Rendering(format!("Failed to get surface texture: {:?}", e))
        })?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Commands Encoder"),
            });

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Set pipeline
            render_pass.set_pipeline(self.sprite_pipeline.pipeline());
            render_pass.set_bind_group(0, &self.projection_bind_group, &[]);

            // Render each batch
            for batch in &self.batches {
                if batch.is_empty() {
                    continue;
                }

                // Get texture for this batch
                let texture_id = match batch.texture_id() {
                    Some(id) => id,
                    None => continue,
                };

                let texture = match self.textures.get(&texture_id) {
                    Some(t) => t,
                    None => continue,
                };

                // Create vertex buffer for this batch
                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Batch Vertex Buffer"),
                            contents: bytemuck::cast_slice(batch.vertices()),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                // Create index buffer for this batch
                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Batch Index Buffer"),
                            contents: bytemuck::cast_slice(batch.indices()),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                // Draw this batch
                render_pass.set_bind_group(1, &texture.bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..batch.indices().len() as u32, 0, 0..1);
            }

            // Render transitions (overlay layer)
            for (kind, progress, fade_color_opt, from_texture_opt, to_texture_opt) in
                transition_commands
            {
                self.render_transition(
                    &mut render_pass,
                    kind,
                    progress,
                    fade_color_opt,
                    from_texture_opt,
                    to_texture_opt,
                )?;
            }
        }

        // Submit commands and present
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Render a transition effect
    fn render_transition(
        &self,
        render_pass: &mut wgpu::RenderPass,
        kind: TransitionKind,
        progress: f32,
        fade_color_opt: Option<narrative_core::Color>,
        _from_texture_opt: &Option<AssetRef>,
        _to_texture_opt: &Option<AssetRef>,
    ) -> EngineResult<()> {
        match kind {
            TransitionKind::FadeBlack => {
                let fade_color = narrative_core::Color::BLACK;
                let params_buffer = self.transition_pipeline.create_fade_params_buffer(
                    &self.device,
                    progress,
                    fade_color,
                );
                let params_bind_group = self
                    .transition_pipeline
                    .create_fade_params_bind_group(&self.device, &params_buffer);

                render_pass.set_bind_group(1, &params_bind_group, &[]);
                self.transition_pipeline.render_fade(
                    render_pass,
                    &self.projection_bind_group,
                    progress,
                    fade_color,
                );
            }
            TransitionKind::FadeWhite => {
                let fade_color = narrative_core::Color::WHITE;
                let params_buffer = self.transition_pipeline.create_fade_params_buffer(
                    &self.device,
                    progress,
                    fade_color,
                );
                let params_bind_group = self
                    .transition_pipeline
                    .create_fade_params_bind_group(&self.device, &params_buffer);

                render_pass.set_bind_group(1, &params_bind_group, &[]);
                self.transition_pipeline.render_fade(
                    render_pass,
                    &self.projection_bind_group,
                    progress,
                    fade_color,
                );
            }
            TransitionKind::FadeColor => {
                let fade_color = fade_color_opt.unwrap_or(narrative_core::Color::BLACK);
                let params_buffer = self.transition_pipeline.create_fade_params_buffer(
                    &self.device,
                    progress,
                    fade_color,
                );
                let params_bind_group = self
                    .transition_pipeline
                    .create_fade_params_bind_group(&self.device, &params_buffer);

                render_pass.set_bind_group(1, &params_bind_group, &[]);
                self.transition_pipeline.render_fade(
                    render_pass,
                    &self.projection_bind_group,
                    progress,
                    fade_color,
                );
            }
            TransitionKind::CrossDissolve => {
                // TODO: Implement cross-dissolve with texture blending
                // This requires loading both from_texture and to_texture
                // For now, this is a placeholder
            }
        }

        Ok(())
    }
}

/// Create an orthographic projection matrix for 2D rendering
/// Maps screen coordinates (0, 0) at top-left to (-1, 1) in clip space
/// and (width, height) at bottom-right to (1, -1) in clip space
fn create_orthographic_projection(width: u32, height: u32) -> [[f32; 4]; 4] {
    let w = width as f32;
    let h = height as f32;

    // Orthographic projection matrix for 2D screen-space coordinates
    // Maps (0, 0) to (-1, 1) and (width, height) to (1, -1)
    // Matrix layout (column-major for WGSL):
    // [ 2/w    0      0   -1 ]
    // [ 0     -2/h    0    1 ]
    // [ 0      0      1    0 ]
    // [ 0      0      0    1 ]
    [
        [2.0 / w, 0.0, 0.0, 0.0],
        [0.0, -2.0 / h, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0, 1.0],
    ]
}
