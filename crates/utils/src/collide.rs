use raylib::math::Vector2;
use crate::point::Point;
pub struct Collide;

impl Collide {
    pub fn point_in_rect(size: &Point, pos: &Point, point: &Vector2) -> bool { 
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

    pub fn line_line(line1_x1: i32, line1_y1: i32, line1_x2: i32, line1_y2: i32, line2_x1: i32, line2_y1: i32, line2_x2: i32, line2_y2: i32) -> bool {
        // calculate distance to the intersection pont
        let ua: f32 = ((line2_x2-line2_x1)*(line1_y1-line2_y1) - (line2_y2-line2_y1)*(line1_x1-line2_x1)) as f32 / ((line2_y2-line2_y1)*(line1_x2-line1_x1) - (line2_x2-line2_x1)*(line1_y2-line1_y1)) as f32;
        let ub: f32 = ((line1_x2-line1_x1)*(line1_y1-line2_y1) - (line1_y2-line1_y1)*(line1_x1-line2_x1)) as f32 / ((line2_y2-line2_y1)*(line1_x2-line1_x1) - (line2_x2-line2_x1)*(line1_y2-line1_y1)) as f32;

        // if ua and ub are between 0-1 lines are colliding
        if ua >= 0.0 && ua <= 1.0 && ub >= 0.0 && ub <= 1.0 {
            return true
        }

        false
    }

    pub fn point_poly(point: Point, poly: &Vec<Point>) -> bool {
        let mut collision = false;
        for i in 0..poly.len() {
            let mut next = i + 1;
            if next == poly.len() {
                next = 0;
            }

            let current_point = poly[i];
            let next_point = poly[next];

            if ((current_point.y >= point.x && next_point.y < point.y) || (current_point.y < point.y && next_point.y >= point.y)) && 
            ((point.x as f32) < ((next_point.x - current_point.x)*(point.y-current_point.y)) as f32 / ((next_point.y-current_point.y) as f32) + (current_point.x as f32)) {
                collision = !collision;
            }
        }

        collision
    }

    pub fn line_poly(start_x: i32, start_y: i32, end_x: i32, end_y: i32, poly: &Vec<Point>) -> bool {
        for i in 0..poly.len() {
            let mut next = i + 1;
            if next == poly.len() {
                next = 0;
            }
            
            let p_line_start_x = poly[i].x;
            let p_line_start_y = poly[i].y;
            let p_line_end_x = poly[next].x;
            let p_line_end_y = poly[next].y;
            
            let collide = Self::line_line(p_line_start_x, p_line_start_y, p_line_end_x, p_line_end_y, start_x, start_y, end_x, end_y);
            if collide {
                return true;
            }
        }
        false
    }

    pub fn ploy_poly(poly1: &Vec<Point>, poly2: &Vec<Point>) -> bool {
        for i in 0..poly1.len() {
            let mut next = i + 1;
            if next == poly1.len() {
                next = 0;
            }

            let current_point = poly1[i];
            let next_point = poly1[next]; // neither of these can fail as we checked above :)

            let collision = Self::line_poly(current_point.x, current_point.y, next_point.x, next_point.y, poly2);
            if collision {
                return true;
            }

            let collision = Self::point_poly(poly2[0], poly1);
            if collision {
                return true;
            }
            let collision = Self::point_poly(poly1[0], poly2);
            if collision {
                return true;
            }
        }
        false
    }
}