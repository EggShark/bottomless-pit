use state::run;

fn main() {
    let s = TestUnit;
    run(Box::new(s));
}

struct TestUnit;

impl state::Game for TestUnit {
    fn render(&self) {
        println!("rendering_stuff");
    }
    fn update(&self) {
        println!("doing game calculations")
    }
}