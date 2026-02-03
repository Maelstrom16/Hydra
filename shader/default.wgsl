// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) screen_position: vec2<f32>,
};

const VERTEX_POSITIONS = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0), // bottom-left
    vec2<f32>( 3.0, -1.0), // bottom-right
    vec2<f32>(-1.0,  3.0), // top-left
);

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(VERTEX_POSITIONS[in_vertex_index], 0.0, 1.0);
    out.screen_position = vec2<f32>((VERTEX_POSITIONS[in_vertex_index]+1.0)*0.5);
    out.screen_position.y = 1 - out.screen_position.y;
    return out;
}

// Fragment shader


@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var screen_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(screen_texture, screen_sampler, in.screen_position);
}