use raylib::prelude::*;

fn main() {
    let screen_width: u16 = 800;
    let screen_height: u16 = 450;

    let(mut rl, thread) = raylib::init()
        .resizable()
        .size(screen_width as i32, screen_height as i32)
        .title("cheesed to meet u")
        .build();
    
    rl.set_exit_key(Some(KeyboardKey::KEY_O));

    while !rl.window_should_close() {
        let mut drawer = rl.begin_drawing(&thread);

        drawer.clear_background(Color::BLUE);
        drawer.draw_text("Cheesed to meet u", 12, 12, 12, Color::GOLD);
    }
}