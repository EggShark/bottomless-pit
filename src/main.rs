use state::run;

fn main() {
    pollster::block_on(run());
}