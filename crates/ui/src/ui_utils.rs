use utils::Point;

use crate::button::Button;
use crate::arrow_selection::ArrowSelector;


pub trait Slectable {
    fn get_pos(&self) -> Point;
    fn select(&mut self);
    fn deslect(&mut self);
}

type Selectables<'a> = Vec<&'a mut dyn Slectable>;

#[derive(Debug, PartialEq)]
pub enum SelectableUiElements {
    ArrowSelector,
    Button,
}


pub struct UiUtils;

impl UiUtils {
    pub fn go_down(items: &mut Selectables, current_selected: Point) -> Point {
        let mut y_dist = u32::MAX;
        let mut array_pos: usize = 0;
        let mut x_dist = i32::MAX;
        for i in 0..items.len() {
            if items[i].get_pos() != current_selected {
                // we cast as a u32 bc a negative will wrap around to u32 max - whatever it was absolute value
                // this makes the negative values the farthest ones away!
                let temp_distacne = (items[i].get_pos().y - current_selected.y) as u32;
                if temp_distacne == 0 && items[i].get_pos().y == current_selected.y {
                    let temp_x = items[i].get_pos().x - current_selected.x;
                    if temp_x < x_dist && temp_x > 0 {
                        array_pos = i;
                        x_dist = temp_x;
                        y_dist = temp_distacne;
                    }
                } else if temp_distacne == y_dist {
                    let temp_x = items[i].get_pos().x - current_selected.x;
                    if temp_x < x_dist && temp_x > 0 {
                        x_dist = temp_x;
                    }
                } else if temp_distacne < y_dist && temp_distacne != 0 {
                    array_pos = i;
                    y_dist = temp_distacne;
                }
            }
        }
        items[array_pos].select();
        items[array_pos].get_pos()
    }
    
    pub fn go_up(items: &mut Selectables, current_selected: Point) -> Point {
        let mut y_dist = u32::MAX;
        let mut array_pos: usize = 0;
        let mut x_pos = 0;
        for i in 0..items.len() {
            let item_pos = items[i].get_pos();
            if item_pos != current_selected {
                let temp_distance = (current_selected.y - item_pos.y) as u32;
                if temp_distance == 0 && item_pos.y == current_selected.y {
                    if item_pos.x < current_selected.x && item_pos.x > x_pos {
                        array_pos = i;
                        x_pos = item_pos.x;
                        y_dist = temp_distance;
                    }
                } else if temp_distance == y_dist {
                    if item_pos.x > x_pos {
                        x_pos = item_pos.x;
                    }
                } else if temp_distance < y_dist && temp_distance != 0 {
                    array_pos = i;
                    y_dist = temp_distance;
                }
            }
        }

        items[array_pos].select();
        items[array_pos].get_pos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn congregatge_selectables<'b>(buttons: &'b mut Vec<Button>, selectors: &'b mut Vec<ArrowSelector>) -> Vec<&'b mut dyn Slectable> {
        let mut selectables: Vec<&'b mut dyn Slectable> = Vec::new();
        for z in buttons.iter_mut() {
            selectables.push(z);
        }
        for x in selectors.iter_mut() {
            selectables.push(x);
        }
        selectables
    }

    #[test]
    fn above_to_bellow_b_to_b() {
        let top: Button = Button::new(Point{x:0, y:1}, Point {x:10, y:10}, None);
        let bottom: Button = Button::new(Point{x: 0,y: 10}, Point{x: 100, y:100}, None);
        let mut buttons = vec![top, bottom];
        let mut selectors = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut selectors);

        let point = UiUtils::go_down(&mut items, Point{x:0, y:0}); // simulates selecting the first
        let point = UiUtils::go_down(&mut items, point); // then selects the bottom

        assert_eq!(Point{x:0, y:10}, point)
    }

    #[test]
    fn above_to_bellow_b_to_b_inverse_array() {
        let top: Button = Button::new(Point{x:0, y:1}, Point {x:10, y:10}, None);
        let bottom: Button = Button::new(Point{x: 0,y: 10}, Point{x: 100, y:100}, None);
        let mut buttons = vec![bottom, top];
        let mut selectors = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut selectors);

        let point = UiUtils::go_down(&mut items, Point{x:0, y:0});
        let point = UiUtils::go_down(&mut items, point);

        assert_eq!(Point{x:0, y:10}, point);
    }

    #[test]
    fn same_y_2_buttons() {
        let left: Button = Button::new(Point{x: 0, y: 1}, Point{x:10, y:10}, None);
        let right: Button = Button::new(Point{x: 100, y: 1}, Point{x:10, y:10}, None);
        let mut buttons = vec![left, right];
        let mut selectors = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut selectors);

        let point = UiUtils::go_down(&mut items, Point{x:0,y:0});
        assert_eq!(Point{x:0, y:1}, point);
        let point = UiUtils::go_down(&mut items, point);
        assert_eq!(Point{x:100, y:1}, point);
    }

    #[test]
    fn same_y_3_buttons() {
        let left: Button = Button::new(Point{x: 0, y: 1}, Point{x:10, y:10}, None);
        let mid: Button = Button::new(Point{x: 100, y: 1}, Point{x:10, y:10}, None);
        let right: Button = Button::new(Point{x: 300, y: 1}, Point{x:10, y:10}, None);
        let mut buttons = vec![left, right, mid];
        let mut selectors = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut selectors);

        let point = UiUtils::go_down(&mut items, Point{x:0,y:0});
        assert_eq!(Point{x:0, y:1}, point);
        let point = UiUtils::go_down(&mut items, point);
        assert_eq!(Point{x: 100, y:1}, point);
    }

    #[test]
    fn same_y_3_buttons_diff_order() {
        let left: Button = Button::new(Point{x: 0, y: 1}, Point{x:10, y:10}, None);
        let mid: Button = Button::new(Point{x: 100, y: 1}, Point{x:10, y:10}, None);
        let right: Button = Button::new(Point{x: 300, y: 1}, Point{x:10, y:10}, None);
        let mut buttons = vec![left, mid, right];
        let mut selectors = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut selectors);

        let point = UiUtils::go_down(&mut items, Point{x:0,y:0});
        assert_eq!(Point{x:0, y:1}, point);
        let point = UiUtils::go_down(&mut items, point);
        assert_eq!(Point{x: 100, y:1}, point);
    }
    
    #[test]
    fn b_a_b_same_y_level() {
        let left: Button = Button::new(Point {x:0, y: 1}, Point{x:10, y:10}, None);
        let right: Button = Button::new(Point {x:40, y: 1}, Point{x:10, y:10}, None);
        let mut buttons = vec![left, right];
        let mid: ArrowSelector = ArrowSelector::new(vec!["s".to_string()], Point{x:20, y:1}, Point{x:10, y:10});
        let mut arrow_selectors = vec![mid];
        let mut items = congregatge_selectables(&mut buttons, &mut arrow_selectors);

        let point = UiUtils::go_down(&mut items, Point {x:0, y: 0});
        assert_eq!(Point{x:0, y:1}, point);
        let point = UiUtils::go_down(&mut items, point);
        assert_eq!(Point{x:20, y:1}, point);
        let point = UiUtils::go_down(&mut items, point);
        assert_eq!(Point{x:40, y:1}, point);
    }

    #[test]
    fn bottom_up_b_b() {
        let top: Button = Button::new(Point{x:0, y:1}, Point {x:10, y:10}, None);
        let bottom: Button = Button::new(Point{x: 0,y: 10}, Point{x: 100, y:100}, None);
        let mut buttons = vec![top, bottom];
        let mut selectors = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut selectors);

        let point = UiUtils::go_up(&mut items, Point{x: 0, y:0});
        assert_eq!(Point{x:0, y:10}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:0, y:1}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:0, y:10}, point);
    }

    #[test]
    fn bottom_up_b_b_inverse() {
        let top: Button = Button::new(Point{x:0, y:1}, Point {x:10, y:10}, None);
        let bottom: Button = Button::new(Point{x: 0,y: 10}, Point{x: 100, y:100}, None);
        let mut buttons = vec![bottom, top];
        let mut selectors = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut selectors);

        let point = UiUtils::go_up(&mut items, Point{x: 0, y:0});
        assert_eq!(Point{x:0, y:10}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:0, y:1}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:0, y:10}, point);
    }

    #[test]
    fn bottom_up_a_a() {
        let top: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x:0, y:1}, Point{x:10, y:10});
        let bottom: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x:0, y:100}, Point{x:10, y:10});
        let mut arrow_selectors = vec![top, bottom];
        let mut buttons: Vec<Button> = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut arrow_selectors);

        let point = UiUtils::go_up(&mut items, Point{x:0,y:0});
        assert_eq!(Point{x:0, y:100}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:0, y:1}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:0, y:100}, point);
    }

    #[test]
    fn bottom_up_a_a_inverse() {
        let top: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x:0, y:1}, Point{x:10, y:10});
        let bottom: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x:0, y:100}, Point{x:10, y:10});
        let mut arrow_selectors = vec![bottom, top];
        let mut buttons: Vec<Button> = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut arrow_selectors);

        let point = UiUtils::go_up(&mut items, Point{x:0,y:0});
        assert_eq!(Point{x:0, y:100}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:0, y:1}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:0, y:1}, point);
    }

    #[test]
    fn bottom_up_same_y_b_b() {
        let left: Button = Button::new(Point{x: 10, y:10}, Point{x:10,y:10}, None);
        let right: Button = Button::new(Point{x: 100, y:10}, Point{x:10,y:10}, None);
        let mut buttons = vec![left, right];
        let mut arrow_selectors: Vec<ArrowSelector> = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut arrow_selectors);

        let point = UiUtils::go_up(&mut items, Point{x:0, y:0});
        assert_eq!(Point{x:100, y:10}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:10, y:10}, point);
    }

    #[test]
    fn bottom_up_same_y_a_a() {
        let left: ArrowSelector = ArrowSelector::new(vec!["s".to_string()], Point{x: 10, y: 10}, Point{x: 10, y:10});
        let right: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x: 30, y: 10}, Point{x: 10, y: 10});
        let mut arrow_selectors = vec![left, right];
        let mut buttons: Vec<Button> = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut arrow_selectors);

        let point = UiUtils::go_up(&mut items, Point{x:0, y:0});
        assert_eq!(Point{x:30, y:10}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:10, y:10}, point);
    }

    #[test]
    fn bottom_up_same_y_a_a_inverse() {
        let left: ArrowSelector = ArrowSelector::new(vec!["s".to_string()], Point{x: 10, y: 10}, Point{x: 10, y:10});
        let right: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x: 30, y: 10}, Point{x: 10, y: 10});
        let mut arrow_selectors = vec![right, left];
        let mut buttons: Vec<Button> = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut arrow_selectors);

        let point = UiUtils::go_up(&mut items, Point{x:0, y:0});
        assert_eq!(Point{x:30, y:10}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:10, y:10}, point);
    }

    #[test]
    fn bottom_up_same_y_b_b_inverse() {
        let left: Button = Button::new(Point{x: 10, y:10}, Point{x:10,y:10}, None);
        let right: Button = Button::new(Point{x: 100, y:10}, Point{x:10,y:10}, None);
        let mut buttons = vec![right, left];
        let mut arrow_selectors: Vec<ArrowSelector> = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut arrow_selectors);

        let point = UiUtils::go_up(&mut items, Point{x:0, y:0});
        assert_eq!(Point{x:100, y:10}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:10, y:10}, point);
    }

    #[test]
    fn bottom_up_3b_same_y() {
        let left: Button = Button::new(Point{x: 10, y:1}, Point{x: 0, y:0}, None);
        let mid: Button = Button::new(Point{x: 20, y:1}, Point{x: 0, y:0}, None);
        let right: Button = Button::new(Point{x: 30, y:1}, Point{x: 0, y:0}, None);
        let mut buttons = vec![left, mid, right];
        let mut arrow_selectors: Vec<ArrowSelector> = Vec::new();
        let mut items = congregatge_selectables(&mut buttons, &mut arrow_selectors);

        let point = UiUtils::go_up(&mut items, Point{x:0, y:0});
        assert_eq!(Point{x:30, y:1}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:20, y:1}, point);
        let point = UiUtils::go_up(&mut items, point);
        assert_eq!(Point{x:10, y:1}, point);
    }
}