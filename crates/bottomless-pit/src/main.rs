use state::{run, Engine, EngineBuilder, Colour};

fn main() {
    let s = TestUnit;
    let engine = EngineBuilder::new()
        .set_clear_colour(Colour::Blue)
        .fullscreen()
        .build()
        .unwrap()
        .run(Box::new(s));

    //run(Box::new(s));
}

struct TestUnit;

impl state::Game for TestUnit {
    fn render(&self) {
        println!("rendering_stuff");
    }
    fn update(&self, engine_handle: &mut Engine) {
        println!("doing game calculations")
    }
}