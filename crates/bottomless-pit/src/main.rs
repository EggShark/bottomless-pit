use state::{Engine, EngineBuilder, Colour, Renderer, Vec2, Key, MouseKey, TextureIndex};

fn main() {
    let mut engine = EngineBuilder::new()
        .set_clear_colour(Colour::White)
        .set_close_key(Key::Q)
        .build()
        .unwrap();

    let s = TestStruct::new(&mut engine);
    engine.run(s);
}

struct TestStruct {
    texture: TextureIndex
}

impl TestStruct {
    pub fn new(engine_handle: &mut Engine) -> Self {
        let texture = engine_handle.create_texture("crates/bottomless-pit/assets/idle.png").unwrap();
        Self {
            texture
        }
    }
}

impl state::Game for TestStruct {
    fn render(&self, render_handle: &mut Renderer) {
        render_handle.draw_line(Vec2{x: 0.0, y: 0.0}, Vec2{x: 200.0, y: 200.0}, Colour::Black);
        render_handle.draw_rectangle(Vec2{x: 0.0, y: 0.0}, 100.0, 200.0, Colour::Purple);
        render_handle.draw_text("hello world 0", Vec2{x: 0.0, y: 0.0}, 40.0, Colour::Black);
        render_handle.draw_textured_rectangle(Vec2{x: 400.0, y: 400.0}, 400.0, 600.0, &self.texture);
        render_handle.draw_textured_rectangle_with_uv(Vec2{x: 100.0, y: 100.0}, 500.0, 500.0, &self.texture, Vec2{x: 0.0, y: 0.0}, 500.0, 500.0);
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