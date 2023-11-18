//! Contains both the Engine and the Engine builder
//! Both of these are crucial to using the engine as the
//! builder lets you customize the engine at the start, and the
//! Engine gives you access to all the crucial logic functions

use image::{GenericImageView, ImageError};
use spin_sleep::SpinSleeper;
use web_time::Instant;
use wgpu::util::DeviceExt;
use wgpu::{CreateSurfaceError, RequestDeviceError};
use winit::error::OsError;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
use winit::window::{BadIcon, Window};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;

use crate::colour::Colour;
use crate::input::{InputHandle, Key, MouseKey};
use crate::render::{make_pipeline, render};
use crate::vectors::Vec2;
use crate::vertex::{Vertex, LineVertex};
use crate::{WHITE_PIXEL, resource};
use crate::{Game, IDENTITY_MATRIX, layouts};
use crate::resource::{Resource, InProgressResource, ResourceType, ResourceId, ResourceError, compare_resources};

pub(crate) type WgpuCache<T> = HashMap<wgpu::Id<T>, T>;

/// The thing that makes the computer go
pub struct Engine {
    input_handle: InputHandle,
    window: Window,
    surface: wgpu::Surface,
    event_loop: Option<EventLoop<BpEvent>>,
    event_loop_proxy: Arc<EventLoopProxy<BpEvent>>, 
    cursor_visibility: bool,
    should_close: bool,
    close_key: Option<Key>,
    target_fps: Option<u16>,
    last_frame: Instant,
    spin_sleeper: SpinSleeper,
    current_frametime: Instant,
    texture_sampler: wgpu::Sampler,
    clear_colour: Colour,
    config: wgpu::SurfaceConfiguration,
    camera_matrix: [f32; 16],
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    wgpu_clump: WgpuClump,
    size: Vec2<u32>,
    in_progress_resources: Vec<InProgressResource>,
    loading: bool,
    defualt_bind_group_id: wgpu::Id<wgpu::BindGroup>,
    default_pipeline_id: wgpu::Id<wgpu::RenderPipeline>,
    line_pipeline_id: wgpu::Id<wgpu::RenderPipeline>,
    pub(crate) pipelines: WgpuCache<wgpu::RenderPipeline>,
    pub(crate) bindgroups: WgpuCache<wgpu::BindGroup>,
}

impl Engine {
    fn new(builder: EngineBuilder) -> Result<Self, BuildError> {
        cfg_if::cfg_if!{
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            } else {
                env_logger::init();
            }
        }

        let cursor_visibility = true;
        let input_handle = InputHandle::new();
        let size: Vec2<u32> = builder.resolution.into();
        let target_fps = builder.target_fps;

        let event_loop: EventLoop<BpEvent> = EventLoopBuilder::with_user_event().build();
        let event_loop_proxy = Arc::new(event_loop.create_proxy());
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

        #[cfg(target_arch="wasm32")]
        {
            let web_window = web_sys::window().expect("could not get window");
            use winit::platform::web::WindowExtWebSys;
            web_window
                .document()
                .and_then(|doc| {
                    let body = doc.body()?;
                    let canvas = web_sys::Element::from(window.canvas());
                    body.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body");
        }

        let backend = if cfg!(target_os = "windows") {
            wgpu::Backends::DX12 // text rendering gets angry on vulkan
        } else {
            wgpu::Backends::all()
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
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

        let limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits,
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
            present_mode: builder.vsync,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&wgpu_clump.device, &config);

        let texture_sampler = wgpu_clump.device.create_sampler(&wgpu::SamplerDescriptor {
            // what to do when given cordinates outside the textures height/width
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // what do when give less or more than 1 pixel to sample
            // linear interprelates between all of them nearest gives the closet colour
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

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

        let texture_format = config.format;

        let white_pixel_image = image::load_from_memory(WHITE_PIXEL).unwrap();
        let white_pixel_rgba = white_pixel_image.to_rgba8();
        let (width, height) = white_pixel_image.dimensions();
        let white_pixel_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let white_pixel_texture = wgpu_clump.device.create_texture(&wgpu::TextureDescriptor {
            size: white_pixel_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            view_formats: &[],
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("White_Pixel"),
        });

        wgpu_clump.queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &white_pixel_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &white_pixel_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            white_pixel_size,
        );

        let white_pixel_view =
            white_pixel_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let white_pixel_sampler = wgpu_clump.device.create_sampler(&wgpu::SamplerDescriptor {
            // what to do when given cordinates outside the textures height/width
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // what do when give less or more than 1 pixel to sample
            // linear interprelates between all of them nearest gives the closet colour
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            wgpu_clump
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("white_pixel_bind_group_layout"),
                });

        let white_pixel = wgpu_clump
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&white_pixel_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&white_pixel_sampler),
                    },
                ],
                label: Some("texture_bind_group"),
            });

        let generic_shader = wgpu_clump
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
            });

        let line_shader = wgpu_clump
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Line_Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line_shader.wgsl").into()),
            });

        let generic_pipeline = make_pipeline(
            &wgpu_clump.device,
            wgpu::PrimitiveTopology::TriangleList,
            &[&texture_bind_group_layout, &camera_bind_group_layout],
            &[Vertex::desc()],
            &generic_shader,
            texture_format,
            Some("Defualt Pipeline"),
        );

        let line_pipeline = make_pipeline(
            &wgpu_clump.device,
            wgpu::PrimitiveTopology::LineList,
            &[&camera_bind_group_layout],
            &[LineVertex::desc()],
            &line_shader,
            texture_format,
            Some("line_renderer"),
        );
    
        let line_id = line_pipeline.global_id();
        let generic_id = generic_pipeline.global_id();
        let mut pipelines = HashMap::new();
        pipelines.insert(line_id, line_pipeline);
        pipelines.insert(generic_id, generic_pipeline);

        let white_pixel_id = white_pixel.global_id();
        let mut bindgroups = HashMap::new();
        bindgroups.insert(white_pixel_id, white_pixel);


        Ok(Self {
            input_handle,
            window,
            surface,
            event_loop: Some(event_loop),
            event_loop_proxy,
            cursor_visibility,
            should_close: false,
            close_key: builder.close_key,
            target_fps,
            last_frame: Instant::now(),
            current_frametime: Instant::now(),
            spin_sleeper: SpinSleeper::default(),
            texture_sampler,
            clear_colour: builder.clear_colour,
            config,
            camera_matrix,
            camera_bind_group,
            camera_buffer,
            wgpu_clump,
            size,
            in_progress_resources: Vec::new(),
            loading: false,
            defualt_bind_group_id: white_pixel_id,
            default_pipeline_id: generic_id,
            line_pipeline_id: line_id,
            pipelines,
            bindgroups,
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
        self.size
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
        self.camera_matrix = matrix;
        self.wgpu_clump.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_matrix]),
        );
    }

    /// nessicary for creating shaders and wanting to include the camera in the layouts
    /// this is required if you would like to use the camera in the shader
    pub fn camera_layout(&self) -> wgpu::BindGroupLayout {
        layouts::create_camera_layout(&self.wgpu_clump.device)
    }

    pub(crate) fn camera_bindgroup(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    /// nessicary for creating shaders and wanting to include textures in the layouts
    /// this is required if you would like to use textures in your shader
    pub fn texture_layout(&self) -> wgpu::BindGroupLayout {
        layouts::create_texture_layout(&self.wgpu_clump.device)
    }

    /// nessicary for creating shaders and wanting to use your own unifroms
    /// this should be pretty generic as it is exposed to both the vertex 
    /// and fragment stage
    pub fn uniform_layout(&self) -> wgpu::BindGroupLayout {
        layouts::create_uniform_layout(&self.wgpu_clump.device)
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
    /// as the next frame is ready unless VSYNC is on then it will draw at the
    /// VYSNC rate, which is dependant on user hardware.
    pub fn remove_target_fps(&mut self) {
        self.target_fps = None;
    }

    /// Will turn off vysnc if the platform suports it, using 
    /// AutoNoVsync for more information check
    /// [PresentMode::AutoNoVsync](https://docs.rs/wgpu/latest/wgpu/enum.PresentMode.html).
    pub fn remove_vsync(&mut self) {
        self.config.present_mode = wgpu::PresentMode::AutoNoVsync;
        self.surface.configure(&self.wgpu_clump.device, &self.config);
    }

    /// Will turn off vysnc if the platform suports it, using 
    /// AutoVsync for more information check
    /// [PresentMode::AutoVsync](https://docs.rs/wgpu/latest/wgpu/enum.PresentMode.html).
    pub fn add_vsync(&mut self) {
        self.config.present_mode = wgpu::PresentMode::AutoVsync;
        self.surface.configure(&self.wgpu_clump.device, &self.config);
    }

    /// Sets a target fps cap. The thread will spin sleep using the
    /// [spin_sleep](https://docs.rs/spin_sleep/latest/spin_sleep/index.html) crate
    /// untill the desired frame time is reached. If the frames are slower than the target
    /// no action is taken to "speed" up the rendering and updating
    pub fn set_target_fps(&mut self, fps: u16) {
        self.target_fps = Some(fps);
    }

    pub fn create_resource<P: AsRef<Path>>(&mut self, path: P) -> ResourceId<Vec<u8>> {
        let typed_id = resource::generate_id::<Vec<u8>>();
        let id = typed_id.get_id();
        let path = path.as_ref();
        let ip_resource = InProgressResource::new(path, id, ResourceType::Bytes);
        
        resource::start_load(&self, path, &ip_resource);
        
        self.in_progress_resources.push(ip_resource);
        typed_id
    }

    pub(crate) fn get_wgpu(&self) -> &WgpuClump {
        &self.wgpu_clump
    }

    pub(crate) fn get_texture_sampler(&self) -> &wgpu::Sampler {
        &self.texture_sampler
    }

    pub(crate) fn get_texture_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub(crate) fn get_proxy(&self) -> Arc<EventLoopProxy<BpEvent>> {
        self.event_loop_proxy.clone()
    }

    /// Used when adding shader options, and textures into the bindgroup cahce !
    pub(crate) fn add_to_bind_group_cache(&mut self, bind_group: wgpu::BindGroup, key: wgpu::Id<wgpu::BindGroup>) {
        self.bindgroups.insert(key, bind_group);
    }

    pub(crate) fn add_to_pipeline_cache(&mut self, pipeline: wgpu::RenderPipeline, key: wgpu::Id<wgpu::RenderPipeline>) {
        self.pipelines.insert(key, pipeline);
    }

    pub(crate) fn defualt_material_bg_id(&self) -> wgpu::Id<wgpu::BindGroup> {
        self.defualt_bind_group_id
    }

    pub(crate) fn defualt_pipe_id(&self) -> wgpu::Id<wgpu::RenderPipeline> {
        self.default_pipeline_id
    }

    pub(crate) fn line_pipe_id(&self) -> wgpu::Id<wgpu::RenderPipeline> {
        self.line_pipeline_id
    }

    pub(crate) fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    pub(crate) fn wgpu_colour(&self) -> wgpu::Color {
        self.clear_colour.into()
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
                    if self.is_loading() {
                        self.update(control_flow);
                    } else {
                        game.update(&mut self);
                        self.update(control_flow);
                        self.current_frametime = Instant::now();
                        
                        match render(&mut game, &mut self) {
                            Ok(_) => {}
                            // reconfigure surface if lost
                            Err(wgpu::SurfaceError::Lost) => self.resize(self.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                            Err(e) => eprintln!("{:?}", e),
                        }
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
                                self.resize(s.into())
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                let s = **new_inner_size;
                                self.resize(s.into());
                            }
                            _ => {}
                        }
                    }
                }
                Event::UserEvent(event) => {
                    self.handle_user_event(event);
                }
                _ => {}
            }
        });
    }

    fn update(&mut self, control_flow: &mut ControlFlow) {
        self.last_frame = Instant::now();
        // self.renderer.bind_group_cache.cache_update();
        self.input_handle.end_of_frame_refresh();
        if let Some(key) = self.close_key {
            if self.input_handle.is_key_down(key) {
                *control_flow = ControlFlow::Exit;
                return;
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
            self.config.width = new_size.x;
            self.config.height = new_size.y;
            self.surface
                .configure(&self.wgpu_clump.device, &self.config);
            self.size = new_size;
        }
    }

    fn handle_user_event(&mut self, event: BpEvent) {
        log::warn!("{:?}", event);
        match event {
            BpEvent::ResourceLoaded(resource) => self.handle_resource(resource),
        }
    }

    fn handle_resource(&mut self, resource: Result<Resource, ResourceError>) {
        // remove ip resource
        let idx = self.in_progress_resources
            .iter()
            .enumerate()
            .find(|(_, ip_resource)| compare_resources(ip_resource, &resource))
            .and_then(|(idx, _)| Some(idx))
            .expect("Finished resource doesnt match any in progress resources");

        self.in_progress_resources.remove(idx);

        match resource {
            Ok(data) => {
                match data.resource_type {
                    ResourceType::Bytes => {},
                    ResourceType::Image => todo!(),
                    ResourceType::Shader => todo!(),
                }
            }
            Err(e) => {
                log::error!("{:?}, loading defualt replacement", e);
            }
        }
    }

    fn is_loading(&self) -> bool {
        self.in_progress_resources.len() > 0
    }
}

/// A builder class that helps create an application
/// with specific details
pub struct EngineBuilder {
    resolution: (u32, u32),
    full_screen: bool,
    target_fps: Option<u16>,
    close_key: Option<Key>,
    clear_colour: Colour,
    window_icon: Option<winit::window::Icon>,
    window_title: String,
    resizable: bool,
    vsync: wgpu::PresentMode,
}

impl EngineBuilder {
    /// Creates a builder with some defualt presets
    /// ```rust
    /// Self {
    ///     resolution: (600, 600),
    ///     full_screen: false,
    ///     target_fps: 30,
    ///     close_key: None,
    ///     clear_colour: Colour::BLACK,
    ///     window_icon: None,
    ///     window_title: "".into(),
    ///     resizable: true,
    ///     vysnc: false,
    /// }
    pub fn new() -> Self {
        Self {
            resolution: (600, 600),
            full_screen: false,
            target_fps: None,
            close_key: None,
            clear_colour: Colour::BLACK,
            window_icon: None,
            window_title: "".into(),
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
            clear_colour: self.clear_colour,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
            vsync: self.vsync
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
            vsync: self.vsync
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
            vsync: self.vsync
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
            clear_colour: self.clear_colour,
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
            clear_colour: self.clear_colour,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: self.resizable,
            vsync: self.vsync,
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
            clear_colour: self.clear_colour,
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
            clear_colour: self.clear_colour,
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
            clear_colour: self.clear_colour,
            window_icon: self.window_icon,
            window_title: self.window_title,
            resizable: false,
            vsync: self.vsync,
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

#[derive(Debug)]
pub(crate) enum BpEvent {
    ResourceLoaded(Result<Resource, ResourceError>),
}