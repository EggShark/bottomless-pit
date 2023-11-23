struct CameraUniform {
    view_proj: mat4x4<f32>,
}

// now why does this struct look like this?
// its becuase we need 16bit alignment on web
struct Time {
    time: f32,
    _junk: f32,
    _junk1: f32,
    _junk4: f32,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> time: Time;

struct VertexInput {
    @location(0) position: vec3<f32>,
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
    out.clip_position = camera.view_proj * vec4<f32>(
        model.position.x,
        model.position.y + sin(time.time + model.position.x * 12.0)/12.0,
        model.position.z,
        1.0
    ); // the vectors on the right the matrices go on the left in order of importance
    out.colour = model.colour;
    return out;
}


// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var r: f32 = (sin((in.clip_position.x + time.time * 16.0)/16.0) + 1.0) / 2.0;
    var b: f32 = (sin((in.clip_position.y + time.time * 16.0)/16.0) + 1.0) / 2.0;
    var g: f32 = (r + b) / 2.0;
    return vec4(r, g, b, 1.0);
}