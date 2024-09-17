use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::input::MouseKey;
use bottomless_pit::render::RenderHandle;
use bottomless_pit::resource::{LoadingOp, ResourceId};
use bottomless_pit::text::{Font, TextMaterial};
use bottomless_pit::vec2;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::Game;

fn main() {
    let mut engine = EngineBuilder::new().build().unwrap();

    let comic = Font::new("examples/Comic.ttf", &mut engine, LoadingOp::Blocking);
    let text_mat = TextMaterial::new("this is a test", Colour::RED, 0.5, 0.5 * 1.3);

    let text_example = TextExample {
        text_mat,
        comic,
        font_size: 0.5,
    };

    engine.run(text_example);
}

struct TextExample {
    text_mat: TextMaterial,
    comic: ResourceId<Font>,
    font_size: f32,
}

impl Game for TextExample {
    fn render<'o>(&'o mut self, mut render: RenderHandle<'o>) {
        let mut render_handle = render.begin_pass(Colour::BLACK);

        self.text_mat
            .add_instance(vec2! { 0.0 }, Colour::WHITE, &render_handle);
        self.text_mat.add_instance_with_rotation(
            Vec2 { x: 100.0, y: 0.0 },
            Colour::WHITE,
            45.0,
            &render_handle,
        );

        self.text_mat.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        self.font_size += 2.0 * engine_handle.get_frame_delta_time();

        self.text_mat.set_font_size(self.font_size, engine_handle);
        self.text_mat
            .set_line_height(self.font_size * 1.3, engine_handle);

        if engine_handle.is_mouse_key_pressed(MouseKey::Left) {
            self.text_mat.set_text_with_font(
                "hello I am talking to you here???",
                Colour::GREEN,
                &self.comic,
                engine_handle,
            );
        }

        self.text_mat.prepare(engine_handle);
    }
}
