mod cache;

use cache::Cache;

pub struct Pipeline {
    transform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    cache: Cache,
    uniform_layout: wgpu::BindGroupLayout,
    uniforms: wgpu::BindGroup,
    raw: wgpu::RenderPipeline,
    instances: wgpu::Buffer,
    current_instances: usize,
    supported_instances: usize,
    current_transform: [f32; 16],
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, filter_mode: wgpu::FilterMode, multisample: wgpu::MultisampleState, render_format: wgpu::TextureFormat, cache_width: u32, cache_height: u32) -> Self {
        build(device, filter_mode, multisample, render_format, cache_width, cache_height)
    }
}

fn build(device: &wgpu::Device, filter_mode: wgpu::FilterMode, multisample: wgpu::MultisampleState, render_format: wgpu::TextureFormat, cache_width: u32, cache_height: u32) -> Pipeline {
    use wgpu::util::DeviceExt;
    let transform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&IDENTITY_MATRIX),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter_mode,
        min_filter: filter_mode,
        mipmap_filter: filter_mode,
        ..Default::default()
    });

    // chache here
    let cache = Cache::new(device, cache_width, cache_height);

    let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("text_render::Pipeline uniforms"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<[f32; 16]>() as u64),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float {filterable: true},
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }
        ],
    });

    let uniforms = create_uniforms(device, &uniform_layout, &transform, &sampler, &cache.view);

    let instances = device.create_buffer(&wgpu::BufferDescriptor{
        label: Some("text_render::Pipeline instances"),
        size: std::mem::size_of::<Instance>() as u64 * Instance::INITIAL_AMOUNT as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        push_constant_ranges: &[],
        bind_group_layouts: &[&uniform_layout],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Text Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("text-shader.wgsl").into()),
    });

    let raw = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Instance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![
                    0 => Float32x3,
                    1 => Float32x2,
                    2 => Float32x2,
                    3 => Float32x2,
                    4 => Float32x4,
                ],
            }],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            front_face: wgpu::FrontFace::Cw,
            strip_index_format: Some(wgpu::IndexFormat::Uint16),
            ..Default::default()
        },
        depth_stencil: None,
        multisample,
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: render_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    });

    Pipeline {
        transform,
        sampler,
        cache,
        uniform_layout,
        uniforms,
        raw,
        instances,
        current_instances: 0,
        supported_instances: Instance::INITIAL_AMOUNT,
        current_transform: [0.0; 16],
    }
}

fn create_uniforms(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, transform: &wgpu::Buffer, sampler: &wgpu::Sampler, cache: &wgpu::TextureView) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("text-redner::Pipeline uniforms"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: transform,
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(cache),
            }
        ]
    })
}

const IDENTITY_MATRIX: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Instance {
    left_top: [f32; 3],
    right_bottom: [f32; 2],
    tex_left_top: [f32; 2],
    tex_right_bottom: [f32; 2],
    color: [f32; 4],
}

impl Instance {
    const INITIAL_AMOUNT: usize = 50_000;
}