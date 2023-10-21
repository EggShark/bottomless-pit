//! Contains everything you need to make some cool shaders including the Shader
//! and UniformData types.

use std::fs;
use std::path::Path;

use encase::ShaderType;
use encase::private::WriteInto;
use wgpu::util::DeviceExt;

use crate::engine_handle::{Engine, WgpuClump};
use crate::vertex::Vertex;
use crate::{layouts, render};

/// An internal representation of an WGSL Shader. Under the hood this creates
/// a new pipeline with or without the support for any extra uniforms. To be utilze 
/// the shader it must be added to a material
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shader {
    pub(crate) pipeline_id: wgpu::Id<wgpu::RenderPipeline>,
}

impl Shader {
    pub fn new<P: AsRef<Path>>(path: P, has_uniforms: bool, engine: &mut Engine) -> Result<Self, std::io::Error> {
        let wgpu = engine.get_wgpu();

        let shader = wgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("User Shader Module"),
            source: wgpu::ShaderSource::Wgsl(fs::read_to_string(path)?.into())
        });

        let pipeline = if has_uniforms {
            render::make_pipeline(
                &wgpu.device,
                wgpu::PrimitiveTopology::TriangleList,
                &[
                    &layouts::create_texture_layout(&wgpu.device),
                    &layouts::create_camera_layout(&wgpu.device),
                    &layouts::create_uniform_layout(&wgpu.device),
                ],
                &[Vertex::desc()],
                &shader,
                engine.get_texture_format(),
                Some("User Shader Pipeline"),
            )
        } else {
            render::make_pipeline(
                &wgpu.device,
                wgpu::PrimitiveTopology::TriangleList,
                &[
                    &layouts::create_texture_layout(&wgpu.device),
                    &layouts::create_camera_layout(&wgpu.device),
                ],
                &[Vertex::desc()],
                &shader,
                engine.get_texture_format(),
                Some("User Shader Pipeline"),
            )
        };

        let id = pipeline.global_id();
        engine.add_to_pipeline_cache(pipeline, id);

        Ok(Self {
            pipeline_id: id,
        })
    }
}

/// `UniformData` contains the byte data of any struct that implements
/// [ShaderType](https://docs.rs/encase/latest/encase/trait.ShaderType.html) which 
/// can be derived. This data needs to be added to a Material upon creation.
#[derive(Debug)]
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