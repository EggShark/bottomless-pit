use raylib::core::math::Vector2;

#[derive(Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Into<Vector2> for Point {
    fn into(self) -> Vector2 {
        Vector2::new(self.x as f32, self.y as f32)
    }
}