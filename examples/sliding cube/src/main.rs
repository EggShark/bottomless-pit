use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::render::Renderer;
use bottomless_pit::colour::Colour;
use bottomless_pit::vectors::Vec2;

fn main() {
    let engine = EngineBuilder::new()
        .set_clear_colour(Colour::BLACK)
    	.set_target_fps(12)
        .build()
        .unwrap();

    let pos = Position {
        pos: Vec2 { x: 0.0, y: 0.0}
    };

    engine.run(pos);
}

struct Position {
    pos: Vec2<f32>,
}

impl Game for Position {
    fn render(&self, render_handle: &mut Renderer) {
        render_handle.draw_rectangle(self.pos, 100.0, 100.0, Colour::RED);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let dt = engine_handle.get_frame_delta_time();
        println!("dt: {}", dt);
        self.pos.x += 100.0 * dt;
    }
}
