use raylib::prelude::*;

pub enum UiElements{
    Text(Text),
    Button(Button),
}

pub struct Text {
    position: (i32, i32),
    font_size: u16,
    text: String,
    // font: Font,
    color: Color,
}

pub struct Button {
    position: Vector2,
    size: Vector2,
    color: Color,
}

impl UiElements {
    pub fn draw(&self, drawer: &mut RaylibDrawHandle) {
        match self {
            Self::Text(x) => {
                x.draw(drawer);
            }
            Self::Button(x) => {
                x.draw(drawer);
            }
        };
    }
}

impl Button {
    pub fn new(x: f32, y: f32, length: f32, height: f32, color: Color) -> Self {
        Self {
            position: Vector2::new(x, y),
            size: Vector2::new(length, height),
            color,
        }
    }

    pub fn hovered(&self, rl: &RaylibHandle) -> bool{
        is_inside(&self.position, &self.size, &rl.get_mouse_position())
    }

    pub fn was_clicked(&self, rl: &RaylibHandle) -> bool {
        is_inside(&self.position, &self.size, &rl.get_mouse_position()) && rl.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON)
    }

    pub fn draw(&self, drawer: &mut RaylibDrawHandle) {
        drawer.draw_rectangle_v(self.position, self.size, self.color);
    }
}

impl Text {
    pub fn new(text: String, position: (i32, i32), font_size: u16, color: Color) -> Self {
        Self {
            text,
            position,
            font_size,
            color,
        }
    }

    pub fn draw(&self, drawer: &mut RaylibDrawHandle) {
        let (x,y) = self.position;
        drawer.draw_text(&self.text, x, y, self.font_size as i32, self.color);
    }
}

// Basic collision code
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