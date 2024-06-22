//! Contains everything you need to make some cool shaders including the Shader
//! and UniformData types.

use std::error::Error;
use std::fmt::Display;
use std::marker::PhantomData;
use std::path::Path;
use std::string::FromUtf8Error;

use encase::private::WriteInto;
use encase::ShaderType;
use wgpu::include_wgsl;
use wgpu::util::DeviceExt;

use crate::context::{GraphicsContext, WgpuClump};
use crate::engine_handle::Engine;
use crate::render::Renderer;
use crate::resource::{self, InProgressResource, ResourceId, ResourceType};
use crate::texture::{SamplerType, UniformTexture};
use crate::vertex::Vertex;
use crate::vectors::Vec2;
use crate::{layouts, render};

/// An internal representation of an WGSL Shader. Under the hood this creates
/// a new pipeline with or without the support for any extra uniforms. To be utilze
/// the shader it must be added to a material
#[derive(Debug)]
pub struct Shader {
    pub(crate) pipeline: wgpu::RenderPipeline,
    options: UntypedShaderOptions,
}

impl Shader {
    /// Attempts to create a shader from a file. This will halt the engine due to resource loading please see
    /// the [resource module](crate::resource) for more information.
    pub fn new<P: AsRef<Path>, T>(
        path: P,
        options: ShaderOptions<T>,
        engine: &mut Engine,
    ) -> ResourceId<Shader> {
        let typed_id = resource::generate_id::<Shader>();
        let id = typed_id.get_id();
        let path = path.as_ref();
        let ip_resource = InProgressResource::new(path, id, ResourceType::Shader(UntypedShaderOptions::from_typed(options, engine.get_context().expect("need context here"))));

        resource::start_load(engine, ip_resource);
        engine.add_in_progress_resource();

        typed_id
    }

    pub(crate) fn from_resource_data(
        data: &[u8],
        options: UntypedShaderOptions,
        engine: &Engine,
    ) -> Result<Self, FromUtf8Error> {
        let context = engine.get_context().unwrap();

        let string = String::from_utf8(data.to_vec())?;
        let shader = context
            .wgpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("User Shader Module"),
                source: wgpu::ShaderSource::Wgsl(string.into()),
            });

        let optional_layout = options.make_layout(&context.wgpu.device);
        let pipeline = if let Some(layout) = optional_layout {
            render::make_pipeline(
                &context.wgpu.device,
                wgpu::PrimitiveTopology::TriangleList,
                &[
                    &layouts::create_texture_layout(&context.wgpu.device),
                    &layouts::create_camera_layout(&context.wgpu.device),
                    &layout
                ],
                &[Vertex::desc()],
                &shader,
                context.get_texture_format(),
                Some("User Shader Pipeline"),
            )
        } else {
            render::make_pipeline(
                &context.wgpu.device,
                wgpu::PrimitiveTopology::TriangleList,
                &[
                    &layouts::create_texture_layout(&context.wgpu.device),
                    &layouts::create_camera_layout(&context.wgpu.device),
                ],
                &[Vertex::desc()],
                &shader,
                context.get_texture_format(),
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
            options: UntypedShaderOptions::EMPTY,
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
            options: UntypedShaderOptions::EMPTY,
        }
    }

    pub(crate) fn update_uniform_data<T: ShaderType + WriteInto>(&self, data: &T, engine: &Engine) -> Result<(), UniformError> {
        self.options.update_uniform_data(data, engine)
    }

    pub(crate) fn update_uniform_texture(&mut self, texture: &UniformTexture, wgpu: &WgpuClump) -> Result<(), UniformError> {
        self.options.update_uniform_texture(texture, wgpu)
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
/// can be derived. This data needs to be added to [ShaderOptions].
#[derive(Debug)]
pub struct UniformData<T> {
    initial_data: Vec<u8>,
    _marker: PhantomData<T>
}

impl<T: ShaderType + WriteInto> UniformData<T> {
    pub fn new(data: &T) -> Self {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(&data).unwrap();
        let byte_array = buffer.into_inner();


        Self {
            initial_data: byte_array,
            _marker: PhantomData,
        }
    }
}

/// `ShaderOptions` controlls the layout of your shader and wether or not you want
/// extra uniforms like a texture or a uniform buffer
#[derive(Debug)]
pub struct ShaderOptions<T> {
    uniform_data: Option<Vec<u8>>,
    uniform_texture: Option<(SamplerType, SamplerType, Vec2<u32>)>,
    _marker: PhantomData<T>,
}

// switch texture with booleans and only store a layout
// figure out how to make view later?

impl<T> ShaderOptions<T> {
    pub const EMPTY: Self = Self {
        uniform_data: None,
        uniform_texture: None,
        _marker: PhantomData,
    };

    /// This will create a shader with a layout of:
    /// ```wgsl
    /// struct MousePos {
    ///     stuff: vec2<f32>,
    ///     _junk: vec2<f32>,
    /// }
    ///
    /// @group(2) @binding(0)
    /// var<uniform> mouse: MousePos;
    /// ```
    pub fn with_uniform_data(engine: &Engine, data: &UniformData<T>) -> Self {

        // dont like doing this bc clone expensive but we need 
        // to make this before wgpu initlizes
        let starting_buffer = data.initial_data.clone();


        Self {
            uniform_data: Some(starting_buffer),
            uniform_texture: None,
            _marker: PhantomData,
        }
    }

    /// This will create a shader with a layout of:
    /// ```wgsl
    /// @group(2) @binding(0)
    /// var light_map: texture_2d<f32>;
    /// @group(2) @binding(1)
    /// var light_map_sampler: sampler;
    /// ```
    pub fn with_uniform_texture(texture: &UniformTexture, engine: &Engine) -> Self {

        let starting_view = texture.make_view();
        let (mag, min) = texture.get_sampler_info();
        let size = texture.get_size();

        Self {
            uniform_data: None,
            uniform_texture: Some((mag, min, size)),
            _marker: PhantomData,
        }
    }

    /// This will create a shader with a layout of:
    /// ```wgsl
    /// @group(2) binding(0)
    /// var<uniform> value: SomeStruct;
    /// @group(2) @binding(1)
    /// var light_map: texture_2d<f32>;
    /// @group(2) @binding(2)
    /// var light_map_sampler: sampler;
    /// ```
    pub fn with_all(engine: &Engine, data: &UniformData<T>, texture: &UniformTexture) -> Self {
        let starting_buffer = data.initial_data.clone();
        let (mag, min) = texture.get_sampler_info();
        let size = texture.get_size();

        Self {
            uniform_data: Some(starting_buffer),
            uniform_texture: Some((mag, min, size)),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub(crate) struct UntypedShaderOptions {
    uniform_data: Option<wgpu::Buffer>,
    uniform_texture: Option<(wgpu::TextureView, wgpu::Sampler)>,
    bind_group: Option<wgpu::BindGroup>,
}

impl UntypedShaderOptions {
    pub(crate) const EMPTY: Self = Self {
        uniform_data: None,
        uniform_texture: None,
        bind_group: None,
    };

    pub(crate) fn from_typed<T>(options: ShaderOptions<T>, context: &GraphicsContext) -> Self {
        let wgpu = &context.wgpu;
        let format = context.get_texture_format();

        let buffer = options.uniform_data.and_then(|d| {
            Some(wgpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("User Uniform Data Buffer"),
                contents: &d,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }))
        });

        let uniform_texture = options.uniform_texture.and_then(|(mag, min, size)| {
            let sampler = wgpu.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                // what do when give less or more than 1 pixel to sample
                // linear interprelates between all of them nearest gives the closet colour
                mag_filter: mag.into(),
                min_filter: min.into(),
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            let texture = wgpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Uniform Texture"),
                size: wgpu::Extent3d { width: size.x, height: size.y, depth_or_array_layers: 1 },
                dimension: wgpu::TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            Some((view, sampler))
        });

        let bind_group = make_layout_internal(&wgpu.device, buffer.is_some(), uniform_texture.is_some())
            .and_then(|layout| {
                let entires = make_entries(buffer.as_ref(), uniform_texture.as_ref().map(|(a, b)| (a, b)), None);
                
                let bind_group = wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor { 
                    label: Some("User Shader Option Bind Group"),
                    layout: &layout,
                    entries: &entires
                });

                Some(bind_group)
            });
        

        Self {
            uniform_data: buffer,
            uniform_texture,
            bind_group,
        }
    }

    pub(crate) fn make_layout(&self, device: &wgpu::Device) -> Option<wgpu::BindGroupLayout> {
        make_layout_internal(device, self.uniform_data.is_some(), self.uniform_texture.is_some())
    }

    fn update_uniform_data<H: ShaderType + WriteInto>(&self, data: &H, engine: &Engine) -> Result<(), UniformError> {
        match &self.uniform_data {
            Some(buffer) => {
                let wgpu = &engine.get_context().expect("NEED CONTEXT BEFORE UPDATING UNIFORM DATA").wgpu;
                let mut uniform_buffer = encase::UniformBuffer::new(Vec::new());
                uniform_buffer.write(&data).unwrap();
                let byte_array = uniform_buffer.into_inner();
    
                wgpu.queue.write_buffer(buffer, 0, &byte_array);
                Ok(())
            },
            None => Err(UniformError::DoesntHaveUniformBuffer)
        }
    }

    fn update_uniform_texture(&mut self, texture: &UniformTexture, wgpu: &WgpuClump) -> Result<(), UniformError> {
        match &mut self.uniform_texture {
            Some(view) => {
                view.0 = texture.make_view();

                self.bind_group = Some(wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Shader Options BindGroup"),
                    entries: &make_entries(self.uniform_data.as_ref(), self.uniform_texture.as_ref().map(|(a, b)| (a, b)), Some(texture.get_sampler())),
                    layout: &self.make_layout(&wgpu.device).unwrap(),
                }));

                Ok(())
            },
            None => Err(UniformError::DoesntHaveUniformTexture)
        }
    }

    pub(crate) fn check_has(&self) -> (bool, bool) {
        (self.uniform_data.is_some(), self.uniform_texture.is_some())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UniformError {
    NotLoadedYet,
    DoesntHaveUniformBuffer,
    DoesntHaveUniformTexture,
}

impl Display for UniformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotLoadedYet => write!(f, "The shader has not loaded yet please try again later"),
            Self::DoesntHaveUniformBuffer => write!(f, "The shader does not have a uniform buffer"),
            Self::DoesntHaveUniformTexture => write!(f, "The shader does not have a unform texture"),
        }
    }
}

impl Error for UniformError {}

fn make_layout_internal(device: &wgpu::Device, has_buffer: bool, has_texture: bool) -> Option<wgpu::BindGroupLayout> {
    match (has_buffer, has_texture) {
        (true, true) => Some(layouts::create_texture_uniform_layout(device)),
        (true, false) => Some(layouts::create_uniform_layout(device)),
        (false, true) => Some(layouts::create_texture_layout(device)),
        (false, false) => None,
    }
}

fn make_entries<'a>(
    uniform_data: Option<&'a wgpu::Buffer>,
    uniform_texture: Option<(&'a wgpu::TextureView, &'a wgpu::Sampler)>,
    other_sampler: Option<&'a wgpu::Sampler>
) -> Vec<wgpu::BindGroupEntry<'a>> {
    let mut entries = Vec::with_capacity(3);

    if let Some(buffer) = uniform_data {
        entries.push(wgpu::BindGroupEntry {
            binding: entries.len() as u32,
            resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
        });
    }

    if let Some((view, sampler)) = uniform_texture {
        entries.push(wgpu::BindGroupEntry {
            binding: entries.len() as u32,
            resource: wgpu::BindingResource::TextureView(&view),
        });
        entries.push(wgpu::BindGroupEntry {
            binding: entries.len() as u32,
            resource: wgpu::BindingResource::Sampler(other_sampler.unwrap_or(&sampler)),
        });
    }

    entries
}