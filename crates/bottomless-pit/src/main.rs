use state::{Engine, EngineBuilder, Colour, Renderer, Vec2, Key, MouseKey};

fn main() {
    let s = TestUnit;
    EngineBuilder::new()
        .set_clear_colour(Colour::Blue)
        .set_close_key(Key::Q)
        .build()
        .unwrap()
        .run(Box::new(s));
}

struct TestUnit;

impl state::Game for TestUnit {
    fn render(&self, render_handle: &mut Renderer) {
        render_handle.draw_line(Vec2{x: 0.0, y: 0.0}, Vec2{x: 200.0, y: 200.0}, Colour::Black);
        render_handle.draw_rectangle(Vec2{x: 0.0, y: 0.0}, 100.0, 200.0, Colour::Purple);
        render_handle.draw_text("hello world 0", Vec2{x: 0.0, y: 0.0}, 40.0, Colour::White);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        if engine_handle.is_key_released(Key::A) {
            println!("input");
        }

        if engine_handle.is_mouse_key_released(MouseKey::Middle) {
            println!("no cliking :(")
        }

        //println!("{:?}", engine_handle.get_mouse_position());
    }
}