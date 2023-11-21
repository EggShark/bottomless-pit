use bottomless_pit::Game;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::input::Key;
use bottomless_pit::render::RenderInformation;
use bottomless_pit::resource::ResourceId;

fn main() {
    let mut engine = EngineBuilder::new()
        .build()
        .unwrap();

    let handle = engine.create_resource("examples/bplogo.png");

    let game = ResourceExample {
        handle,
    };

    engine.run(game);
}

struct ResourceExample {
    handle: ResourceId<Vec<u8>>
}

impl Game for ResourceExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {

    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let data = engine_handle.get_byte_resource(self.handle).unwrap();
    }
}