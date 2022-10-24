use raylib::prelude::*;
use utils::{GameState, Point};
use ui::{UiScene};
use super::settings::{Settings, Resolutions};
use super::player::player::Player;

#[derive(Debug)]
pub struct Game { 
    state: GameState,
    pub settings: Settings,
    ui_scene: UiScene,
    player: Option<Player>
}

impl Game {
    pub fn new(settings: Settings) -> Self {
        let ui_scene = UiScene::from_game_state(&GameState::default());
        Self {
            state: GameState::default(),
            settings,
            ui_scene,
            player: None,
        }
    }

    pub fn update(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        // the logic loop for the game
        match self.state {
            GameState::MainMenu => {
                self.main_menu_update(handle, thread);
            },
            GameState::SettingsMenu => {
                self.settings_update(handle);
            },
            GameState::Ingame => {
                println!("look ma im in game");
            },
            GameState::Testing => {
                self.testing_update(handle);
            }
            GameState::Quit => unreachable!()
        }
    }

    pub fn draw(&self, mut d_handle: RaylibDrawHandle) {
        // the drawing loop for the game
        d_handle.clear_background(Color::GREEN);
        self.ui_scene.draw(&mut d_handle);

        match &self.player {
            Some(player) => {
                player.draw(&mut d_handle);
            }
            None => {},
        }
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

    fn into_testing(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.state = GameState::Testing;
        self.ui_scene = UiScene::default();
        self.player = Some(Player::make_baller(rl, thread, Point{x: 0, y: 50}));
    }

    fn main_menu_update(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        self.ui_scene.slection_check(handle);

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

        if self.ui_scene.buttons[3].was_clicked(handle) {
            return self.into_testing(handle, thread);
        }
    }

    fn settings_update(&mut self, handle: &mut RaylibHandle) {
        self.ui_scene.slection_check(handle);
        
        for x in 0..self.ui_scene.selectors.len() {
            self.ui_scene.selectors[x].update(handle);
        }

        if self.ui_scene.buttons[0].was_clicked(handle) {
            self.into_main();
        }

        if self.ui_scene.buttons[1].was_clicked(handle) {
            self.apply_settings(handle);
        }
    }

    fn apply_settings(&mut self, rl: &mut RaylibHandle) {
        let mut selections: Vec<i8> = Vec::new();
        for x in 0..self.ui_scene.selectors.len() {
            selections.push(self.ui_scene.selectors[x].get_curr_selection());
        }

        let resolution = Resolutions::from(selections[0] as u8);

        let (width, height) = resolution.len_width();

        let volume: u8 = selections[1] as u8 + 1;
        rl.set_window_size(width as i32, height as i32);
        self.settings.update_settings(resolution, volume);
    }

    // quick and dirty way to put stuff for testing
    fn testing_update(&mut self, rl: &mut RaylibHandle) {
        self.player.as_mut()
            .unwrap()
            .update(rl, &self.settings.keys);
    }
}