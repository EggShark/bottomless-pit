mod button;
mod scene;

use scene::Scene;
use button::{Button, Text, UiElements};
use raylib::prelude::*;

enum AppState {
    Menued,
    InGame,
}

fn main() {
    let screen_width: u16 = 800;
    let screen_height: u16 = 450;
    
    let mut state = AppState::Menued;

    let(mut rl, thread) = raylib::init()
        .resizable()
        .size(screen_width as i32, screen_height as i32)
        .title("cheesed to meet u")
        .build();
    
    rl.set_target_fps(30);
    rl.set_exit_key(None);
    
    while !rl.window_should_close() {
        let mut drawer = rl.begin_drawing(&thread);
        match state {
            AppState::Menued => todo!(),
            AppState::InGame => unimplemented!(),
        }
    }
}
