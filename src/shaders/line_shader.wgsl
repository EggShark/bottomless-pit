struct EngineUniforms {
    camera: mat3x3<f32>,
    screen_size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> engine: EngineUniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) colour: vec4<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) colour: vec4<f32>,
}

// vectex shader
@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos: vec3<f32> = engine.camera * vec3<f32>(model.position, 1.0); // the vectors on the right the matrices go on the left in order of importance
    
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
    return in.colour;
}