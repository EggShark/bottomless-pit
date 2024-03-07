use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::material::LineMaterial;
use bottomless_pit::render::RenderInformation;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::Game;
use bottomless_pit::vec2;

fn main() {
    let engine = EngineBuilder::new().build().unwrap();

    let line_material = LineMaterial::new(&engine);

    let game = LineExample {
        material: line_material,
    };

    engine.run(game);
}

struct LineExample {
    material: LineMaterial,
}

impl Game for LineExample {
    fn render<'pass, 'others>(
        &'others mut self,
        mut render_handle: RenderInformation<'pass, 'others>,
    ) where
        'others: 'pass,
    {
        self.material.add_line(
            vec2! { 0.0, 0.0 },
            vec2! { 100.0, 100.0 },
            Colour::WHITE,
            &render_handle,
        );

        self.material.add_screenspace_line(vec2!(-1.0, 1.0), vec2!(1.0, -1.0), Colour::RED, &render_handle);

        self.material.draw(&mut render_handle);
    }

    fn update(&mut self, _engine_handle: &mut Engine) {}
}
