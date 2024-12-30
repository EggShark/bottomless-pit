//! Contains both the Engine and the Engine builder
//! Both of these are crucial to using the engine as the
//! builder lets you customize the engine at the start, and the
//! Engine gives you access to all the crucial logic functions

use glyphon::{Attrs, Metrics, Shaping};

use image::ImageError;
use spin_sleep::SpinSleeper;
use std::num::NonZeroU64;
use std::path::Path;
use web_time::Instant;
use wgpu::{CreateSurfaceError, RequestDeviceError};
use winit::application::ApplicationHandler;
use winit::error::OsError;
use winit::event::*;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::BadIcon;

use crate::context::{GraphicsContext, WindowOptions};
use crate::input::{InputHandle, Key, ModifierKeys, MouseKey};
use crate::render::render;
use crate::resource;
use crate::resource::{
    InProgressResource, Loader, LoadingOp, Resource, ResourceError, ResourceId, ResourceManager,
    ResourceType,
};
use crate::shader::{FinalShaderOptions, IntermediateOptions, Shader};
use crate::text::Font;
use crate::texture::{SamplerType, Texture};
use crate::vectors::Vec2;
use crate::Game;

/// The thing that makes the computer go
pub struct Engine {
    event_loop: Option<EventLoop<BpEvent>>,
    input_handle: InputHandle,
    window_options: Option<WindowOptions>,
    proxy: EventLoopProxy<BpEvent>,
    cursor_visibility: bool,
    should_close: bool,
    close_key: Option<Key>,
    target_fps: Option<u16>,
    last_frame: Instant,
    spin_sleeper: SpinSleeper,
    current_frametime: Instant,
    size: Vec2<u32>,
    pub(crate) resource_manager: ResourceManager,
    pub(crate) loader: Loader,
    pub(crate) defualt_resources: DefualtResources,
    ma_frame_time: f32,
    pub(crate) context: Option<GraphicsContext>,
}

impl Engine {
    fn new(builder: EngineBuilder) -> Result<Self, BuildError> {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
            } else {
                env_logger::init();
            }
        }

        let cursor_visibility = true;
        let input_handle = InputHandle::new();
        let size: Vec2<u32> = builder.resolution.into();
        let target_fps = builder.target_fps;

        let event_loop: EventLoop<BpEvent> = EventLoop::with_user_event().build().unwrap();
        let proxy = event_loop.create_proxy();

        let resource_manager = ResourceManager::new();

        let close_key = builder.close_key;

        let line_id = resource::generate_id::<Shader>();
        let generic_id = resource::generate_id::<Shader>();
        let white_pixel_id = resource::generate_id::<Texture>();

        let defualt_resources = DefualtResources {
            default_pipeline_id: generic_id,
            defualt_texture_id: white_pixel_id,
            line_pipeline_id: line_id,
        };

        Ok(Self {
            input_handle,
            event_loop: Some(event_loop),
            window_options: Some(builder.into()),
            proxy,
            cursor_visibility,
            should_close: false,
            close_key,
            target_fps,
            last_frame: Instant::now(),
            current_frametime: Instant::now(),
            spin_sleeper: SpinSleeper::default(),
            size,
            context: None,
            resource_manager,
            loader: Loader::new(),
            defualt_resources,
            ma_frame_time: 0.0,
        })
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

    /// This will tell you if certain keys like CTRL or Shift
    /// are modifying the current key presses.
    pub fn check_modifiers(&self, modifer: ModifierKeys) -> bool {
        self.input_handle.check_modifiers(modifer)
    }

    /// returns the text vule of any keys held down helpfull for text
    /// entry. As if Shift + w is held this will return `Some("W")`
    pub fn get_current_text(&self) -> Option<&str> {
        self.input_handle.get_text_value()
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

    /// Returns how much the mouse has moved in the last frame
    pub fn get_mouse_delta(&self) -> Vec2<f32> {
        self.input_handle.get_mouse_delta()
    }

    /// Checks if the window has focus
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn window_has_focus(&self) -> bool {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context.window.has_focus()
    }

    /// Checks if the window is maximized not fullscreened
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn is_window_maximized(&self) -> bool {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context.window.is_maximized()
    }

    /// Checks to see if the window is minimized
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn is_window_minimized(&self) -> bool {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context.window.is_minimized().unwrap_or(false)
    }

    /// Checks to see if the window is fullscreen not maximized
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn is_window_fullscreen(&self) -> bool {
        // based on limited docs knowledge this should work
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context.window.fullscreen().is_some()
    }

    /// Will maximize the window
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn maximize_window(&self) {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context.window.set_maximized(true);
    }

    /// Will minimize the window
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn minimize_window(&self) {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context.window.set_minimized(true);
    }

    /// Will close the window and stop the program
    pub fn close(&mut self) {
        self.should_close = true;
    }

    /// Will attempt to set the window icon for more details check the
    /// [winit docs](https://docs.rs/winit/latest/winit/window/struct.Window.html#method.set_window_icon)
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn set_window_icon(&self, path: &str) -> Result<(), IconError> {
        let image = image::open(path)?.into_rgba8();
        let (width, height) = image.dimensions();
        let image_bytes = image.into_raw();
        let icon = winit::window::Icon::from_rgba(image_bytes, width, height)?;

        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");

        context.window.set_window_icon(Some(icon));
        Ok(())
    }

    /// Sets the window title
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn set_window_title(&self, title: &str) {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context.window.set_title(title);
    }

    /// Changes the Position of the window in Physical Pixles
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn set_window_position(&self, x: f32, y: f32) {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context
            .window
            .set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
    }

    /// Sets the physical minimum size of the window
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn set_window_min_size(&self, width: f32, height: f32) {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context
            .window
            .set_min_inner_size(Some(winit::dpi::PhysicalSize::new(width, height)));
    }

    /// Gets the physical postion of the window
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn get_window_position(&self) -> Option<Vec2<i32>> {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        match context.window.outer_position() {
            Ok(v) => Some((v.x, v.y).into()),
            Err(_) => None,
        }
    }

    /// Gets the phyisical size of the window,
    pub fn get_window_size(&self) -> Vec2<u32> {
        self.size
    }

    /// Gets the scale factor to help handle diffrence between phyiscial and logical pixels
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn get_window_scale_factor(&self) -> f64 {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context.window.scale_factor()
    }

    /// Toggels fullscreen mode may fail on certain Operating Systems
    /// check the [winit docs](https://docs.rs/winit/latest/winit/window/struct.Window.html#method.set_fullscreen)
    /// for more information
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn toggle_fullscreen(&self) {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        if self.is_window_fullscreen() {
            context
                .window
                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        } else {
            context.window.set_fullscreen(None);
        }
    }

    /// Hides the cursor
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn hide_cursor(&mut self) {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context.window.set_cursor_visible(false);
        self.cursor_visibility = false;
    }

    /// Shows the cursor if its hidden
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn show_cursor(&mut self) {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");
        context.window.set_cursor_visible(true);
        self.cursor_visibility = true;
    }

    /// Gets the time since the previous frame or change in time between now and last frame
    pub fn get_frame_delta_time(&self) -> f32 {
        Instant::now().duration_since(self.last_frame).as_secs_f32()
    }

    /// Returns a moving average of the last two frame times
    pub fn get_stable_fps(&self) -> f32 {
        1.0 / self.ma_frame_time
    }

    /// Gets the current target fps
    pub fn get_target_fps(&self) -> Option<u16> {
        self.target_fps
    }

    /// Will uncap the framerate and cause the engine to redner and update as soon
    /// as the next frame is ready unless VSYNC is on then it will draw at the
    /// VYSNC rate, which is dependant on user hardware.
    pub fn remove_target_fps(&mut self) {
        self.target_fps = None;
    }

    /// Will turn off vysnc if the platform suports it, using
    /// AutoNoVsync for more information check
    /// [PresentMode::AutoNoVsync](https://docs.rs/wgpu/latest/wgpu/enum.PresentMode.html).
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn remove_vsync(&mut self) {
        let context = self
            .context
            .as_mut()
            .expect("Context hasnt been created yet run inside impl Game");
        context.config.present_mode = wgpu::PresentMode::AutoNoVsync;
        context
            .surface
            .configure(&context.wgpu.device, &context.config);
    }

    /// Will turn off vysnc if the platform suports it, using
    /// AutoVsync for more information check
    /// [PresentMode::AutoVsync](https://docs.rs/wgpu/latest/wgpu/enum.PresentMode.html).
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn add_vsync(&mut self) {
        let context = self
            .context
            .as_mut()
            .expect("Context hasnt been created yet run inside impl Game");
        context.config.present_mode = wgpu::PresentMode::AutoVsync;
        context
            .surface
            .configure(&context.wgpu.device, &context.config);
    }

    /// Sets a target fps cap. The thread will spin sleep using the
    /// [spin_sleep](https://docs.rs/spin_sleep/latest/spin_sleep/index.html) crate
    /// untill the desired frame time is reached. If the frames are slower than the target
    /// no action is taken to "speed" up the rendering and updating
    pub fn set_target_fps(&mut self, fps: u16) {
        self.target_fps = Some(fps);
    }

    /// Measures string based on the default font. To measure a string with a custom font
    /// use [TextMaterial::get_measurements()](../text/struct.TextMaterial.html#method.get_measurements)
    /// # Panics
    /// When called outside of the functions in the [Game] trait
    pub fn measure_string(&mut self, text: &str, font_size: f32, line_height: f32) -> Vec2<f32> {
        let size = self.get_window_size();
        let scale_factor = self.get_window_scale_factor();

        let context = self
            .context
            .as_mut()
            .expect("Context hasnt been created yet run inside impl Game");

        let mut buffer = glyphon::Buffer::new(
            &mut context.text_renderer.font_system,
            Metrics::new(font_size, line_height),
        );

        let physical_width = (size.x as f64 * scale_factor) as f32;
        let physical_height = (size.y as f64 * scale_factor) as f32;

        buffer.set_size(
            &mut context.text_renderer.font_system,
            Some(physical_height),
            Some(physical_width),
        );
        buffer.set_text(
            &mut context.text_renderer.font_system,
            text,
            Attrs::new(),
            Shaping::Basic,
        );

        let height = buffer.lines.len() as f32 * buffer.metrics().line_height;
        let run_width = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .max_by(f32::total_cmp)
            .unwrap_or(0.0);

        Vec2 {
            x: run_width,
            y: height,
        }
    }

    /// Loads in a byte vector resource, can be used to load arbitary files.
    pub fn create_resource<P: AsRef<Path>>(
        &mut self,
        path: P,
        loading_op: LoadingOp,
    ) -> ResourceId<Vec<u8>> {
        let typed_id = resource::generate_id::<Vec<u8>>();
        let id = typed_id.get_id();
        let path = path.as_ref();
        let ip_resource = InProgressResource::new(path, id, ResourceType::Bytes, loading_op);

        self.loader.blocking_load(ip_resource, self.proxy.clone());
        typed_id
    }

    /// This returns the current number of resources that
    /// are loading
    pub fn get_loading_resource_count(&self) -> usize {
        self.loader.get_loading_resources()
    }

    /// Attemps to fetch a byte resource.
    ///
    /// Returns `None` if the resource isnt loaded yet.
    pub fn get_byte_resource(&self, id: ResourceId<Vec<u8>>) -> Option<&Vec<u8>> {
        self.resource_manager.get_byte_resource(&id)
    }

    pub(crate) fn get_proxy(&self) -> EventLoopProxy<BpEvent> {
        self.proxy.clone()
    }

    pub(crate) fn get_resources(&self) -> &ResourceManager {
        &self.resource_manager
    }

    pub(crate) fn defualt_material_bg_id(&self) -> ResourceId<Texture> {
        self.defualt_resources.defualt_texture_id
    }

    pub(crate) fn defualt_pipe_id(&self) -> ResourceId<Shader> {
        self.defualt_resources.default_pipeline_id
    }

    pub(crate) fn line_pipe_id(&self) -> ResourceId<Shader> {
        self.defualt_resources.line_pipeline_id
    }

    /// Takes the struct that implements the Game trait and starts the winit event loop running the game
    pub fn run<T>(mut self, game: T)
    where
        T: Game + 'static,
    {
        let event_loop = self.event_loop.take().unwrap(); //should never panic
        event_loop.run_app(&mut (game, self)).unwrap();
    }

    fn update(&mut self, elwt: &ActiveEventLoop) {
        self.last_frame = Instant::now();
        let dt = self
            .last_frame
            .duration_since(self.current_frametime)
            .as_secs_f32();

        self.ma_frame_time = (self.ma_frame_time + dt) / 2.0;

        if self.should_close {
            elwt.exit();
            return;
        }

        self.input_handle.end_of_frame_refresh();
        if let Some(key) = self.close_key {
            if self.input_handle.is_key_down(key) {
                elwt.exit();
                return;
            }
        }

        if let Some(frame_rate) = self.target_fps {
            let frame_time = Instant::now()
                .duration_since(self.current_frametime)
                .as_nanos() as u64;
            let desired_time_between_frames = 1000000000 / frame_rate as u64;
            if frame_time < desired_time_between_frames {
                self.spin_sleeper
                    .sleep_ns(desired_time_between_frames - frame_time);
            }
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.input_handle.process_input(event)
    }

    fn resize(&mut self, new_size: Vec2<u32>) {
        let context = self
            .context
            .as_mut()
            .expect("Context hasnt been created yet run inside impl Game");

        log::info!("RESZING");

        if new_size.x > 0 && new_size.y > 0 {
            context.config.width = new_size.x;
            context.config.height = new_size.y;
            context
                .surface
                .configure(&context.wgpu.device, &context.config);
            self.size = new_size;
            context.wgpu.queue.write_buffer(
                &context.camera_buffer,
                48,
                bytemuck::cast_slice(&[new_size.x as f32, new_size.y as f32]),
            );
        }
    }

    fn handle_resource(&mut self, resource: Result<Resource, ResourceError>) {
        match resource {
            Ok(data) => {
                self.loader.remove_item_loading(data.loading_op);
                match data.resource_type {
                    ResourceType::Bytes => self.add_finished_bytes(data.data, data.id, &data.path),
                    ResourceType::Image(mag, min) => {
                        self.add_finished_image(data.data, data.id, mag, min, &data.path)
                    }
                    ResourceType::Shader(options) => {
                        self.add_finished_shader(data.data, data.id, options, &data.path)
                    }
                    ResourceType::Font => self.add_finished_font(data),
                }
            }
            Err(e) => {
                log::error!(
                    "could not load resource: {:?}, becuase: {:?}, loading defualt replacement",
                    e,
                    e.error
                );

                self.loader.remove_item_loading(e.loading_op);

                match e.resource_type {
                    ResourceType::Bytes => self.add_defualt_bytes(e.id),
                    ResourceType::Image(..) => self.add_defualt_image(e.id),
                    ResourceType::Shader(_) => self.add_defualt_shader(e.id),
                    ResourceType::Font => self.add_defualt_font(e.id),
                }
            }
        }
    }

    fn add_finished_bytes(&mut self, data: Vec<u8>, id: NonZeroU64, path: &Path) {
        let typed_id: ResourceId<Vec<u8>> = ResourceId::from_number(id);
        self.resource_manager.insert_bytes(typed_id, data);
        log::info!("byte resource at: {:?} loaded succesfully", path);
    }

    fn add_finished_image(
        &mut self,
        data: Vec<u8>,
        id: NonZeroU64,
        mag: SamplerType,
        min: SamplerType,
        path: &Path,
    ) {
        let typed_id: ResourceId<Texture> = ResourceId::from_number(id);
        let texture = Texture::from_resource_data(self, None, data, mag, min);
        match texture {
            Ok(texture) => {
                self.resource_manager.insert_texture(typed_id, texture);
                log::info!("texture resource at: {:?} loaded succesfully", path);
            }
            Err(e) => log::error!("{:?}, loading defualt replacement", e),
        }
    }

    fn add_finished_shader(
        &mut self,
        data: Vec<u8>,
        id: NonZeroU64,
        options: IntermediateOptions,
        path: &Path,
    ) {
        let final_option =
            FinalShaderOptions::from_intermediate(options, self.context.as_ref().unwrap());

        let typed_id: ResourceId<Shader> = ResourceId::from_number(id);
        let shader = Shader::from_resource_data(&data, final_option, self);
        match shader {
            Ok(shader) => {
                self.resource_manager.insert_pipeline(typed_id, shader);
                log::info!("shader resource at: {:?} loaded succesfully", path);
            }
            Err(e) => {
                log::error!("{:?}. loading defualt replacement", e);
                self.add_defualt_shader(id);
            }
        }
    }

    fn add_finished_font(&mut self, resource: Resource) {
        let typed_id: ResourceId<Font> = ResourceId::from_number(resource.id);

        let context = self.context.as_mut().unwrap();

        let font = context.text_renderer.load_font_from_bytes(&resource.data);
        self.resource_manager.insert_font(typed_id, font);
        log::info!("Font resource at: {:?} loaded succesfully", resource.path);
    }

    fn add_defualt_bytes(&mut self, id: NonZeroU64) {
        let typed_id: ResourceId<Vec<u8>> = ResourceId::from_number(id);
        self.resource_manager.insert_bytes(typed_id, Vec::new());
    }

    fn add_defualt_image(&mut self, id: NonZeroU64) {
        let typed_id: ResourceId<Texture> = ResourceId::from_number(id);
        let image = Texture::default(self);
        self.resource_manager.insert_texture(typed_id, image);
    }

    fn add_defualt_shader(&mut self, id: NonZeroU64) {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");

        let typed_id: ResourceId<Shader> = ResourceId::from_number(id);
        let shader = Shader::defualt(&context.wgpu, context.get_texture_format());
        self.resource_manager.insert_pipeline(typed_id, shader);
    }

    fn add_defualt_font(&mut self, id: NonZeroU64) {
        let context = self
            .context
            .as_ref()
            .expect("Context hasnt been created yet run inside impl Game");

        let typed_id: ResourceId<Font> = ResourceId::from_number(id);
        let font = Font::from_str(context.text_renderer.get_defualt_font_name());
        self.resource_manager.insert_font(typed_id, font);
    }

    pub(crate) fn is_loading(&self) -> bool {
        // self.loader.get_loading_resources() > 0
        #[cfg(not(target_arch = "wasm32"))]
        {
            false
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.loader.is_blocked()
        }
    }
}

impl<T: Game> ApplicationHandler<BpEvent> for (T, Engine) {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (_, engine) = self;

        if engine.context.is_none() {
            engine.context = Some(GraphicsContext::from_active_loop(
                event_loop,
                engine.window_options.take().unwrap(),
                &mut engine.resource_manager,
                &engine.defualt_resources,
            ))
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (game, engine) = self;
        if window_id == engine.context.as_ref().unwrap().window.id() && !engine.input(&event) {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    engine.resize(physical_size.into());
                    game.on_resize(physical_size.into(), engine);
                }
                WindowEvent::RedrawRequested => {
                    if engine.is_loading() {
                        engine.update(event_loop);
                    } else {
                        game.update(engine);
                        engine.update(event_loop);
                        engine.current_frametime = Instant::now();

                        match render(game, engine) {
                            Ok(_) => {}
                            // reconfigure surface if lost
                            Err(wgpu::SurfaceError::Lost) => engine.resize(engine.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                event_loop.exit();
                            }
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                }
                e => {
                    log::info!("{:?}", e);
                }
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: BpEvent) {
        let (_, engine) = self;
        match event {
            BpEvent::ResourceLoaded(resource) => engine.handle_resource(resource),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let (_, engine) = self;
        engine.context.as_ref().unwrap().window.request_redraw();
    }
}

/// A builder class that helps create an application
/// with specific details
pub struct EngineBuilder {
    pub(crate) resolution: (u32, u32),
    pub(crate) full_screen: bool,
    target_fps: Option<u16>,
    close_key: Option<Key>,
    pub(crate) window_icon: Option<winit::window::Icon>,
    pub(crate) window_title: String,
    pub(crate) resizable: bool,
    pub(crate) vsync: wgpu::PresentMode,
}

impl EngineBuilder {
    /// Creates a builder with some defualt presets
    /// ```rust
    /// Self {
    ///     resolution: (600, 600),
    ///     full_screen: false,
    ///     target_fps: 30,
    ///     close_key: None,
    ///     window_icon: None,
    ///     window_title: "Bottonless-Pit Game".into(),
    ///     resizable: true,
    ///     vysnc: false,
    /// }
    pub fn new() -> Self {
        Self {
            resolution: (600, 600),
            full_screen: false,
            target_fps: None,
            close_key: None,
            window_icon: None,
            window_title: "Bottomless-Pit Game".into(),
            resizable: true,
            vsync: wgpu::PresentMode::AutoVsync,
        }
    }

    /// Overides the defualt resolution
    pub fn with_resolution(self, resolution: (u32, u32)) -> Self {
        Self {
            resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: self.close_key,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
            vsync: self.vsync,
        }
    }

    /// Will cause the window to be fullscreen upon launch
    pub fn fullscreen(self) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: true,
            target_fps: self.target_fps,
            close_key: self.close_key,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
            vsync: self.vsync,
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
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
            vsync: self.vsync,
        }
    }

    /// Will cause the framerate to be uncapped if the platform supports it using
    /// wgpu's [PresentMode::AutoNoVsync](https://docs.rs/wgpu/latest/wgpu/enum.PresentMode.html)
    /// by defualt the engine uses
    /// [PresentMode::AutoVsync](https://docs.rs/wgpu/latest/wgpu/enum.PresentMode.html)
    pub fn remove_vsync(self) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: self.close_key,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
            vsync: wgpu::PresentMode::AutoNoVsync,
        }
    }

    /// Sets a key that will instantly close the window
    pub fn set_close_key(self, key: Key) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: Some(key),
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
            vsync: self.vsync,
        }
    }

    /// Sets the title of the window
    pub fn set_window_title(self, title: &str) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: self.close_key,
            window_icon: self.window_icon,
            window_title: title.into(),
            resizable: self.resizable,
            vsync: self.vsync,
        }
    }

    /// Sets the window icon
    pub fn set_window_icon(self, icon: winit::window::Icon) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: self.close_key,
            window_icon: Some(icon),
            window_title: self.window_title,
            resizable: self.resizable,
            vsync: self.vsync,
        }
    }

    /// Prevents the window from being resized during runtime
    pub fn unresizable(self) -> Self {
        Self {
            resolution: self.resolution,
            full_screen: self.full_screen,
            target_fps: self.target_fps,
            close_key: self.close_key,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: false,
            vsync: self.vsync,
        }
    }

    /// Attempts to buld the Engine
    pub fn build(self) -> Result<Engine, BuildError> {
        Engine::new(self)
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// All the errors that can occur when creating core engine resources
#[derive(Debug)]
pub enum BuildError {
    /// Occurs when the operating system cannot do certain actions,
    /// typically means the window could not be created.
    WindowOsError(OsError),
    /// Happens when the surface cannont be created. This can happen
    /// on certian OS's or when the browser does not suport WebGL
    CreateSurfaceError(CreateSurfaceError),
    /// Occurs when the WGPU adapter cannot be found.
    FailedToCreateAdapter,
    /// Occurs when the WGPU device cannot be made. This usually
    /// means the OS does not support the minimum graphics features.
    RequestDeviceError(RequestDeviceError),
    #[cfg(target_arch = "wasm32")]
    /// This occurs when the code cannot fetch the JavaScript Window element.
    CantGetWebWindow,
    #[cfg(target_arch = "wasm32")]
    /// This occurs when the Document element cannout be found.
    CantGetDocument,
    #[cfg(target_arch = "wasm32")]
    /// This is any error that can come from the calling of JavaScript
    /// APIs.
    CantGetBody,
    #[cfg(target_arch = "wasm32")]
    JsError(wasm_bindgen::JsValue),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WindowOsError(e) => write!(f, "{}", e),
            Self::CreateSurfaceError(e) => write!(f, "{}", e),
            Self::FailedToCreateAdapter => write!(f, "unable to create Adapater"),
            Self::RequestDeviceError(e) => write!(f, "{}", e),
            #[cfg(target_arch = "wasm32")]
            Self::CantGetWebWindow => write!(f, "could not get web Window"),
            #[cfg(target_arch = "wasm32")]
            Self::CantGetDocument => write!(f, "could not get HTML document"),
            #[cfg(target_arch = "wasm32")]
            Self::CantGetBody => write!(f, "could nto get HTML body tag"),
            #[cfg(target_arch = "wasm32")]
            Self::JsError(e) => write!(f, "{:?}", e),
        }
    }
}

impl std::error::Error for BuildError {}

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

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for BuildError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::JsError(value)
    }
}

/// Errors that can occur when setting the window Icon.
#[derive(Debug)]
pub enum IconError {
    /// Occurs because the image does not meet certain
    /// requirments please see [winit docs](https://docs.rs/winit/latest/winit/window/enum.BadIcon.html).
    BadIcon(BadIcon),
    /// The image file was not a valid image
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

pub(crate) struct DefualtResources {
    pub(crate) defualt_texture_id: ResourceId<Texture>,
    pub(crate) default_pipeline_id: ResourceId<Shader>,
    pub(crate) line_pipeline_id: ResourceId<Shader>,
}

#[derive(Debug)]
pub(crate) enum BpEvent {
    ResourceLoaded(Result<Resource, ResourceError>),
}
