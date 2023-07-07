use bottomless_pit::{*, engine_handle::EngineBuilder, shader::ShaderIndex};
use vectors::Vec2;
use colour::Colour;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_window_title("Testing Triangle")
        .with_resolution((400, 400))
        .build()
        .unwrap();

    let shader = engine.create_shader("shader.wgsl").unwrap();

    let s = Unit(shader);

    engine.run(s);
}

struct Unit(ShaderIndex);

impl Game for Unit {
    fn render(&self, render_handle: &mut render::Renderer) {
        render_handle.set_shader(&self.0);

        render_handle.draw_triangle_with_coloured_verticies(
            Vec2{x: 100.0, y: 0.0},
            Vec2{x: 200.0, y: 200.0},
            Vec2{x: 0.0, y: 200.0},
            Colour::Red,
            Colour::Green,
            Colour::Blue,
        );

        render_handle.set_to_defualt_shader();

        render_handle.draw_triangle_with_coloured_verticies(
            Vec2{x: 300.0, y: 200.0},
            Vec2{x: 400.0, y: 400.0},
            Vec2{x: 200.0, y: 400.0},
            Colour::Red,
            Colour::Green,
            Colour::Blue,
        );
    }

    fn update(&mut self, engine_handle: &mut engine_handle::Engine) {
        
    }
}
