mod button;
mod arrow_selection;

use raylib::text::measure_text;
use raylib::drawing::RaylibDrawHandle;
use raylib::core::text::{RaylibFont, WeakFont};
use super::utils::Collide;

pub use button::Button;
pub use arrow_selection::ArrowSelector;

fn center_text_x_pos(text: &str, x: u16, width: u16, font_size: i32) -> u16 {
    let size = measure_text(text, font_size);
    let text_pos = ((width/ 2) - (size/2) as u16) + x;
    text_pos
}

fn center_text_y(font: WeakFont, font_size: i32, y: u16, obj_height: u16) -> u16 {
    let text_height = font.base_size();
    let scale_factor = font_size/font.base_size();

    let text_size = text_height * scale_factor;

    ((obj_height/2) - (text_size as u16/2)) + y
}

pub fn center_text(text: &str, x: u16, y: u16, width: u16, height: u16, font_size: i32, font: WeakFont) -> (u16, u16) {
    let x = center_text_x_pos(text, x, width, font_size);

    let y = center_text_y(font, font_size, y, height);

    (x, y)
}

#[derive(Debug, PartialEq)]
pub struct UiScene {
    pub buttons: Vec<Button>,
    pub selectors: Vec<ArrowSelector>
}

impl Default for UiScene {
    fn default() -> Self {
        Self {
            buttons: Vec::new(),
            selectors: Vec::new(),
        }
    }
}

impl UiScene {
    pub fn init_main() -> Self {
        let quit = Button::new((10, 10), (100, 40), Some("Quit".to_string()));
        let go_to_game = Button::new((10, 80), (100, 40), Some("to game".to_string()));

        let buttons = vec![quit, go_to_game];

        Self{
            buttons,
            selectors: Vec::new(),
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        for button in self.buttons.iter() {
            button.draw(d_handle);
        }

        for selector in self.selectors.iter() {
            selector.draw(d_handle);
        } 
    }
}