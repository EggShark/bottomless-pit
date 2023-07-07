use bottomless_pit::{*, engine_handle::EngineBuilder};
use vectors::Vec2;
use colour::Colour;

fn main() {
    let s = Unit;
    let mut engine = EngineBuilder::new()
        .set_window_title("Testing Triangle")
        .with_resolution((400, 400))
        .build()
        .unwrap();

    let shader = engine.create_shader("shader.wgsl").unwrap();

    engine.run(s);
}

struct Unit;

impl Game for Unit {
    fn render(&self, render_handle: &mut render::Renderer) {
        render_handle.draw_triangle_with_coloured_verticies(
            Vec2{x: 200.0, y: 0.0},
            Vec2{x: 400.0, y: 400.0},
            Vec2{x: 0.0, y: 400.0},
            Colour::Red,
            Colour::Green,
            Colour::Blue,
        )
    }

    fn update(&mut self, engine_handle: &mut engine_handle::Engine) {
        
    }
}
