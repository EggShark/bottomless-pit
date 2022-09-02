mod button;

use raylib::prelude::*;


fn main() {
    let screen_width: u16 = 800;
    let screen_height: u16 = 450;


    let(mut rl, thread) = raylib::init()
        .resizable()
        .size(screen_width as i32, screen_height as i32)
        .title("cheesed to meet u")
        .build();
    
    rl.set_target_fps(30);
    rl.set_exit_key(None);

    let testing = button::Button::new((10, 10), (100, 40), Some("Hello".to_string()));

    while !rl.window_should_close() {
        let mut d_handle = rl.begin_drawing(&thread);
        testing.draw(&mut d_handle);

        if testing.was_clicked(&d_handle) {
            println!("hello");
        }

        d_handle.clear_background(Color::YELLOW);
    }
}
