use bottomless_pit::camera::Camera;
use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::input::Key;
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::Game;

fn main() {
    let mut engine = EngineBuilder::new().build().unwrap();

    let line_material = MaterialBuilder::new().build(&mut engine);
    let mut camera = Camera::new(&engine);

    camera.set_translation(Vec2{x: 1.0, y: 1.0}, &engine);

    let game = CameraExample {
        material: line_material,
        camera,
    };

    engine.run(game);
}

struct CameraExample {
    material: Material,
    camera: Camera,
}

impl Game for CameraExample {
    fn render<'pass, 'others>(
        &'others mut self,
        mut render_handle: RenderInformation<'pass, 'others>,
    ) where
        'others: 'pass,
    {
        self.material.add_rectangle(Vec2 { x: 0.0, y: 0.0 }, Vec2{x: 200.0, y: 200.0}, Colour::WHITE, &render_handle);

        self.camera.set_active(&mut render_handle);
        self.material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {

    }
}