use state::{run, Engine};

fn main() {
    let s = TestUnit;
    run(Box::new(s));
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