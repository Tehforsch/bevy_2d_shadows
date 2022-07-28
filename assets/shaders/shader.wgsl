struct Data {
    some_random_float: f32;
};

#import bevy_sprite::mesh2d_view_bind_group
[[group(0), binding(0)]]
var<uniform> view: View;
#import bevy_sprite::mesh2d_struct
[[group(1), binding(0)]]
var texture: texture_2d<f32>;
[[group(1), binding(1)]]
var texture_sampler: sampler;
[[group(1), binding(2)]]
var<uniform> data: Data;
[[group(2), binding(0)]]
var<uniform> mesh: Mesh2d;

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view.view_proj * mesh.model * vec4<f32>(vertex.position, 1.0);
    out.tex_coords = vertex.tex_coords;
    return out;
}

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color = textureSample(texture, texture_sampler, in.tex_coords);
    return color;
}
