mod button;

use button::Button;
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

    let mut circle_x: i32 = 0;
    let mut circle_y: i32 = 0;

    let testing = Button::new(20.0, 20.0, 100.0, 10.0, Color::LIME);

    while !rl.window_should_close() {
        let mut drawer = rl.begin_drawing(&thread);

        if testing.was_clicked(&drawer) {
            println!("clicked");
        }

        // right now movment is tied to frame rate
        if drawer.is_key_down(KeyboardKey::KEY_RIGHT) {
            circle_x += 5;
        } 
        if drawer.is_key_down(KeyboardKey::KEY_LEFT) {
            circle_x -= 5;
        }
        if drawer.is_key_down(KeyboardKey::KEY_DOWN) {
            circle_y += 5;
        }
        if drawer.is_key_down(KeyboardKey::KEY_UP) {
            circle_y -= 5;
        }
        
        draw_button(&testing, &mut drawer);
        drawer.draw_rectangle(circle_x, circle_y, 10, 10, Color::WHITE);
        drawer.clear_background(Color::BLUE);
        drawer.draw_text("Cheesed to meet u", 12, 12, 1, Color::GOLD);
    }
}

fn draw_button(button: &Button, drawer: &mut RaylibDrawHandle) {
    drawer.draw_rectangle_v(button.position, button.size, button.color);
}