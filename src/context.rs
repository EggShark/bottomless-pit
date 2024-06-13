use std::sync::Arc;
use crate::{resource, WHITE_PIXEL};
use crate::layouts;
use crate::render::make_pipeline;
use crate::text::TextRenderer;
use crate::texture::Texture;
use crate::shader::Shader;
use crate::vectors::Vec2;
use crate::vertex::LineVertex;

use image::GenericImageView;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};
use wgpu::util::DeviceExt;


pub(crate) struct GraphicsContext {
    wgpu_clump: WgpuClump,
    window: Arc<Window>,
    texture_sampler: wgpu::Sampler,
    config: wgpu::SurfaceConfiguration,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    text_renderer: TextRenderer,
}

impl GraphicsContext {
    pub fn from_active_loop(event_loop: &ActiveEventLoop, window_options: WindowOptions) {
        // should never fail as we will always set it
        let size: Vec2<u32> = window_options.attributes.inner_size.unwrap().into();

        let window = Arc::new(event_loop.create_window(window_options.attributes).unwrap());

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let web_window = web_sys::window().ok_or(BuildError::CantGetWebWindow)?;
            let canvas = web_sys::Element::from(window.canvas().unwrap());
            let document = web_window.document().ok_or(BuildError::CantGetDocument)?;

            match document.get_element_by_id(&builder.window_title) {
                Some(element) => {
                    let array = js_sys::Array::new();
                    array.push(&wasm_bindgen::JsValue::from(canvas));
                    element.replace_with_with_node(&array)?;
                }
                None => {
                    log::warn!(
                        "coudn't find desitantion <canvas> with id: {}, appending to body",
                        builder.window_title
                    );
                    canvas.set_id(&builder.window_title);
                    let body = document.body().ok_or(BuildError::CantGetBody)?;
                    body.append_child(&canvas)?;
                }
            }
        }

        let backend = wgpu::Backends::all();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }));

        let adapter = match adapter {
            Some(a) => a,
            None => panic!("AHHHHHHHH no adapter"),
        };

        let limits = adapter.limits();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                label: None,
            },
            None,
        )).unwrap();

        let wgpu_clump = WgpuClump { device, queue };

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.x,
            height: size.y,
            present_mode: window_options.presentation,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
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

        let camera_matrix = [
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            size.x as f32,
            size.y as f32,
            0.0,
            0.0,
        ];

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

        let texture_bind_group_layout = layouts::create_texture_layout(&wgpu_clump.device);

        let white_pixel_bind_group =
            wgpu_clump
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

        let white_pixel = Texture::new_direct(
            white_pixel_view,
            white_pixel_bind_group,
            Vec2 { x: 1.0, y: 1.0 },
        );

        let line_shader = wgpu_clump
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Line_Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line_shader.wgsl").into()),
            });

        let line_pipeline = make_pipeline(
            &wgpu_clump.device,
            wgpu::PrimitiveTopology::LineList,
            &[&camera_bind_group_layout],
            &[LineVertex::desc()],
            &line_shader,
            texture_format,
            Some("line_renderer"),
        );

        let text_renderer = TextRenderer::new(&wgpu_clump);

        let line_id = resource::generate_id::<Shader>();
        let generic_id = resource::generate_id::<Shader>();
        let line_shader = Shader::from_pipeline(line_pipeline);
        let generic_shader = Shader::defualt(&wgpu_clump, texture_format);


    }
}

struct WindowOptions {
    attributes: WindowAttributes,
    presentation: wgpu::PresentMode,
}

pub(crate) struct WgpuClump {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}
