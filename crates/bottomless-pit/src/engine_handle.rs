use image::{ImageError, GenericImageView};
use rayon::prelude::*;
use winit::window::BadIcon;
use winit::window::Window;

use crate::Colour;
use crate::DrawQueues;
use crate::TextureCache;
use crate::InputHandle;
use crate::cache::TextureIndex;
use crate::render::Renderer;
use crate::input::Key;
use crate::texture::{Texture, create_texture};
use crate::vectors::Vec2;
use crate::camera::{Camera, CameraController};

pub struct Engine {
    renderer: Renderer,
    input_handle: InputHandle,
    window: Window,
    cursor_visibility: bool,
    camera_matrix: [f32; 16],
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
}


impl Engine {
    fn new(winodw_builder: winit::window::WindowBuilder) -> Self {
        todo!()
    }

    pub fn create_texture(&mut self, path: &str) -> TextureIndex {
        create_texture(&mut self.renderer.texture_cahce, &self.renderer.wgpu_things, path)
    }

    pub fn create_many_textures(&mut self, paths: &[&str]) -> Vec<TextureIndex> {
        let textures = paths.par_iter()
            .map(|path| Texture::from_path(&self.renderer.wgpu_things, None, path).unwrap())
            .collect::<Vec<Texture>>();

        textures.into_iter().map(|texture| self.renderer.texture_cahce.add_texture(texture)).collect::<Vec<TextureIndex>>()
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

    pub fn change_camera_matrix(&mut self, matrix: [f32; 16]) {
        self.camera_matrix = matrix;
        self.renderer.wgpu_things.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_matrix]));
    }
}

pub struct EngineBuilder {
    resolution: (u32, u32),
    full_screen: bool,
    target_fps: u32,
    close_key: Option<Key>,
    clear_colour: Colour,
    window_icon: Option<winit::window::Icon>,
    window_title: String,
    resizable: bool,
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self {
            resolution: (600, 600),
            full_screen: false,
            target_fps: 30,
            close_key: None,
            clear_colour: Colour::Black,
            window_icon: None,
            window_title: "".into(),
            resizable: true
        }
    }

    pub fn with_resolution(self, resolution: (u32, u32)) -> Self {
        Self {
            resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: self.close_key,
            clear_colour: self.clear_colour,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
        }
    }

    pub fn fullscreen(self) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: true,
            target_fps: self.target_fps,
            close_key: self.close_key,
            clear_colour: self.clear_colour,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
        }
    }

    pub fn set_target_fps(self, fps: u32) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: fps,
            close_key: self.close_key,
            clear_colour: self.clear_colour,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
        }
    }

    pub fn set_close_key(self, key: Key) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: Some(key),
            clear_colour: self.clear_colour,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
        }
    }

    pub fn set_clear_colour(self, colour: Colour) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: self.close_key,
            clear_colour: colour,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
        }
    }

    pub fn set_window_title(self, title: &str) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: self.close_key,
            clear_colour: self.clear_colour,
            window_icon: self.window_icon,
            window_title: title.into(),
            resizable: self.resizable,
        }
    }

    pub fn set_window_icon(self, icon: winit::window::Icon) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: self.close_key,
            clear_colour: self.clear_colour,
            window_icon: Some(icon),
            window_title: self.window_title,
            resizable: self.resizable,
        }
    }

    pub fn unresizable(self) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: self.close_key,
            clear_colour: self.clear_colour,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: false,
        }
    }

    pub fn build(self) -> Engine {
        let window_builder = winit::window::WindowBuilder::new()
            .with_title(self.window_title)
            .with_inner_size(winit::dpi::PhysicalSize::new(self.resolution.0, self.resolution.1))
            .with_resizable(self.resizable)
            .with_window_icon(self.window_icon);
        todo!()
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

// just made to avoid data clumps
pub(crate) struct DeviceQueue {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}