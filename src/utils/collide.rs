use raylib::math::Vector2;

pub struct Collide;

impl Collide {
    pub fn point_in_rect(size: (u16, u16), pos: (u16, u16), point: &Vector2) -> bool{
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
}