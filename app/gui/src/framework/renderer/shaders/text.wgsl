// Text shader for rendering glyphs from texture atlas

struct Uniforms {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var glyph_texture: texture_2d<f32>;

@group(0) @binding(2)
var glyph_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Convert pixel position to clip space (-1 to 1)
    let clip_x = (input.position.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (input.position.y / uniforms.screen_size.y) * 2.0;

    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.uv = input.uv;
    out.color = input.color;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the glyph texture (grayscale mask)
    let mask = textureSample(glyph_texture, glyph_sampler, input.uv).r;

    // Apply color with mask as alpha
    return vec4<f32>(input.color.rgb, input.color.a * mask);
}
