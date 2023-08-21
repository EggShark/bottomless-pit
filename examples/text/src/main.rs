use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::colour::Colour;
use bottomless_pit::text::{TextRenderer, Text};
use bottomless_pit::input::MouseKey;
use bottomless_pit::vectors::Vec2;

fn main() {
    let engine = EngineBuilder::new()
        .set_clear_colour(Colour::BLACK)
        .build()
        .unwrap();

    let mut text_render = TextRenderer::new(&engine);
    let mut text = Text::new(Vec2{x: 0.0, y: 0.0}, 20.0, 30.0, &mut text_render, &engine);
    text.set_text("HELLO WORLD!", Colour::GREEN, &mut text_render);


    let text_example = TextExample {
        text_handle: text_render,
        text,
        red_or_green: true,
    };

    engine.run(text_example);
}

struct TextExample {
    text_handle: TextRenderer,
    text: Text,
    red_or_green: bool
}

impl Game for TextExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.text_handle.draw_text(&self.text, &mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        if engine_handle.is_mouse_key_pressed(MouseKey::Left) {
            self.red_or_green = !self.red_or_green;
            let colour = if self.red_or_green {
                Colour::RED
            } else {
                Colour::GREEN
            };

            self.text.set_text("HELLO WORLD!", colour, &mut self.text_handle);
        }
    }
}
