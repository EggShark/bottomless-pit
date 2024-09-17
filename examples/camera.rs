use bottomless_pit::camera::Camera;
use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::input::Key;
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderHandle;
use bottomless_pit::resource::LoadingOp;
use bottomless_pit::text::TextMaterial;
use bottomless_pit::texture::Texture;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::{vec2, Game};

fn main() {
    let mut engine = EngineBuilder::new().build().unwrap();

    let texture = Texture::new(&mut engine, "examples/bplogo.png", LoadingOp::Blocking);

    let material = MaterialBuilder::new()
        .add_texture(texture)
        .build(&mut engine);

    let camera = Camera::default();

    let text = TextMaterial::new(
        "Mouse pos: 0,0 \n Mouse pos: 0, 0",
        Colour::WHITE,
        15.0,
        20.0,
    );

    let game = CameraExample {
        material,
        text,
        camera,
    };

    engine.run(game);
}

struct CameraExample {
    material: Material,
    text: TextMaterial,
    camera: Camera,
}

impl Game for CameraExample {
    fn render<'o>(&'o mut self, mut render: RenderHandle<'o>) {
        let mut render_handle = render.begin_pass(Colour::BLACK);

        self.material.add_rectangle(
            Vec2 { x: 0.0, y: 0.0 },
            Vec2 { x: 300.0, y: 300.0 },
            Colour::WHITE,
            &render_handle,
        );

        self.camera.set_active(&mut render_handle);
        self.material.draw(&mut render_handle);

        render_handle.reset_camera();
        self.text
            .add_instance(vec2!(0.0), Colour::WHITE, &render_handle);
        self.text.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let dt = engine_handle.get_frame_delta_time();
        let mouse_pos = engine_handle.get_mouse_position();
        let size = engine_handle.get_window_size();

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

        if engine_handle.is_key_pressed(Key::Enter) {
            self.camera.rotation += 45.0;
        }

        let trans_mouse = self.camera.transform_point(mouse_pos, size);

        self.text.set_text(
            &format!(
                "Screen mouse pos: {:.3}, {:.3}\nWorld mouse pos: {:.3}, {:.3}",
                mouse_pos.x, mouse_pos.y, trans_mouse.x, trans_mouse.y
            ),
            Colour::WHITE,
            engine_handle,
        );

        self.text.prepare(engine_handle);
    }
}
