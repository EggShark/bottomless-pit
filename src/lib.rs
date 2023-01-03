mod texture;
mod camera;
use cgmath::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0 , NUM_INSTANCES_PER_ROW as f32 * 0.5);

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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    clear_color: wgpu::Color,
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    cur_tex: bool,
}

impl State {
    async fn new(window: &Window) -> Self {
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

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let camera = camera::Camera::new((0.0, 1.0, 2.0), (0.0, 0.0, 0.0), cgmath::Vector3::unit_y(), (config.width as f32/config.height as f32), 45.0, 0.1, 100.0);
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
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });


        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(TEST_PENTAGRAM),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(TEST_INDICIES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let diffuse_bytes = include_bytes!("../assets/happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, Some("diffuse_texture"), diffuse_bytes).unwrap();

        let texuture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
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
            label: Some("texture_bind_group_layout"),
        });
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            layout: &texuture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texuture_bind_group_layout,
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
                buffers: &[Vertex::desc(), InstanceRaw::desc()], // specfies what type of vertices we want to pass to the shader,
            },
            fragment: Some(wgpu::FragmentState{ // techically optional. Used to store colour data to the surface
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState{ // tells wgpu what colour outputs it should set up.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE), // specifies that the blending should just replace old pixel data wiht new data,
                    write_mask: wgpu::ColorWrites::ALL, // writes all colours
                })],
            }),
            primitive: wgpu::PrimitiveState{
                topology: wgpu::PrimitiveTopology::TriangleList, // every 3 verticies is one triangle
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // triagnle must be counter-clock wise to be considered facing forawrd
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

        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3{x: x as f32, y: 0.0, z: z as f32} - INSTANCE_DISPLACEMENT;
                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Instance {
                    position,
                    rotation,
                }
            })
        }).collect::<Vec<Instance>>();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<InstanceRaw>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let num_indices = TEST_INDICIES.len() as u32;
        let cur_tex = false;
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
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            diffuse_texture,
            instances,
            instance_buffer,
            cur_tex,
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
        let mut res = false;
        res = self.camera_controller.process_events(event);
        res = res || match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.cur_tex {
                    self.swap_texture("assets/happy-tree-cartoon.png");
                } else {
                    self.swap_texture("assets/idle.png");
                }
                self.cur_tex = !self.cur_tex;
                true
            },
            _ => false,
        };

        res
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        for instance in self.instances.iter_mut() {
            let rotation_amout = cgmath::Quaternion::from_angle_y(cgmath::Rad(0.01));
            let current = instance.rotation;
            instance.rotation = rotation_amout * current;
        }

        let instance_data = self.instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<InstanceRaw>>();
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instance_data));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });

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
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // you can only have 1 index buffer at a time
        render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as u32); // draw() ignores the indices
        // render_pass.draw(0..self.num_vertices, 0..1); // tell it to draw something with x verticies and 1 instance of it
        drop(render_pass);
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn set_background_colour(&mut self, colour: wgpu::Color) {
        self.clear_color = colour;
    }

    fn swap_texture(&mut self, path: &str) {
        println!("{}", path);
        let image = std::fs::read(path).unwrap();
        let texture = texture::Texture::from_bytes(&self.device, &self.queue, Some(path), &image).unwrap();
        let texuture_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
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
            label: Some("texture_bind_group_layout"),
        });
        self.diffuse_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texuture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });
        self.diffuse_texture = texture;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout{
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

const TEST_PENTAGRAM: &[Vertex] = &[
    Vertex{position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614]}, // A
    Vertex{position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354]}, // B
    Vertex{position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397]}, // C
    Vertex{position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914]}, // D
    Vertex{position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641]}, // E
];


const TEST_INDICIES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw { 
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout{
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // to a 32x4 is only the four points of the vertex not counting the fact than an instance is 4x4 so we need 4 entries
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32;12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {},
                    // reconfigure surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
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
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
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
