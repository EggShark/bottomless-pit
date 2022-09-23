mod ui_elements;
mod game;
mod settings;
mod player;

use animation::PlayerAnimation;
use utils::Point;
use settings::Settings;
use game::Game;

fn main() {
    let settings = Settings::load_from_file();

    let settings = match settings {
        Ok(settings) => {
            settings
        },
        Err(_) => {
            Settings::default()
        }
    };

    let (length, height) = settings.get_resoultion();

    let(mut rl, thread) = raylib::init()
        .size(length as i32, height as i32)
        .title("cheesed to meet u")
        .resizable()
        .build();
    
    rl.set_target_fps(30);
    rl.set_exit_key(None);

    let mut test_animation = PlayerAnimation::new("assets/walk_forwards.png", 2, &mut rl, &thread);

    let mut game = Game::new(settings);

    while !game.should_close(&rl) {
        game.update(&mut rl);

        let mut d_handle = rl.begin_drawing(&thread);
        test_animation.update();
        test_animation.draw(&mut d_handle, Point{x: 100, y:100});
        //d_handle.draw_texture(&player, 100, 100, Color::WHITE);
        game.draw(d_handle);
    }
}
