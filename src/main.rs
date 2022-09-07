mod ui_elements;
mod game;
mod settings;

use settings::Settings;
use game::Game;

fn main() {
    let settings = Settings::default();
    settings.write_to_file().unwrap();

    println!("{:?}", settings);

    let(mut rl, thread) = raylib::init()
        .transparent()
        .size(settings.length as i32, settings.height as i32)
        .title("cheesed to meet u")
        .build();
    
    rl.set_target_fps(30);
    rl.set_exit_key(None);

    let mut game = Game::new();

    while !game.should_close(&rl) {
        game.update(&rl);

        let d_handle = rl.begin_drawing(&thread);
        game.draw(d_handle);
    }
}
