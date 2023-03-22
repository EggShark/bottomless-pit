mod texture;
mod rect;
mod vertex;
mod input;
mod draw_queue;
mod matrix_math;
mod colour;
mod text;
mod engine_handle;
mod render;
mod vectors;

pub use engine_handle::{Engine, EngineBuilder, BuildError};
pub use render::Renderer;
pub use vectors::{Vec2, Vec3};
pub use matrix_math::*;
pub use colour::Colour;
pub use input::Key;
pub use texture::TextureIndex;
use input::InputHandle;
use texture::{Texture, TextureCache};
use vertex::{Vertex, LineVertex};
use draw_queue::{DrawQueues, BindGroups};

pub trait Game {
    fn render(&self, render_handle: &mut Renderer);
    fn update(&mut self, engine_handle: &mut Engine);
    fn on_close(&self) {

    } 
}

const IDENTITY_MATRIX: [f32; 16] = [
    1.0,  0.0,  0.0,  0.0,
    0.0,  1.0,  0.0,  0.0,
    0.0,  0.0,  1.0,  0.0,
    0.0,  0.0,  0.0,  1.0,
];

// just the data for png of a white pixel didnt want it in a seperate file so here is a hard coded const!
const WHITE_PIXEL: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84, 8, 91, 99, 248, 15, 4, 0, 9, 251, 3, 253, 159, 31, 44, 0, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130];