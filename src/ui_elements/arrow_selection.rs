use raylib::core::RaylibHandle;
use raylib::prelude::RaylibDraw;
use raylib::drawing::RaylibDrawHandle;
use raylib::color::Color;
use raylib::core::text::measure_text;
use super::center_text_x_pos;

pub struct ArrowSelector {
    pos: (u16, u16),
    size: (u16, u16),
    options: u8,
    curr_option: u8,
    display_text: String,
}

impl ArrowSelector {
    pub fn new(options: u8, display_text: &str, pos: (u16, u16), size: (u16, u16)) -> Self {
        Self {
            pos,
            size,
            options,
            curr_option: 0,
            display_text: display_text.to_string()
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        d_handle.draw_rectangle(self.pos.0 as i32, self.pos.1 as i32, self.size.0 as i32, self.size.1 as i32, Color::WHITE);

        let pos = center_text_x_pos(&self.display_text, self.pos.0, self.size.0, 20);
        d_handle.draw_text(&self.display_text, pos as i32, (self.pos.1 - 2) as i32, 20, Color::BLACK); 
    }
}