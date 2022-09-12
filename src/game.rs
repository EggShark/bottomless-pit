use raylib::prelude::*;
use super::ui_elements::UiScene;

#[derive(Debug, PartialEq)]
pub struct Game { 
    state: GameState,
    ui_scene: UiScene,
}

#[derive(Debug, PartialEq)]
pub enum GameState {
    MainMenu,
    SettingsMenu,
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
        let ui_scene = UiScene::init_main();
        Self {
            state: GameState::default(),
            ui_scene,
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle) {
        // the logic loop for the game
        match self.state {
            GameState::MainMenu => {
                self.main_menu_update(handle);
            },
            GameState::SettingsMenu => {
                
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
        self.ui_scene.draw(&mut drawer);
    }

    pub fn set_state(&mut self, state: GameState) {
        self.state =  state;
    }

    pub fn should_close(&self, rl: &RaylibHandle) -> bool{
        self.state == GameState::Quit || rl.window_should_close()
    }

    pub fn into_game(&mut self) {
        // load what the game needs here
        self.state = GameState::Ingame;
        self.ui_scene = UiScene::default();
    }

    fn main_menu_update(&mut self, handle: &RaylibHandle) {
        if self.ui_scene.buttons[0].was_clicked(handle) {
            self.state = GameState::Quit;
        }

        if self.ui_scene.buttons[1].was_clicked(handle) {
            self.into_game();
        }
    }
}

// pub fn main_menu_update(&self, game: &mut Game, handle: &RaylibHandle) {
//     if self.buttons[0].was_clicked(handle) {
//         game.set_state(GameState::Quit);
//     }
//     if self.buttons[1].was_clicked(handle) {
//         // TODO replace with a transition functionn for deloading and stuff
//         game.into_game();
//     }
// }