use bottomless_pit::colour::Colour;
use bottomless_pit::engine_handle::{Engine, EngineBuilder};
use bottomless_pit::input::{Key, MouseKey};
use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::RenderHandle;
use bottomless_pit::texture::Texture;
use bottomless_pit::vectors::Vec2;
use bottomless_pit::vec2;
use bottomless_pit::Game;

fn main() {
    let mut engine = EngineBuilder::new()
        .build()
        .unwrap();

    let texture = Texture::new(&mut engine, "examples/bplogo.png");

    let texture_material = MaterialBuilder::new()
        .add_texture(texture)
        .build(&mut engine);
    let regular_material = MaterialBuilder::new().build(&mut engine);

    let pos = Position {
        pos: vec2! { 0.0 },
        regular_material,
        texture_material,
        state: false,
    };

    engine.run(pos);
}

struct Position {
    pos: Vec2<f32>,
    regular_material: Material<()>,
    texture_material: Material<()>,
    state: bool,
}

impl Game for Position {
    fn render<'o>(
        &'o mut self,
        mut render: RenderHandle<'o>,
    ) {
        let mut render_handle = render.begin_pass(Colour::BLACK);

        let defualt_size = vec2! { 50.0 };
        self.regular_material.add_rectangle(
            vec2! { 0.0 },
            defualt_size,
            Colour::RED,
            &render_handle,
        );
        self.regular_material.add_rectangle(
            self.pos,
            vec2! { 100.0 },
            Colour::RED,
            &render_handle,
        );
        self.texture_material.add_rectangle(
            vec2! { 0.0, 50.0 },
            defualt_size,
            Colour::WHITE,
            &render_handle,
        );
        self.texture_material.add_rectangle_with_uv(
            Vec2 { x: 0.0, y: 100.0 },
            defualt_size,
            vec2! { 311.0 },
            vec2! { 311.0 },
            Colour::WHITE,
            &render_handle,
        );
        self.regular_material.add_rectangle_with_rotation(
            Vec2 { x: 0.0, y: 150.0 },
            defualt_size,
            Colour::GREEN,
            45.0,
            &render_handle,
        );

        let points = [
            Vec2 { x: 0.0, y: 300.0 },
            Vec2 { x: 80.0, y: 290.0 },
            Vec2 { x: 100.0, y: 400.0 },
            Vec2 { x: 60.0, y: 400.0 },
        ];
        let uvs = [
            vec2! { 0.0 },
            Vec2 { x: 1.0, y: 0.0 },
            vec2! { 1.0 },
            Vec2 { x: 0.0, y: 1.0 },
        ];

        self.regular_material
            .add_custom(points, uvs, 0.0, Colour::RED, &render_handle);

        self.regular_material.add_screenspace_rectangle(vec2!(0.0, 0.0), vec2!(1.0), Colour::YELLOW, &render_handle);

        if self.state {
            self.regular_material.add_rectangle(vec2!(300.0), vec2!(100.0), Colour::GREEN, &render_handle);
        }

        self.texture_material.draw(&mut render_handle);
        self.regular_material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut Engine) {
        let dt = engine_handle.get_frame_delta_time();
        self.pos.x += 100.0 * dt;
        if engine_handle.is_mouse_key_pressed(MouseKey::Left) {
            let new_texture = Texture::new(engine_handle, "examples/eggshark.png");
            self.texture_material.change_texture(new_texture);
        }

        self.state = engine_handle.is_key_down(Key::Space);
    }
}
