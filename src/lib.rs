use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    clear_color: wgpu::Color,
    num_vertices: u32,
    num_indices: u32,
    cur_shape: bool,
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
            b: 0.0,
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState{
                module: &shader,
                entry_point: "vs_main", //specify the entry point (can be whatever as long as it exists)
                buffers: &[Vertex::desc(),], // specfies what type of vertices we want to pass to the shader,
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

        let num_vertices = TEST_PENTAGRAM.len() as u32;
        let num_indices = TEST_INDICIES.len() as u32;
        let cur_shape = true;
        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            num_vertices,
            cur_shape,
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
        match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.cur_shape = !self.cur_shape;
                let mut shape = Vec::new();
                let mut indicies = Vec::new();
                if self.cur_shape {
                    shape = TEST_PENTAGRAM.iter().map(|p| *p).collect::<Vec<Vertex>>();
                    indicies = TEST_INDICIES.iter().map(|p| *p).collect::<Vec<u16>>();
                } else {
                    let num_vertices = 16;
                    let angle = std::f32::consts::PI * 2.0 / num_vertices as f32;
                    let challenge_verts = (0..num_vertices)
                        .map(|i| {
                            let theta = angle * i as f32;
                            Vertex {
                                position: [0.5 * theta.cos(), -0.5 * theta.sin(), 0.0],
                                colour: [(1.0 + theta.cos()) / 2.0, (1.0 + theta.sin()) / 2.0, 1.0],
                            }
                        })
                        .collect::<Vec<_>>();

                    let num_triangles = num_vertices - 2;
                    let challenge_indices = (1u16..num_triangles + 1)
                        .into_iter()
                        .flat_map(|i| vec![i + 1, i, 0])
                        .collect::<Vec<_>>();
                    shape = challenge_verts;
                    indicies = challenge_indices;
                }
                self.change_buffer(&shape, &indicies);
                true
            },
            _ => false,
        }
    }

    fn update(&mut self) {

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
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // you can only have 1 index buffer at a time
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // draw() ignores the indices
        // render_pass.draw(0..self.num_vertices, 0..1); // tell it to draw something with x verticies and 1 instance of it
        drop(render_pass);
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn change_buffer(&mut self, shape: &[Vertex], indicies: &[u16]) {
        self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(shape),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(indicies),
            usage: wgpu::BufferUsages::INDEX,
        });

        self.num_vertices = shape.len() as u32;
        self.num_indices = indicies.len() as u32;
    }

    fn set_background_colour(&mut self, colour: wgpu::Color) {
        self.clear_color = colour;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    colour: [f32; 3],
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
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}

const TEST_TRIANGLE: &[Vertex] = &[
    Vertex{position: [0.0, 0.5, 0.0], colour: [1.0, 0.0, 0.0]},
    Vertex{position: [-0.5, -0.5, 0.0], colour: [0.0, 1.0, 0.0]},
    Vertex{position: [0.5, -0.5, 0.0], colour: [0.0, 0.0, 1.0]},
];

const TEST_PENTAGRAM: &[Vertex] = &[
    Vertex{position: [-0.0868241, 0.49240386, 0.0], colour: [0.5, 0.0, 0.5]}, // A
    Vertex{position: [-0.49513406, 0.06958647, 0.0], colour: [0.5, 0.0, 0.5]}, // B
    Vertex{position: [-0.21918549, -0.44939706, 0.0], colour: [0.5, 0.0, 0.5]}, // C
    Vertex{position: [0.35966998, -0.3473291, 0.0], colour: [0.5, 0.0, 0.5]}, // D
    Vertex{position: [0.44147372, 0.2347359, 0.0], colour: [0.5, 0.0, 0.5]}, // E
];

const TEST_INDICIES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

const COOL_SHAPE: &[Vertex] = &[
    Vertex{position: [0.5, 0.5, 0.0], colour: [0.0, 1.0, 1.0]},
    Vertex{position: [0.5, 1.0, 0.0], colour: [1.0, 1.0, 0.0]},
    Vertex{position: [1.0, 1.0, 0.0], colour: [1.0, 1.0, 1.0]},
    Vertex{position: [1.0, 0.0, 0.0], colour: [1.0, 1.0, 1.0]},
    Vertex{position: [-1.0, 0.0, 0.0], colour: [1.0, 1.0, 1.0]},
    Vertex{position: [-1.0, 0.5, 0.0], colour: [1.0, 1.0, 1.0]},
];

const COOL_INDICIES: &[u16] = &[
    1, 0, 2,
    2, 0, 3,
    0, 5, 3,
    3, 5, 4,
];

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
