use raylib::prelude::*;
use super::ui_elements::Button;
use std::collections::HashMap;


pub struct Game { 
    state: GameState,
}

pub enum GameState {
    MainMenu,
    Ingame,
}

impl Default for GameState {
    fn default() -> Self {
        Self::MainMenu
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::default(),
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle) {
        // the logic loop for the game
        match self.state {
            GameState::MainMenu => {
                println!("look ma im in a menu");
            },
            GameState::Ingame => {
                println!("look ma im in game");
            },
        }
    }

    pub fn draw(&self, mut drawer: RaylibDrawHandle) {
        // the drawing loop for the game
        drawer.clear_background(Color::RED);
    }
}