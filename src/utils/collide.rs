use raylib::math::Vector2;
use super::Point;
pub struct Collide;

impl Collide {
    pub fn point_in_rect(size: &Point, pos: &Point, point: &Vector2) -> bool{
        if point.x < pos.x as f32 {
            return false
        }
        if point.y < pos.y as f32 {
            return false
        }
        if point.y > (pos.y + size.y) as f32 {
            return false
        }
        if point.x > (pos.x + size.x) as f32 {
            return false
        }
    
        true
    }
}