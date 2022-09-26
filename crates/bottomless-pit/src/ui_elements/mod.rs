mod button;
mod arrow_selection;
mod ui_scene;

use raylib::ffi::DrawEllipse;
use utils::Point;
use super::game::GameState;

pub use button::Button;
pub use arrow_selection::ArrowSelector;
pub use ui_scene::UiScene;

#[derive(Debug)]
pub enum SelectableUiElements {
    ArrowSelector,
    Button,
}

pub struct UiUtils;

impl UiUtils {
    pub fn advance(buttons: &Vec<Button>, arrow_selectors: &Vec<ArrowSelector>, current_selected: Point) -> (usize, SelectableUiElements) {
        let mut a_dist:i32 = i32::MAX;
        let mut b_dist: i32 = i32::MAX;
        let mut array_pos: usize = 0;
        let mut a_pos: usize = 0;
        let mut b_pos: usize = 0;
        let mut select_type = SelectableUiElements::Button;

        for i in 0..buttons.len() {
            let temp_distacne = buttons[i].get_pos().y - current_selected.y;
            if !(temp_distacne <= 0) {
                if temp_distacne < b_dist {
                    b_pos = i;
                    b_dist = temp_distacne;
                }
            } else if buttons[i].get_pos() != current_selected && buttons[i].get_pos().x > current_selected.x {
                    b_pos = i;
                    b_dist = temp_distacne;
            }
        }

        for i in 0..arrow_selectors.len() {
            let temp_distacne = arrow_selectors[i].get_pos().y - current_selected.y;
            let pos = arrow_selectors[i].get_pos();
            if !(temp_distacne <= 0) {
                if temp_distacne < a_dist {
                    a_pos = i;
                    a_dist = temp_distacne;
                }
            } else if pos != current_selected && pos.x > current_selected.x {
                a_pos = i;
                a_dist = temp_distacne;
                
            }
        }

        if a_dist == i32::MAX && b_dist < 0 {
            return Self::advance(buttons, arrow_selectors, Point{x: 0, y: 0});
        } else if b_dist == i32::MAX && b_dist < 0{
            return Self::advance(buttons, arrow_selectors, Point{x: 0, y: 0});
        } else if b_dist < 0 && a_dist < 0 {
            return Self::advance(buttons, arrow_selectors, Point{x: 0, y: 0});
        }
        else if a_dist < 0 {
            array_pos = b_pos;
            select_type = SelectableUiElements::Button;
        } else if b_dist < 0{
            array_pos = a_pos;
            select_type = SelectableUiElements::ArrowSelector;
        } else if a_dist < b_dist {
            array_pos = a_pos;
            select_type = SelectableUiElements::ArrowSelector;
        } else {
            array_pos = b_pos;
            select_type = SelectableUiElements::Button;
        }

        (array_pos, select_type)
    }
    
    pub fn go_back(buttons: &Vec<Button>, arrow_selectors: &Vec<ArrowSelector>, current_selected: Point) -> (usize, SelectableUiElements) {
        let mut a_dist:i32 = i32::MAX;
        let mut b_dist: i32 = i32::MAX;
        let mut array_pos: usize = 0;
        let mut a_pos: usize = 0;
        let mut b_pos: usize = 0;
        let mut select_type = SelectableUiElements::Button;

        for i in 0..buttons.len() {
            let temp_distacne = current_selected.y - buttons[i].get_pos().y;
            println!("{}, {}", temp_distacne, b_dist);
            if !(temp_distacne >= 0) {
                if temp_distacne < b_dist {
                    b_pos = i;
                    b_dist = temp_distacne;
                }
            } else if buttons[i].get_pos() != current_selected && buttons[i].get_pos().x > current_selected.x {
                    b_pos = i;
                    b_dist = temp_distacne;
            }
        }

        for i in 0..arrow_selectors.len() {
            let temp_distacne = current_selected.y - arrow_selectors[i].get_pos().y;
            println!("{}, {}", temp_distacne, a_dist);
            if !(temp_distacne >= 0) {
                if temp_distacne < b_dist {
                    a_pos = i;
                    a_dist = temp_distacne;
                }
            } else if arrow_selectors[i].get_pos() != current_selected && arrow_selectors[i].get_pos().x > current_selected.x {
                    a_pos = i;
                    a_dist = temp_distacne;
            }
        }

        println!("{}, {}", a_dist, b_dist);


        if a_dist < 0 {
            array_pos = b_pos;
            select_type = SelectableUiElements::Button;
        } else if b_dist < 0{
            array_pos = a_pos;
            select_type = SelectableUiElements::ArrowSelector;
        } else if a_dist < b_dist {
            array_pos = a_pos;
            select_type = SelectableUiElements::ArrowSelector;
        } else {
            array_pos = b_pos;
            select_type = SelectableUiElements::Button;
        }
    
        (array_pos, select_type)
    }
}