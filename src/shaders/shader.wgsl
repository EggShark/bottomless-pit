struct CameraUniform {
    view_proj: mat3x3<f32>,
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
    
    // out.clip_position = vec4(camera.view_proj * vec3<f32>(model.position, 0.0), 1.0); // the vectors on the right the matrices go on the left in order of importance
    // var test_mat: mat3x3<f32> = mat3x3(
    //     0.5, 0.0, 0.5,
    //     0.0, 0.5, -0.5,
    //     0.0, 0.0, 1.0,
    // );
    
    var final_pos = vec3(model.position, 1.0) * camera.view_proj;
    final_pos = final_pos / final_pos.z;
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