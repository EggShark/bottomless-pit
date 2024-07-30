use std::f32::consts::PI;

use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderHandle;
use bottomless_pit::resource::LoadingOp;
use bottomless_pit::shader::{Shader, ShaderOptions, UniformData};
use bottomless_pit::vectors::Vec2;
use bottomless_pit::Game;
use bottomless_pit::vec2;

use encase::ShaderType;

fn main() {
    let mut engine = EngineBuilder::new()
        .build()
        .unwrap();

    let data = MousePos {
        x: 0.0,
        y: 0.0,
        _junk: 0.0,
        _padding2: 0.0,
    };
    
    let mouse_uniform_data = UniformData::new(&data);
    let mouse_shader = Shader::new("examples/mouse.wgsl", ShaderOptions::with_uniform_data(&mouse_uniform_data), &mut engine, LoadingOp::Blocking);
    
    // On wasm we need this to be 16 bytes aligned so we have added this instead of
    // a 0.0_f32
    let circle_uniform_data = UniformData::new(&data);
    let circle_shader = Shader::new("examples/movement.wgsl", ShaderOptions::with_uniform_data(&circle_uniform_data), &mut engine, LoadingOp::Blocking);
    
    let mouse_material = MaterialBuilder::new()
        .set_shader(mouse_shader)
        .build(&mut engine);

    let circle_material = MaterialBuilder::new()
        .set_shader(circle_shader)
        .build(&mut engine);

    let defualt_material = MaterialBuilder::new().build(&mut engine);

    let game = ShaderExample {
        data,
        mouse_material,
        circle_material,
        defualt_material,
        theta: data,
    };

    engine.run(game);
}

#[derive(ShaderType, Clone, Copy)]
struct MousePos {
    x: f32,
    y: f32,
    _junk: f32,
    _padding2: f32,
}

struct ShaderExample {
    mouse_material: Material<MousePos>,
    circle_material: Material<MousePos>,
    defualt_material: Material<()>,
    data: MousePos,
    theta: MousePos,
}

impl Game for ShaderExample {
    fn render<'o>(
        &'o mut self,
        mut render: RenderHandle<'o>,
    ) {
        let mut render_handle = render.begin_pass(Colour::BLACK);

        self.mouse_material.add_rectangle(
            vec2! { 0.0 },
            vec2! { 100.0 },
            Colour::RED,
            &render_handle,
        );
        self.circle_material.add_rectangle(
            vec2! { 100.0 },
            vec2! { 100.0 },
            Colour::RED,
            &render_handle,
        );
        self.defualt_material.add_rectangle(
            Vec2 { x: 0.0, y: 200.0 },
            vec2! { 100.0 },
            Colour::RED,
            &render_handle,
        );

        self.mouse_material.draw(&mut render_handle);
        self.circle_material.draw(&mut render_handle);
        self.defualt_material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let dt = engine_handle.get_frame_delta_time();

        self.theta.x = (self.theta.x + dt) % (2.0 * PI);

        let size = engine_handle.get_window_size();
        let mouse_pos = engine_handle.get_mouse_position();

        let new_data = MousePos {
            x: mouse_pos.x / size.x as f32,
            y: mouse_pos.y / size.y as f32,
            _junk: 0.0,
            _padding2: 0.0,
        };

        self.data = new_data;
        self.mouse_material
            .update_uniform_data(&self.data, &engine_handle)
            .unwrap();
        self.circle_material
            .update_uniform_data(&self.theta, &engine_handle)
            .unwrap();
    }
}
