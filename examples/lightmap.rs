use std::f32::consts::PI;
use std::cmp::Ordering;

use bottomless_pit::engine_handle::Engine;
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderHandle;
use bottomless_pit::shader::{Shader, ShaderOptions, UniformData, UniformError};
use bottomless_pit::{*, engine_handle::EngineBuilder};
use bottomless_pit::texture::UniformTexture;
use encase::ShaderType;
use vectors::Vec2;
use colour::Colour;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_window_title("Lightmap")
        .with_resolution((800, 800))
        .build()
        .unwrap();
    
    let uniform_texture = UniformTexture::new(&engine, engine.get_window_size());

    let light = Light {
        colour: Colour::ORANGE,
        pos_x: 0.0,
        pos_y: 0.0,
        brightness: 0.75,
        aspect_ratio: 1.0,
    };

    let light_data = UniformData::new(&light);

    let shader_options = ShaderOptions::with_all(&engine, &light_data, &uniform_texture);
    let light_shader = Shader::new("examples/light.wgsl", shader_options, &mut engine);

    let material = MaterialBuilder::new()
        .set_shader(light_shader)
        .build(&mut engine);

    let ocluder_material = MaterialBuilder::new()
        .build(&mut engine);

    let rectangles = vec![
        Rectangle::new(Vec2{x: 120.0, y: 20.0}, Vec2{x: 50.0, y: 50.0}),
        Rectangle::new(Vec2{x: 270.0, y: 70.0}, Vec2{x: 50.0, y: 50.0}),
        Rectangle::new(Vec2{x: 130.0, y: 280.0}, Vec2{x: 50.0, y: 50.0}),
        Rectangle::new(Vec2{x: 220.0, y: 300.0}, Vec2{x: 50.0, y: 50.0}),
        Rectangle::new(Vec2{x: 350.0, y: 350.0}, Vec2{x: 100.0, y: 100.0}),
    ];

    let s = TextureExample {
        material,
        ocluder_material,
        light,
        uniform_texture,
        rectangles,
        mouse_pos: ZEROS,
    };

    engine.run(s);
}

struct TextureExample {
    material: Material,
    ocluder_material: Material,
    light: Light,
    uniform_texture: UniformTexture,
    rectangles: Vec<Rectangle>,
    mouse_pos: Vec2<f32>,
}

const ZEROS: Vec2<f32> = Vec2{x: 0.0, y: 0.0};

impl Game for TextureExample {
    fn render<'o>(&'o mut self, mut render_handle: RenderHandle<'o>) {
        
        self.create_shadow_map(&mut render_handle);

        let mut p2 = render_handle.begin_pass(Colour::BLACK);
        let size = p2.get_size();
        let size = Vec2{x: size.x as f32, y: size.y as f32};

        self.material.add_rectangle(Vec2 {x: 0.0, y: 0.0}, size, Colour::WHITE, &p2);
        self.material.draw(&mut p2);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let mouse_pos = engine_handle.get_mouse_position();
        self.mouse_pos = mouse_pos;
        let window_size = engine_handle.get_window_size();

        self.light.pos_x = mouse_pos.x / window_size.x as f32;
        self.light.pos_y = mouse_pos.y / window_size.y as f32;
        self.material.update_uniform_data(&self.light, &engine_handle).unwrap();
    }

    fn on_resize(&mut self, new_size: Vec2<u32>, engine_handle: &mut Engine) {
        self.light.aspect_ratio = new_size.x as f32 / new_size.y as f32;
        match self.material.update_uniform_data(&self.light, &engine_handle) {
            Ok(_) => {},
            Err(e) => {
                match e {
                    UniformError::NotLoadedYet => {},
                    _ => panic!("{}", e),
                }
            } 
        } 

        match self.material.update_uniform_texture(&mut self.uniform_texture, new_size, engine_handle) {
            Ok(_) => {},
            Err(e) => {
                match e {
                    UniformError::NotLoadedYet => {},
                    _ => panic!("{}", e),
                }
            }
        }
    }
}

impl TextureExample {
    fn create_shadow_map<'o>(&mut self, render_handle: &mut RenderHandle<'o>) {
        let mut p1 = render_handle.begin_texture_pass(&mut self.uniform_texture, Colour::WHITE);

        let light_pos = self.mouse_pos;

        for rect in self.rectangles.iter() {
            for (segment_1, segment_2) in rect.create_segments() {
                let vert_1 = segment_1;
                let vert_2 = segment_1 + 
                    Vec2{
                        x: 300.0*(segment_1.x - light_pos.x),
                        y: 300.0*(segment_1.y - light_pos.y),
                    };
                let vert_3 = segment_2;
                let vert_4 = segment_2 + 
                    Vec2{
                        x: 300.0*(segment_2.x - light_pos.x),
                        y: 300.0*(segment_2.y - light_pos.y),
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

                self.ocluder_material.add_custom(arr, [ZEROS; 4], 0.0, Colour::BLACK, &p1);
            }
        }

        // makes sure there is not light in the squares
        self.rectangles
            .iter()
            .for_each(|rect| self.ocluder_material.add_rectangle(rect.pos, rect.size, Colour::BLACK, &mut p1));

        self.ocluder_material.draw(&mut p1);
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
    aspect_ratio: f32,
}

// Convex Hull Algo
fn compare_points(p1: &Vec2<f32>, p2: &Vec2<f32>) -> Ordering {
    let angle_one = get_angle(&ZEROS, p1);
    let angle_two = get_angle(&ZEROS, p2);
    if angle_one < angle_two {
        return Ordering::Less;
    }

    let d1 = get_distance(&ZEROS, p1);
    let d2 = get_distance(&ZEROS, p2);
    if (angle_one == angle_two) && (d1 < d2) {
        return Ordering::Less;
    }

    Ordering::Greater
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