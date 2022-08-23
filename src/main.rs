mod button;
mod scene;

use scene::Scene;
use button::{Button, Text, UiElements};
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

    let testing = Button::new(20.0, 20.0, 100.0, 10.0, Color::LIME);

    let h = Button::new(10.0, 10.0, 100.0, 20.0, Color::GOLD);
    let y = Text::new("Hey Hey".to_string(), (30, 30), 12, Color::BLACK);

    let z = UiElements::Button(h);
    let b = UiElements::Text(y);

    let x = vec![z, b];

    let yes = Scene::from_vec(x);

    while !rl.window_should_close() {
        let mut drawer = rl.begin_drawing(&thread);

        if testing.was_clicked(&drawer) {
            println!("clicked");
        }
        
        testing.draw(&mut drawer);

        yes.testing(&mut drawer);
    }
}
