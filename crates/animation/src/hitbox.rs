use raylib::{core::math::Vector2, prelude::{RaylibDrawHandle, RaylibDraw, Color}};
use utils::Point;

pub enum HitboxType {
    DamageAble,
    DamageDealing,
}

pub struct HitBox {
    polygon: Vec<Point>,
    kind: HitboxType,
}

impl HitBox {
    // you have to be carfull when doing the poly point order
    // something to adress latter
    pub fn new(polygon: Vec<Point>, kind: HitboxType) -> Self {
        Self {
            polygon,
            kind,
        }
    }

    pub fn draw_hibox(&self, d_handle: &mut RaylibDrawHandle) {
        for i in 0..self.polygon.len() - 1 {
            d_handle.draw_line(self.polygon[i].x, self.polygon[i].y, self.polygon[i + 1].x, self.polygon[i + 1].y, Color::RED);
        }
        d_handle.draw_line(self.polygon[0].x, self.polygon[0].y, self.polygon[self.polygon.len() - 1].x, self.polygon[self.polygon.len() - 1].y, Color::RED);
    }
}