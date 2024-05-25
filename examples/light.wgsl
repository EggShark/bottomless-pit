struct EngineUniforms {
    camera: mat3x3<f32>,
    screen_size: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> engine: EngineUniforms;

struct Light {
    colour: vec4<f32>,
    position: vec2<f32>,
    brightness: f32,
    aspect_ratio: f32,
}

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
    var pos: vec3<f32> = engine.camera * vec3<f32>(model.position.x, model.position.y, 1.0); // the vectors on the right the matrices go on the left in order of importance
    
    pos = pos / pos.z;
    pos.x = 2.0 * pos.x / engine.screen_size.x - 1.0;
    pos.y = ((2.0 * pos.y / engine.screen_size.y) - 1.0) * -1.0;
    out.clip_position = vec4(pos.xy, 0.0, 1.0);
    
    
    out.colour = model.colour;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var x: f32 = in.tex_coords.x - light.position.x;
    var y: f32 = in.tex_coords.y - light.position.y;

    x = x * light.aspect_ratio;

    var distance = sqrt(x * x + y * y);
    // TODO: Add blur
    var brightness: f32 = max(0.0, 0.7-distance);
    return textureSample(light_map, light_map_sampler, in.tex_coords) * (brightness * light.colour);
}