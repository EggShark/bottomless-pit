use raylib::prelude::*;

pub struct Button {
    pub position: Vector2,
    pub size: Vector2,
    pub color: Color,
}

impl Button {
    pub fn new(x: f32, y: f32, width: f32, height: f32, color: Color) -> Self {
        Self {
            position: Vector2::new(x, y),
            size: Vector2::new(width, height),
            color,
        }
    }

    pub fn was_clicked(&self, rl: &RaylibHandle) -> bool {
        is_inside(&self.position, &self.size, &rl.get_mouse_position()) && rl.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON)
    }
}

fn is_inside(pos: &Vector2, size: &Vector2, point: &Vector2) -> bool {
    // the pos is the top left corrner of the rect
    if point.x < pos.x {
        return false
    }
    if point.y < pos.y {
        return false
    }
    if point.y > pos.y + size.y {
        return false
    }
    if point.x > pos.x + size.x {
        return false
    }
    true
}