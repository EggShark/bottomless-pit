mod game;
mod settings;
mod player;
mod app;
mod menu_manger;

use settings::Settings;
use app::App;

fn main() {
    let settings = Settings::load_from_file();

    let settings = match settings {
        Ok(settings) => {
            println!("settings loaded succsesfully");
            settings
        },
        Err(e) => {
            println!("settings could not be loaded {}", e);
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
    
    let mut app = App::new(settings);

    while !app.should_close(&rl) {
        app.update(&mut rl, &thread);
        let d_handle = rl.begin_drawing(&thread);
        app.draw(d_handle);
    }
}