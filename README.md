# Bottomless-pit
Bottomless-pit is a simple 2D game engine that is still a work in progress.
This library is inspired slightly by Raylib and other rust based game engines like GGEZ.
All Bottomless-pit does currently is handle keyboard and mouse input while also providing
a simple way to draw objects to the screen. The shape and texutre renderer is written
in house, but the text rendering is powered by [glyphon](https://github.com/grovesNL/glyphon).
To get started start by implementing the Game trait on any struct you like
```rust,no_run
use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::render::RenderHandle;
use bottomless_pit::Game;
fn main() {
    let mut engine = EngineBuilder::new()
        .build()
        .expect("Failed to create the engine!");
    let game = CoolGame::new(&mut engine);
    engine.run(game);
}

struct CoolGame {
    // put whatever you want here
}

impl CoolGame {
    pub fn new(engine_handle: &mut Engine) -> Self {
        // you can even load assets before the game opens
        CoolGame {}
    }
}

impl Game for CoolGame {
    fn render<'o>(&'o mut self, mut render: RenderHandle<'o>) {
        let render_pass = render.begin_pass(Colour::BLACK);
        // render whatever you want
    }
    fn update(&mut self, engine_handle: &mut Engine) {
        // this is where all your logic should go
    }
}
```