use utils::{Point, Text};
use raylib::consts::KeyboardKey;
use raylib::prelude::RaylibDraw;
use raylib::drawing::RaylibDrawHandle;
use raylib::color::Color;
use raylib::core::math::Vector2;

use crate::ui_utils::Selectable;

#[derive(Debug, PartialEq)]
pub struct KeyChanger {
    pos: Point,
    size: Point,
    selected: bool,
    text: String,
    key_selcted: KeyboardKey,
}

impl Selectable for KeyChanger {
    fn deslect(&mut self) {
        self.selected = false;
    }
    fn get_pos(&self) -> Point {
        self.pos
    }
    fn select(&mut self) {
        self.selected = true;
    }
}

impl KeyChanger {
    pub fn new(pos: Point, size: Point, text: String, key_selcted: KeyboardKey) -> Self {
        Self {
            pos,
            size,
            selected: false,
            text,
            key_selcted,
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        let color = if self.selected {
            Color::RED
        } else {
            Color::WHITE
        };
        let text_pos = Text::center_text_x_pos(&self.text, self.pos.x, self.size.x, 20);


        d_handle.draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, color);
        d_handle.draw_text(&self.text, text_pos, self.pos.y, 20, Color::BLACK);
    }
}