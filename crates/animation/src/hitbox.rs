use raylib::prelude::{RaylibDrawHandle, RaylibDraw, Color};
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
        let color = match self.kind {
            HitboxType::DamageAble => Color::BLUE,
            HitboxType::DamageDealing => Color::RED,
        };

        for i in 0..self.polygon.len() - 1 {
            d_handle.draw_line(self.polygon[i].x, self.polygon[i].y, self.polygon[i + 1].x, self.polygon[i + 1].y, color);
        }
        d_handle.draw_line(self.polygon[0].x, self.polygon[0].y, self.polygon[self.polygon.len() - 1].x, self.polygon[self.polygon.len() - 1].y, color);
    }

    // cant always return a bool just for test time it will
    pub fn collision_check(&self, other: &HitBox) -> bool {
        utils::Collide::ploy_poly(&self.polygon, &other.polygon)
    }

    pub fn shift_x(&mut self, amount: i32) {
        for point in self.polygon.iter_mut() {
            point.x = point.x + amount;
        }
    }

    pub fn shift_y(&mut self, amount: i32) {
        for point in self.polygon.iter_mut() {
            point.y = point.y + amount
        }
    }
}