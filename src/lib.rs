//! # Bottomless-pit
//! Bottomless-pit is a simple 2D game engine that is still a work in progress.
//! This library is inspired slightly by Raylib and other rust based game engines like GGEZ.
//! All Bottomless-pit does currently is handle keyboard and mouse input while also providing
//! a simple way to draw objects to the screen. The shape and texutre renderer is written
//! in house, but the text rendering is powered by [glyphon](https://github.com/grovesNL/glyphon).
//!
//! To get started start by implmenting the Game trait on any struct you like
//! ```rust,no_run
//! use bottomless_pit::Game;
//! use bottomless_pit::engine_handle::{Engine, EngineBuilder};
//! use bottomless_pit::render::RenderInformation;
//!
//! fn main() {
//!     let engine = EngineBuilder::new()
//!         .build()
//!         .expect("Failed to create the engine!");
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
//!     fn render<'o>(&'o mut self, mut render_handle: RenderHandle<'o>) {
//!         // render what ever you want
//!     }
//!     fn update(&mut self, engine_handle: &mut Engine) {
//!         // this is where all your logic should go
//!     }
//! }
#![allow(clippy::needless_doctest_main)]

pub mod buffer;
pub mod camera;
pub mod colour;
mod context;
pub mod engine_handle;
pub mod input;
mod layouts;
pub mod material;
pub mod matrix_math;
pub mod render;
pub mod resource;
pub mod shader;
pub mod text;
pub mod texture;
pub mod vectors;
mod vertex;

#[cfg(feature = "mint")]
pub use mint;

use engine_handle::Engine;
use render::RenderHandle;
use vectors::Vec2;
/// The Trait needed for structs to be used in with the Engine
pub trait Game {
    /// Rendering code goes here
    fn render<'o>(&'o mut self, render_handle: RenderHandle<'o>);
    /// Updating code goes here
    fn update(&mut self, engine_handle: &mut Engine);
    /// Things to do when the window closes
    fn on_close(&self) {}
    fn on_resize(&mut self, _new_window_size: Vec2<u32>, _engine_handle: &mut Engine) {}
}

#[rustfmt::skip]
// just the data for png of a white pixel didnt want it in a seperate file so here is a hard coded const!
const WHITE_PIXEL: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84, 8, 91, 99, 248, 15, 4, 0, 9, 251, 3, 253, 159, 31, 44, 0, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130];
const ERROR_TEXTURE_DATA: &[u8] = include_bytes!("error.png");
