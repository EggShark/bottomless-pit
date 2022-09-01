mod main_menu;
mod sub_menu_test;


// idea is that menus are only in scope here in this function or module so in the main loop where its
// match state {
//      menued => menu_hander(), menus in scope and "in memory and in scope"
//      ingame => game_handler(), games in sesion so not happening
// }

enum Menus {
    Main,
    SubMenuTest
}

let mut current_menu = Menus::Main;

pub fn menu_handler() {
    match current_menu {
        Menus::Main => main_menu::loop(),
        Menus::SubMenuTest => sub_menu_test::loop(),
    }
}