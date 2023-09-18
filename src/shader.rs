use std::fs;
use std::path::Path;

use encase::ShaderType;
use encase::private::WriteInto;
use wgpu::util::DeviceExt;

use crate::engine_handle::{Engine, WgpuClump};
use crate::vertex::Vertex;
use crate::{layouts, render};

pub struct ShaderBuilder<'a> {
    shader: wgpu::ShaderModule,
    layouts: &'a [&'a wgpu::BindGroupLayout],
}

impl<'a> ShaderBuilder<'a> {
    pub fn new<P: AsRef<Path>>(engine: &Engine, path: P) -> Result<Self, std::io::Error> {
        let wgpu = engine.get_wgpu();

        let shader = wgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("User Shader Module"),
            source: wgpu::ShaderSource::Wgsl(fs::read_to_string(path)?.into())
        });

        Ok(Self {
            shader,
            layouts: &[],
        })
    }

    pub fn set_layouts(self, layouts: &'a[&'a wgpu::BindGroupLayout]) -> Self {
        Self {
            shader: self.shader,
            layouts,
        }
    }

    pub fn register(self, engine: &mut Engine) -> RegisteredShader {
        RegisteredShader::from_builder(self, engine)
    }
}

pub struct RegisteredShader {
    pub(crate) pipeline_id: wgpu::Id<wgpu::RenderPipeline>,
}

impl RegisteredShader {
    fn from_builder(builder: ShaderBuilder, engine: &mut Engine) -> Self {
        let device = &engine.get_wgpu().device;
        let pipeline = render::make_pipeline(
            device,
            wgpu::PrimitiveTopology::TriangleList,
            builder.layouts,
            &[Vertex::desc()],
            &builder.shader,
            engine.get_texture_format(),
            Some("User Shader Pipeline"),
        );
        let pipeline_id = pipeline.global_id();

        engine.add_to_pipeline_cache(pipeline, pipeline_id);

        Self {
            pipeline_id,
        }
    }
}

pub struct UniformData {
    initial_data: Vec<u8>,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl UniformData {
    pub fn new<T: ShaderType + WriteInto>(engine: &Engine, data: &T) -> Self {
        let wgpu = engine.get_wgpu();
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(&data).unwrap();
        let byte_array = buffer.into_inner();

        let layout = layouts::create_uniform_layout(&wgpu.device);

        Self {
            initial_data: byte_array,
            bind_group_layout: layout,
        }
    }

    pub(crate) fn extract_buffer_and_bindgroup(&self, wgpu: &WgpuClump) -> (wgpu::Buffer, wgpu::BindGroup) {
        let buffer = wgpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("User uniform buffer"),
            contents: &self.initial_data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("User Uniform BindGroup"),
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                resource: buffer.as_entire_binding(),
                binding: 0,
            }],
        });

        (buffer, bind_group)
    }
}