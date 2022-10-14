mod game;
mod settings;
mod player;

use player::Player;
use utils::Point;
use settings::Settings;
use game::Game;
use animation::{HitBox, HitboxType};

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
    let test = HitBox::new(vec![Point{x:100, y:100}, Point{x: 200, y:100}, Point{x:200, y:200}, Point{x:100, y:200}], HitboxType::DamageAble);
    let test2 = HitBox::new(vec![Point{x:150, y:150}, Point{x: 300, y:600}, Point{x: 100, y: 600}], HitboxType::DamageDealing);

    while !game.should_close(&rl) {
        player.update(&rl);
        game.update(&mut rl);

        let mut d_handle = rl.begin_drawing(&thread);
        test.draw_hibox(&mut d_handle);
        test2.draw_hibox(&mut d_handle);
        println!("{}", test.collision_check(&test2));
        player.draw(&mut d_handle);
        game.draw(d_handle);
    }
}
