use utils::AppState;
use super::Settings;
use super::menu_manger::MenuManager;
use super::game::Game;
use raylib::prelude::*;
// state machines all the way down

pub struct App {
    settings: Settings,
    state: AppState,
    menu: Option<MenuManager>,
    game: Option<Game>,
}

impl App {
    pub fn new(settings: Settings) -> Self {
        let mut app = Self {
            settings,
            state: AppState::InMenu,
            game: None,
            menu: None,
        };
        app.menu = Some(MenuManager::new(&app.settings.keys));

        app
    }

    pub fn update(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let keys = self.settings.keys;
        match self.menu.as_mut() {
            Some(menu) => {
                menu.update(rl, &keys, &mut self.settings);

                if let Some(state) = menu.get_change() {
                    self.change_state(state, rl, thread);
                }
            },
            None => {},
        }

        match self.game.as_mut() {
            Some(game) => {
                game.update(rl);
            },
            None => {},
        }
    }

    pub fn draw(&self, mut d_handle: RaylibDrawHandle) {
        d_handle.clear_background(Color::GREEN);
        match &self.menu {
            Some(menu) => {
                menu.draw(&mut d_handle);
            },
            None => {},
        }

        match &self.game {
            Some(game) => {
                game.draw(d_handle);
            },
            None => {},
        }
    }

    pub fn should_close(&self, rl: &RaylibHandle) -> bool{
        self.state == AppState::Quit || rl.window_should_close()
    }

    pub fn change_state(&mut self, state: AppState, rl: &mut RaylibHandle, thread: &RaylibThread) {
        match state {
            AppState::InGame => {
                self.menu = None;
                self.game = Some(Game::new(self.settings.keys, rl, thread));
            },
            AppState::InMenu => {},
            AppState::Quit => self.state = AppState::Quit,
        }
    }
}
// need to add this here somewhere
// fn apply_settings(&mut self, rl: &mut RaylibHandle) {
//     let mut selections: Vec<i8> = Vec::new();
//     for x in 0..self.ui_scene.selectors.len() {
//         selections.push(self.ui_scene.selectors[x].get_curr_selection());
//     }

//     let resolution = Resolutions::from(selections[0] as u8);

//     let (width, height) = resolution.len_width();

//     let volume: u8 = selections[1] as u8 + 1;
//     rl.set_window_size(width as i32, height as i32);
//     self.settings.update_settings(resolution, volume);
// }

// fn apply_key_binds(&mut self) {
//     let mut new_binds: [KeyboardKey; 7] = [KeyboardKey::KEY_NULL; 7];
//     for i in 0..self.ui_scene.key_changers.len() {
//         new_binds[i] = self.ui_scene.key_changers[i].get_key();
//     }

//     let keys = Inputs::new(new_binds);
//     self.settings.update_bindings(keys)
// }