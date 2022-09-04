use raylib::prelude::*;

pub struct Button {
    pos: (u16, u16),
    size: (u16, u16),
    text: Option<String>,
}

impl Button {
    pub fn new(pos: (u16, u16), size: (u16, u16), text: Option<String>) -> Self {
        Self {
            pos,
            size,
            text,
        }
    }

    pub fn draw(&self ,d: &mut RaylibDrawHandle) {
        d.draw_rectangle(self.pos.0 as i32, self.pos.1 as i32, self.size.0 as i32, self.size.1 as i32, Color::WHITE);

        match &self.text {
            Some(text) => {
                d.draw_text(text, self.pos.0 as i32, self.pos.1 as i32, 20, Color::BLACK);
            },
            None => {}
        }
    }

    pub fn was_clicked(&self, rl: &RaylibHandle) -> bool {
        point_in_rect(self.size, self.pos, &rl.get_mouse_position()) && rl.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON)
    }
}

fn point_in_rect(size: (u16, u16), pos: (u16, u16), point: &Vector2) -> bool{
    if point.x < pos.0 as f32 {
        return false
    }
    if point.y < pos.1 as f32 {
        return false
    }
    if point.y > (pos.1 + size.1) as f32 {
        return false
    }
    if point.x > (pos.0 + size.0) as f32 {
        return false
    }

    true
}