use image::{ImageError, GenericImageView};
use rayon::prelude::*;
use wgpu::util::DeviceExt;
use wgpu::{CreateSurfaceError, RequestDeviceError};
use winit::event::*;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::{BadIcon, Window};
use winit::error::OsError;

use crate::{Colour, IDENTITY_MATRIX, Game};
use crate::InputHandle;
use crate::TextureIndex;
use crate::render::Renderer;
use crate::input::Key;
use crate::texture::{Texture, create_texture};
use crate::vectors::Vec2;

pub struct Engine {
    renderer: Renderer,
    input_handle: InputHandle,
    window: Window,
    config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface,
    event_loop: Option<EventLoop<()>>,
    cursor_visibility: bool,
    camera_matrix: [f32; 16],
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
}


impl Engine {
    fn new(builder: EngineBuilder) -> Result<Self, BuildError> {
        let cursor_visibility = true;
        let input_handle = InputHandle::new();
        let size: Vec2<u32> = builder.resolution.into();

        let event_loop = EventLoop::new();
        let window_builder = winit::window::WindowBuilder::new()
            .with_title(builder.window_title)
            .with_inner_size(winit::dpi::PhysicalSize::new(builder.resolution.0, builder.resolution.1))
            .with_resizable(builder.resizable)
            .with_window_icon(builder.window_icon);

        let window_builder = if builder.full_screen {
            window_builder.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        } else {
            window_builder
        };

        let window = window_builder.build(&event_loop)?;

        let backend = if cfg!(target_os = "windows") {
            wgpu::Backends::DX12 // text rendering gets angry on vulkan
        } else {
            wgpu::Backends::all()
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor{ 
            backends: backend, 
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });

        let surface = unsafe {
            instance.create_surface(&window)
        }?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }));

        let adapter = match adapter {
            Some(a) => Ok(a),
            None => Err(BuildError::FailedToCreateAdapter)
        }?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))?;

        let wgpu_clump = WgpuClump {
            device,
            queue,
        };

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.x,
            height: size.y,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&wgpu_clump.device, &config);

        let camera_matrix = IDENTITY_MATRIX;
        let camera_buffer = wgpu_clump.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        let camera_bind_group_layout = wgpu_clump.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = wgpu_clump.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let renderer = Renderer::new(wgpu_clump, &surface, &adapter, window.inner_size(), &camera_bind_group_layout, builder.clear_colour, config.format);
        Ok(Self {
            renderer,
            input_handle,
            window,
            surface,
            config,
            event_loop: Some(event_loop),
            cursor_visibility,
            camera_matrix,
            camera_bind_group,
            camera_buffer,
        })
        
    }

    pub fn create_texture(&mut self, path: &str) -> TextureIndex {
        create_texture(&mut self.renderer.texture_cache, &self.renderer.wgpu_clump, path)
    }

    pub fn create_many_textures(&mut self, paths: &[&str]) -> Vec<TextureIndex> {
        let textures = paths.par_iter()
            .map(|path| Texture::from_path(&self.renderer.wgpu_clump, None, path).unwrap())
            .collect::<Vec<Texture>>();

        textures.into_iter().map(|texture| self.renderer.texture_cache.add_texture(texture)).collect::<Vec<TextureIndex>>()
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
        self.renderer.size
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
        self.renderer.wgpu_clump.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_matrix]));
    }

    pub fn run(mut self, mut game: Box<dyn Game>) -> () {
        let event_loop = self.event_loop.take().unwrap(); //should never panic
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    game.render(&mut self.renderer);
                    match self.renderer.render(self.window.inner_size().into(), &self.camera_bind_group, &self.surface) {
                        Ok(_) => {},
                        // reconfigure surface if lost
                        Err(wgpu::SurfaceError::Lost) => self.resize(self.renderer.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                    game.update(&mut self);
                    self.update();
                }
                Event::MainEventsCleared => {
                    //RedrawRequested will only trigger once, unless we manually request it
                    self.window.request_redraw();
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => if !self.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            let s = *physical_size;
                            self.resize(s.into());
                        },
                        WindowEvent::ScaleFactorChanged{new_inner_size, ..} => {
                            let s = **new_inner_size;
                            self.resize(s.into());
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        });
    }

    fn update(&mut self) {
        self.renderer.texture_cache.chache_update();
        self.input_handle.end_of_frame_refresh();
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        println!("{:?}", event);
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput{
                    virtual_keycode,
                    state,
                    ..
                },
                ..
            } => {
                self.input_handle.process_input(virtual_keycode, *state)
            },
            _ => false,
        }
    }

    fn resize(&mut self, new_size: Vec2<u32>) {
        if new_size.x > 0 && new_size.y > 0 {
            self.config.width = new_size.x;
            self.config.height = new_size.y;
            self.surface.configure(&self.renderer.wgpu_clump.device, &self.config);
            self.renderer.size = new_size;
        }
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

    pub fn build(self) -> Result<Engine, BuildError> {        
        Engine::new(self)
    }
}

#[derive(Debug)]
pub enum BuildError {
    WindowOsError(OsError),
    CreateSurfaceError(CreateSurfaceError),
    FailedToCreateAdapter,
    RequestDeviceError(RequestDeviceError),
}

impl From<OsError> for BuildError {
    fn from(value: OsError) -> Self {
        Self::WindowOsError(value)
    }
}

impl From<CreateSurfaceError> for BuildError {
    fn from(value: CreateSurfaceError) -> Self {
        Self::CreateSurfaceError(value)
    }
}

impl From<RequestDeviceError> for BuildError {
    fn from(value: RequestDeviceError) -> Self {
        Self::RequestDeviceError(value)
    }
}
//impl std::error::Error for BuildError {}

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
pub(crate) struct WgpuClump {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}