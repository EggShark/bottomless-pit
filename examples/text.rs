use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::colour::Colour;
use bottomless_pit::text::{TextRenderer, Text, TextMaterial};
use bottomless_pit::input::MouseKey;
use bottomless_pit::vectors::Vec2;

fn main() {
    let engine = EngineBuilder::new()
        .set_clear_colour(Colour::BLACK)
        .build()
        .unwrap();

    let mut text_render = TextRenderer::new(&engine);

    let text_mat = TextMaterial::new(Vec2{x: 0.0, y: 0.0}, "A", Colour::RED, 10.0, 10.0, &mut text_render, &engine);


    let text_example = TextExample {
        text_handle: text_render,
        red_or_green: true,
    };

    engine.run(text_example);
}

struct TextExample {
    text_handle: TextRenderer,
    red_or_green: bool
}

impl Game for TextExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {

    }

    fn update(&mut self, engine_handle: &mut Engine) {

    }
}
