use raylib::prelude::*;
use utils::{GameState, Point};
use ui::{UiScene};
use input_handler::Inputs;
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
        let ui_scene = UiScene::from_game_state(&GameState::default(), &settings.keys);
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
            GameState::KeySettings => {
                self.key_settings_update(handle);
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
                draw_healthbar(player, &mut d_handle);
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
        self.ui_scene = UiScene::from_game_state(&self.state, &self.settings.keys);
    }

    fn into_settings(&mut self) {
        self.state = GameState::SettingsMenu;
        self.ui_scene = UiScene::from_game_state(&self.state, &self.settings.keys);
    }

    fn into_key_settings(&mut self) {
        self.state = GameState::KeySettings;
        self.ui_scene = UiScene::from_game_state(&self.state, &self.settings.keys);
    }

    fn into_main(&mut self) {
        self.state = GameState::MainMenu;
        self.ui_scene = UiScene::from_game_state(&self.state, &self.settings.keys);
    }

    fn into_testing(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.state = GameState::Testing;
        self.ui_scene = UiScene::from_game_state(&self.state, &self.settings.keys);
        self.player = Some(Player::make_baller(rl, thread, Point{x: 0, y: 250}));
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
            return self.into_main();
        }

        if self.ui_scene.buttons[2].was_clicked(handle) {
            return self.into_key_settings();
        }

        if self.ui_scene.buttons[1].was_clicked(handle) {
            self.apply_settings(handle);
        }
    }

    fn key_settings_update(&mut self, handle: &mut RaylibHandle) {
        for changer in self.ui_scene.key_changers.iter_mut() {
            changer.update(handle);
        }

        if self.ui_scene.buttons[0].was_clicked(handle) {
            return self.into_settings();
        }

        if self.ui_scene.buttons[1].was_clicked(handle) {
            self.apply_key_binds();
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

    fn apply_key_binds(&mut self) {
        let mut new_binds: [KeyboardKey; 7] = [KeyboardKey::KEY_NULL; 7];
        for i in 0..self.ui_scene.key_changers.len() {
            new_binds[i] = self.ui_scene.key_changers[i].get_key();
        }

        let keys = Inputs::new(new_binds);
        self.settings.update_bindings(keys)
    }

    // quick and dirty way to put stuff for testing
    fn testing_update(&mut self, rl: &mut RaylibHandle) {
        self.ui_scene.slection_check(rl);
        self.ui_scene.key_changers[0].update(rl);
        self.player.as_mut()
            .unwrap()
            .update(rl, &self.settings.keys);
    }
}

fn draw_healthbar(player: &Player, d_handle: &mut RaylibDrawHandle) {
    // assume 100 = 100% fill should take up 1/2 or 1/3 of the screen?
    let window_width = d_handle.get_screen_width();
    let hp = player.get_health();
    let fill = ((window_width / 3)as f32 * (hp/100.0)).round() as i32;

    d_handle.draw_rectangle(20, 20, fill, 40, Color::RED);
    d_handle.draw_rectangle_lines(20, 20, window_width / 3, 40, Color::BLACK);
}