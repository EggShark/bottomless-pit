use std::f32::consts::PI;

use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::shader::Shader;
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::colour::Colour;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::shader::UniformData;

fn main() {
    let mut engine = EngineBuilder::new()
        .with_resolution((500, 500))
        .remove_vsync()
        .set_clear_colour(Colour::BLACK)
        .build()
        .unwrap();

    let uniform_data = UniformData::new(&engine, &0.0_f32);

    let mouse_shader = Shader::new("examples/sinewaves.wgsl", true, &mut engine)
        .unwrap();

    let regular_material = MaterialBuilder::new()
        .set_uniform(&uniform_data)
        .set_shader(mouse_shader)
        .build(&mut engine);

    let pos = Position {
        regular_material,
        time: 0.0
    };

    engine.run(pos);
}



struct Position {
    regular_material: Material,
    time: f32,
}

impl Game for Position {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.regular_material.add_regular_n_gon(120, 200.0, Vec2{x: 250.0, y: 250.0}, Colour::BLUE, &render_handle);

        self.regular_material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        self.time = (self.time + engine_handle.get_frame_delta_time()) % (2.0*PI);
        self.regular_material.update_uniform_data(&self.time, &engine_handle);
    }
}
