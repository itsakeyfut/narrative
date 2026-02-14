// Cross-dissolve transition shader (blend between two textures)

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Uniforms {
    projection: mat4x4<f32>,
}

struct TransitionParams {
    // Transition progress (0.0 = from_texture, 1.0 = to_texture)
    progress: f32,
    // Padding for alignment
    _padding0: f32,
    _padding1: f32,
    _padding2: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var from_texture: texture_2d<f32>;
@group(1) @binding(1) var from_sampler: sampler;
@group(1) @binding(2) var to_texture: texture_2d<f32>;
@group(1) @binding(3) var to_sampler: sampler;
@group(2) @binding(0) var<uniform> params: TransitionParams;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample both textures
    let from_color = textureSample(from_texture, from_sampler, in.tex_coords);
    let to_color = textureSample(to_texture, to_sampler, in.tex_coords);

    // Linear interpolation based on progress
    return mix(from_color, to_color, params.progress);
}
