#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;
@group(1) @binding(2)
var light_map: texture_2d<f32>;
@group(1) @binding(3)
var light_map_sampler: sampler;
@group(2) @binding(0)
var<uniform> mesh: Mesh2d;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view.view_proj * mesh.model * vec4<f32>(vertex.position, 1.0);
    out.tex_coords = vertex.tex_coords;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(texture, texture_sampler, in.tex_coords);

    let lightmap_dimensions_int = textureDimensions(light_map);
    let lightmap_dimensions = vec2<f32>(f32(lightmap_dimensions_int.x), f32(lightmap_dimensions_int.y));
    let uv = in.clip_position.xy / lightmap_dimensions;

    let light = textureSample(light_map, light_map_sampler, uv);

    return color * light;
}
