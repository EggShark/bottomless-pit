// if you plan on targeting wasm, you should use web_time::Instant instead of
// std::time::Instant since the standard library one will panic if you try to
// use it
use web_time::Instant;

use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderHandle;
use bottomless_pit::text::TextMaterial;
use bottomless_pit::vec2;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::Game;

// changing window size will cause all these other constants to be correct
const WINDOW_SIZE: u32 = 600;
const SQUARE_SIZE: f32 = (WINDOW_SIZE as f32) / 10.0;
const SQUARE_INITIAL_POS: Vec2<f32> = Vec2 {
    x: (WINDOW_SIZE as f32) / 2.0 - SQUARE_SIZE / 2.0,
    y: (WINDOW_SIZE as f32) / 2.0 - SQUARE_SIZE / 2.0,
};
const PATH_RADIUS: f32 = (WINDOW_SIZE as f32) * 0.25;

fn main() {
    let mut engine = EngineBuilder::new()
        // change this to what your heart desires (or get rid of it entirely)
        .set_target_fps(15)
        .with_resolution((WINDOW_SIZE, WINDOW_SIZE))
        .build()
        .unwrap();

    let text_material = TextMaterial::new("", Colour::GREEN, 12.0, 12.0 * 1.3);

    let regular_material = MaterialBuilder::new().build(&mut engine);

    let pos = State {
        pos: SQUARE_INITIAL_POS,
        regular_material,
        text_material,
        start_time: Instant::now(),
    };

    engine.run(pos);
}

struct State {
    pos: Vec2<f32>,
    text_material: TextMaterial,
    regular_material: Material,
    start_time: Instant,
}

impl Game for State {
    fn render<'o>(&'o mut self, mut render: RenderHandle<'o>) {
        let mut render_handle = render.begin_pass(Colour::BLACK);

        self.regular_material.add_rectangle(
            self.pos,
            vec2! { SQUARE_SIZE },
            Colour::RED,
            &render_handle,
        );

        self.text_material
            .add_instance(vec2! { 0.0 }, Colour::WHITE, &render_handle);

        self.regular_material.draw(&mut render_handle);
        self.text_material.draw(&mut render_handle);
    }

    fn update(&mut self, engine: &mut Engine) {
        self.text_material.set_text(
            &format!(
                "FPS: {fps}\nStable FPS: {stable}",
                fps = engine.get_fps(),
                stable = engine.get_stable_fps()
            ),
            Colour::GREEN,
            engine,
        );

        self.text_material.prepare(engine);

        let secs = self.start_time.elapsed().as_secs_f32();

        self.pos.x = SQUARE_INITIAL_POS.x + secs.cos() * PATH_RADIUS;
        self.pos.y = SQUARE_INITIAL_POS.y + secs.sin() * PATH_RADIUS;
    }
}
