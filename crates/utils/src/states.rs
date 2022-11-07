#[derive(Debug, Copy, Clone ,PartialEq)]
pub enum AppState {
    InMenu,
    InGame,
    Quit,
}

#[derive(Debug, Copy, Clone ,PartialEq)]
pub enum GameState {
    InGame,
}

#[derive(Debug, Copy, Clone ,PartialEq)]
pub enum MenuState {
    MainMenu,
    SettingsMenu,
    KeySettings,
}
