//! Contains everything you need to make some cool shaders including the Shader
//! and UniformData types.

use std::fs;
use std::path::Path;

use encase::ShaderType;
use encase::private::WriteInto;
use wgpu::util::DeviceExt;

use crate::engine_handle::{Engine, WgpuClump};
use crate::texture::UniformTexture;
use crate::vertex::Vertex;
use crate::vectors::Vec2;
use crate::{layouts, render};

/// An internal representation of an WGSL Shader. Under the hood this creates
/// a new pipeline with or without the support for any extra uniforms. To be utilze 
/// the shader it must be added to a material
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shader {
    pub(crate) pipeline_id: wgpu::Id<wgpu::RenderPipeline>,
}

impl Shader {
    pub fn new<P: AsRef<Path>>(path: P, engine: &mut Engine) -> Result<Self, std::io::Error> {
        let wgpu = engine.get_wgpu();

        let shader = wgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("User Shader Module"),
            source: wgpu::ShaderSource::Wgsl(fs::read_to_string(path)?.into())
        });

        let pipeline = render::make_pipeline(
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
        );

        let id = pipeline.global_id();
        engine.add_to_pipeline_cache(pipeline, id);

        Ok(Self {
            pipeline_id: id,
        })
    }

    pub fn new_with_uniforms<P: AsRef<Path>>(path: P, uniform: &UniformData, engine: &mut Engine) -> Result<Self, std::io::Error> {
        let wgpu = engine.get_wgpu();

        let shader = wgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("User Shader Module"),
            source: wgpu::ShaderSource::Wgsl(fs::read_to_string(path)?.into())
        });

        let pipeline = render::make_pipeline(
            &wgpu.device,
            wgpu::PrimitiveTopology::TriangleList,
            &[
                &layouts::create_texture_layout(&wgpu.device),
                &layouts::create_camera_layout(&wgpu.device),
                &uniform.bind_group_layout,
                
            ],
            &[Vertex::desc()],
            &shader,
            engine.get_texture_format(),
            Some("User Shader Pipeline"),
        );

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
    buffer: wgpu::Buffer,
    texture_view: Option<wgpu::TextureView>,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl UniformData {
    pub fn new<T: ShaderType + WriteInto>(engine: &Engine, data: &T) -> Self {
        let wgpu = engine.get_wgpu();
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(&data).unwrap();
        let byte_array = buffer.into_inner();

        let buffer = wgpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("User uniform buffer"),
            contents: &byte_array,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = layouts::create_uniform_layout(&wgpu.device);

        Self {
            buffer,
            texture_view: None,
            bind_group_layout: layout,
        }
    }

    pub fn new_with_extra_texture<T: ShaderType + WriteInto>(data: &T, texture: &UniformTexture, engine: &Engine) -> Self {
        let wgpu = engine.get_wgpu();
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(&data).unwrap();
        let byte_array = buffer.into_inner();

        let buffer = wgpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("User uniform buffer"),
            contents: &byte_array,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = layouts::create_uniform_and_texture_layout(&wgpu.device);

        Self {
            buffer,
            texture_view: Some(texture.get_view()),
            bind_group_layout: layout,
        }
    }

    pub fn update_uniform_data<T: ShaderType + WriteInto>(&self, data: &T, engine: &Engine) {
        let wgpu = engine.get_wgpu();
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(&data).unwrap();
        let byte_array = buffer.into_inner();

        wgpu.queue.write_buffer(&self.buffer, 0, &byte_array);
    }

    pub fn change_texture_size(&mut self, new_size: Vec2<u32>, texture: &mut UniformTexture, engine: &Engine) {
        assert!(self.texture_view.is_some());

        texture.change_size(new_size, engine);
        self.texture_view = Some(texture.get_view());
    }

    pub(crate) fn create_bind_group(&self, sampler: &wgpu::Sampler, wgpu: &WgpuClump) -> wgpu::BindGroup {
        match &self.texture_view {
            Some(view) => {
                wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor{
                    label: Some("User Uniform BindGroup"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            resource: self.buffer.as_entire_binding(),
                            binding: 0,
                        },
                        wgpu::BindGroupEntry {
                            resource: wgpu::BindingResource::TextureView(view),
                            binding: 1,
                        },
                        wgpu::BindGroupEntry {
                            resource: wgpu::BindingResource::Sampler(sampler),
                            binding: 2,
                        }
                    ],
                })
            },
            None => {
                wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("User Uniform BindGroup"),
                    layout: &self.bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        resource: self.buffer.as_entire_binding(),
                        binding: 0,
                    }]
                })
            }
        }
    }
}