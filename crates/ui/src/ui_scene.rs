use raylib::{drawing::RaylibDrawHandle, RaylibHandle};
use raylib::consts::KeyboardKey;
use utils::{Point, GameState};
use crate::button::Button;
use crate::arrow_selection::ArrowSelector;
use crate::ui_utils::{UiUtils, SelectableUiElements, Slectable};

#[derive(Debug, PartialEq)]
pub struct UiScene {
    pub buttons: Vec<Button>,
    pub selectors: Vec<ArrowSelector>,
    pub current_selection: Point,
}

impl Default for UiScene {
    fn default() -> Self {
        Self {
            buttons: Vec::new(),
            selectors: Vec::new(),
            current_selection: Point{x:0,y:0}
        }
    }
}

impl UiScene {
    fn init_main() -> Self {
        let quit = Button::new(Point{x: 10, y: 10}, Point{x: 100, y:40}, Some("Quit".to_string()));
        let go_to_game = Button::new(Point{x:10, y:80}, Point{x:100, y:40}, Some("to game".to_string()));
        let go_to_settings = Button::new(Point{x:10, y:180}, Point{x:100, y:40}, Some("Settings".to_string()));
        let test: Button = Button::new(Point {x: 800, y: 180}, Point{x: 100, y:40}, None);
        let mid_test: Button = Button::new(Point{x: 400, y: 180}, Point{x: 100, y:40}, None);

        let buttons = vec![quit, go_to_game, go_to_settings, mid_test, test];

        let yat = ArrowSelector::new(vec![String::from("a")], Point{x: 200, y:1}, Point{x:400, y:100});
        let arrow_test = ArrowSelector::new(vec![String::from("a")], Point{x: 200, y:600}, Point{x:400, y:100});

        Self {
            buttons,
            selectors: vec![arrow_test, yat],
            current_selection: Point{x:0,y:0},
        }
    }

    fn init_settings_menu() -> Self {
        let back = Button::new(Point{x:200, y:400}, Point{x:100, y:40}, Some("Go Back".to_string()));
        let submit = Button::new(Point{x:400, y:400}, Point{x:100, y:40}, Some("Apply".to_string()));
        let test = Button::new(Point {x: 100, y: 600}, Point {x: 100, y: 40}, None);
        
        let buttons = vec![back, submit, test];

        let resolution = ArrowSelector::new(vec![String::from("1920x1080"), String::from("1280x720"), String::from("854x360")], Point{x:200, y:0}, Point{x:400, y:100});
        let voloptions: Vec<String> = vec![String::from("1"), String::from("2"), String::from("3"), String::from("4"), String::from("5"), String::from("6"), String::from("7"), String::from("8"), String::from("9"), String::from("10")];
        let volume = ArrowSelector::new(voloptions, Point{x:200, y:200}, Point{x:400, y:100});

        let selectors = vec![resolution, volume];

        Self {
            buttons,
            selectors,
            current_selection: Point{x:0, y:0},
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
        let current_selction = self.current_selection;
        deslect(&mut self.buttons, &mut self.selectors, current_selction);
        let mut selectables = congregatge_selectables(&mut self.buttons, &mut self.selectors);
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            let new_selection = UiUtils::go_down(&mut selectables, current_selction);
            self.current_selection = new_selection;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            let new_selection = UiUtils::go_up(&mut selectables, current_selction);
            self.current_selection = new_selection;
        }
    }
}

fn deslect(buttons: &mut Vec<Button>, selectors: &mut Vec<ArrowSelector>, cur_selction: Point) {
    for z in buttons.iter_mut() {
        if z.get_pos() != cur_selction {
            z.deslect();
        }
    }
    for x in selectors.iter_mut() {
        if x.get_pos() != cur_selction {
            x.deslect();
        }
    }
}

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