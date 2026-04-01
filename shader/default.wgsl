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
    out.screen_position = VERTEX_POSITIONS[in_vertex_index] * vec2<f32>(1.0, -1.0);
    return out;
}


// Fragment shader

struct Uniforms {
    screen_size: vec2<f32>,
    target_aspect: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var screen_texture: texture_2d<f32>;
@group(0) @binding(2)
var screen_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_aspect = uniforms.screen_size.x / uniforms.screen_size.y;

    var screen_position = in.screen_position;
    var mask = 1.0;
    
    if screen_aspect < uniforms.target_aspect {
        // letterbox
        let scale_factor = screen_aspect / uniforms.target_aspect;
        screen_position.y = screen_position.y / scale_factor;
        if (screen_position.y < -1.0 || screen_position.y > 1.0) {
            mask = 0.0;
        }
    } else if screen_aspect > uniforms.target_aspect {
        // pillarbox
        let scale_factor = uniforms.target_aspect / screen_aspect;
        screen_position.x = screen_position.x / scale_factor;
        if (screen_position.x < -1.0 || screen_position.x > 1.0) {
            mask = 0.0;
        }
    }

    screen_position = (screen_position + 1.0) * 0.5;

    return textureSample(screen_texture, screen_sampler, screen_position) * mask;
}