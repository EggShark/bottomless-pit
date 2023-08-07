//! # Bottomless-pit (working title)
//! Bottomless-pit is a simple 2D game engine that is still a work in progress.
//! This library is inspired slightly by Raylib and other rust based game engines like GGEZ.
//! All Bottomless-pit does currently is handle keyboard and mouse input while also providing
//! a simple way to draw objects to the screen. The shape and texutre renderer is written
//! in house, but the text rendering is powered by [wgpu_glyph](https://github.com/hecrj/wgpu_glyph).
//!
//! To get started start by implmenting the Game trait on any struct you like
//! ```rust,no_run
//! use bottomless_pit::Game;
//! use bottomless_pit::engine_handle{Engine, EngineBuilder};
//! use bottomless_pit::render::Renderer;
//!
//! fn main() {
//!     let engine = EngineBuilder::new()
//!         .build()
//!         .expect("Failed to crate the engine!");
//!     let game = CoolGame::new(&mut engine);
//!
//!     engine.run(game);
//! }
//!
//! struct CoolGame {
//!     // put whatever you want here
//! }
//!
//! impl CoolGame {
//!     pub fn new(engine_handle: &mut Engine) {
//!         // you can even load assets before the game opens
//!     }
//! }
//!
//! impl Game for CoolGame {
//!     fn render(&self, render_handle: &mut Renderer) {
//!         // render what ever you want
//!     }
//!     fn update(&mut self, engine_handle: &mut Engine) {
//!         // this is where all your logic should go
//!     }
//! }
#![allow(clippy::needless_doctest_main)]

pub mod colour;
pub mod engine_handle;
pub mod input;
mod layouts;
pub mod matrix_math;
mod material;
mod rect;
mod resource_cache;
pub mod render;
mod text;
pub mod texture;
pub mod vectors;
pub mod shader;
mod vertex;
mod wgpu_glyph;

pub use crevice;
use engine_handle::Engine;
use render::Renderer;
/// The Trait needed for structs to be used in with the Engine
pub trait Game {
    /// Rendering code goes here
    fn render(&self, render_handle: &mut Renderer);
    /// updating code goes here
    fn update(&mut self, engine_handle: &mut Engine);
    /// Things to do when the window closes
    fn on_close(&self) {}
}

#[rustfmt::skip]
const IDENTITY_MATRIX: [f32; 16] = [
    1.0,  0.0,  0.0,  0.0,
    0.0,  1.0,  0.0,  0.0,
    0.0,  0.0,  1.0,  0.0,
    0.0,  0.0,  0.0,  1.0,
];

#[rustfmt::skip]
// just the data for png of a white pixel didnt want it in a seperate file so here is a hard coded const!
const WHITE_PIXEL: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84, 8, 91, 99, 248, 15, 4, 0, 9, 251, 3, 253, 159, 31, 44, 0, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130];
