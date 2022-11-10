use input_handler::Inputs;
use raylib::RaylibHandle;
use raylib::consts::KeyboardKey;
use raylib::prelude::RaylibDrawHandle;
use ui::UiScene;
use utils::{MenuState, AppState};
use crate::settings::{Settings, Resolutions};


pub struct MenuManager {
    ui_scene: UiScene,
    state: MenuState,
    change: Option<AppState> // what is used to interface with the app above
}

impl MenuManager {
    pub fn new(keys: &Inputs) -> Self {
        Self {
            ui_scene: UiScene::from_menu_state(MenuState::MainMenu, keys),
            state: MenuState::MainMenu,
            change: None,
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        self.ui_scene.draw(d_handle);
    }

    pub fn update(&mut self, rl: &mut RaylibHandle, keys: &Inputs, settings: &mut Settings) {
        match self.state {
            MenuState::KeySettings => self.update_key_settings(rl, keys, settings),
            MenuState::SettingsMenu => self.update_settings_menu(rl, keys, settings),
            MenuState::MainMenu => self.update_main_menu(rl, keys),
        }
    }

    pub fn get_change(&self) -> Option<AppState> {
        self.change
    }

    fn change_state(&mut self, new_state: MenuState, keys: &Inputs) {
        self.state = new_state;
        self.ui_scene = UiScene::from_menu_state(new_state, keys)
    }

    fn update_key_settings(&mut self, rl: &mut RaylibHandle, keys: &Inputs, settings: &mut Settings) {
        for changer in self.ui_scene.key_changers.iter_mut() {
            changer.update(rl);
        }

        if self.ui_scene.buttons[0].was_clicked(rl) {
            return self.change_state(MenuState::SettingsMenu, keys);
        }

        if self.ui_scene.buttons[1].was_clicked(rl) {
            self.apply_key_binds(settings);
        }
    }

    fn update_main_menu(&mut self, rl: &RaylibHandle, keys: &Inputs) {
        self.ui_scene.slection_check(rl);

        if self.ui_scene.buttons[0].was_clicked(rl) {
            self.change = Some(AppState::Quit);
            return;
        }

        if self.ui_scene.buttons[1].was_clicked(rl) {
            self.change = Some(AppState::InGame);
            return;
            // need to return early to back out after clearing the vec
            // otherwise it will tried to read [2] when there is nothing there
        }

        if self.ui_scene.buttons[2].was_clicked(rl) {
            return self.change_state(MenuState::SettingsMenu, keys);
        }
    }

    fn update_settings_menu(&mut self, rl: &mut RaylibHandle, keys: &Inputs, settings: &mut Settings) {
        self.ui_scene.slection_check(rl);
        
        for x in 0..self.ui_scene.selectors.len() {
            self.ui_scene.selectors[x].update(rl);
        }

        if self.ui_scene.buttons[0].was_clicked(rl) {
            return self.change_state(MenuState::MainMenu, keys);
        }

        if self.ui_scene.buttons[2].was_clicked(rl) {
            return self.change_state(MenuState::KeySettings, keys);
        }

        if self.ui_scene.buttons[1].was_clicked(rl) {
            self.apply_settings(rl, settings);
        }
    }

    fn apply_settings(&self, rl: &mut RaylibHandle, settings: &mut Settings) {
        let mut selections: Vec<i8> = Vec::new();
        for x in 0..self.ui_scene.selectors.len() {
            selections.push(self.ui_scene.selectors[x].get_curr_selection());
        }

        let resolution = Resolutions::from(selections[0] as u8);

        let (width, height) = resolution.len_width();

        let volume: u8 = selections[1] as u8 + 1;
        rl.set_window_size(width as i32, height as i32);

        settings.update_settings(resolution, volume);
    }

    fn apply_key_binds(&self, settings: &mut Settings) {
        let mut new_binds: [KeyboardKey; 7] = [KeyboardKey::KEY_NULL; 7];
        for i in 0..self.ui_scene.key_changers.len() {
            new_binds[i] = self.ui_scene.key_changers[i].get_key();
        }

        let keys = Inputs::new(new_binds);
        settings.update_bindings(keys)
    }
}