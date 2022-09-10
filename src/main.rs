mod ui_elements;
mod game;
mod settings;

use settings::Settings;
use ui_elements::ArrowSelector;
use game::Game;
use raylib::prelude::*;

fn main() {
    let settings = Settings::default();
    settings.write_to_file().unwrap();

    println!("{:?}", settings);

    let(mut rl, thread) = raylib::init()
        .size(settings.length as i32, settings.height as i32)
        .title("cheesed to meet u")
        .resizable()
        .build();
    
    rl.set_target_fps(30);
    rl.set_exit_key(None);

    let testing = ArrowSelector::new(vec![String::from("Cheese"), String::from("Cheesing")], (300, 300), (400, 100));

    let mut game = Game::new();

    while !game.should_close(&rl) {
        game.update(&rl);

        let mut d_handle = rl.begin_drawing(&thread);
        testing.draw(&mut d_handle);
        game.draw(d_handle);
    }
}
