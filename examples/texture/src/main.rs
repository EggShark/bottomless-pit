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
    let texture = Texture::from_path(&engine, Some("texture"), "bplogo.png").unwrap();
    let texture = MaterialBuilder::new().add_texture(texture).build(&mut engine);
    let defualt = MaterialBuilder::new().build(&mut engine);
    
    let s = TextureExample {
        texture,
        regular: defualt,
        texture_switch: true,
        pos: Vec2{x: 0.0, y: 0.0}
    };

    engine.run(s);
}

struct TextureExample {
    texture: Material,
    regular: Material,
    texture_switch: bool,
    pos: Vec2<f32>,
}

impl Game for TextureExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        if self.texture_switch {
            self.texture.add_rectangle(Vec2{x: 0.0, y: 0.0}, Vec2{x: 400.0, y: 400.0}, Colour::WHITE, &render_handle);
            self.texture.draw(&mut render_handle);
        } else {
            self.regular.add_rectangle(Vec2{x: 0.0, y: 0.0}, Vec2{x: 400.0, y: 400.0}, Colour::WHITE, &render_handle);
            self.regular.draw(&mut render_handle);
        }
    }

    fn update(&mut self, engine_handle: &mut engine_handle::Engine) {
        let dt = engine_handle.get_frame_delta_time();
        println!("{}", dt);
        if engine_handle.is_mouse_key_pressed(MouseKey::Left) {
            self.texture_switch = !self.texture_switch;
        }
        self.pos.x += 100.0 * dt;
    }
}
