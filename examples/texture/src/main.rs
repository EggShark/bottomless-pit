use bottomless_pit::{*, engine_handle::EngineBuilder};
use bottomless_pit::texture::TextureIndex;
use bottomless_pit::input::MouseKey;
use vectors::Vec2;
use colour::Colour;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_window_title("Testing Triangle")
        .with_resolution((400, 400))
        .build()
        .unwrap();
    let texture = engine.create_texture("bplogo.png").unwrap();
    let s = Unit(texture, true);
    engine.run(s);
}

struct Unit(TextureIndex, bool);

impl Game for Unit {
    fn render(&self, render_handle: &mut render::Renderer) {
        if self.1 {
            render_handle.draw_textured_rectangle(Vec2{x: 0.0, y: 0.0}, 400.0, 400.0, &self.0);
        } else {
            render_handle.draw_rectangle(Vec2{x: 0.0, y: 0.0}, 400.0, 400.0, Colour::Red);
        }
    }

    fn update(&mut self, engine_handle: &mut engine_handle::Engine) {
        println!("{}", engine_handle.get_frame_delta_time());
        if engine_handle.is_mouse_key_pressed(MouseKey::Left) {
            self.1 = !self.1;
        }
    }
}
