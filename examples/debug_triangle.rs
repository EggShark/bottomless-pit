use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderHandle;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::Game;
use bottomless_pit::vec2;

fn main() {
    let mut engine = EngineBuilder::new()
        .with_resolution((400, 400))
        .set_clear_colour(Colour::BLACK)
        .build()
        .unwrap();

    let material = MaterialBuilder::new().build(&mut engine);

    let pos = DebugTriangle { material };

    engine.run(pos);
}

struct DebugTriangle {
    material: Material,
}

impl Game for DebugTriangle {
    fn render<'o>(
        &'o mut self,
        mut render: RenderHandle<'o>,
    ) {
        let mut render_handle = render.begin_pass();

        self.material.add_triangle_with_coloured_verticies(
            [
                vec2! { 200.0, 0.0 },
                // supplying one value to the macro will
                // make x and y the same value
                vec2! { 400.0 },
                vec2! { 0.0, 400.0 },
            ],
            [Colour::RED, Colour::GREEN, Colour::BLUE],
            &render_handle,
        );
        self.material.add_rectangle(
            vec2! { 0.0 },
            vec2! { 100.0 },
            Colour::RED,
            &render_handle,
        );
        self.material.draw(&mut render_handle);
    }

    fn update(&mut self, _engine_handle: &mut Engine) {}
}
