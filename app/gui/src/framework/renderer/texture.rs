//! Texture renderer for rendering images

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// Instance data for a single textured quad
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TextureInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub opacity: f32,
    pub _padding: [f32; 3],
}

impl TextureInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        1 => Float32x2,  // position
        2 => Float32x2,  // size
        3 => Float32,    // opacity
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextureInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Vertex data for the texture quad
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct TextureVertex {
    position: [f32; 2],
}

impl TextureVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextureVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Unit quad vertices (0,0) to (1,1) for texture mapping
const TEXTURE_VERTICES: &[TextureVertex] = &[
    TextureVertex {
        position: [0.0, 0.0],
    },
    TextureVertex {
        position: [1.0, 0.0],
    },
    TextureVertex {
        position: [1.0, 1.0],
    },
    TextureVertex {
        position: [0.0, 1.0],
    },
];

const TEXTURE_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

/// Uniform data for the texture shader
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct TextureUniforms {
    screen_size: [f32; 2],
    _padding: [f32; 2],
}

use std::collections::HashMap;

/// Per-texture instance data
struct TextureInstanceData {
    buffer: wgpu::Buffer,
    capacity: u32,
    count: u32,
}

/// Renderer for textured quads (images, sprites, backgrounds)
///
/// # Architecture
///
/// This renderer uses a prepare/render pattern to avoid GPU synchronization issues:
/// 1. **Prepare phase** (before render pass): Update all GPU buffers
/// 2. **Render phase** (inside render pass): Issue draw calls
///
/// # Lifecycle per frame
///
/// ```text
/// begin_frame()
///   └─> Reset uniforms_updated flag
///
/// prepare(texture_id, instances, screen_size)  // Called for each texture
///   ├─> Update uniform buffer (once per frame)
///   ├─> Create/resize instance buffer if needed
///   └─> Write instance data to GPU
///
/// [Render pass begins]
///
/// render(render_pass, texture_id, bind_group)  // Called for each texture
///   ├─> Set pipeline and bind groups
///   ├─> Bind vertex and instance buffers
///   └─> Draw indexed instances
///
/// [Render pass ends]
/// ```
///
/// # Instance management
///
/// Each texture ID has its own instance buffer, allowing efficient batching
/// of multiple sprites using the same texture. Buffers are automatically resized
/// as needed with power-of-two growth to minimize reallocations.
///
/// # GPU synchronization
///
/// The uniform buffer (screen_size) is shared across all textures and updated
/// only once per frame (controlled by `uniforms_updated` flag). This prevents
/// GPU sync issues that cause flickering.
pub struct TextureRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    /// Per-texture instance buffers (keyed by texture ID)
    instance_data: HashMap<u64, TextureInstanceData>,
    /// Track if uniforms have been updated this frame
    uniforms_updated: bool,
}

impl TextureRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/texture.wgsl").into()),
        });

        // Create uniform bind group layout
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Uniform Bind Group Layout"),
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
                label: Some("Texture Bind Group Layout"),
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

        // Create uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture Uniform Buffer"),
            contents: bytemuck::cast_slice(&[TextureUniforms {
                screen_size: [800.0, 600.0],
                _padding: [0.0, 0.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create uniform bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Texture Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            immediate_size: 0,
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Texture Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TextureVertex::desc(), TextureInstance::desc()],
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
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture Vertex Buffer"),
            contents: bytemuck::cast_slice(TEXTURE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture Index Buffer"),
            contents: bytemuck::cast_slice(TEXTURE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            uniform_bind_group_layout,
            texture_bind_group_layout,
            instance_data: HashMap::new(),
            uniforms_updated: false,
        }
    }

    /// Begin a new frame (call this before any prepare() calls)
    pub fn begin_frame(&mut self) {
        self.uniforms_updated = false;
    }

    /// Prepare texture rendering for a specific texture ID
    ///
    /// This must be called BEFORE the render pass to avoid GPU sync issues
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_id: u64,
        instances: &[TextureInstance],
        screen_size: (u32, u32),
    ) {
        // Update uniform buffer only once per frame (shared across all textures)
        if !self.uniforms_updated {
            queue.write_buffer(
                &self.uniform_buffer,
                0,
                bytemuck::cast_slice(&[TextureUniforms {
                    screen_size: [screen_size.0 as f32, screen_size.1 as f32],
                    _padding: [0.0, 0.0],
                }]),
            );
            self.uniforms_updated = true;
        }

        if instances.is_empty() {
            // Remove this texture's data if it exists
            self.instance_data.remove(&texture_id);
            return;
        }

        // Get or create instance data for this texture
        let required_capacity = instances.len() as u32;
        let data = self.instance_data.entry(texture_id).or_insert_with(|| {
            let new_capacity = required_capacity.max(64).next_power_of_two();
            let buffer_size = (new_capacity as usize) * std::mem::size_of::<TextureInstance>();
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Texture {} Instance Buffer", texture_id)),
                size: buffer_size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            TextureInstanceData {
                buffer,
                capacity: new_capacity,
                count: 0,
            }
        });

        // Resize buffer if needed
        if required_capacity > data.capacity {
            let new_capacity = required_capacity.max(64).next_power_of_two();
            let buffer_size = (new_capacity as usize) * std::mem::size_of::<TextureInstance>();
            data.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Texture {} Instance Buffer", texture_id)),
                size: buffer_size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            data.capacity = new_capacity;
        }

        // Write instance data
        queue.write_buffer(&data.buffer, 0, bytemuck::cast_slice(instances));
        data.count = required_capacity;
    }

    /// Render prepared texture instances for a specific texture ID
    ///
    /// prepare() must be called before this for the given texture_id
    pub fn render<'rp>(
        &self,
        render_pass: &mut wgpu::RenderPass<'rp>,
        texture_id: u64,
        texture_bind_group: &'rp wgpu::BindGroup,
    ) {
        let Some(data) = self.instance_data.get(&texture_id) else {
            return;
        };

        if data.count == 0 {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, data.buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..data.count);
    }

    /// Get the texture bind group layout for creating texture bind groups
    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_instance_layout() {
        // Verify TextureInstance memory layout matches GPU expectations
        assert_eq!(
            std::mem::size_of::<TextureInstance>(),
            32,
            "TextureInstance size should be 32 bytes (2*f32 + 2*f32 + f32 + 3*f32 padding)"
        );

        // Verify alignment
        assert_eq!(
            std::mem::align_of::<TextureInstance>(),
            4,
            "TextureInstance should be 4-byte aligned"
        );

        // Create instance and verify it can be safely cast to bytes
        let instances = [TextureInstance {
            position: [100.0, 200.0],
            size: [256.0, 512.0],
            opacity: 0.8,
            _padding: [0.0, 0.0, 0.0],
        }];

        let bytes: &[u8] = bytemuck::cast_slice(&instances);
        assert_eq!(bytes.len(), 32, "Byte representation should be 32 bytes");
    }

    #[test]
    fn test_texture_vertex_layout() {
        // Verify TextureVertex memory layout
        assert_eq!(
            std::mem::size_of::<TextureVertex>(),
            8,
            "TextureVertex size should be 8 bytes (2*f32)"
        );

        assert_eq!(
            std::mem::align_of::<TextureVertex>(),
            4,
            "TextureVertex should be 4-byte aligned"
        );
    }

    #[test]
    fn test_texture_uniforms_layout() {
        // Verify TextureUniforms memory layout
        assert_eq!(
            std::mem::size_of::<TextureUniforms>(),
            16,
            "TextureUniforms size should be 16 bytes (2*f32 + 2*f32 padding)"
        );

        assert_eq!(
            std::mem::align_of::<TextureUniforms>(),
            4,
            "TextureUniforms should be 4-byte aligned"
        );
    }

    #[test]
    fn test_texture_constants() {
        // Verify vertex and index data
        assert_eq!(TEXTURE_VERTICES.len(), 4, "Should have 4 vertices for quad");
        assert_eq!(
            TEXTURE_INDICES.len(),
            6,
            "Should have 6 indices for 2 triangles"
        );

        // Verify indices form two triangles
        assert_eq!(TEXTURE_INDICES, &[0, 1, 2, 0, 2, 3]);

        // Verify vertices form unit quad (0,0) to (1,1)
        assert_eq!(TEXTURE_VERTICES[0].position, [0.0, 0.0]);
        assert_eq!(TEXTURE_VERTICES[1].position, [1.0, 0.0]);
        assert_eq!(TEXTURE_VERTICES[2].position, [1.0, 1.0]);
        assert_eq!(TEXTURE_VERTICES[3].position, [0.0, 1.0]);
    }

    #[test]
    fn test_bytemuck_traits() {
        // Verify all structs implement Pod and Zeroable
        fn assert_pod_zeroable<T: bytemuck::Pod + bytemuck::Zeroable>() {}

        assert_pod_zeroable::<TextureInstance>();
        assert_pod_zeroable::<TextureVertex>();
        assert_pod_zeroable::<TextureUniforms>();
    }
}
