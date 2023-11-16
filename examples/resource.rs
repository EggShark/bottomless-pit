use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::render::RenderInformation;

fn main() {
    let engine = EngineBuilder::new()
        .build()
        .unwrap();

    let game = LineExample;

    engine.create_resource("examples/bplogo.ng");

    engine.run(game);
}

struct LineExample;

impl Game for LineExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {

    }

    fn update(&mut self, _engine_handle: &mut Engine) {
        
    }
}