struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct Light {
    colour: vec4<f32>,
    position: vec2<f32>,
    brightness: f32,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> light: Light;
@group(2) @binding(1)
var light_map: texture_2d<f32>;
@group(2) @binding(2)
var light_map_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) colour: vec4<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) colour: vec4<f32>,
}

// vertex shader
@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 0.0, 1.0); // the vectors on the right the matrices go on the left in order of importance
    out.colour = model.colour;
    return out;
}


// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var x: f32 = in.tex_coords.x - light.position.x;
    var y: f32 = in.tex_coords.y - light.position.y;
    var distance = sqrt(x * x + y * y);
    var brightness: f32 = max(0.0, 0.7-distance);
    return textureSample(light_map, light_map_sampler, in.tex_coords) * (brightness * light.colour);
}