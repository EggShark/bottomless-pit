use bottomless_pit::{*, engine_handle::EngineBuilder, shader::{ShaderIndex, ShaderOptions}};
use vectors::Vec2;
use colour::Colour;
use crevice::std140::AsStd140;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_window_title("Testing Triangle")
        .with_resolution((400, 400))
        .build()
        .unwrap();

    let shader = engine.create_shader("shader.wgsl", vec![engine.texture_layout(), engine.camera_layout(), engine.uniform_layout(),]).unwrap();
    let options = ShaderOptions::new(&MousePos { x: 0.0, y: 0.0 }, &mut engine);

    let s = ShaderExample {
        shader,
        options,
        mouse: MousePos { x: 0.0, y: 0.0 },
    };

    engine.run(s);
}

#[derive(AsStd140)]
struct MousePos{
    x: f32,
    y: f32,
}

struct ShaderExample {
    shader: ShaderIndex,
    options: ShaderOptions,
    mouse: MousePos,
}

impl Game for ShaderExample {
    fn render(&self, render_handle: &mut render::Renderer) {
        render_handle.set_shader(&self.shader);
        render_handle.set_shader_options(&self.options);

        render_handle.draw_triangle_with_coloured_verticies(
            Vec2{x: 100.0, y: 0.0},
            Vec2{x: 200.0, y: 200.0},
            Vec2{x: 0.0, y: 200.0},
            Colour::Red,
            Colour::Green,
            Colour::Blue,
        );

        // render_handle.set_to_defualt_shader();

        render_handle.draw_triangle_with_coloured_verticies(
            Vec2{x: 300.0, y: 200.0},
            Vec2{x: 400.0, y: 400.0},
            Vec2{x: 200.0, y: 400.0},
            Colour::Red,
            Colour::Green,
            Colour::Blue,
        );
    }

    fn update(&mut self, engine_handle: &mut engine_handle::Engine) {
        let mouse_pos = engine_handle.get_mouse_position();
        self.mouse.x = mouse_pos.x;
        self.mouse.y = mouse_pos.y;
        self.options.update_uniform(&self.mouse, engine_handle);
    }
}
