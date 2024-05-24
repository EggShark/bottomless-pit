//! Contains everything you need to make some cool shaders including the Shader
//! and UniformData types.

use std::path::Path;
use std::string::FromUtf8Error;

use encase::private::WriteInto;
use encase::ShaderType;
use wgpu::include_wgsl;
use wgpu::util::DeviceExt;

use crate::engine_handle::{Engine, WgpuClump};
use crate::render::Renderer;
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

        resource::start_load(engine, ip_resource);
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

    pub(crate) fn update_uniform_data<T: ShaderType + WriteInto>(&self, data: &T, engine: &Engine) {
        self.options.update_uniform_data(data, engine);
    }

    pub(crate) fn update_uniform_texture(&mut self, texture: &UniformTexture, wgpu: &WgpuClump) {
        self.options.update_uniform_texture(texture, wgpu);
    }

    pub(crate) fn set_active<'o, 'p>(&'o self, renderer: &mut Renderer<'o, 'p>) {
        renderer.pass.set_pipeline(&self.pipeline);
        
        if let Some(bind_group) = &self.options.bind_group {
            renderer.pass.set_bind_group(2, bind_group, &[]);
        }
    }
}

/// `UniformData` contains the byte data of any struct that implements
/// [ShaderType](https://docs.rs/encase/latest/encase/trait.ShaderType.html) which
/// can be derived. This data needs to be added to a Material upon creation.
#[derive(Debug)]
pub struct UniformData {
    initial_data: Vec<u8>,
}

impl UniformData {
    pub fn new<T: ShaderType + WriteInto>(data: &T) -> Self {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(&data).unwrap();
        let byte_array = buffer.into_inner();


        Self {
            initial_data: byte_array,
        }
    }
}

#[derive(Debug)]
pub struct ShaderOptions {
    uniform_data: Option<wgpu::Buffer>,
    uniform_texture: Option<(wgpu::TextureView, wgpu::Sampler)>,
    bind_group: Option<wgpu::BindGroup>,
}

impl ShaderOptions {
    //TODO add support for custom texture samplers
    pub const EMPTY: Self = Self {
        uniform_data: None,
        uniform_texture: None,
        bind_group: None,
    };

    pub fn with_uniform_data(engine: &Engine, data: &UniformData) -> Self {
        let device = &engine.get_wgpu().device;

        let starting_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("User uniform buffer"),
                contents: &data.initial_data,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = Self::make_layout_internal(device, Some(&starting_buffer), None)
            .and_then(|layout| Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Shader Options BindGroup"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(starting_buffer.as_entire_buffer_binding()),
                    }
                ],
            })));

        Self {
            uniform_data: Some(starting_buffer),
            uniform_texture: None,
            bind_group,
        }
    }

    pub fn with_uniform_texture(texture: &UniformTexture, engine: &Engine) -> Self {
        let device = &engine.get_wgpu().device;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
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

        let starting_view = texture.make_view();

        let bind_group = Self::make_layout_internal(device, None, Some((&starting_view, &sampler)))
            .and_then(|layout| Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Shader Options BindGroup"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&starting_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
            })));

        Self {
            uniform_data: None,
            uniform_texture: Some((starting_view, sampler)),
            bind_group,
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
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

        let bind_group = Self::make_layout_internal(device, Some(&starting_buffer), Some((&starting_view, &sampler)))
            .and_then(|layout| Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Shader Options BindGroup"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(starting_buffer.as_entire_buffer_binding()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&starting_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
            })));

        Self {
            uniform_data: Some(starting_buffer),
            uniform_texture: Some((starting_view, sampler)),
            bind_group,
        }
    }

    fn update_uniform_data<T: ShaderType + WriteInto>(&self, data: &T, engine: &Engine) {
        if let Some(buffer) = &self.uniform_data {
            let wgpu = engine.get_wgpu();
            let mut uniform_buffer = encase::UniformBuffer::new(Vec::new());
            uniform_buffer.write(&data).unwrap();
            let byte_array = uniform_buffer.into_inner();

            wgpu.queue.write_buffer(buffer, 0, &byte_array);
        }
    }

    fn update_uniform_texture(&mut self, texture: &UniformTexture, wgpu: &WgpuClump) {
        if let Some(view) = &mut self.uniform_texture {
            view.0 = texture.make_view();

            self.bind_group = Some(wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Shader Options BindGroup"),
                entries: &self.make_entries(),
                layout: &self.make_layout(&wgpu.device).unwrap(),
            }));
        }
    }

    fn make_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        let mut entries = Vec::with_capacity(3);

        if let Some(buffer) = &self.uniform_data {
            entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            });
        }

        if let Some((view, sampler)) = &self.uniform_texture {
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            });
        }

        entries
    }

    pub(crate) fn make_layout(&self, device: &wgpu::Device) -> Option<wgpu::BindGroupLayout> {
        Self::make_layout_internal(device, self.uniform_data.as_ref(), self.uniform_texture.as_ref().map(|(a, b)| (a, b)))
    }

    fn make_layout_internal(device: &wgpu::Device, uniform_data: Option<&wgpu::Buffer>, uniform_texture: Option<(&wgpu::TextureView, &wgpu::Sampler)>) -> Option<wgpu::BindGroupLayout> {
        match (uniform_data, uniform_texture) {
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