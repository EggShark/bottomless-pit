use std::f32::consts::PI;

use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::{TexturePass, RenderInformation};
use bottomless_pit::shader::{UniformData, Shader};
use bottomless_pit::{*, engine_handle::EngineBuilder};
use bottomless_pit::texture::UniformTexture;
use encase::ShaderType;
use vectors::Vec2;
use colour::Colour;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_window_title("Lightmap")
        .with_resolution((400, 400))
        .set_clear_colour(Colour::BLACK)
        .build()
        .unwrap();
    
    let uniform_texture = UniformTexture::new(&engine, engine.get_window_size());

    let light = Light {
        colour: Colour::ORANGE,
        pos_x: 0.0,
        pos_y: 0.0,
        brightness: 0.75,
        _pad: 0.0,
    };

    let uniform_data = UniformData::new_with_extra_texture(&light, &uniform_texture, &engine);
    let shader = Shader::new_with_uniforms("examples/lightmap.wgsl", &uniform_data, &mut engine).unwrap();

    let material = MaterialBuilder::new()
        .set_uniform(&uniform_data)
        .set_shader(shader)
        .build(&mut engine);

    let ocluder_material = MaterialBuilder::new().build(&mut engine);

    let rectangles = vec![
        Rectangle::new(Vec2{x: 120.0, y: 20.0}, Vec2{x: 50.0, y: 50.0}),
        Rectangle::new(Vec2{x: 270.0, y: 70.0}, Vec2{x: 50.0, y: 50.0}),
        Rectangle::new(Vec2{x: 130.0, y: 280.0}, Vec2{x: 50.0, y: 50.0}),
        Rectangle::new(Vec2{x: 220.0, y: 300.0}, Vec2{x: 50.0, y: 50.0}),
    ];

    let s = TextureExample {
        material,
        ocluder_material,
        uniform_data,
        light,
        uniform_texture,
        rectangles,
    };

    engine.run(s);
}

struct TextureExample {
    material: Material,
    ocluder_material: Material,
    uniform_data: UniformData,
    light: Light,
    uniform_texture: UniformTexture,
    rectangles: Vec<Rectangle>,
}

const ZEROS: Vec2<f32> = Vec2{x: 0.0, y: 0.0};

impl TextureExample {
    fn create_shadow_map(&mut self, engine_handle: &mut engine_handle::Engine) {
        let light_pos = engine_handle.get_mouse_position();
        let mut texture_pass = TexturePass::new(engine_handle);
        let view = self.uniform_texture.get_view();
        let size = self.uniform_texture.get_size();
        let mut pain = texture_pass.begin_pass(engine_handle, &view, size, Colour::WHITE);

        for rect in self.rectangles.iter() {
            for (segment_1, segment_2) in rect.create_segments() {
                let vert_1 = segment_1;
                let vert_2 = segment_1 + 
                    Vec2{
                        x: 30.0*(segment_1.x - light_pos.x),
                        y: 30.0*(segment_1.y - light_pos.y),
                    };
                let vert_3 = segment_2;
                let vert_4 = segment_2 + 
                    Vec2{
                        x: 30.0*(segment_2.x - light_pos.x),
                        y: 30.0*(segment_2.y - light_pos.y),
                    };

                let mut arr = [vert_1, vert_2, vert_3, vert_4];

                let center_point = Vec2{
                    x: (vert_1.x + vert_2.x + vert_3.x + vert_4.x)/4.0,
                    y: (vert_1.y + vert_2.y + vert_3.y + vert_4.y)/4.0,
                };

                for point in arr.iter_mut() {
                    *point = *point-center_point;
                }

                arr.sort_by(|left, right| compare_points(left, right));
                for point in arr.iter_mut() {
                    *point = *point+center_point;
                }

                self.ocluder_material.add_custom(arr, [ZEROS; 4], 0.0, Colour::BLACK, &pain);
            }
        }

        self.ocluder_material.draw(&mut pain);

        drop(pain);

        texture_pass.finish_pass(&engine_handle);
    }
}

fn get_angle(center_point: &Vec2<f32>, point: &Vec2<f32>) -> f32 {
    let x = point.x - center_point.x;
    let y = point.y - center_point.y;
    let mut angle = f32::atan2(y, x);
    if angle <= 0.0 {
        angle += 2.0*PI;
    }

    angle
}

fn get_distance(p1: &Vec2<f32>, p2: &Vec2<f32>) -> f32 {
    let x = p1.x - p2.x;
    let y = p1.y - p2.y;
    (x*x + y*y).sqrt()
}

// Convex Hull Algo
fn compare_points(p1: &Vec2<f32>, p2: &Vec2<f32>) -> std::cmp::Ordering {
    let angle_one = get_angle(&ZEROS, p1);
    let angle_two = get_angle(&ZEROS, p2);
    if angle_one < angle_two {
        return std::cmp::Ordering::Less;
    }
    let d1 = get_distance(&ZEROS, p1);
    let d2 = get_distance(&ZEROS, p2);
    if (angle_one == angle_two) && (d1 < d2) {
        return std::cmp::Ordering::Less;
    }

    std::cmp::Ordering::Greater
}

impl Game for TextureExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.material.add_rectangle(Vec2 {x: 0.0, y: 0.0}, Vec2{x: 400.0, y: 400.0}, Colour::WHITE, &render_handle);
        self.material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut engine_handle::Engine) {
        let mouse_pos = engine_handle.get_mouse_position();
        let window_size = engine_handle.get_window_size();

        self.light.pos_x = mouse_pos.x / window_size.x as f32;
        self.light.pos_y = mouse_pos.y / window_size.y as f32;
        self.uniform_data.update_uniform_data(&self.light, &engine_handle);

        self.create_shadow_map(engine_handle);
    }
}

struct Rectangle {
    pos: Vec2<f32>,
    size: Vec2<f32>,
}

impl Rectangle {
    fn new(pos: Vec2<f32>, size: Vec2<f32>) -> Self {
        Self {
            pos,
            size
        }
    }

    fn create_segments(&self) -> [(Vec2<f32>, Vec2<f32>); 4] {
        let p1 = self.pos;
        let p2 = Vec2{x: self.pos.x + self.size.x, y: self.pos.y};
        let p3 = Vec2{x: self.pos.x + self.size.x, y: self.pos.y + self.size.y};
        let p4 = Vec2{x: self.pos.x, y: self.size.y + self.pos.y};
        [(p1, p2), (p2, p3), (p3, p4), (p4, p1)]
    }
}

#[derive(ShaderType)]
struct Light {
    colour: Colour,
    pos_x: f32,
    pos_y: f32,
    brightness: f32,
    _pad: f32,
}