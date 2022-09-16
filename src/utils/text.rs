use raylib::core::text::{RaylibFont, WeakFont};
use raylib::text::measure_text;

pub struct Text;

impl Text {
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
        let x = Self::center_text_x_pos(text, x, width, font_size);
    
        let y = Self::center_text_y(font, font_size, y, height);
    
        (x, y)
    }
    
}