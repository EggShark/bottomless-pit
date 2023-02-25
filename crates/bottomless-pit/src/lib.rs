mod cache;
mod texture;
mod rect;
mod camera;
mod vertex;
mod line;
mod input;
mod draw_queue;
mod matrix_math;
mod colour;

use cgmath::{Point2};
use cache::{TextureCache, TextureIndex};
use input::InputHandle;
use rect::{TexturedRect, Rectangle};
use texture::Texture;
use vertex::Vertex;
use image::GenericImageView;
use wgpu::util::DeviceExt;
use wgpu_glyph::{orthographic_projection, GlyphCruncher};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window}
};
use draw_queue::{DrawQueues, BindGroups};
use line::{Line, LineVertex};
use matrix_math::*;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    camera: camera::Camera,
    camera_controller: camera::CameraController,
    camera_buffer: wgpu::Buffer,
    camera_uniform: camera::CameraUniform,
    camera_bind_group: wgpu::BindGroup,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    draw_queues: DrawQueues,
    glyph_brush: wgpu_glyph::GlyphBrush<(), wgpu_glyph::ab_glyph::FontArc>,
    clear_color: wgpu::Color,
    texture_cahce: TextureCache,
    textured_rect: TexturedRect,
    coloured_rect: Rectangle,
    rendering_stuff: MyRenderingStuff,
    input_handle: InputHandle,
    counter: f32,
}

impl State {
    fn new(window: &Window) -> Self {
        let mut texture_cahce = TextureCache::new();
        let size = window.inner_size();
        // the insance is a handle to our GPU
        // Backends all means Vulkan + Metal + DX12 (probgonna use vulkan <3)
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe {
            instance.create_surface(window)
        };
        let clear_color = wgpu::Color{
            r: 0.0,
            g: 0.0,
            b: 255.0,
            a: 0.0
        };


        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).unwrap();


        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )).unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        };
        surface.configure(&device, &config);

        let camera = camera::Camera::new((0.0, 0.0, 1.0), (0.0, 0.0, 0.0), cgmath::Vector3::unit_y(), config.width as f32/config.height as f32, 45.0, 0.1, 100.0);
        let camera_controller = camera::CameraController::new(0.2);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let diffuse_bytes = include_bytes!("../assets/trans-test.png");
        let diffuse_texture = texture::create_texture_from_bytes(&mut texture_cahce, &device, &queue, diffuse_bytes);
        let diffuse_rect = rect::TexturedRect::new(diffuse_texture, [-0.0, 0.0], [0.5, 0.5]);

        let coloured_rect = rect::Rectangle::new([-1.0, 1.0], [1.0, 0.5], [1.0, 0.0, 0.0, 1.0]);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &Texture::make_bind_group_layout(&device),
                &camera_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState{
                module: &shader,
                entry_point: "vs_main", //specify the entry point (can be whatever as long as it exists)
                buffers: &[Vertex::desc()], // specfies what type of vertices we want to pass to the shader,
            },
            fragment: Some(wgpu::FragmentState{ // techically optional. Used to store colour data to the surface
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState{ // tells wgpu what colour outputs it should set up.
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // specifies that the blending should just replace old pixel data wiht new data,
                    write_mask: wgpu::ColorWrites::ALL, // writes all colours
                })],
            }),
            primitive: wgpu::PrimitiveState{
                topology: wgpu::PrimitiveTopology::TriangleList, // every 3 verticies is one triangle
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw, // triagnle must be counter-clock wise to be considered facing forawrd
                cull_mode: Some(wgpu::Face::Back), // all triagnles not front facing are culled
                // setting this to anything other than fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // requires Features::DEPTH_CLIP_CONTROLL,
                unclipped_depth: false,
                // requires Features::CONSERVATIVE_RASTERIZATION,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState{
                count: 1, // determines how many samples the pipeline will use
                mask: !0, // how many samples the pipeline will use (in this case all of them)
                alpha_to_coverage_enabled: false, // something to do with AA
            },
            multiview: None,
        });
        let minecraft_mono = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!("../assets/Minecraft-Mono.ttf")).unwrap();
        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(minecraft_mono)
            .build(&device, wgpu::TextureFormat::Bgra8UnormSrgb);
        
        let rendering_stuff = MyRenderingStuff::new(&device, &queue, &[&camera_bind_group_layout], config.format);
        Self {
            surface,
            device,
            queue,
            config,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            size,
            clear_color,
            render_pipeline,
            draw_queues: DrawQueues::new(),
            textured_rect: diffuse_rect,
            texture_cahce,
            glyph_brush,
            coloured_rect,
            rendering_stuff,
            input_handle: InputHandle::new(),
            counter: 0.0,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        // true means this function has handled it and the main loop doesnt need too
        // false measn the main loop needs to worry about it
        let input_handler = match event {
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
        };
        self.camera_controller.process_events(event) || input_handler
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        self.counter = (self.counter + 1.0) % 360.0;
        self.input_handle.end_of_frame_refresh();
        let test_line = Line::new([0.1, 0.0], [0.5, 0.0], [0.0, 0.0, 0.0, 1.0]);
        let test_rect = Rectangle::new([-0.5, 0.0], [0.5, 0.5], [0.0, 0.0, 0.0, 1.0]);
        self.draw_queues.add_line(test_line.start, test_line.end);
        self.draw_queues.add_textured_rectange(&mut self.texture_cahce, &self.textured_rect, &self.device);
        self.draw_queues.add_rectangle(&test_rect);
        self.draw_queues.add_rectangle(&self.coloured_rect);
        self.texture_cahce.chache_update();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });
        
        let render_items = self.draw_queues.process_queued(&self.device);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment{
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        if render_items.number_of_rectangle_indicies != 0 {
            render_pass.set_vertex_buffer(0, render_items.rectangle_buffer.slice(..));
            render_pass.set_index_buffer(render_items.rectangle_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            let mut current_bind_group = &render_items.rectangle_bind_group_switches[0];
            for (idx, bind_group_switch_point) in render_items.rectangle_bind_group_switches.iter().enumerate() {
                if bind_group_switch_point.bind_group != current_bind_group.bind_group {
                    current_bind_group = &bind_group_switch_point;
                }
                let bind_group = match &current_bind_group.bind_group {
                    &BindGroups::WhitePixel => &self.rendering_stuff.white_pixel,
                    &BindGroups::Custom {bind_group} => &self.texture_cahce[bind_group].bind_group,
                };
                render_pass.set_bind_group(0, bind_group, &[]);
                let draw_range = match render_items.rectangle_bind_group_switches.get(idx + 1) {
                    Some(switch_point) => current_bind_group.point as u32..switch_point.point as u32,
                    None => current_bind_group.point as u32..render_items.number_of_rectangle_indicies,
                };
                println!("{:?}", draw_range);
                render_pass.draw_indexed(draw_range, 0, 0..1);
            }
        }
        render_pass.set_pipeline(&self.rendering_stuff.line_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, render_items.line_buffer.slice(..));
        render_pass.draw(0..render_items.number_of_line_verticies, 0..1);
        drop(render_pass);

        let mut staging_belt = wgpu::util::StagingBelt::new(100);
        let test_section = wgpu_glyph::Section {
            screen_position: (1.0, 1.0),
            bounds: (self.size.width as f32, self.size.height as f32),
            text: vec![wgpu_glyph::Text::new("ll").with_scale(40.0).with_z(0.0).with_color([0.0, 0.0, 0.0, 1.0,])],
            ..Default::default()
        };
        let text_transform = flatten_matrix(unflatten_matrix(orthographic_projection(self.size.width, self.size.height)) * get_text_rotation_matrix(&test_section, self.counter, &mut self.glyph_brush));
        self.glyph_brush.queue(test_section);
        
        self.glyph_brush.draw_queued_with_transform(
            &self.device, &mut staging_belt, &mut encoder, &view, &self.camera_bind_group, text_transform,
        ).unwrap();

        staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn create_texture(&mut self, path: &str) -> TextureIndex {
        let texture = Texture::from_path(&self.device, &self.queue, None, path).unwrap();
        self.texture_cahce.add_texture(texture)
    }

    fn create_texture_from_bytes(&mut self, bytes: &[u8]) -> TextureIndex {
        let texture = Texture::from_bytes(&self.device, &self.queue, None, bytes).unwrap();
        self.texture_cahce.add_texture(texture)
    }

    fn set_background_colour(&mut self, colour: wgpu::Color) {
        self.clear_color = colour;
    }
}

struct MyRenderingStuff {
    white_pixel: wgpu::BindGroup,
    line_pipeline: wgpu::RenderPipeline,
}

impl MyRenderingStuff {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, bind_group_layouts: &[&wgpu::BindGroupLayout], texture_format: wgpu::TextureFormat) -> Self {
        let white_pixel_image = image::load_from_memory(WHITE_PIXEL).unwrap();
        let white_pixel_rgba = white_pixel_image.to_rgba8();
        let (width, height) = white_pixel_image.dimensions();
        let white_pixel_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let white_pixel_texture = device.create_texture(&wgpu::TextureDescriptor{
            size: white_pixel_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("White_Pixel"),
        });

        queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &white_pixel_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &white_pixel_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(width * 4),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            white_pixel_size,
        );

        let white_pixel_view = white_pixel_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let white_pixel_sampler = device.create_sampler(&wgpu::SamplerDescriptor{
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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {filterable: true},
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
            label: Some("white_pixel_bind_group_layout"),
        });

        let white_pixel = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&white_pixel_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&white_pixel_sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line_shader.wgsl").into()),
        });

        let line_pipeline = make_pipeline(device, wgpu::PrimitiveTopology::LineList, bind_group_layouts, &[line::LineVertex::desc()], &line_shader, texture_format, Some("line_renderer"));

        Self {
            white_pixel,
            line_pipeline,
        }
    }
}

pub fn run(game: Box<dyn Game>) -> ! {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window);
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                game.render();
                match state.render() {
                    Ok(_) => {},
                    // reconfigure surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
                game.update();
                state.update();
            }
            Event::MainEventsCleared => {
                //RedrawRequested will only trigger once, unless we manually request it
                window.request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged{new_inner_size, ..} => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    });
}

pub trait Game {
    fn render(&self);
    fn update(&self);
    fn on_close(&self) {

    } 
}

fn make_pipeline(
    device: &wgpu::Device, 
    topology: wgpu::PrimitiveTopology, 
    bind_group_layouts: &[&wgpu::BindGroupLayout], 
    vertex_buffers: &[wgpu::VertexBufferLayout], 
    shader: &wgpu::ShaderModule, 
    texture_format: wgpu::TextureFormat, label: Option<&str>
) -> wgpu::RenderPipeline {
    let layout_label = match label {
        Some(label) => Some(format!("{} layout", label)),
        None => None
    };

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: layout_label.as_deref(), // somehow converss Option<String> to Option<&str>
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(&layout),
        vertex: wgpu::VertexState{
            module: &shader,
            entry_point: "vs_main", //specify the entry point (can be whatever as long as it exists)
            buffers: vertex_buffers, // specfies what type of vertices we want to pass to the shader,
        },
        fragment: Some(wgpu::FragmentState{ // techically optional. Used to store colour data to the surface
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState{ // tells wgpu what colour outputs it should set up.
                format: texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING), // specifies that the blending should just replace old pixel data wiht new data,
                write_mask: wgpu::ColorWrites::ALL, // writes all colours
            })],
        }),
        primitive: wgpu::PrimitiveState{
            topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw, // triagnle must be counter-clock wise to be considered facing forawrd
            cull_mode: Some(wgpu::Face::Back), // all triagnles not front facing are culled
            // setting this to anything other than fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // requires Features::DEPTH_CLIP_CONTROLL,
            unclipped_depth: false,
            // requires Features::CONSERVATIVE_RASTERIZATION,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState{
            count: 1, // determines how many samples the pipeline will use
            mask: !0, // how many samples the pipeline will use (in this case all of them)
            alpha_to_coverage_enabled: false, // something to do with AA
        },
        multiview: None,
    })
}

const IDENTITY_MATRIX: [f32; 16] = [
    1.0,  0.0,  0.0,  0.0,
    0.0,  1.0,  0.0,  0.0,
    0.0,  0.0,  1.0,  0.0,
    0.0,  0.0,  0.0,  1.0,
];

// just the data for png of a white pixel didnt want it in a seperate file so here is a hard coded const!
const WHITE_PIXEL: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84, 8, 91, 99, 248, 15, 4, 0, 9, 251, 3, 253, 159, 31, 44, 0, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130];