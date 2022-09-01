mod button;
mod menus;
mod scene;

use menus::{Menus, menu_handler};
use button::{Button, Text, UiElements};
use raylib::prelude::*;

#[derive(PartialEq)]
pub enum AppState {
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
        match &state {
            AppState::Menued => menus::menu_handler(&mut state, &mut drawer),
            AppState::InGame => unimplemented!(),
        }
    }
}
