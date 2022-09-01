mod main_menu;
mod sub_menu_test;

use raylib::prelude::RaylibDrawHandle;
use super::AppState;
use super::scene::Menu;
use super::Button;
use super::UiElements;
// idea is that menus are only in scope here in this function or module so in the main loop where its
// match state {
//      menued => menu_hander(), menus in scope and "in memory and in scope"
//      ingame => game_handler(), games in sesion so not happening
// }

pub enum Menus {
    Main,
    SubMenuTest
}

pub fn menu_handler(state: &mut AppState, d: &mut RaylibDrawHandle) {
    let mut current = main_menu::init_menu();
    let mut current_menu = Menus::Main;

    println!("menued");

    while state == &AppState::Menued && !d.window_should_close() {
        println!("g");
        match &current_menu {
            &Menus::Main => main_menu::loops(&current, d),
            &Menus::SubMenuTest => sub_menu_test::loops(&current, d),
        }
    }
}