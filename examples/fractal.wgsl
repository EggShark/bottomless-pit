struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct Time {
    time: f32,
    aspect_ratio: f32,
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
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0); // the vectors on the right the matrices go on the left in order of importance
    out.colour = model.colour;
    return out;
}



fn palette(t: f32, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, d: vec3<f32>) -> vec3<f32> {
    return a + b * cos(6.28318*(c*t+d));
}

fn sdfBox(p: vec2<f32>, box: vec2<f32>) -> f32 {
    var d: vec2<f32> = abs(p)-box;
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0); 
}

fn sdCross(p: vec2f, b: vec2f) -> f32 {
  var q: vec2f = abs(p);
  q = select(q.xy, q.yx, q.y > q.x);
  let t = q - b;
  let k = max(t.y, t.x);
  let w = select(vec2f(b.y - q.x, -k), t, k > 0.);
  return sign(k) * length(max(w, vec2f(0.)));
}

fn sdPentagon(p: vec2f, r: f32) -> f32 {
  let k = vec3f(0.809016994, 0.587785252, 0.726542528);
  var q: vec2f = vec2f(abs(p.x), p.y);
  q = q - 2. * min(dot(vec2f(-k.x, k.y), q), 0.) * vec2f(-k.x, k.y);
  q = q - 2. * min(dot(vec2f(k.x, k.y), q), 0.) * vec2f(k.x, k.y);
  q = q - vec2f(clamp(q.x, -r * k.z, r * k.z), r);
  return length(q) * sign(q.y);
}

fn sdHexagon(p: vec2f, r: f32) -> f32 {
  let k = vec3f(-0.866025404, 0.5, 0.577350269);
  var q: vec2f = abs(p);
  q = q - 2. * min(dot(k.xy, q), 0.) * k.xy;
  q = q - vec2f(clamp(q.x, -k.z * r, k.z * r), r);
  return length(q) * sign(q.y);
}

fn sdHorseshoe(p: vec2f, sc: vec2f, r: f32, l: f32, w: f32) -> f32 {
  var q: vec2f = vec2f(abs(p.x), p.y);
  let m = length(p);
  q = q * mat2x2<f32>(vec2f(-sc.y, sc.x), vec2f(sc.x, sc.y));
  q = vec2f(select(m * sign(-sc.y), q.x, q.y > 0.0 || q.x > 0.), select(m, q.y, q.x > 0.));
  q = vec2f(q.x, abs(q.y - r)) - vec2f(l, w);
  return length(max(q, vec2f(0.))) + min(0., max(q.x, q.y));
}

fn sdHeart(p: vec2f) -> f32 {
  let q = vec2f(abs(p.x), p.y);
  let w = q - vec2f(0.25, 0.75);
  if (q.x + q.y > 1.0) { return sqrt(dot(w, w)) - sqrt(2.) / 4.; }
  let u = q - vec2f(0., 1.);
  let v = q - 0.5 * max(q.x + q.y, 0.);
  return sqrt(min(dot(u, u), dot(v, v))) * sign(q.x - q.y);
}

fn sdStar5(p: vec2f, r: f32, rf: f32) -> f32 {
  let k1 = vec2f(0.809016994375, -0.587785252292);
  let k2 = vec2f(-k1.x, k1.y);
  var q: vec2f = vec2f(abs(p.x), p.y);
  q = q - 2. * max(dot(k1, q), 0.) * k1;
  q = q - 2. * max(dot(k2, q), 0.) * k2;
  q.x = abs(q.x);
  q.y = q.y - r;
  let ba = rf * vec2f(-k1.y, k1.x) - vec2f(0., 1.);
  let h = clamp(dot(q, ba) / dot(ba, ba), 0., r);
  return length(q - ba * h) * sign(q.y * ba.x - q.x * ba.y);
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // flips so it mics gsls
    var uv: vec2<f32> = vec2(in.tex_coords.x, abs(in.tex_coords.y - 1.0));
    uv.x *= time.aspect_ratio; // prevents stretching
    uv = (uv - 0.5) * 2.0; // centers the UV so 0.0 is center of screen

    var uv0: vec2<f32> = uv;

    var final_colour: vec3<f32> = vec3(0.0); 

    for (var i: i32 = 0; i < 3; i++) {
        uv = fract(uv * 1.3) - 0.5; // the magic only gives the decimal part of inside
        var distance: f32 = (length(uv) + sdStar5(uv, 0.2, 0.3)/2.0 + sdfBox(uv, vec2(0.2, 0.2))) * exp(-length(uv0)); 
        //gets the length but remvoes uniformity with exp(-length(uv0))
        var colour: vec3<f32> = palette(
            length(uv0) + f32(i)*0.4 + time.time*0.2,
            vec3(1.028, 0.708, 0.218),
            vec3(0.468, 0.788, 0.468),
            vec3(0.628, 1.000, 1.000),
            vec3(3.118, 1.188, 1.458),
        ); // makes fancy colours
        // sin makes circles
        // abs just makes it real colours
        // 0.01 / adds the "neon" effect
        // pow increases contrast
        distance = pow(0.01 / abs(tan(distance * 7.0 + time.time)/7.0), 4.0);

        final_colour += colour * distance;
    }


    return vec4(final_colour, 1.0);
}