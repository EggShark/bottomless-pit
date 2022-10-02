use raylib::prelude::*;
use utils::{Collide, Point};
use crate::ui_utils::Slectable;

#[derive(Debug, PartialEq)]
pub struct Button {
    pos: Point,
    size: Point,
    selected: bool,
    text: Option<String>,
}

impl Slectable for Button {
    fn get_pos(&self) -> Point {
        self.pos
    }
    fn deslect(&mut self) {
        self.selected = false;
    }
    fn select(&mut self) {
        self.selected = true;
    }
}

impl Button {
    pub fn new(pos: Point, size: Point, text: Option<String>) -> Self {
        Self {
            pos,
            size,
            selected: false,
            text,
        }
    }

    pub fn draw(&self ,d: &mut RaylibDrawHandle) {
        let color = if self.selected {
            Color::RED
        } else {
            Color::WHITE
        };

        d.draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, color);

        match &self.text {
            Some(text) => {
                d.draw_text(text, self.pos.x, self.pos.y, 20, Color::BLACK);
            },
            None => {}
        }
    }

    pub fn was_clicked(&self, rl: &RaylibHandle) -> bool {
        Collide::point_in_rect(&self.size, &self.pos, &rl.get_mouse_position()) && rl.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON)
    }
}