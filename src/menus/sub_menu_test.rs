use super::Menu;
use super::Button;
use super::UiElements;
use super::RaylibDrawHandle;
use raylib::color::Color;

pub fn loops(menu: &Menu, d: &mut RaylibDrawHandle) {
    println!("z");
}

pub fn init_menu() -> Menu {
    let back_button = UiElements::Button(Button::new(10.0, 10.0, 100.0, 40.0, Color::RED));

    let vec = vec![back_button];

    Menu::from_vec(vec)
}