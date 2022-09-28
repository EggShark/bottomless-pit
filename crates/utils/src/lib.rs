mod text;
mod point;
mod collide;

pub use crate::text::Text;
pub use crate::point::Point;
pub use crate::collide::Collide;

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