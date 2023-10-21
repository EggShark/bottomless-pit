use bottomless_pit::material::{Material, MaterialBuilder};
use bottomless_pit::render::{TexturePass, RenderInformation};
use bottomless_pit::shader::{UniformData, Shader};
use bottomless_pit::{*, engine_handle::EngineBuilder};
use bottomless_pit::texture::UniformTexture;
use vectors::Vec2;
use colour::Colour;

fn main() {
    let mut engine = EngineBuilder::new()
        .set_window_title("Testing Triangle")
        .with_resolution((400, 400))
        .build()
        .unwrap();
    
    let uniform_texture = UniformTexture::new(&engine, engine.get_window_size());

    let uniform_data = UniformData::new_with_extra_texture(&0.0_f32, &uniform_texture, &engine);
    let shader = Shader::new_with_uniforms("examples/lightmap.wgsl", &uniform_data, &mut engine).unwrap();

    let material = MaterialBuilder::new()
        .set_uniform(&uniform_data)
        .set_shader(shader)
        .build(&mut engine);

    let ocluder_material = MaterialBuilder::new().build(&mut engine);

    let s = TextureExample {
        material,
        ocluder_material,
        uniform_data,
        pos: Vec2{x: 0.0, y: 0.0},
        uniform_texture,
    };

    engine.run(s);
}

struct TextureExample {
    material: Material,
    ocluder_material: Material,
    uniform_data: UniformData,
    pos: Vec2<f32>,
    uniform_texture: UniformTexture,
}

impl Game for TextureExample {
    fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.material.add_rectangle(Vec2 {x: 0.0, y: 0.0}, Vec2{x: 400.0, y: 400.0}, Colour::BROWN, &render_handle);
        self.material.draw(&mut render_handle);
    }

    fn update(&mut self, engine_handle: &mut engine_handle::Engine) {
        let dt = engine_handle.get_frame_delta_time();

        self.pos = self.pos + Vec2{x: 2.0 * dt, y: 2.0 * dt};

        let mut texture_pass = TexturePass::new(engine_handle);
        let view = self.uniform_texture.get_view();
        let mut pain = texture_pass.begin_pass(engine_handle, &view);

        self.ocluder_material.add_rectangle(self.pos, Vec2{x: 10.0, y: 10.0}, Colour::PINK, &pain);
        self.ocluder_material.draw(&mut pain);

        drop(pain);

        texture_pass.finish_pass(&engine_handle);
    }
}
