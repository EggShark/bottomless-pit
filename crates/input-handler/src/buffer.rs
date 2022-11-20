use utils::Point;
use crate::inputs::MovmentKeys;

pub enum InputDirection {
    Up,
    Down,
    Left,
    Right,
}

// example of number pad noation
//  1 | 7 8 9
//  0 | 4 5 6
// -1 | 1 2 3
//    |-1 0 1

pub fn key_to_direction(key: &MovmentKeys) -> Point {
    match key {
        MovmentKeys::LeftKey => Point {x: -1, y: 0},
        MovmentKeys::RightKey => Point {x: 1, y: 0},
        MovmentKeys::UpKey => Point {x: 0, y: 1},
        MovmentKeys::DownKey => Point{x: 0, y: -1},
        MovmentKeys::None => Point {x: 0, y: 0},
    }
}

pub fn inputs_to_point(sequence: &[MovmentKeys]) -> Point {
    let points: Vec<Point> = sequence.iter().map(|input| key_to_direction(input)).collect();

    let mut x_sum = 0;
    let mut y_sum = 0;
    for point in points {
        y_sum += point.y;
        x_sum += point.x;
    }    

    Point {x: x_sum, y: y_sum}
}

pub fn point_to_numpad(point: Point) -> i32 {
    match point {
        Point{x: -1, y: -1} => 1,
        Point{x: 0, y: -1} => 2,
        Point{x:1, y: -1} => 3,
        Point{x:-1, y: 0} => 4,
        Point{x: 0, y:0} => 5,
        Point{x:1, y: 0} => 6,
        Point{x: -1, y: 1} => 7,
        Point{x: 0, y: 1} => 8,
        Point{x: 1, y:1} => 9,
        _ => 5
    }
}

pub fn inputs_to_numpad(sequence: &[MovmentKeys]) -> i32 {
    let points: Vec<Point> = sequence.iter().map(|input| key_to_direction(input)).collect();

    let mut x_sum = 0;
    let mut y_sum = 0;
    for point in points {
        y_sum += point.y;
        x_sum += point.x;
    }    

    let sum_point = Point{x: x_sum, y: y_sum};

    point_to_numpad(sum_point)
}

// I LOVE WRAPPING ARRAYS TO GIVE SPECIFIC FUNCTIONALITY
#[derive(Debug)]
pub struct InputBuffer {
    inputs: [i32; 20],
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            inputs: [5; 20],
        }
    }

    pub fn new_input(&mut self, input: i32) {
        self.inputs.rotate_right(1); // shifts array to the right making 20 -> 0
        self.inputs[0] = input; // makes the "newest" input at 0
    }

    pub fn check_sequence(&self, sequence: &[i32], max_duration: i32) -> bool {
        let mut w = sequence.len() as i32 - 1;

        for i in 0..max_duration {
            let key = self.inputs[i as usize];

            if key == sequence[w as usize] {
                w -= 1;
            }
            if w == -1 {
                return true;
            }
        }

        false
    }

    pub fn get(&self, i: usize) -> i32 {
        self.inputs[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 2+2)
    }

    #[test]
    fn point_numpad_conversion() {
        let one = Point{x:-1, y:-1};
        let two = Point{x:0, y:-1};
        let three = Point{x:1, y:-1};
        let four = Point{x:-1, y:0};
        let five = Point{x: 0, y: 0};
        let six = Point{x:1, y:0};
        let seven = Point{x: -1, y:1};
        let eight = Point{x:0, y:1};
        let nine = Point{x:1,y:1};
        assert_eq!(point_to_numpad(one),1);
        assert_eq!(point_to_numpad(two), 2);
        assert_eq!(point_to_numpad(three), 3);
        assert_eq!(point_to_numpad(four), 4);
        assert_eq!(point_to_numpad(five), 5);
        assert_eq!(point_to_numpad(six), 6);
        assert_eq!(point_to_numpad(seven), 7);
        assert_eq!(point_to_numpad(eight), 8);
        assert_eq!(point_to_numpad(nine), 9);
    }

    #[test]
    fn down_left_numpad() {
        let input = [MovmentKeys::DownKey, MovmentKeys::LeftKey];

        assert_eq!(inputs_to_numpad(&input), 1);
    }

    #[test] 
    fn qcf_input() {
        let mut buffer = InputBuffer::new();
        buffer.new_input(2);
        buffer.new_input(3);
        buffer.new_input(6);

        let qcf = [2, 3, 6];

        let test = buffer.check_sequence(&qcf, 3);

        assert_eq!(true, test);
    }
}