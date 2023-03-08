use image::{ImageError, GenericImageView};
use rayon::prelude::*;
use winit::window::BadIcon;
use winit::window::Window;

use crate::DrawQueues;
use crate::TextureCache;
use crate::InputHandle;
use crate::cache::TextureIndex;
use crate::render::Render;
use crate::input::Key;
use crate::texture::{Texture, create_texture};
use crate::vectors::Vec2;

pub struct Engine {
    renderer: Render,
    draw_queues: DrawQueues,
    device: wgpu::Device,
    queue: wgpu::Queue,
    input_handle: InputHandle,
    texture_cahce: TextureCache,
    window: Window,
    cursor_visibility: bool,
}


impl Engine {
    pub fn create_texture(&mut self, path: &str) -> TextureIndex {
        create_texture(&mut self.texture_cahce, &self.device, &self.queue, path)
    }

    pub fn create_many_textures(&mut self, paths: &[&str]) -> Vec<TextureIndex> {
        let textures = paths.par_iter()
            .map(|path| Texture::from_path(&self.device, &self.queue, None, path).unwrap())
            .collect::<Vec<Texture>>();

        textures.into_iter().map(|texture| self.texture_cahce.add_texture(texture)).collect::<Vec<TextureIndex>>()
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.input_handle.is_key_down(key)
    }

    pub fn is_key_up(&self, key: Key) -> bool {
        self.input_handle.is_key_up(key)
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.input_handle.is_key_pressed(key)
    }

    pub fn is_key_released(&self, key: Key) -> bool {
        self.input_handle.is_key_released(key)
    }

    pub fn window_has_focus(&self) -> bool {
        self.window.has_focus()
    }

    pub fn is_window_maximized(&self) -> bool {
        self.window.is_maximized()
    }

    pub fn is_window_minimized(&self) -> bool {
        match self.window.is_minimized() {
            Some(value) => value,
            None => false,
        }
    }

    pub fn is_window_fullscreen(&self) -> bool {
        // based on limited docs knowledge this should work
        match self.window.fullscreen() {
            Some(_) => true,
            None => false,
        }
    }

    pub fn maximize_window(&self) {
        self.window.set_maximized(true);
    }

    pub fn minimize_window(&self) {
        self.window.set_minimized(true);
    }

    pub fn set_window_icon(&self, path: &str) -> Result<(), IconError> {
        let image = image::open(path)?.into_rgba8();
        let (width, height) = image.dimensions();
        let image_bytes = image.into_raw();
        let icon = winit::window::Icon::from_rgba(image_bytes, width, height)?;
        self.window.set_window_icon(Some(icon));
        Ok(())
    }

    pub fn set_window_title(&self, title: &str) {
        self.window.set_title(title);
    }

    pub fn set_window_position(&self, x: f32, y: f32) {
        self.window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
    }

    pub fn set_window_min_size(&self, width: f32, height: f32) {
        self.window.set_min_inner_size(Some(winit::dpi::PhysicalSize::new(width, height)));
    }

    pub fn get_window_position(&self) -> Option<Vec2<i32>>{
        match self.window.outer_position() {
            Ok(v) => Some((v.x, v.y).into()),
            Err(_) => None,
        }
    }

    pub fn get_window_size(&self) -> Vec2<u32> {
        self.window.inner_size().into()
    }

    pub fn get_window_scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    pub fn toggle_fullscreen(&self) {
        if self.is_window_fullscreen() {
            self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        } else {
            self.window.set_fullscreen(None);
        }
    }

    pub fn hide_cursor(&mut self) {
        self.window.set_cursor_visible(false);
        self.cursor_visibility = false;
    }

    pub fn show_cursor(&mut self) {
        self.window.set_cursor_visible(true);
        self.cursor_visibility = true;
    }
}

#[derive(Debug)]
pub enum IconError {
    BadIcon(BadIcon),
    IconLoadingError(ImageError)
}

impl From<BadIcon> for IconError {
    fn from(value: BadIcon) -> Self {
        Self::BadIcon(value)
    }
}

impl From<ImageError> for IconError {
    fn from(value: ImageError) -> Self {
        Self::IconLoadingError(value)
    }
}

impl std::fmt::Display for IconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadIcon(e) => write!(f, "{}", e),
            Self::IconLoadingError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for IconError {}