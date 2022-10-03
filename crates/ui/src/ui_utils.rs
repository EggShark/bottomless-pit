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
    pub fn advance(items: &mut Selectables, current_selected: Point) {
        // let mut x_dist = i32::MAX;
        // for i in 0..buttons.len() {
        //     if buttons[i].get_pos() != current_selected {
        //         // we cast as a u32 bc a negative will wrap around to u32 max - whatever it was absolute value
        //         // this makes the negative values the farthest ones away!
        //         let temp_distacne = (buttons[i].get_pos().y - current_selected.y) as u32;
        //         if temp_distacne == 0 && buttons[i].get_pos().y == current_selected.y {
        //             // dont use type casting here as I do not want this to loop around
        //             let temp_x = buttons[i].get_pos().x - current_selected.x;
        //             if temp_x < x_dist && temp_x > 0 {
        //                 b_pos = i;
        //                 b_dist = temp_distacne;
        //                 x_dist = temp_x;
        //             }
        //         } else if temp_distacne == b_dist {
        //             let temp_x = buttons[i].get_pos().x - current_selected.x;
        //             if temp_x < x_dist && temp_x > 0 {
        //                 x_dist = temp_x;
        //             }
        //         } else if temp_distacne < b_dist && temp_distacne != 0 {
        //             b_pos = i;
        //             b_dist = temp_distacne;
        //         }
        //     }
        // }
        items[0].select();
    }
    
    pub fn go_back(items: &Selectables, current_selected: Point) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn above_to_bellow_b_to_b() {
        let top: Button = Button::new(Point{x:0, y:1}, Point {x:10, y:10}, None);
        let bottom: Button = Button::new(Point{x: 0,y: 10}, Point{x: 100, y:100}, None);
        let buttons = vec![top, bottom];

        let (pos, _) = UiUtils::advance(&buttons, &Vec::new(), Point{x:0, y:0}); // simulates selecting the first
        let (pos, kind) = UiUtils::advance(&buttons, &Vec::new(), buttons[pos].get_pos()); // then selects the bottom

        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
    }

    #[test]
    fn above_to_bellow_b_to_b_inverse_array() {
        let top: Button = Button::new(Point{x:0, y:1}, Point {x:10, y:10}, None);
        let bottom: Button = Button::new(Point{x: 0,y: 10}, Point{x: 100, y:100}, None);
        let buttons = vec![bottom, top];

        let (pos, _) = UiUtils::advance(&buttons, &Vec::new(), Point{x:0, y:0});
        let (pos, kind) = UiUtils::advance(&buttons, &Vec::new(), buttons[pos].get_pos());

        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
    }

    #[test]
    fn same_y_2_buttons() {
        let left: Button = Button::new(Point{x: 0, y: 1}, Point{x:10, y:10}, None);
        let right: Button = Button::new(Point{x: 100, y: 1}, Point{x:10, y:10}, None);

        let buttons = vec![left, right];

        let (pos, kind) = UiUtils::advance(&buttons, &Vec::new(), Point{x:0,y:0});
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::advance(&buttons, &Vec::new(), buttons[pos].get_pos());
        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
    }

    #[test]
    fn same_y_3_buttons() {
        let left: Button = Button::new(Point{x: 0, y: 1}, Point{x:10, y:10}, None);
        let mid: Button = Button::new(Point{x: 100, y: 1}, Point{x:10, y:10}, None);
        let right: Button = Button::new(Point{x: 300, y: 1}, Point{x:10, y:10}, None);

        let buttons = vec![left, right, mid];

        let (pos, kind) = UiUtils::advance(&buttons, &Vec::new(), Point{x:0,y:0});
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::advance(&buttons, &Vec::new(), buttons[pos].get_pos());
        assert_eq!((2, SelectableUiElements::Button), (pos, kind));
    }

    #[test]
    fn same_y_3_buttons_diff_order() {
        let left: Button = Button::new(Point{x: 0, y: 1}, Point{x:10, y:10}, None);
        let mid: Button = Button::new(Point{x: 100, y: 1}, Point{x:10, y:10}, None);
        let right: Button = Button::new(Point{x: 300, y: 1}, Point{x:10, y:10}, None);

        let buttons = vec![left, mid, right];

        let (pos, kind) = UiUtils::advance(&buttons, &Vec::new(), Point{x:0,y:0});
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::advance(&buttons, &Vec::new(), buttons[pos].get_pos());
        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
    }
    
    #[test]
    fn b_a_b_same_y_level() {
        let left: Button = Button::new(Point {x:0, y: 1}, Point{x:10, y:10}, None);
        let right: Button = Button::new(Point {x:40, y: 1}, Point{x:10, y:10}, None);

        let buttons = vec![left, right];

        let mid: ArrowSelector = ArrowSelector::new(vec!["s".to_string()], Point{x:20, y:1}, Point{x:10, y:10});
        let arrow_selectors = vec![mid];

        let (pos, kind) = UiUtils::advance(&buttons, &arrow_selectors, Point {x:0, y: 0});
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::advance(&buttons, &arrow_selectors, buttons[pos].get_pos());
        assert_eq!((0, SelectableUiElements::ArrowSelector), (pos, kind));
        let (pos, kind) = UiUtils::advance(&buttons, &arrow_selectors, arrow_selectors[pos].get_pos());
        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
    }

    #[test]
    fn a_b_a_same_y_level() {
        let mid: Button = Button::new(Point {x: 40, y: 1}, Point {x: 10, y: 10}, None);
        let buttons = vec![mid];
        let left: ArrowSelector =  ArrowSelector::new(vec!["a".to_string()], Point{x:0, y: 1}, Point{x:10, y:10});
        let right: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x:100, y: 1}, Point{x:10, y:10});
        let arrow_selectors = vec![left, right];

        let (pos, kind) = UiUtils::advance(&buttons, &arrow_selectors, Point{x: 0, y:0});
        assert_eq!((0, SelectableUiElements::ArrowSelector), (pos, kind));
        let (pos, kind) = UiUtils::advance(&buttons, &arrow_selectors, arrow_selectors[pos].get_pos());
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::advance(&buttons, &arrow_selectors, buttons[pos].get_pos());
        assert_eq!((1, SelectableUiElements::ArrowSelector), (pos, kind));
    }

    #[test]
    fn bottom_up_b_b() {
        let top: Button = Button::new(Point{x:0, y:1}, Point {x:10, y:10}, None);
        let bottom: Button = Button::new(Point{x: 0,y: 10}, Point{x: 100, y:100}, None);
        let buttons = vec![top, bottom];

        let (pos, kind) = UiUtils::go_back(&buttons, &Vec::new(), Point{x: 0, y:0});
        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &Vec::new(), buttons[pos].get_pos());
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &Vec::new(), buttons[pos].get_pos());
        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
    }

    #[test]
    fn bottom_up_b_b_inverse() {
        let top: Button = Button::new(Point{x:0, y:1}, Point {x:10, y:10}, None);
        let bottom: Button = Button::new(Point{x: 0,y: 10}, Point{x: 100, y:100}, None);
        let buttons = vec![bottom, top];

        let (pos, kind) = UiUtils::go_back(&buttons, &Vec::new(), Point{x: 0, y:0});
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &Vec::new(), buttons[pos].get_pos());
        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &Vec::new(), buttons[pos].get_pos());
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
    }

    #[test]
    fn bottom_up_a_a() {
        let top: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x:0, y:1}, Point{x:10, y:10});
        let bottom: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x:0, y:100}, Point{x:10, y:10});
        let arrow_selectors = vec![top, bottom];
        let buttons: Vec<Button> = Vec::new();

        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, Point{x:0,y:0});
        assert_eq!((1, SelectableUiElements::ArrowSelector), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, arrow_selectors[pos].get_pos());
        assert_eq!((0, SelectableUiElements::ArrowSelector), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, arrow_selectors[pos].get_pos());
        assert_eq!((1, SelectableUiElements::ArrowSelector), (pos, kind));
    }

    #[test]
    fn bottom_up_a_a_inverse() {
        let top: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x:0, y:1}, Point{x:10, y:10});
        let bottom: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x:0, y:100}, Point{x:10, y:10});
        let arrow_selectors = vec![bottom, top];
        let buttons: Vec<Button> = Vec::new();

        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, Point{x:0,y:0});
        assert_eq!((0, SelectableUiElements::ArrowSelector), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, arrow_selectors[pos].get_pos());
        assert_eq!((1, SelectableUiElements::ArrowSelector), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, arrow_selectors[pos].get_pos());
        assert_eq!((0, SelectableUiElements::ArrowSelector), (pos, kind));
    }

    #[test]
    fn bottom_up_same_y_b_b() {
        let left: Button = Button::new(Point{x: 10, y:10}, Point{x:10,y:10}, None);
        let right: Button = Button::new(Point{x: 100, y:10}, Point{x:10,y:10}, None);
        let buttons = vec![left, right];
        let arrow_selectors: Vec<ArrowSelector> = Vec::new();

        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, Point{x:0, y:0});
        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, buttons[pos].get_pos());
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
    }

    #[test]
    fn bottom_up_same_y_a_a() {
        let left: ArrowSelector = ArrowSelector::new(vec!["s".to_string()], Point{x: 10, y: 10}, Point{x: 10, y:10});
        let right: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x: 30, y: 10}, Point{x: 10, y: 10});
        let arrow_selectors = vec![left, right];
        let buttons: Vec<Button> = Vec::new();

        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, Point{x:0, y:0});
        assert_eq!((1, SelectableUiElements::ArrowSelector), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, arrow_selectors[pos].get_pos());
        assert_eq!((0, SelectableUiElements::ArrowSelector), (pos, kind));
    }

    #[test]
    fn bottom_up_same_y_a_a_inverse() {
        let left: ArrowSelector = ArrowSelector::new(vec!["s".to_string()], Point{x: 10, y: 10}, Point{x: 10, y:10});
        let right: ArrowSelector = ArrowSelector::new(vec!["a".to_string()], Point{x: 30, y: 10}, Point{x: 10, y: 10});
        let arrow_selectors = vec![right, left];
        let buttons: Vec<Button> = Vec::new();

        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, Point{x:0, y:0});
        assert_eq!((0, SelectableUiElements::ArrowSelector), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, arrow_selectors[pos].get_pos());
        assert_eq!((1, SelectableUiElements::ArrowSelector), (pos, kind));
    }

    #[test]
    fn bottom_up_same_y_b_b_inverse() {
        let left: Button = Button::new(Point{x: 10, y:10}, Point{x:10,y:10}, None);
        let right: Button = Button::new(Point{x: 100, y:10}, Point{x:10,y:10}, None);
        let buttons = vec![right, left];
        let arrow_selectors: Vec<ArrowSelector> = Vec::new();

        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, Point{x:0, y:0});
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, buttons[pos].get_pos());
        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
    }

    #[test]
    fn bottom_up_3b_same_y() {
        let left: Button = Button::new(Point{x: 10, y:1}, Point{x: 0, y:0}, None);
        let mid: Button = Button::new(Point{x: 20, y:1}, Point{x: 0, y:0}, None);
        let right: Button = Button::new(Point{x: 30, y:1}, Point{x: 0, y:0}, None);
        let buttons = vec![left, mid, right];
        let arrow_selectors: Vec<ArrowSelector> = Vec::new();

        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, Point{x:0, y:0});
        assert_eq!((2, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, buttons[pos].get_pos());
        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
        let (pos, kind) = UiUtils::go_back(&buttons, &arrow_selectors, buttons[pos].get_pos());
        assert_eq!((0, SelectableUiElements::Button), (pos, kind));
    }
}