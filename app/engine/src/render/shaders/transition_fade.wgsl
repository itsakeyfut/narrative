// Fade transition shader (fade to/from solid color)

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
    // Transition progress (0.0 = fully visible, 1.0 = fully faded)
    progress: f32,
    // Fade color (RGB)
    fade_color: vec3<f32>,
    // Padding for alignment
    _padding: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var<uniform> params: TransitionParams;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fade effect: blend between transparent and fade color based on progress
    let alpha = params.progress;
    return vec4<f32>(params.fade_color, alpha);
}
