use raylib::prelude::*;
use super::{Collide, Point};

#[derive(Debug, PartialEq)]
pub struct Button {
    pos: Point,
    size: Point,
    text: Option<String>,
}

impl Button {
    pub fn new(pos: Point, size: Point, text: Option<String>) -> Self {
        Self {
            pos,
            size,
            text,
        }
    }

    pub fn draw(&self ,d: &mut RaylibDrawHandle) {
        d.draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, Color::WHITE);

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