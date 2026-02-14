//! Sprite rendering pipeline

use bytemuck::{Pod, Zeroable};

/// Sprite vertex for GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SpriteVertex {
    /// Position [x, y]
    pub position: [f32; 2],
    /// Texture coordinates [u, v]
    pub tex_coords: [f32; 2],
    /// Color [r, g, b, a]
    pub color: [f32; 4],
}

impl SpriteVertex {
    /// Get the vertex buffer layout descriptor
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        // Calculate offsets for vertex attributes
        // position: [f32; 2] at offset 0
        // tex_coords: [f32; 2] at offset 8 (after position)
        // color: [f32; 4] at offset 16 (after position + tex_coords)
        const POSITION_OFFSET: u64 = 0;
        const TEX_COORDS_OFFSET: u64 = POSITION_OFFSET + std::mem::size_of::<[f32; 2]>() as u64;
        const COLOR_OFFSET: u64 = TEX_COORDS_OFFSET + std::mem::size_of::<[f32; 2]>() as u64;

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: POSITION_OFFSET,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Texture coordinates
                wgpu::VertexAttribute {
                    offset: TEX_COORDS_OFFSET,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: COLOR_OFFSET,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Sprite rendering pipeline
pub struct SpritePipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl SpritePipeline {
    /// Create a new sprite pipeline
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sprite.wgsl").into()),
        });

        // Create uniform bind group layout (projection matrix)
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Sprite Uniform Bind Group Layout"),
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

        // Create texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Sprite Texture Bind Group Layout"),
                entries: &[
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            immediate_size: 0,
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[SpriteVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            uniform_bind_group_layout,
            texture_bind_group_layout,
        }
    }

    /// Get the render pipeline
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    /// Get the uniform bind group layout
    pub fn uniform_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.uniform_bind_group_layout
    }

    /// Get the texture bind group layout
    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_vertex_creation() {
        let vertex = SpriteVertex {
            position: [1.0, 2.0],
            tex_coords: [0.5, 0.5],
            color: [1.0, 0.0, 0.0, 1.0],
        };

        assert_eq!(vertex.position, [1.0, 2.0]);
        assert_eq!(vertex.tex_coords, [0.5, 0.5]);
        assert_eq!(vertex.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_sprite_vertex_copy() {
        let v1 = SpriteVertex {
            position: [1.0, 2.0],
            tex_coords: [0.5, 0.5],
            color: [1.0, 0.0, 0.0, 1.0],
        };

        let v2 = v1;

        assert_eq!(v1.position, v2.position);
        assert_eq!(v1.tex_coords, v2.tex_coords);
        assert_eq!(v1.color, v2.color);
    }

    #[test]
    fn test_sprite_vertex_debug() {
        let vertex = SpriteVertex {
            position: [1.0, 2.0],
            tex_coords: [0.5, 0.5],
            color: [1.0, 0.0, 0.0, 1.0],
        };

        let debug_str = format!("{:?}", vertex);
        assert!(debug_str.contains("SpriteVertex"));
    }

    #[test]
    fn test_sprite_vertex_zero() {
        let vertex: SpriteVertex = bytemuck::Zeroable::zeroed();

        assert_eq!(vertex.position, [0.0, 0.0]);
        assert_eq!(vertex.tex_coords, [0.0, 0.0]);
        assert_eq!(vertex.color, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_sprite_vertex_bytemuck_cast() {
        let vertex = SpriteVertex {
            position: [1.0, 2.0],
            tex_coords: [0.5, 0.5],
            color: [1.0, 0.0, 0.0, 1.0],
        };

        // Test Pod/Zeroable traits work for bytemuck
        let bytes: &[u8] = bytemuck::bytes_of(&vertex);
        assert_eq!(bytes.len(), std::mem::size_of::<SpriteVertex>());
    }

    #[test]
    fn test_sprite_vertex_array() {
        let vertices = [
            SpriteVertex {
                position: [0.0, 0.0],
                tex_coords: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            SpriteVertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];

        assert_eq!(vertices.len(), 2);
        assert_eq!(vertices[0].position, [0.0, 0.0]);
        assert_eq!(vertices[1].position, [1.0, 1.0]);
    }

    // Pipeline tests require wgpu device, which is async and integration-test scope
    // Moved to integration tests or app tests
}
