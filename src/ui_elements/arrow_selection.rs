use raylib::core::RaylibHandle;
use raylib::prelude::RaylibDraw;
use raylib::drawing::RaylibDrawHandle;
use raylib::color::Color;
use raylib::core::math::Vector2;
use raylib::consts::MouseButton;
use super::{Collide, center_text};

#[derive(Debug, PartialEq)]
pub struct ArrowSelector {
    pos: (u16, u16),
    size: (u16, u16),
    options: u8,
    curr_option: i8,
    display_text: Vec<String>,
}

impl ArrowSelector {
    pub fn new(display_text: Vec<String>, pos: (u16, u16), size: (u16, u16)) -> Self {
        Self {
            pos,
            size,
            options: display_text.len() as u8,
            curr_option: 0,
            display_text: display_text
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        d_handle.draw_rectangle(self.pos.0 as i32, self.pos.1 as i32, self.size.0 as i32, self.size.1 as i32, Color::WHITE);

        let (x, y) = center_text(&self.display_text[self.curr_option as usize], self.pos.0, self.pos.1, self.size.0, self.size.1, 20, d_handle.get_font_default());
        d_handle.draw_text(&self.display_text[self.curr_option as usize], x as i32, y as i32, 20, Color::BLACK); 

        let left_first_triangle_x = (self.pos.0 + (self.size.0 / 4)) - 10;
        let left_first_triangle_y = self.size.1/2 + self.pos.1;
        let left_v = Vector2::new(left_first_triangle_x as f32, left_first_triangle_y as f32);
        let bottom_right_v = Vector2::new(left_first_triangle_x as f32 + 20.0, left_first_triangle_y as f32 - 10.0);
        let top_right_v = Vector2::new(left_first_triangle_x as f32 + 20.0, left_first_triangle_y as f32 + 10.0);
        // triangles need to be drawn in counter clock wise order
        d_handle.draw_triangle(top_right_v, bottom_right_v, left_v, Color::BLACK);

        let right_first_point_x: f32 = (self.pos.0 as f32 + (self.size.0 as f32 * 0.75)) + 10.0;
        let right_first_point_y: f32 = self.size.1 as f32 /2.0 + self.pos.1 as f32;
        let right_first_v = Vector2::new(right_first_point_x, right_first_point_y);
        let right_bottom_v = Vector2::new(right_first_point_x - 20.0, right_first_point_y - 10.0);
        let right_top_v = Vector2::new(right_first_point_x - 20.0, right_first_point_y + 10.0);

        d_handle.draw_triangle(right_top_v, right_first_v, right_bottom_v, Color::BLACK);
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        // change options on click
        let mouse_pos: Vector2 = rl.get_mouse_position();
        let ((r_x, r_y), (r_width, r_height)) = self.get_right_sqaure();
        let over_right = Collide::point_in_rect((r_width, r_height), (r_x, r_y), &mouse_pos);

        let ((l_x, l_y), (l_width, l_height)) = self.get_left_square();
        let over_left = Collide::point_in_rect((l_width, l_height), (l_x, l_y), &mouse_pos);
        let click =  rl.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON);

        if over_left && click {
            self.curr_option -= 1;
            if self.curr_option < 0 {
                self.curr_option = self.options as i8 - 1;
            }
        }

        if over_right && click {
            self.curr_option += 1;
            if self.curr_option == self.options as i8 {
                self.curr_option = 0;
            }
        }
    }

    pub fn get_curr_selection(&self) -> i8 {
        self.curr_option
    }

    fn get_right_sqaure(&self) -> ((u16, u16), (u16, u16)){
        let x_pos = (self.pos.0 + (self.size.0 as f32 * 0.75) as u16 ) - 10;
        let y_pos = self.size.1/2 + self.pos.1 - 10;
        ((x_pos, y_pos), (20, 20))
    }

    fn get_left_square(&self) -> ((u16, u16), (u16, u16)) {
        let x_pos = (self.pos.0 + (self.size.0 / 4)) - 10;
        let y_pos = self.size.1/2 + self.pos.1 - 10;
        ((x_pos, y_pos), (20, 20))
    }
}