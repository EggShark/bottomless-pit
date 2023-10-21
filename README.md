# Bottomless-pit (working title)
Bottomless-pit is a simple 2D game engine that is still a work in progress.
This library is inspired slightly by Raylib and other rust based game engines like GGEZ.
All Bottomless-pit does currently is handle keyboard and mouse input while also providing
a simple way to draw objects to the screen. The shape and texutre renderer is written
in house, but the text rendering is powered by [glyphon](https://github.com/grovesNL/glyphon).

To get started start by implmenting the Game trait on any struct you like
```rust,no_run
use bottomless_pit::{Engine, EngineBuilder, Renderer, Game};
fn main() {
    let engine = EngineBuilder::new()
        .build()
        .expect("Failed to crate the engine!");
    let game = CoolGame::new(&mut engine);
    engine.run(game);
}

struct CoolGame {
    // put whatever you want here
}

impl CoolGame {
    pub fn new(engine_handle: &mut Engine) {
        // you can even load assets before the game opens
    }
}

impl Game for CoolGame {
    fn render(&self, render_handle: &mut Renderer) {
        // render what ever you want
    }
    fn update(&mut self, engine_handle: &mut Engine) {
        // this is where all your logic should go
    }
}
```