use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::colour::Colour;
use bottomless_pit::vectors::Vec2;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_clear_colour(Colour::BLACK)
        .build()
        .unwrap();

    let material = MaterialBuilder::new().build(&mut engine);

    let pos = Position {
        pos: Vec2 { x: 0.0, y: 0.0},
        material,
    };

    engine.run(pos);
}

struct Position {
    pos: Vec2<f32>,
    material: Material,
}

impl Game for Position {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.material.add_rectangle(self.pos, Vec2{x: 100.0, y: 100.0}, Colour::RED, &render_handle);
        self.material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let dt = engine_handle.get_frame_delta_time();
        println!("{}", dt);
        self.pos.x += 100.0 * dt;
    }
}
