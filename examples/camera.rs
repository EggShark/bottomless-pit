use bottomless_pit::camera::Camera;
use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::input::Key;
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderInformation;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::{vec2, Game};
use bottomless_pit::texture::Texture;

fn main() {
    let mut engine = EngineBuilder::new().build().unwrap();

    let texture = Texture::new(&mut engine, "examples/bplogo.png");

    let line_material = MaterialBuilder::new()
        .add_texture(texture)
        .build(&mut engine);
    
    let mut camera = Camera::new(&engine);

    camera.center = vec2!(150.0, -150.0);

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
        self.material.add_rectangle(Vec2 { x: -0.0, y: -0.0 }, Vec2{x: 300.0, y: 300.0}, Colour::WHITE, &render_handle);

        self.camera.set_active(&mut render_handle);
        self.material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let dt = engine_handle.get_frame_delta_time();

        let move_factor = 15.0;

        if engine_handle.is_key_down(Key::A) {
            self.camera.center.x -= move_factor * dt;
        }

        if engine_handle.is_key_down(Key::D) {
            self.camera.center.x += move_factor * dt;
        }

        if engine_handle.is_key_down(Key::W) {
            self.camera.center.y += move_factor * dt;
        }

        if engine_handle.is_key_down(Key::S) {
            self.camera.center.y -= move_factor * dt;
        }

        if engine_handle.is_key_down(Key::Left) {
            self.camera.rotation += move_factor * dt;
        }

        if engine_handle.is_key_down(Key::Right) {
            self.camera.rotation -= move_factor * dt;
        }

        if engine_handle.is_key_down(Key::L) {
            self.camera.scale += vec2!(2.0 * dt, 2.0 * dt);
        }

        if engine_handle.is_key_down(Key::K) {
            self.camera.scale -= vec2!(2.0 * dt, 2.0 * dt);
        }

        println!("{:?}", self.camera.center);
    }
}