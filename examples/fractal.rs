use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::shader::Shader;
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::colour::Colour;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::shader::UniformData;
use encase::ShaderType;

fn main() {
    let mut engine = EngineBuilder::new()
        .with_resolution((500, 500))
        .remove_vsync()
        .set_clear_colour(Colour::BLACK)
        .build()
        .unwrap();

    let uniform_data = UniformData::new(&engine, &Uniforms { time: 0.0, aspect: 0.0});

    let mouse_shader = Shader::new_with_uniforms("examples/fractal.wgsl", &uniform_data, &mut engine)
        .unwrap();

    let regular_material = MaterialBuilder::new()
        .set_uniform(&uniform_data)
        .set_shader(mouse_shader)
        .build(&mut engine);

    let pos = Position {
        regular_material,
        uniform_data,
        uniform: Uniforms { time: 0.0, aspect: 0.0}
    };

    engine.run(pos);
}

struct Position {
    regular_material: Material,
    uniform_data: UniformData,
    uniform: Uniforms,
}

impl Game for Position {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        let size = render_handle.get_size();
        let size = Vec2{x: size.x as f32, y: size.y as f32};

        self.regular_material.add_rectangle(Vec2{x: 0.0, y: 0.0}, size, Colour::WHITE, &render_handle);

        self.regular_material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let dt = engine_handle.get_frame_delta_time();
        self.uniform.time += dt;
        self.uniform_data.update_uniform_data(&self.uniform, &engine_handle);
    }

    fn on_resize(&mut self, new_size: Vec2<u32>, engine: &mut Engine) {
        self.uniform.aspect = new_size.x as f32 / new_size.y as f32;
        self.uniform_data.update_uniform_data(&self.uniform, &engine);
    }
}

#[derive(ShaderType)]
struct Uniforms {
    time: f32,
    aspect: f32,
}