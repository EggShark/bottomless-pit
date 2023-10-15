use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::colour::Colour;
use bottomless_pit::text::{TextRenderer, TextMaterial, Font};
use bottomless_pit::input::MouseKey;
use bottomless_pit::vectors::Vec2;

fn main() {
    let engine = EngineBuilder::new()
        .set_clear_colour(Colour::BLACK)
        .build()
        .unwrap();

    let mut text_render = TextRenderer::new(&engine);
    let comic = text_render.load_font_from_bytes(include_bytes!("Comic.ttf").to_vec());

    let text_mat = TextMaterial::new("AA", Colour::RED, 100.0, 100.0, &mut text_render, &engine);


    let text_example = TextExample {
        text_handle: text_render,
        text_mat,
        comic,
    };

    engine.run(text_example);
}

struct TextExample {
    text_handle: TextRenderer,
    text_mat: TextMaterial,
    comic: Font,
}

impl Game for TextExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.text_mat.add_instance(Vec2{x: 0.0, y: 0.0}, Colour::WHITE, &render_handle);
        self.text_mat.add_instance_with_rotation(Vec2{x: 100.0, y: 0.0}, Colour::WHITE, 45.0, &render_handle);

        self.text_mat.draw(&mut self.text_handle, &mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        if engine_handle.is_mouse_key_pressed(MouseKey::Left) {
            self.text_mat.set_text_with_font("hel", Colour::GREEN, &self.comic, &mut self.text_handle, &engine_handle);
            self.text_mat.set_font_size(40.0, &mut self.text_handle, &engine_handle);
        }
    }
}
