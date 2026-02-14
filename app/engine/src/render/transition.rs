//! Transition pipeline for scene transitions

use crate::error::EngineResult;
use narrative_core::Color;
use wgpu::util::DeviceExt;

/// Uniform data for fade transitions
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct FadeParams {
    /// Transition progress (0.0 = fully visible, 1.0 = fully faded)
    progress: f32,
    /// Fade color RGB
    fade_color: [f32; 3],
    /// Padding for alignment (unused)
    _padding: f32,
}

/// Uniform data for dissolve transitions
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DissolveParams {
    /// Transition progress (0.0 = from_texture, 1.0 = to_texture)
    progress: f32,
    /// Padding for alignment
    _padding: [f32; 3],
}

/// Vertex for fullscreen quad rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransitionVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl TransitionVertex {
    /// Vertex layout descriptor
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TransitionVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Texture coordinates
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }

    /// Create fullscreen quad vertices
    /// Returns vertices for a quad covering the entire screen
    pub fn fullscreen_quad() -> [TransitionVertex; 4] {
        [
            // Top-left
            TransitionVertex {
                position: [-1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            // Top-right
            TransitionVertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            // Bottom-right
            TransitionVertex {
                position: [1.0, -1.0],
                tex_coords: [1.0, 1.0],
            },
            // Bottom-left
            TransitionVertex {
                position: [-1.0, -1.0],
                tex_coords: [0.0, 1.0],
            },
        ]
    }

    /// Create fullscreen quad indices (CCW winding)
    pub fn fullscreen_indices() -> [u16; 6] {
        [0, 1, 2, 2, 3, 0]
    }
}

/// Pipeline for rendering transition effects
pub struct TransitionPipeline {
    /// Fade transition pipeline
    fade_pipeline: wgpu::RenderPipeline,
    /// Cross-dissolve transition pipeline
    dissolve_pipeline: wgpu::RenderPipeline,
    /// Uniform bind group layout (for projection matrix)
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    /// Fade params bind group layout
    fade_params_layout: wgpu::BindGroupLayout,
    /// Dissolve texture bind group layout
    dissolve_texture_layout: wgpu::BindGroupLayout,
    /// Dissolve params bind group layout
    dissolve_params_layout: wgpu::BindGroupLayout,
    /// Fullscreen quad vertex buffer
    vertex_buffer: wgpu::Buffer,
    /// Fullscreen quad index buffer
    index_buffer: wgpu::Buffer,
}

impl TransitionPipeline {
    /// Create a new transition pipeline
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> EngineResult<Self> {
        // Create bind group layouts
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Transition Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let fade_params_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Fade Params Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let dissolve_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Dissolve Texture Bind Group Layout"),
                entries: &[
                    // from_texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // from_sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // to_texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // to_sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let dissolve_params_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Dissolve Params Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Load shaders
        let fade_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fade Transition Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/transition_fade.wgsl").into()),
        });

        let dissolve_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cross-Dissolve Transition Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/transition_dissolve.wgsl").into(),
            ),
        });

        // Create fade pipeline
        let fade_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fade Transition Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &fade_params_layout],
            immediate_size: 0,
        });

        let fade_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fade Transition Pipeline"),
            layout: Some(&fade_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &fade_shader,
                entry_point: Some("vs_main"),
                buffers: &[TransitionVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fade_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Create dissolve pipeline
        let dissolve_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Dissolve Transition Pipeline Layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout,
                    &dissolve_texture_layout,
                    &dissolve_params_layout,
                ],
                immediate_size: 0,
            });

        let dissolve_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Dissolve Transition Pipeline"),
            layout: Some(&dissolve_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &dissolve_shader,
                entry_point: Some("vs_main"),
                buffers: &[TransitionVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &dissolve_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Create fullscreen quad buffers
        let vertices = TransitionVertex::fullscreen_quad();
        let indices = TransitionVertex::fullscreen_indices();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transition Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transition Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Ok(Self {
            fade_pipeline,
            dissolve_pipeline,
            uniform_bind_group_layout,
            fade_params_layout,
            dissolve_texture_layout,
            dissolve_params_layout,
            vertex_buffer,
            index_buffer,
        })
    }

    /// Get the uniform bind group layout
    pub fn uniform_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.uniform_bind_group_layout
    }

    /// Get the fade params bind group layout
    pub fn fade_params_layout(&self) -> &wgpu::BindGroupLayout {
        &self.fade_params_layout
    }

    /// Get the dissolve texture bind group layout
    pub fn dissolve_texture_layout(&self) -> &wgpu::BindGroupLayout {
        &self.dissolve_texture_layout
    }

    /// Get the dissolve params bind group layout
    pub fn dissolve_params_layout(&self) -> &wgpu::BindGroupLayout {
        &self.dissolve_params_layout
    }

    /// Render a fade transition
    pub fn render_fade(
        &self,
        render_pass: &mut wgpu::RenderPass,
        uniform_bind_group: &wgpu::BindGroup,
        _progress: f32,
        _fade_color: Color,
    ) {
        render_pass.set_pipeline(&self.fade_pipeline);
        render_pass.set_bind_group(0, uniform_bind_group, &[]);

        // Create params bind group (will be created by renderer)
        // Note: This is a placeholder - actual implementation needs bind group from renderer
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    /// Render a cross-dissolve transition
    pub fn render_dissolve(
        &self,
        render_pass: &mut wgpu::RenderPass,
        uniform_bind_group: &wgpu::BindGroup,
        texture_bind_group: &wgpu::BindGroup,
        params_bind_group: &wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.dissolve_pipeline);
        render_pass.set_bind_group(0, uniform_bind_group, &[]);
        render_pass.set_bind_group(1, texture_bind_group, &[]);
        render_pass.set_bind_group(2, params_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    /// Create fade params buffer
    pub fn create_fade_params_buffer(
        &self,
        device: &wgpu::Device,
        progress: f32,
        fade_color: Color,
    ) -> wgpu::Buffer {
        let params = FadeParams {
            progress,
            fade_color: [fade_color.r, fade_color.g, fade_color.b],
            _padding: 0.0,
        };

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fade Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }

    /// Create fade params bind group
    pub fn create_fade_params_bind_group(
        &self,
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fade Params Bind Group"),
            layout: &self.fade_params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    /// Create dissolve params buffer
    pub fn create_dissolve_params_buffer(
        &self,
        device: &wgpu::Device,
        progress: f32,
    ) -> wgpu::Buffer {
        let params = DissolveParams {
            progress,
            _padding: [0.0; 3],
        };

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dissolve Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }

    /// Create dissolve params bind group
    pub fn create_dissolve_params_bind_group(
        &self,
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Dissolve Params Bind Group"),
            layout: &self.dissolve_params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    /// Create dissolve texture bind group
    pub fn create_dissolve_texture_bind_group(
        &self,
        device: &wgpu::Device,
        from_view: &wgpu::TextureView,
        from_sampler: &wgpu::Sampler,
        to_view: &wgpu::TextureView,
        to_sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Dissolve Texture Bind Group"),
            layout: &self.dissolve_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(from_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(from_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(to_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(to_sampler),
                },
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_vertex_layout() {
        let layout = TransitionVertex::layout();
        assert_eq!(
            layout.array_stride,
            std::mem::size_of::<TransitionVertex>() as wgpu::BufferAddress
        );
        assert_eq!(layout.step_mode, wgpu::VertexStepMode::Vertex);
        assert_eq!(layout.attributes.len(), 2);
    }

    #[test]
    fn test_fullscreen_quad_vertices() {
        let vertices = TransitionVertex::fullscreen_quad();
        assert_eq!(vertices.len(), 4);

        // Check corners in NDC space
        assert_eq!(vertices[0].position, [-1.0, 1.0]); // Top-left
        assert_eq!(vertices[1].position, [1.0, 1.0]); // Top-right
        assert_eq!(vertices[2].position, [1.0, -1.0]); // Bottom-right
        assert_eq!(vertices[3].position, [-1.0, -1.0]); // Bottom-left

        // Check UV coordinates
        assert_eq!(vertices[0].tex_coords, [0.0, 0.0]);
        assert_eq!(vertices[1].tex_coords, [1.0, 0.0]);
        assert_eq!(vertices[2].tex_coords, [1.0, 1.0]);
        assert_eq!(vertices[3].tex_coords, [0.0, 1.0]);
    }

    #[test]
    fn test_fullscreen_indices() {
        let indices = TransitionVertex::fullscreen_indices();
        assert_eq!(indices.len(), 6);

        // Check winding order (CCW)
        assert_eq!(indices, [0, 1, 2, 2, 3, 0]);
    }

    #[test]
    fn test_fade_params_size() {
        // Verify struct size for GPU alignment (20 bytes due to padding)
        assert_eq!(std::mem::size_of::<FadeParams>(), 20);
    }

    #[test]
    fn test_dissolve_params_size() {
        // Verify struct size for GPU alignment
        assert_eq!(std::mem::size_of::<DissolveParams>(), 16);
    }

    #[test]
    fn test_fade_params_creation() {
        let color = narrative_core::Color::rgb(0.5, 0.6, 0.7);
        let params = FadeParams {
            progress: 0.75,
            fade_color: [color.r, color.g, color.b],
            _padding: 0.0,
        };

        assert_eq!(params.progress, 0.75);
        assert_eq!(params.fade_color, [0.5, 0.6, 0.7]);
    }

    #[test]
    fn test_dissolve_params_creation() {
        let params = DissolveParams {
            progress: 0.5,
            _padding: [0.0; 3],
        };

        assert_eq!(params.progress, 0.5);
    }
}
