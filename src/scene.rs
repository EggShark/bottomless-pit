use super::UiElements;
use raylib::drawing::RaylibDrawHandle;

pub struct Scene {
    elements: Vec<UiElements>,
}

impl Scene {
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

    pub fn testing(&self, drawer: &mut RaylibDrawHandle) {
        for element in self.elements.iter() {
            element.draw(drawer);
        }
    }
}