mod game;
mod settings;
mod player;

use player::player::Player;
use utils::Point;
use settings::Settings;
use game::Game;
use animation::{HitBox, HitboxType};
use raylib::prelude::*;

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

    let mut game = Game::new(settings);

    let mut player = Player::make_baller(&mut rl, &thread, Point{x: 200, y: 200});
    let texture = rl.load_texture(&thread, "assets/slash_test.png").unwrap();
    let frame_rec = Rectangle::new((0 * (texture.width()/1 as i32))as f32, 0.0, (texture.width() as f32 / 7 as f32) * 1 as f32, texture.height() as f32);
    
    while !game.should_close(&rl) {
        player.update(&rl);
        game.update(&mut rl);

        let mut d_handle = rl.begin_drawing(&thread);
        d_handle.draw_texture_rec(&texture, frame_rec, Vector2::new(600.0, 600.0), Color::WHITE);
        player.draw(&mut d_handle);
        game.draw(d_handle);
    }
}
