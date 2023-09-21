use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::colour::Colour;
use bottomless_pit::vectors::Vec2;

fn main() {
    let mut engine = EngineBuilder::new()
        .with_resolution((400, 400))
        .set_clear_colour(Colour::BLACK)
        .build()
        .unwrap();

    let material = MaterialBuilder::new().build(&mut engine);

    let pos = DebugTriangle {
        material,
    };

    engine.run(pos);
}

struct DebugTriangle {
    material: Material,
}

impl Game for DebugTriangle {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.material.add_triangle_with_coloured_verticies(
            Vec2{x: 200.0, y: 0.0},
            Vec2{x: 400.0, y: 400.0},
            Vec2{x: 0.0, y: 400.0},
            Colour::RED,
            Colour::GREEN,
            Colour::BLUE,
            &render_handle,
        );
        self.material.add_rectangle(Vec2{x: 0.0, y: 0.0}, Vec2{x: 100.0, y: 100.0}, Colour::RED, &render_handle);
        self.material.draw(&mut render_handle);
    }

    fn update(&mut self, _engine_handle: &mut Engine) {

    }
}
