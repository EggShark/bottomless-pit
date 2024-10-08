use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::input::MouseKey;
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderHandle;
use bottomless_pit::resource::LoadingOp;
use bottomless_pit::texture::Texture;
use bottomless_pit::vec2;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::Game;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_window_title("Testing Triangle")
        .with_resolution((400, 400))
        .build()
        .unwrap();

    let texture = Texture::new(&mut engine, "examples/bplogo.png", LoadingOp::Blocking);

    let texture = MaterialBuilder::new()
        .add_texture(texture)
        .build(&mut engine);
    let defualt = MaterialBuilder::new().build(&mut engine);

    let s = TextureExample {
        current: texture,
        other: defualt,
        pos: vec2! { 0.0 },
    };

    engine.run(s);
}

struct TextureExample {
    current: Material,
    other: Material,
    pos: Vec2<f32>,
}

impl Game for TextureExample {
    fn render<'o>(&'o mut self, mut render: RenderHandle<'o>) {
        let mut render_handle = render.begin_pass(Colour::BLACK);

        self.current.add_rectangle(
            vec2! { 0.0 },
            vec2! { 400.0 },
            Colour::WHITE,
            &render_handle,
        );
        self.current.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let dt = engine_handle.get_frame_delta_time();
        if engine_handle.is_mouse_key_pressed(MouseKey::Left) {
            std::mem::swap(&mut self.other, &mut self.current);
        }
        self.pos.x += 100.0 * dt;
    }
}
