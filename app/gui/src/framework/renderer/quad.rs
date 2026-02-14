//! Quad renderer for drawing rectangles

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// Instance data for a single quad
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct QuadInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
}

impl QuadInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
        1 => Float32x2,  // position
        2 => Float32x2,  // size
        3 => Float32x4,  // color
        4 => Float32,    // corner_radius
        5 => Float32,    // border_width
        6 => Float32x4,  // border_color
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Vertex data for the quad (just corner positions)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct QuadVertex {
    position: [f32; 2],
}

impl QuadVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Unit quad vertices (0,0) to (1,1)
const QUAD_VERTICES: &[QuadVertex] = &[
    QuadVertex {
        position: [0.0, 0.0],
    },
    QuadVertex {
        position: [1.0, 0.0],
    },
    QuadVertex {
        position: [1.0, 1.0],
    },
    QuadVertex {
        position: [0.0, 1.0],
    },
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

/// Uniform data for the quad shader
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct QuadUniforms {
    screen_size: [f32; 2],
    _padding: [f32; 2],
}

/// Renderer for quads (rectangles)
pub struct QuadRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    instance_buffer: Option<wgpu::Buffer>,
    /// Current capacity of instance buffer in number of instances
    instance_buffer_capacity: u32,
    instance_count: u32,
    /// Track if uniforms have been updated this frame
    uniforms_updated: bool,
}

impl QuadRenderer {
    /// Begin a new frame (call this before any prepare() calls)
    pub fn begin_frame(&mut self) {
        self.uniforms_updated = false;
    }

    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/quad.wgsl").into()),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Uniform Buffer"),
            contents: bytemuck::cast_slice(&[QuadUniforms {
                screen_size: [800.0, 600.0],
                _padding: [0.0, 0.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Quad Bind Group Layout"),
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

        // Create bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Quad Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Quad Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Quad Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[QuadVertex::desc(), QuadInstance::desc()],
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
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            instance_buffer: None,
            instance_buffer_capacity: 0,
            instance_count: 0,
            uniforms_updated: false,
        }
    }

    /// Prepare quad rendering
    ///
    /// This must be called BEFORE the render pass to avoid GPU sync issues
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[QuadInstance],
        screen_size: (u32, u32),
    ) {
        // Update uniform buffer only once per frame (shared across all quads)
        if !self.uniforms_updated {
            queue.write_buffer(
                &self.uniform_buffer,
                0,
                bytemuck::cast_slice(&[QuadUniforms {
                    screen_size: [screen_size.0 as f32, screen_size.1 as f32],
                    _padding: [0.0, 0.0],
                }]),
            );
            self.uniforms_updated = true;
        }

        if instances.is_empty() {
            self.instance_count = 0;
            return;
        }

        // Reuse or create instance buffer (buffer pooling for performance)
        // See Issue #250 for GPU optimization tracking
        let required_capacity = instances.len() as u32;
        if required_capacity > self.instance_buffer_capacity {
            // Need a larger buffer - allocate with some extra capacity to reduce reallocations
            let new_capacity = required_capacity.max(64).next_power_of_two();
            let buffer_size = (new_capacity as usize) * std::mem::size_of::<QuadInstance>();
            let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Quad Instance Buffer"),
                size: buffer_size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_buffer = Some(instance_buffer);
            self.instance_buffer_capacity = new_capacity;
        }

        // Write instance data to existing buffer
        if let Some(ref buffer) = self.instance_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
        }
        self.instance_count = required_capacity;
    }

    /// Render prepared quad instances
    ///
    /// prepare() must be called before this
    pub fn render<'rp>(&self, render_pass: &mut wgpu::RenderPass<'rp>) {
        if self.instance_count == 0 {
            return;
        }

        // Render (use if-let to satisfy the borrow checker and avoid unwrap)
        if let Some(ref instance_buffer) = self.instance_buffer {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..self.instance_count);
        }
    }
}
