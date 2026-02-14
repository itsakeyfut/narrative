// Quad shader for rendering UI rectangles

struct Uniforms {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct InstanceInput {
    @location(1) quad_position: vec2<f32>,
    @location(2) quad_size: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) corner_radius: f32,
    @location(5) border_width: f32,
    @location(6) border_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) corner_radius: f32,
    @location(4) border_width: f32,
    @location(5) border_color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate pixel position
    let pixel_pos = instance.quad_position + vertex.position * instance.quad_size;

    // Convert to clip space (-1 to 1)
    let clip_x = (pixel_pos.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (pixel_pos.y / uniforms.screen_size.y) * 2.0;

    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.color = instance.color;
    out.local_pos = vertex.position * instance.quad_size;
    out.size = instance.quad_size;
    out.corner_radius = instance.corner_radius;
    out.border_width = instance.border_width;
    out.border_color = instance.border_color;

    return out;
}

// Signed distance function for rounded rectangle
fn sd_rounded_rect(p: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let half_size = size * 0.5;
    let q = abs(p - half_size) - half_size + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let radius = min(in.corner_radius, min(in.size.x, in.size.y) * 0.5);

    // Calculate distance from edge
    let dist = sd_rounded_rect(in.local_pos, in.size, radius);

    // Anti-aliased edge
    let aa = 1.0;
    let alpha = 1.0 - smoothstep(-aa, aa, dist);

    if alpha < 0.01 {
        discard;
    }

    var final_color = in.color;

    // Border rendering
    if in.border_width > 0.0 {
        let inner_dist = sd_rounded_rect(in.local_pos, in.size - vec2<f32>(in.border_width * 2.0), max(0.0, radius - in.border_width));
        let border_alpha = smoothstep(-aa, aa, inner_dist);
        final_color = mix(in.color, in.border_color, border_alpha);
    }

    return vec4<f32>(final_color.rgb, final_color.a * alpha);
}
