struct CameraUniform {
    camera: mat3x3<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

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
    
    var final_pos = camera.camera * vec3(model.position, 1.0);
    final_pos = final_pos / final_pos.z;
    final_pos.x = 2.0 * final_pos.x / 600.0 - 1.0;
    final_pos.y = ((2.0 * final_pos.y / 600.0) - 1.0) * -1.0;
    out.clip_position = vec4(final_pos.xy, 0.0, 1.0);
    out.colour = model.colour;
    return out;
}


// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords) * in.colour;
}