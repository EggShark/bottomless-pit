use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::{*, engine_handle::EngineBuilder};
use bottomless_pit::texture::Texture;
use bottomless_pit::input::MouseKey;
use vectors::Vec2;
use colour::Colour;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_window_title("Testing Triangle")
        .with_resolution((400, 400))
        .build()
        .unwrap();
    let texture = Texture::from_path(&engine, Some("texture"), "../bplogo.png").unwrap();
    let texture = MaterialBuilder::new().add_texture(texture).build(&mut engine);
    let defualt = MaterialBuilder::new().build(&mut engine);
    
    let s = TextureExample {
        current: texture,
        other: defualt,
        pos: Vec2{x: 0.0, y: 0.0}
    };

    engine.run(s);
}

struct TextureExample {
    current: Material,
    other: Material,
    pos: Vec2<f32>,
}

impl Game for TextureExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
            self.current.add_rectangle(Vec2{x: 0.0, y: 0.0}, Vec2{x: 400.0, y: 400.0}, Colour::WHITE, &render_handle);
            self.current.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut engine_handle::Engine) {
        let dt = engine_handle.get_frame_delta_time();
        println!("{}", dt);
        if engine_handle.is_mouse_key_pressed(MouseKey::Left) {
            std::mem::swap(&mut self.other, &mut self.current);
        }
        self.pos.x += 100.0 * dt;
    }
}
