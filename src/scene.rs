use super::UiElements;
use super::Button;
use raylib::drawing::RaylibDrawHandle;

pub struct Menu {
    elements: Vec<UiElements>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            elements: Vec::new()
        }
    }

    pub fn from_vec(elements: Vec<UiElements>) -> Self {
        Self {
            elements
        }
    }

    pub fn add_item(&mut self, item: UiElements) {
        self.elements.push(item);
    }

    pub fn draw(&self, drawer: &mut RaylibDrawHandle) {
        for element in self.elements.iter() {
            element.draw(drawer);
        }
    }

    pub fn get_buttons(&self) -> Vec<&Button> {
        let mut buttons = Vec::new();
        for element in self.elements.iter() {
            match element {
                UiElements::Button(b) => buttons.push(b),
                _ => {}
            }
        }
        buttons
    }
}