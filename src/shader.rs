//! Contains everything you need to make some cool shaders including the Shader
//! and UniformData types.

use std::path::Path;
use std::string::FromUtf8Error;

use encase::ShaderType;
use encase::private::WriteInto;
use wgpu::util::DeviceExt;

use crate::engine_handle::{Engine, WgpuClump};
use crate::resource::{self, ResourceId, ResourceType, InProgressResource};
use crate::vertex::Vertex;
use crate::{layouts, render};

/// An internal representation of an WGSL Shader. Under the hood this creates
/// a new pipeline with or without the support for any extra uniforms. To be utilze 
/// the shader it must be added to a material
#[derive(Debug)]
pub struct Shader {
    pub(crate) pipeline: wgpu::RenderPipeline
}

impl Shader {
    pub fn new<P: AsRef<Path>>(path: P, has_uniforms: bool, engine: &mut Engine) -> ResourceId<Shader> {
        let typed_id = resource::generate_id::<Shader>();
        let id = typed_id.get_id();
        let path = path.as_ref();
        let ip_resource = InProgressResource::new(path, id, ResourceType::Shader(has_uniforms));

        resource::start_load(&engine, path, &ip_resource);
        engine.add_in_progress_resource(ip_resource);

        typed_id
    }

    pub(crate) fn from_resource_data(data: &[u8], has_uniforms: bool, engine: &Engine) -> Result<Self, FromUtf8Error> {
        let wgpu = engine.get_wgpu();

        let string = String::from_utf8(data.to_vec())?;
        let shader = wgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("User Shader Module"),
            source: wgpu::ShaderSource::Wgsl(string.into())
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

        Ok(Self {
            pipeline,
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