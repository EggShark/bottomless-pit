use raylib::prelude::*;
use super::ui_elements::Button;

pub struct Game { 
    state: GameState,
    buttons: Vec<Button>,
}

#[derive(PartialEq)]
pub enum GameState {
    MainMenu,
    Ingame,
    Quit,
}

impl Default for GameState {
    fn default() -> Self {
        Self::MainMenu
    }
}

impl Game {
    pub fn new() -> Self {
        let buttons = init_main();
        Self {
            state: GameState::default(),
            buttons,
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle) {
        // the logic loop for the game
        match self.state {
            GameState::MainMenu => {
                if self.buttons[0].was_clicked(handle) {
                    self.state = GameState::Quit;
                }
                if self.buttons[1].was_clicked(handle) {
                    // TODO replace with a transition functionn for deloading and stuff
                    self.state = GameState::Ingame;
                }
            },
            GameState::Ingame => {
                println!("look ma im in game");
            },
            GameState::Quit => unreachable!()
        }
    }

    pub fn draw(&self, mut drawer: RaylibDrawHandle) {
        // the drawing loop for the game
        drawer.clear_background(Color::GREEN);
        for button in self.buttons.iter() {
            button.draw(&mut drawer);
        }
    }

    pub fn close_check(&self) -> bool{
        self.state == GameState::Quit
    }
}

fn init_main() -> Vec<Button> {
    let quit = Button::new((10, 10), (100, 40), Some("Quit".to_string()));
    let go_to_game = Button::new((10, 80), (100, 40), Some("to game".to_string()));

    vec![quit, go_to_game]
}