//! Contains both the Engine and the Engine builder
//! Both of these are crucial to using the engine as the
//! builder lets you customize the engine at the start, and the
//! Engine gives you access to all the crucial logic functions

use image::ImageError;
use spin_sleep::SpinSleeper;
use std::time::Instant;
use wgpu::util::DeviceExt;
use wgpu::{CreateSurfaceError, RequestDeviceError};
use winit::error::OsError;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{BadIcon, Window};

use crate::colour::Colour;
use crate::input::{InputHandle, Key, MouseKey};
use crate::render::Renderer;
use crate::shader::{ShaderIndex, create_shader};
use crate::texture::{create_texture, TextureError, TextureIndex};
use crate::vectors::Vec2;
use crate::{text, Game, IDENTITY_MATRIX, layouts};

/// The thing that makes the computer go
pub struct Engine {
    renderer: Renderer,
    input_handle: InputHandle,
    window: Window,
    surface: wgpu::Surface,
    event_loop: Option<EventLoop<()>>,
    cursor_visibility: bool,
    should_close: bool,
    close_key: Option<Key>,
    target_fps: Option<u16>,
    last_frame: Instant,
    spin_sleeper: SpinSleeper,
    current_frametime: Instant,
}

impl Engine {
    fn new(builder: EngineBuilder) -> Result<Self, BuildError> {
        let cursor_visibility = true;
        let input_handle = InputHandle::new();
        let size: Vec2<u32> = builder.resolution.into();
        let target_fps = builder.target_fps;

        let event_loop = EventLoop::new();
        let window_builder = winit::window::WindowBuilder::new()
            .with_title(builder.window_title)
            .with_inner_size(winit::dpi::PhysicalSize::new(
                builder.resolution.0,
                builder.resolution.1,
            ))
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

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });

        let surface = unsafe { instance.create_surface(&window) }?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }));

        let adapter = match adapter {
            Some(a) => Ok(a),
            None => Err(BuildError::FailedToCreateAdapter),
        }?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))?;

        let wgpu_clump = WgpuClump { device, queue };

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
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
        let camera_buffer =
            wgpu_clump
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_matrix]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group_layout = layouts::create_camera_layout(&wgpu_clump.device);

        let camera_bind_group = wgpu_clump
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

        let renderer = Renderer::new(
            wgpu_clump,
            window.inner_size(),
            camera_bind_group,
            camera_bind_group_layout,
            camera_buffer,
            builder.clear_colour,
            config,
        );

        Ok(Self {
            renderer,
            input_handle,
            window,
            surface,
            event_loop: Some(event_loop),
            cursor_visibility,
            should_close: false,
            close_key: builder.close_key,
            target_fps,
            last_frame: Instant::now(),
            current_frametime: Instant::now(),
            spin_sleeper: SpinSleeper::default(),
        })
    }

    /// Attempts to create a texture
    pub fn create_texture(&mut self, path: &str) -> Result<TextureIndex, TextureError> {
        create_texture(
            &mut self.renderer.bind_group_cache,
            &self.renderer.wgpu_clump,
            path,
        )
    }

    /// Loads in the shader to the cache and returns the index
    pub fn create_shader(&mut self, path: &str, layouts: Vec<wgpu::BindGroupLayout>, label: Option<&str>) -> Result<ShaderIndex, std::io::Error> {
        create_shader(
            &mut self.renderer.shader_cache,
            layouts,
            path,
            &self.renderer.wgpu_clump,
            &self.renderer.config,
            label,
        )
    }

    /// Checks if a key is down
    pub fn is_key_down(&self, key: Key) -> bool {
        self.input_handle.is_key_down(key)
    }

    /// Checks if a key is up
    pub fn is_key_up(&self, key: Key) -> bool {
        self.input_handle.is_key_up(key)
    }

    /// Only returns `true` on the frame where the key is first pressed down
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.input_handle.is_key_pressed(key)
    }

    /// Only returns `true` on the frame where the key first returns back up
    pub fn is_key_released(&self, key: Key) -> bool {
        self.input_handle.is_key_released(key)
    }

    /// Checks if a mouse key is down
    pub fn is_mouse_key_down(&self, key: MouseKey) -> bool {
        self.input_handle.is_mouse_key_down(key)
    }

    /// Checks if a mouse key is up
    pub fn is_mouse_key_up(&self, key: MouseKey) -> bool {
        self.input_handle.is_mouse_key_up(key)
    }

    /// Only returns `true` on the frame where the key is first pressed down
    pub fn is_mouse_key_pressed(&self, key: MouseKey) -> bool {
        self.input_handle.is_mouse_key_pressed(key)
    }

    /// Only returns `true` on the frame where the key first returns back up
    pub fn is_mouse_key_released(&self, key: MouseKey) -> bool {
        self.input_handle.is_mouse_key_released(key)
    }

    /// Gives the current position of the mouse in physical pixels
    pub fn get_mouse_position(&self) -> Vec2<f32> {
        self.input_handle.get_mouse_position()
    }

    /// Checks if the window has focus
    pub fn window_has_focus(&self) -> bool {
        self.window.has_focus()
    }

    /// Checks if the window is maximized not fullscreened
    pub fn is_window_maximized(&self) -> bool {
        self.window.is_maximized()
    }

    /// Checks to see if the window is minimized
    pub fn is_window_minimized(&self) -> bool {
        self.window.is_minimized().unwrap_or(false)
    }

    /// Checks to see if the window is fullscreen not maximized
    pub fn is_window_fullscreen(&self) -> bool {
        // based on limited docs knowledge this should work
        self.window.fullscreen().is_some()
    }

    /// Will maximize the window
    pub fn maximize_window(&self) {
        self.window.set_maximized(true);
    }

    /// Will minimize the window
    pub fn minimize_window(&self) {
        self.window.set_minimized(true);
    }

    /// Will close the window and stop the program
    pub fn close(&mut self) {
        self.should_close = true;
    }

    /// Will attempt to set the window icon for more details check the [winit docs](https://docs.rs/winit/latest/winit/window/struct.Window.html#method.set_window_icon)
    pub fn set_window_icon(&self, path: &str) -> Result<(), IconError> {
        let image = image::open(path)?.into_rgba8();
        let (width, height) = image.dimensions();
        let image_bytes = image.into_raw();
        let icon = winit::window::Icon::from_rgba(image_bytes, width, height)?;
        self.window.set_window_icon(Some(icon));
        Ok(())
    }

    /// Sets the window title
    pub fn set_window_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Changes the Position of the window in PhysicalPixles
    pub fn set_window_position(&self, x: f32, y: f32) {
        self.window
            .set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
    }

    /// Sets the physical minimum size of the window
    pub fn set_window_min_size(&self, width: f32, height: f32) {
        self.window
            .set_min_inner_size(Some(winit::dpi::PhysicalSize::new(width, height)));
    }

    /// Gets the physical postion of the window
    pub fn get_window_position(&self) -> Option<Vec2<i32>> {
        match self.window.outer_position() {
            Ok(v) => Some((v.x, v.y).into()),
            Err(_) => None,
        }
    }

    /// Gets the phyisical size of the window,
    pub fn get_window_size(&self) -> Vec2<u32> {
        self.renderer.size
    }

    /// Gets the scale factor to help handle diffrence between phyiscial and logical pixels
    pub fn get_window_scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    /// Toggels fullscreen mode may fail on certain Operating Systems
    /// check the [winit docs](https://docs.rs/winit/latest/winit/window/struct.Window.html#method.set_fullscreen) for more information
    pub fn toggle_fullscreen(&self) {
        if self.is_window_fullscreen() {
            self.window
                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        } else {
            self.window.set_fullscreen(None);
        }
    }

    /// Hides the cursor
    pub fn hide_cursor(&mut self) {
        self.window.set_cursor_visible(false);
        self.cursor_visibility = false;
    }

    /// Shows the cursor if its hidden
    pub fn show_cursor(&mut self) {
        self.window.set_cursor_visible(true);
        self.cursor_visibility = true;
    }

    /// Will update the camera matrix as of version 0.1.0 it will effect
    /// all things drawn is also 3D
    pub fn change_camera_matrix(&mut self, matrix: [f32; 16]) {
        self.renderer.camera_matrix = matrix;
        self.renderer.wgpu_clump.queue.write_buffer(
            &self.renderer.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.renderer.camera_matrix]),
        );
    }

    /// nessicary for creating shaders and wanting to include the camera in the layouts
    /// this is required if you would like to use the camera in the shader
    pub fn camera_layout(&self) -> wgpu::BindGroupLayout {
        layouts::create_camera_layout(&self.renderer.wgpu_clump.device)
    }

    /// nessicary for creating shaders and wanting to include textures in the layouts
    /// this is required if you would like to use textures in your shader
    pub fn texture_layout(&self) -> wgpu::BindGroupLayout {
        layouts::create_texture_layout(&self.renderer.wgpu_clump.device)
    }

    /// nessicary for creating shaders and wanting to use your own unifroms
    /// this should be pretty generic as it is exposed to both the vertex 
    /// and fragment stage
    pub fn uniform_layout(&self) -> wgpu::BindGroupLayout {
        layouts::create_uniform_layout(&self.renderer.wgpu_clump.device)
    }

    /// Measures a peice of text and gives a Vec2 of the width and height
    pub fn measure_text(&mut self, text: &str, scale: f32) -> Vec2<f32> {
        text::measure_text(text, &mut self.renderer.glyph_brush, scale)
    }

    /// Gets the time since the previous frame or change in time between now and last frame
    pub fn get_frame_delta_time(&self) -> f32 {
        let dt = Instant::now().duration_since(self.last_frame).as_secs_f32();
        dt
    }

    /// Gets the current target fps
    pub fn get_target_fps(&self) -> Option<u16> {
        self.target_fps
    }

    /// Will uncap the framerate and cause the engine to redner and update as soon
    /// as the next frame is ready
    pub fn remove_target_fps(&mut self) {
        self.target_fps = None;
    }

    /// Sets a target fps cap. The thread will spin sleep using the
    /// [spin_sleep](https://docs.rs/spin_sleep/latest/spin_sleep/index.html) crate
    /// untill the desired frame time is reached. If the frames are slower than the target
    /// no action is taken to "speed" up the rendering and updating
    pub fn set_target_fps(&mut self, fps: u16) {
        self.target_fps = Some(fps);
    }

    pub(crate) fn get_wgpu(&self) -> &WgpuClump {
        &self.renderer.wgpu_clump
    }

    /// Used when adding shader options into the bindgroup cahce !
    pub(crate) fn add_to_bind_group_cache(&mut self, bind_group: wgpu::BindGroup, key: u32) {
        self.renderer.bind_group_cache.add_item(bind_group, key);
    }

    /// Takes the struct that implements the Game trait and starts the winit event loop running the game
    pub fn run<T: 'static>(mut self, mut game: T) -> !
    where
        T: Game,
    {
        let event_loop = self.event_loop.take().unwrap(); //should never panic
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    self.current_frametime = Instant::now();
                    game.render(&mut self.renderer);
                    match self.renderer.render(
                        self.window.inner_size().into(),
                        &self.surface,
                    ) {
                        Ok(_) => {}
                        // reconfigure surface if lost
                        Err(wgpu::SurfaceError::Lost) => self.resize(self.renderer.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                    game.update(&mut self);
                    self.update();
                    if self.should_close {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::MainEventsCleared => {
                    //RedrawRequested will only trigger once, unless we manually request it
                    self.window.request_redraw();
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => {
                    if !self.input(event) {
                        match event {
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(physical_size) => {
                                let s = *physical_size;
                                self.resize(s.into());
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                let s = **new_inner_size;
                                self.resize(s.into());
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        });
    }

    fn update(&mut self) {
        self.last_frame = Instant::now();
        self.renderer.bind_group_cache.cache_update();
        self.input_handle.end_of_frame_refresh();
        if let Some(key) = self.close_key {
            if self.input_handle.is_key_down(key) {
                self.should_close = true;
            }
        }

        if let Some(frame_rate) = self.target_fps {
            let frame_time = Instant::now().duration_since(self.current_frametime).as_nanos() as u64;
            let desired_time_between_frames = 1000000000 / frame_rate as u64;
            if frame_time < desired_time_between_frames {
                self.spin_sleeper.sleep_ns(desired_time_between_frames-frame_time);
            }
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.input_handle.process_input(event)
    }

    fn resize(&mut self, new_size: Vec2<u32>) {
        if new_size.x > 0 && new_size.y > 0 {
            self.renderer.config.width = new_size.x;
            self.renderer.config.height = new_size.y;
            self.surface
                .configure(&self.renderer.wgpu_clump.device, &self.renderer.config);
            self.renderer.size = new_size;
        }
    }
}

/// The main entry point for the Engine
pub struct EngineBuilder {
    resolution: (u32, u32),
    full_screen: bool,
    target_fps: Option<u16>,
    close_key: Option<Key>,
    clear_colour: Colour,
    window_icon: Option<winit::window::Icon>,
    window_title: String,
    resizable: bool,
}

impl EngineBuilder {
    /// Creates a builder with some defualt presets
    /// ```rust
    /// Self {
    ///     resolution: (600, 600),
    ///     full_screen: false,
    ///     target_fps: 30,
    ///     close_key: None,
    ///     clear_colour: Colour::Black,
    ///     window_icon: None,
    ///     window_title: "".into(),
    ///     resizable: true,
    /// }
    pub fn new() -> Self {
        Self {
            resolution: (600, 600),
            full_screen: false,
            target_fps: None,
            close_key: None,
            clear_colour: Colour::Black,
            window_icon: None,
            window_title: "".into(),
            resizable: true,
        }
    }

    /// Overides the defualt resolution
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

    /// Will cause the window to be fullscreen upon launch
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

    /// Sets a target fps cap. The thread will spin sleep using the
    /// [spin_sleep](https://docs.rs/spin_sleep/latest/spin_sleep/index.html) crate
    /// untill the desired frame time is reached. If the frames are slower than the target
    /// no action is taken to "speed" up the rendering and updating
    pub fn set_target_fps(self, fps: u16) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: Some(fps),
            close_key: self.close_key,
            clear_colour: self.clear_colour,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
        }
    }

    /// Sets a key that will instantly close the window
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

    /// Sets the colour that the background will be
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

    /// Sets the title of the window
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

    /// Sets the window icon
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

    /// Prevents the window from being resized during runtime
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

    /// Attempts to buld the Engine
    pub fn build<'a>(self) -> Result<Engine, BuildError> {
        Engine::new(self)
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
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
    IconLoadingError(ImageError),
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
