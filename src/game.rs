use raylib::prelude::*;
use super::ui_elements::UiScene;
use super::settings::Settings;

#[derive(Debug, PartialEq)]
pub struct Game { 
    state: GameState,
    settings: Settings,
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
    pub fn new(settings: Settings) -> Self {
        let ui_scene = UiScene::from_game_state(&GameState::default());
        Self {
            state: GameState::default(),
            settings,
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
                self.settings_update(handle);
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

    pub fn should_close(&self, rl: &RaylibHandle) -> bool{
        self.state == GameState::Quit || rl.window_should_close()
    }

    fn into_game(&mut self) {
        // load what the game needs here
        self.state = GameState::Ingame;
        self.ui_scene = UiScene::from_game_state(&self.state);
    }

    fn into_settings(&mut self) {
        self.state = GameState::SettingsMenu;
        self.ui_scene = UiScene::from_game_state(&self.state);
    }

    fn into_main(&mut self) {
        self.state = GameState::MainMenu;
        self.ui_scene = UiScene::from_game_state(&self.state);
    }

    fn main_menu_update(&mut self, handle: &RaylibHandle) {
        if self.ui_scene.buttons[0].was_clicked(handle) {
            self.state = GameState::Quit;
        }

        if self.ui_scene.buttons[1].was_clicked(handle) {
            return self.into_game();
            // need to return early to back out after clearing the vec
            // otherwise it will tried to read [2] when there is nothing there
        }

        if self.ui_scene.buttons[2].was_clicked(handle) {
            return self.into_settings();
        }
    }

    fn settings_update(&mut self, handle: &RaylibHandle) {
        for x in 0..self.ui_scene.selectors.len() {
            self.ui_scene.selectors[x].update(handle);
        }

        if self.ui_scene.buttons[0].was_clicked(handle) {
            self.into_main();
        }

        if self.ui_scene.buttons[1].was_clicked(handle) {
            self.apply_settings();
        }
    }

    fn apply_settings(&mut self) {
        let mut selections: Vec<i8> = Vec::new();
        for x in 0..self.ui_scene.selectors.len() {
            selections.push(self.ui_scene.selectors[x].get_curr_selection());
        }

        let (width, height): (u16, u16) = match selections[0] {
            0 => {
                (1920, 1080)
            }
            1 => {
                (1280, 720)
            }
            2 => {
                (854, 360)
            }
            _ => unreachable!(),
        };
        let volume: u8 = selections[1] as u8 + 1;

        self.settings.update_settings(width, height, volume);
    }
}