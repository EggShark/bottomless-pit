mod button;
mod arrow_selection;

use raylib::text::measure_text;

pub use button::Button;
pub use arrow_selection::ArrowSelector;

pub fn center_text_x_pos(text: &str, x: u16, width: u16, font_size: i32) -> u16 {
    let size = measure_text(text, font_size as i32);
    let text_pos = ((width/ 2) - (size/2) as u16) + x;
    text_pos
}