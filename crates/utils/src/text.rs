use raylib::core::text::{RaylibFont, WeakFont};
use raylib::text::measure_text;
use crate::point::Point;

pub struct Text;

impl Text {
    pub fn center_text_x_pos(text: &str, x: i32, width: i32, font_size: i32) -> i32 {
        let size = measure_text(text, font_size);
        let text_pos = ((width/ 2) - (size/2)) + x;
        text_pos
    }
    
    pub fn center_text_y(font: WeakFont, font_size: i32, y: i32, obj_height: i32) -> i32 {
        let text_height = font.base_size();
        let scale_factor = font_size/font.base_size();
    
        let text_size = text_height * scale_factor;
    
        ((obj_height/2) - (text_size/2)) + y
    }
    
    pub fn center_text(text: &str, pos: &Point, size: &Point, font_size: i32, font: WeakFont) -> (i32, i32) {
        let x = Self::center_text_x_pos(text, pos.x, size.x, font_size);
    
        let y = Self::center_text_y(font, font_size, pos.y, size.y);
    
        (x, y)
    }
    
}