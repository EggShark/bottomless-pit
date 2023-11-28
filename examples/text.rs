use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::colour::Colour;
use bottomless_pit::resource::ResourceId;
use bottomless_pit::text::{TextMaterial, Font};
use bottomless_pit::input::MouseKey;
use bottomless_pit::vectors::Vec2;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_clear_colour(Colour::BLACK)
        .build()
        .unwrap();


    let comic = Font::new("examples/Comic.ttf", &mut engine);
    let text_mat = TextMaterial::new("AA", Colour::RED, 100.0, 100.0, &mut engine);


    let text_example = TextExample {
        text_mat,
        comic,
    };

    engine.run(text_example);
}

struct TextExample {
    text_mat: TextMaterial,
    comic: ResourceId<Font>,
}

impl Game for TextExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.text_mat.add_instance(Vec2{x: 0.0, y: 0.0}, Colour::WHITE, &render_handle);
        self.text_mat.add_instance_with_rotation(Vec2{x: 100.0, y: 0.0}, Colour::WHITE, 45.0, &render_handle);

        self.text_mat.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        if engine_handle.is_mouse_key_pressed(MouseKey::Left) {
            self.text_mat.set_text_with_font("hel", Colour::GREEN, &self.comic, engine_handle);
            self.text_mat.set_font_size(40.0, engine_handle);
            self.text_mat.prepare(engine_handle);
        }
    }
}
