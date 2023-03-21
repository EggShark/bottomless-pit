use state::{Engine, EngineBuilder, Colour, Renderer, Vec2};

fn main() {
    let s = TestUnit;
    let engine = EngineBuilder::new()
        .set_clear_colour(Colour::Blue)
        .fullscreen()
        .build()
        .unwrap()
        .run(Box::new(s));
}

struct TestUnit;

impl state::Game for TestUnit {
    fn render(&self, render_handle: &mut Renderer) {
        render_handle.draw_line(Vec2{x: 0.0, y: 1.0}, Vec2{x: 1.0, y: 0.0}, Colour::Black);
        render_handle.draw_rectangle(Vec2{x: -1.0, y: 0.0}, 1.0, 1.0, Colour::Purple);
        println!("rendering_stuff");
    }
    fn update(&mut self, engine_handle: &mut Engine) {
        println!("doing game calculations")
    }
}