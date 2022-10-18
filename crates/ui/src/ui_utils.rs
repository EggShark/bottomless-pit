use utils::Point;


pub trait Selectable {
    fn get_pos(&self) -> Point;
    fn select(&mut self);
    fn deslect(&mut self);
}

type Selectables<'a> = Vec<&'a mut dyn Selectable>;

#[derive(Debug, PartialEq)]
pub enum SelectableUiElements {
    ArrowSelector,
    Button,
}


pub struct UiUtils;

impl UiUtils {
    // make sure go_down and go_up are fed a SORTED array!!
    pub fn go_down(items: &mut Selectables, current_selected: usize) -> usize {
        let element = items.get_mut(current_selected + 1);
        match element {
            Some(element) => {
                element.select();
                current_selected + 1
            },
            None => {
                items[0].select();
                0
            }
        }
    }
    
    pub fn go_up(items: &mut Selectables, current_selected: usize) -> usize {
        let x = if current_selected == 0 {
            items.len() - 1
        } else {
            current_selected - 1
        };

        let element = items.get_mut(x);
        match element {
            Some(element) => {
                element.select();
                x
            },
            None => {
                items[x].select();
                x
            }
        }
    }
}