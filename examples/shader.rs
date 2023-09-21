use std::f32::consts::PI;

use bottomless_pit::Game;
use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::material::{MaterialBuilder, Material};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::shader::{Shader, UniformData};
use bottomless_pit::vectors::Vec2;
use encase::ShaderType;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_clear_colour(Colour::WHITE)
        .build()
        .unwrap();

    let mouse_shader = Shader::new("examples/mouse.wgsl", true, &mut engine)
        .unwrap();

    let circle_shader = Shader::new("examples/movement.wgsl", true, &mut engine)
        .unwrap();

    let data = MousePos{x: 0.0, y: 0.0};
    let mouse_uniform_data = UniformData::new(&engine, &data);
    let circle_uniform_data = UniformData::new(&engine, &0.0_f32);

    let mouse_material = MaterialBuilder::new()
        .set_shader(mouse_shader)
        .set_uniform(&mouse_uniform_data)
        .build(&mut engine);

    let circle_material = MaterialBuilder::new()
        .set_shader(circle_shader)
        .set_uniform(&circle_uniform_data)
        .build(&mut engine);

    let game = ShaderExample {
        data,
        mouse_material,
        circle_material,
        theta: 0.0,
    };

    engine.run(game);
}

#[derive(ShaderType)]
struct MousePos {
    x: f32,
    y: f32,
}

struct ShaderExample {
    mouse_material: Material,
    circle_material: Material,
    data: MousePos,
    theta: f32,
}

impl Game for ShaderExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.mouse_material.add_rectangle(Vec2{x: 0.0, y: 0.0}, Vec2{x: 100.0, y: 100.0}, Colour::RED, &render_handle);
        self.circle_material.add_rectangle(Vec2{x: 100.0, y: 100.0}, Vec2{x: 100.0, y: 100.0}, Colour::RED, &render_handle);


        self.mouse_material.draw(&mut render_handle);
        self.circle_material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let dt = engine_handle.get_frame_delta_time();

        self.theta = (self.theta + dt) % (2.0*PI);

        let size = engine_handle.get_window_size();
        let mouse_pos = engine_handle.get_mouse_position();
        let new_data = MousePos{
            x: mouse_pos.x/size.x as f32,
            y: mouse_pos.y/size.y as f32,
        };
        self.data = new_data;
        self.mouse_material.update_uniform_data(&self.data, &engine_handle);
        self.circle_material.update_uniform_data(&self.theta, &engine_handle);
    }
}