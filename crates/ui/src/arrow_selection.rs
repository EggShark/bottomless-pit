use raylib::core::RaylibHandle;
use raylib::prelude::RaylibDraw;
use raylib::drawing::RaylibDrawHandle;
use raylib::color::Color;
use raylib::core::math::Vector2;
use raylib::consts::MouseButton;
use utils::{Collide, Text, Point};
use crate::ui_utils::Selectable;

#[derive(Debug, PartialEq)]
pub struct ArrowSelector {
    pos: Point,
    size: Point,
    options: u8,
    curr_option: i8,
    selected: bool,
    display_text: Vec<String>,
}

impl Selectable for ArrowSelector {
    fn get_pos(&self) -> Point {
        self.pos
    }
    fn select(&mut self) {
        self.selected = true;
    }
    fn deslect(&mut self) {
        self.selected = false;
    }
}

impl ArrowSelector {
    pub fn new(display_text: Vec<String>, pos: Point, size: Point) -> Self {
        Self {
            pos,
            size,
            options: display_text.len() as u8,
            curr_option: 0,
            selected: false,
            display_text: display_text,
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        let color = if self.selected {
            Color::RED
        } else {
            Color::WHITE
        };

        d_handle.draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, color);

        let (x, y) = Text::center_text(&self.display_text[self.curr_option as usize], &self.pos, &self.size, 20, d_handle.get_font_default());
        d_handle.draw_text(&self.display_text[self.curr_option as usize], x as i32, y as i32, 20, Color::BLACK); 

        let left_first_triangle_x = (self.pos.x + (self.size.x / 4)) - 10;
        let left_first_triangle_y = self.size.y/2 + self.pos.y;
        let left_v = Vector2::new(left_first_triangle_x as f32, left_first_triangle_y as f32);
        let bottom_right_v = Vector2::new(left_first_triangle_x as f32 + 20.0, left_first_triangle_y as f32 - 10.0);
        let top_right_v = Vector2::new(left_first_triangle_x as f32 + 20.0, left_first_triangle_y as f32 + 10.0);
        // triangles need to be drawn in counter clock wise order
        d_handle.draw_triangle(top_right_v, bottom_right_v, left_v, Color::BLACK);

        let right_first_point_x: f32 = (self.pos.x as f32 + (self.size.x as f32 * 0.75)) + 10.0;
        let right_first_point_y: f32 = self.size.y as f32 /2.0 + self.pos.y as f32;
        let right_first_v = Vector2::new(right_first_point_x, right_first_point_y);
        let right_bottom_v = Vector2::new(right_first_point_x - 20.0, right_first_point_y - 10.0);
        let right_top_v = Vector2::new(right_first_point_x - 20.0, right_first_point_y + 10.0);

        d_handle.draw_triangle(right_top_v, right_first_v, right_bottom_v, Color::BLACK);
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        // change options on click
        let mouse_pos: Vector2 = rl.get_mouse_position();
        let (pos, size) = self.get_right_sqaure();
        let over_right = Collide::point_in_rect(&size, &pos, &mouse_pos);

        let (pos, size) = self.get_left_square();
        let over_left = Collide::point_in_rect(&size, &pos, &mouse_pos);
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

    fn get_right_sqaure(&self) -> (Point, Point){
        let x = (self.pos.x + (self.size.x as f32 * 0.75) as i32) - 10;
        let y = self.size.y/2 + self.pos.y - 10;
        (Point{x, y}, Point{x:20, y:20})
    }

    fn get_left_square(&self) -> (Point, Point) {
        let x = (self.pos.x + (self.size.x / 4)) - 10;
        let y = self.size.y/2 + self.pos.y - 10;
        (Point{x, y}, Point{x:20, y:20})
    }
}