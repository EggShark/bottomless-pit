use raylib::core::math::Vector2;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn sort_vec(mut points: Vec<Point>) -> Vec<Point>{
        points.sort_by(|a, b| {
            let y_order = a.y.cmp(&b.y);
            if y_order == std::cmp::Ordering::Equal {
                a.x.cmp(&b.x)
            } else {
                y_order
            }
        });
        points
    }
}

impl Into<Vector2> for Point {
    fn into(self) -> Vector2 {
        Vector2::new(self.x as f32, self.y as f32)
    }
}

impl From<Vector2> for Point {
    fn from(vector: Vector2) -> Self {
        Self{
            x: vector.x as i32,
            y: vector.y as i32,
        }
    }
}