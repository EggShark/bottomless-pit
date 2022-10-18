use raylib::{drawing::RaylibDrawHandle, RaylibHandle};
use raylib::consts::KeyboardKey;
use utils::{Point, GameState};
use crate::button::Button;
use crate::arrow_selection::ArrowSelector;
use crate::ui_utils::{UiUtils, Selectable};

#[derive(Debug, PartialEq)]
pub struct UiScene {
    pub buttons: Vec<Button>,
    pub selectors: Vec<ArrowSelector>,
    pub current_selection: usize,
    first_selection: bool
}

impl Default for UiScene {
    fn default() -> Self {
        Self {
            buttons: Vec::new(),
            selectors: Vec::new(),
            current_selection: 0,
            first_selection: true,
        }
    }
}

impl UiScene {
    fn init_main() -> Self {
        let quit = Button::new(Point{x: 10, y: 10}, Point{x: 100, y:40}, Some("Quit".to_string()));
        let go_to_game = Button::new(Point{x:10, y:80}, Point{x:100, y:40}, Some("to game".to_string()));
        let go_to_settings = Button::new(Point{x:10, y:180}, Point{x:100, y:40}, Some("Settings".to_string()));
        let buttons = vec![quit, go_to_game, go_to_settings];

        Self {
            buttons,
            selectors: Vec::new(),
            current_selection: 0,
            first_selection: true,
        }
    }

    fn init_settings_menu() -> Self {
        let back = Button::new(Point{x:200, y:400}, Point{x:100, y:40}, Some("Go Back".to_string()));
        let submit = Button::new(Point{x:400, y:400}, Point{x:100, y:40}, Some("Apply".to_string()));
        
        let buttons = vec![back, submit];

        let resolution = ArrowSelector::new(vec![String::from("1920x1080"), String::from("1280x720"), String::from("854x360")], Point{x:200, y:0}, Point{x:400, y:100});
        let voloptions: Vec<String> = vec![String::from("1"), String::from("2"), String::from("3"), String::from("4"), String::from("5"), String::from("6"), String::from("7"), String::from("8"), String::from("9"), String::from("10")];
        let volume = ArrowSelector::new(voloptions, Point{x:200, y:200}, Point{x:400, y:100});

        let selectors = vec![resolution, volume];

        Self {
            buttons,
            selectors,
            current_selection: 0,
            first_selection: true,
        }
    }

    pub fn from_game_state(state: &GameState) -> Self {
        match state {
            GameState::MainMenu => {
                Self::init_main()
            }
            GameState::SettingsMenu => {
                Self::init_settings_menu()
            }
            _ => {
                Self::default()
            }
        }
    }   

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        for button in self.buttons.iter() {
            button.draw(d_handle);
        }

        for selector in self.selectors.iter() {
            selector.draw(d_handle);
        } 
    }

    pub fn slection_check(&mut self, rl: &RaylibHandle) {
        // this allows for mouseless navigtion of the menus
        // by first sorting the array and then depedning on
        // which key was pressed increment down or increment
        // up through the list
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            if self.first_selection {
                self.first_selection = false;
                let mut selectables = congregatge_selectables(&mut self.buttons, &mut self.selectors);
                selectables[0].select();
                // fixes a bug where it would selected the second element first as it would just advance
                // to the second element 'ignoreing' the first
            } else {
                let mut selectables = congregatge_selectables(&mut self.buttons, &mut self.selectors);
                let new_selection = UiUtils::go_down(&mut selectables, self.current_selection);
                self.current_selection = new_selection;
                deslect(&mut selectables, self.current_selection);
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            let mut selectables = congregatge_selectables(&mut self.buttons, &mut self.selectors);
            let new_selection = UiUtils::go_up(&mut selectables, self.current_selection);
            self.current_selection = new_selection;
            deslect(&mut selectables, self.current_selection);
        }
    }
}

fn deslect(elements: &mut Vec<&mut dyn Selectable>, current_selected: usize) {
    for i in 0..elements.len() {
        if i != current_selected {
            elements[i].deslect();
        }
    }
}

fn congregatge_selectables<'b>(buttons: &'b mut Vec<Button>, selectors: &'b mut Vec<ArrowSelector>) -> Vec<&'b mut dyn Selectable> {
    let mut selectables: Vec<&'b mut dyn Selectable> = Vec::new();
    for z in buttons.iter_mut() {
        selectables.push(z);
    }
    for x in selectors.iter_mut() {
        selectables.push(x);
    }
    let selectables = sort_by_points(selectables);
    selectables
}

fn sort_by_points(mut list: Vec<&mut dyn Selectable>) -> Vec<&mut dyn Selectable> {
    list.sort_by(|a, b| {
        let y_order = a.get_pos().y.cmp(&b.get_pos().y);
        if y_order == std::cmp::Ordering::Equal {
            a.get_pos().x.cmp(&b.get_pos().x)
        } else {
            y_order
        }
    });
    list
}