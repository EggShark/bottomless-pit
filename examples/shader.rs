use bottomless_pit::Game;
use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::material::{MaterialBuilder, Material};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::shader::{ShaderBuilder, UniformData};
use bottomless_pit::vectors::Vec2;
use encase::ShaderType;

fn main() {
    let mut engine = EngineBuilder::new()
        .build()
        .unwrap();

    let shader = ShaderBuilder::new(&engine, "examples/solidcolour.wgsl")
        .unwrap()
        .set_layouts(&[&engine.texture_layout(), &engine.camera_layout(), &engine.uniform_layout()])
        .register(&mut engine);

    let data = MousePos{x: 0.0, y: 0.0};
    let uniform_data = UniformData::new(&engine, &data);

    let material = MaterialBuilder::new()
        .set_shader(shader)
        .set_uniform(&uniform_data)
        .build(&mut engine);

    let game = ShaderExample {
        data,
        regular_material: material,
    };

    engine.run(game);
}

#[derive(ShaderType)]
struct MousePos {
    x: f32,
    y: f32,
}

struct ShaderExample {
    regular_material: Material,
    data: MousePos,
}

impl Game for ShaderExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.regular_material.add_rectangle(Vec2{x: 0.0, y: 0.0}, Vec2{x: 100.0, y: 100.0}, Colour::RED, &render_handle);

        self.regular_material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let size = engine_handle.get_window_size();
        let mouse_pos = engine_handle.get_mouse_position();
        let new_data = MousePos{
            x: mouse_pos.x/size.x as f32,
            y: mouse_pos.y/size.y as f32,
        };
        self.data = new_data;
        self.regular_material.update_uniform_data(&self.data, &engine_handle);
    }
}