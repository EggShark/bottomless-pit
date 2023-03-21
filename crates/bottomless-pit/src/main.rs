use state::{run, Engine, EngineBuilder, Colour};

fn main() {
    let engine = EngineBuilder::new()
        .set_clear_colour(Colour::Blue)
        .fullscreen()
        .build()
        .unwrap();

    let s = TestUnit;
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