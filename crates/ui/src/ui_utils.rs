use utils::Point;

use crate::button::Button;
use crate::arrow_selection::ArrowSelector;


#[derive(Debug, PartialEq)]
pub enum SelectableUiElements {
    ArrowSelector,
    Button,
}


pub struct UiUtils;

impl UiUtils {
    pub fn advance(buttons: &Vec<Button>, arrow_selectors: &Vec<ArrowSelector>, current_selected: Point) -> (usize, SelectableUiElements) {
        let mut a_dist: u32 = u32::MAX;
        let mut b_dist: u32 = u32::MAX;
        let mut array_pos: usize = 0;
        let mut a_pos: usize = 0;
        let mut b_pos: usize = 0;
        let mut select_type = SelectableUiElements::Button;

        for i in 0..buttons.len() {
            // we cast as a u32 bc a negative will wrap around to u32 max - whatever it was absolute value
            // this makes the negative values the farthest ones away!
            let temp_distacne = (buttons[i].get_pos().y - current_selected.y) as u32;
            if temp_distacne == 0 && buttons[i].get_pos().x > current_selected.x {
                b_pos = i;
                b_dist = temp_distacne;
            } else if temp_distacne < b_dist && temp_distacne != 0 {
                b_pos = i;
                b_dist = temp_distacne;
            }
        }

        for i in 0..arrow_selectors.len() {
            let temp_distacne = (arrow_selectors[i].get_pos().y - current_selected.y) as u32;
            if temp_distacne == 0 && arrow_selectors[i].get_pos().x > current_selected.x {
                a_pos = i;
                a_dist = temp_distacne;
            }
            else if temp_distacne < a_dist && temp_distacne != 0 {
                a_pos = i;
                a_dist = temp_distacne;
            }
        }

        if a_dist < b_dist {
            select_type = SelectableUiElements::ArrowSelector;
            array_pos = a_pos;
        } else {
            select_type = SelectableUiElements::Button;
            array_pos = b_pos;
        }

        (array_pos, select_type)
    }
    
    pub fn go_back(buttons: &Vec<Button>, arrow_selectors: &Vec<ArrowSelector>, current_selected: Point) -> (usize, SelectableUiElements) {
        let mut a_dist: u32 = u32::MAX;
        let mut b_dist: u32 = u32::MAX;
        let mut array_pos: usize = 0;
        let mut a_pos: usize = 0;
        let mut b_pos: usize = 0;
        let mut select_type = SelectableUiElements::Button;

        for i in 0..buttons.len() {
            let temp_distacne = (current_selected.y - buttons[i].get_pos().y) as u32;
            if temp_distacne == 0 && buttons[i].get_pos().x < current_selected.x {
                b_dist = temp_distacne;
                b_pos = i;
            } else if temp_distacne < b_dist && temp_distacne != 0 {
                b_dist = temp_distacne;
                b_pos = i;
            }
        }

        for i in 0..arrow_selectors.len() {
            let temp_distance: u32 = (current_selected.y - arrow_selectors[i].get_pos().y) as u32;
            if temp_distance == 0 && arrow_selectors[i].get_pos().x < current_selected.x {
                a_dist = temp_distance;
                a_pos = i;
            } else if temp_distance < a_dist && temp_distance != 0 {
                a_dist = temp_distance;
                a_pos = i;
            }
        }

        if a_dist < b_dist {
            select_type = SelectableUiElements::ArrowSelector;
            array_pos = a_pos;
        } else {
            select_type = SelectableUiElements::Button;
            array_pos = b_pos;
        }

        (array_pos, select_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn above_to_bellow_b_to_b() {
        let top: Button = Button::new(Point{x:0, y:0}, Point {x:10, y:10}, None);
        let bottom: Button = Button::new(Point{x: 0,y: 10}, Point{x: 100, y:100}, None);
        let buttons = vec![top, bottom];

        let (pos, kind) = UiUtils::advance(&buttons, &Vec::new(), Point{x:0, y:0});

        assert_eq!((1, SelectableUiElements::Button), (pos, kind));
    }
}