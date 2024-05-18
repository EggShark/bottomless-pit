//! Contains everything you need to make some cool shaders including the Shader
//! and UniformData types.

use std::path::Path;
use std::string::FromUtf8Error;

use encase::private::WriteInto;
use encase::ShaderType;
use wgpu::include_wgsl;
use wgpu::util::DeviceExt;

use crate::engine_handle::{Engine, WgpuClump};
use crate::resource::{self, InProgressResource, ResourceId, ResourceType};
use crate::texture::UniformTexture;
use crate::vertex::Vertex;
use crate::{layouts, render};

/// An internal representation of an WGSL Shader. Under the hood this creates
/// a new pipeline with or without the support for any extra uniforms. To be utilze
/// the shader it must be added to a material
#[derive(Debug)]
pub struct Shader {
    pub(crate) pipeline: wgpu::RenderPipeline,
    options: ShaderOptions,
}

impl Shader {
    /// Attempts to create a shader from a file. This will halt the engine due to resource loading please see
    /// the [resource module](crate::resource) for more information.
    pub fn new<P: AsRef<Path>>(
        path: P,
        options: ShaderOptions,
        engine: &mut Engine,
    ) -> ResourceId<Shader> {
        let typed_id = resource::generate_id::<Shader>();
        let id = typed_id.get_id();
        let path = path.as_ref();
        let ip_resource = InProgressResource::new(path, id, ResourceType::Shader(options));

        resource::start_load(engine, path, ip_resource);
        engine.add_in_progress_resource();

        typed_id
    }

    /// Attempts to create a shader from a byte array, this will not halt the engine. See the
    /// [resource module](crate::resource) for more information on this halting behavior.
    pub fn from_btyes(engine: &mut Engine, options: ShaderOptions, bytes: &[u8]) -> ResourceId<Shader> {
        let shader = Self::from_resource_data(bytes, options, engine).unwrap_or_else(|e| {
            log::warn!("{}, occured loading defualt replacement", e);
            Self::defualt(engine.get_wgpu(), engine.get_texture_format())
        });

        let typed_id = resource::generate_id::<Shader>();
        engine.resource_manager.insert_pipeline(typed_id, shader);

        typed_id
    }

    pub(crate) fn from_resource_data(
        data: &[u8],
        options: ShaderOptions,
        engine: &Engine,
    ) -> Result<Self, FromUtf8Error> {
        let wgpu = engine.get_wgpu();

        let string = String::from_utf8(data.to_vec())?;
        let shader = wgpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("User Shader Module"),
                source: wgpu::ShaderSource::Wgsl(string.into()),
            });

        let optional_layout = options.make_layout(&wgpu.device);
        let pipeline = if let Some(layout) = optional_layout {
            render::make_pipeline(
                &wgpu.device,
                wgpu::PrimitiveTopology::TriangleList,
                &[
                    &layouts::create_texture_layout(&wgpu.device),
                    &layouts::create_camera_layout(&wgpu.device),
                    &layout
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
            options,
        })
    }

    pub(crate) fn from_pipeline(pipeline: wgpu::RenderPipeline) -> Self {
        Self {
            pipeline,
            options: ShaderOptions::EMPTY,
        }
    }

    pub(crate) fn defualt(wgpu: &WgpuClump, texture_format: wgpu::TextureFormat) -> Self {
        let shader_descriptor = include_wgsl!("shaders/shader.wgsl");
        let shader = wgpu.device.create_shader_module(shader_descriptor);
        let pipeline = render::make_pipeline(
            &wgpu.device,
            wgpu::PrimitiveTopology::TriangleList,
            &[
                &layouts::create_texture_layout(&wgpu.device),
                &layouts::create_camera_layout(&wgpu.device),
            ],
            &[Vertex::desc()],
            &shader,
            texture_format,
            Some("Defualt Shader From Error"),
        );

        Self {
            pipeline,
            options: ShaderOptions::EMPTY,
        }
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

    pub(crate) fn extract_buffer_and_bindgroup(
        &self,
        wgpu: &WgpuClump,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let buffer = wgpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

#[derive(Debug)]
pub struct ShaderOptions {
    uniform_data: Option<wgpu::Buffer>,
    uniform_texture: Option<wgpu::TextureView>,
}

impl ShaderOptions {
    //TODO add support for custom texture samplers
    pub const EMPTY: Self = Self {
        uniform_data: None,
        uniform_texture: None,
    };

    pub fn with_uniform_data(engine: &Engine, data: &UniformData) -> Self {
        let device = &engine.get_wgpu().device;

        let starting_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("User uniform buffer"),
                contents: &data.initial_data,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        Self {
            uniform_data: Some(starting_buffer),
            uniform_texture: None,
        }
    }

    pub fn with_uniform_texture(texture: &UniformTexture) -> Self {
        let starting_view = texture.make_view();

        Self {
            uniform_data: None,
            uniform_texture: Some(starting_view),
        }
    }

    pub fn with_all(engine: &Engine, data: &UniformData, texture: &UniformTexture) -> Self {
        let device = &engine.get_wgpu().device;

        let starting_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("User uniform buffer"),
                contents: &data.initial_data,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let starting_view = texture.make_view();

        Self {
            uniform_data: Some(starting_buffer),
            uniform_texture: Some(starting_view),
        }
    }

    pub(crate) fn make_layout(&self, device: &wgpu::Device) -> Option<wgpu::BindGroupLayout> {
        match (&self.uniform_data, &self.uniform_texture) {
            (Some(_), Some(_)) => Some(layouts::create_texture_uniform_layout(device)),
            (Some(_), None) => Some(layouts::create_uniform_layout(device)),
            (None, Some(_)) => Some(layouts::create_texture_layout(device)),
            (None, None) => None,
        }
    }

    pub(crate) fn check_has(&self) -> (bool, bool) {
        (self.uniform_data.is_some(), self.uniform_texture.is_some())
    }
}