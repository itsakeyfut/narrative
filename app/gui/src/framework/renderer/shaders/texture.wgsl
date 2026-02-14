// Texture shader for rendering images

struct Uniforms {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct InstanceInput {
    @location(1) tex_position: vec2<f32>,
    @location(2) tex_size: vec2<f32>,
    @location(3) opacity: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) opacity: f32,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate pixel position
    let pixel_pos = instance.tex_position + vertex.position * instance.tex_size;

    // Convert to clip space (-1 to 1)
    let clip_x = (pixel_pos.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (pixel_pos.y / uniforms.screen_size.y) * 2.0;

    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.tex_coords = vertex.position; // Use unit quad positions as tex coords
    out.opacity = instance.opacity;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, texture_sampler, in.tex_coords);
    return vec4<f32>(tex_color.rgb, tex_color.a * in.opacity);
}
