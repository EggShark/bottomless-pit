use super::Menu;
use super::Button;
use super::UiElements;
use super::RaylibDrawHandle;
use raylib::color::Color;
use raylib::RaylibHandle;
use raylib::ffi::CloseWindow;

pub fn loops(menu: &Menu, d: &mut RaylibDrawHandle) {
    draw_loop(menu, d);
    logic_loop(menu, d);

}

pub fn init_menu() -> Menu {
    let quit_button = UiElements::Button(Button::new(10.0, 10.0, 100.0, 40.0, Color::RED));
    let switch_menu = UiElements::Button(Button::new(20.0, 10.0, 100.0, 40.0, Color::RED));

    let vec = vec![quit_button, switch_menu];

    Menu::from_vec(vec)
}

fn draw_loop(menu: &Menu, d: &mut RaylibDrawHandle) {
    menu.draw(d);
}

fn logic_loop(menu: &Menu, rl: &RaylibHandle) {
    let buttons = menu.get_buttons();

    if buttons[0].was_clicked(rl) {
        unsafe {
            CloseWindow();
        }
    }
    if buttons[1].was_clicked(rl) {
        // swtich menu
    }
}