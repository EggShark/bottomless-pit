//! Cointains the interface into the texture cache and by
//! extension accsss the texture interface

use crate::engine_handle::Engine;
use crate::resource::{self, InProgressResource, Resource, ResourceId, ResourceType};
use crate::vectors::Vec2;
use crate::{layouts, ERROR_TEXTURE_DATA};
use image::{GenericImageView, ImageError};
use std::fmt::Display;
use std::io::Error;
use std::path::Path;

/// Contains all the information need to render an image/texture to the screen.
/// In order to be used it must be put inside a [Material](crate::material::Material)
pub struct Texture {
    pub(crate) _view: wgpu::TextureView,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) size: Vec2<f32>,
}

impl Texture {
    /// Attempts to both read a file at the specified path and turn it into an image. This will halt the engine
    /// untill loading is finished please see the [resource module](crate::resource) module for more information
    /// on how resource loading works.
    pub fn new<P>(engine: &mut Engine, path: P) -> ResourceId<Texture>
    where
        P: AsRef<Path>,
    {
        let typed_id = resource::generate_id::<Texture>();
        let id = typed_id.get_id();
        let path = path.as_ref();
        let ip_resource = InProgressResource::new(path, id, ResourceType::Image(SamplerType::LinearInterpolation, SamplerType::NearestNeighbor));

        resource::start_load(engine, path, &ip_resource);

        engine.add_in_progress_resource(ip_resource);
        typed_id
    }

    /// Attempts to both read a file at the specified path and turn it into an image. This will halt the engine
    /// untill loading is finished please see the [resource module](crate::resource) module for more information
    /// on how resource loading works. This also lets you choose how you would like the texture sampled when 
    /// it samples more than one or less than one pixel.
    pub fn new_with_sampler<P>(engine: &mut Engine, path: P, sampler: SamplerType) -> ResourceId<Texture>
    where
        P: AsRef<Path>,
    {
        let typed_id = resource::generate_id::<Texture>();
        let id = typed_id.get_id();
        let path = path.as_ref();
        let ip_resource = InProgressResource::new(path, id, ResourceType::Image(sampler, sampler));

        resource::start_load(engine, path, &ip_resource);
        engine.add_in_progress_resource(ip_resource);

        typed_id
    }

    /// Attempts to load the file at the path and then turn it into a texture. This also allows you to select what sampling type to use
    /// for both the `mag_sampler`, when the texture is being drawn larger than the orignal resolution and `min_sampler`, when the texture 
    /// is being drawn smaller than the original resolution.
    pub fn new_with_mag_min_sampler<P>(engine: &mut Engine, path: P, mag_sampler: SamplerType, min_sampler: SamplerType) -> ResourceId<Texture> 
    where
        P: AsRef<Path>
    {
        let typed_id = resource::generate_id::<Texture>();
        let id = typed_id.get_id();
        let path = path.as_ref();
        let ip_resource = InProgressResource::new(path, id, ResourceType::Image(mag_sampler, min_sampler));

        resource::start_load(engine, path, &ip_resource);
        engine.add_in_progress_resource(ip_resource);

        typed_id
    }

    /// Attempts to load an image from a byte array. This is done staticly as it does not halt the engine
    /// for more information on resource loading see [resource module](crate::resource).
    pub fn from_btyes(
        engine: &mut Engine,
        label: Option<&str>,
        bytes: &[u8],
    ) -> ResourceId<Texture> {
        let img = image::load_from_memory(bytes)
            .map(|img| Self::from_image(engine, img, label, SamplerType::LinearInterpolation, SamplerType::NearestNeighbor))
            .unwrap_or_else(|e| {
                log::warn!("{}, occured loading default", e);
                Self::default(engine)
            });

        let typed_id = resource::generate_id::<Texture>();
        engine.resource_manager.insert_texture(typed_id, img);

        typed_id
    }

    pub(crate) fn from_resource_data(
        engine: &Engine,
        label: Option<&str>,
        resource: Resource,
        mag_sampler: SamplerType,
        min_sampler: SamplerType
    ) -> Result<Self, TextureError> {
        let img = image::load_from_memory(&resource.data)?;
        Ok(Self::from_image(engine, img, label, mag_sampler, min_sampler))
    }

    pub(crate) fn new_direct(
        view: wgpu::TextureView,
        bind_group: wgpu::BindGroup,
        size: Vec2<f32>,
    ) -> Self {
        Self {
            _view: view,
            bind_group,
            size,
        }
    }

    pub(crate) fn default(engine: &Engine) -> Self {
        let image = image::load_from_memory(ERROR_TEXTURE_DATA).unwrap();
        Self::from_image(engine, image, Some("Error Texture"), SamplerType::LinearInterpolation, SamplerType::NearestNeighbor)
    }

    fn from_image(engine: &Engine, img: image::DynamicImage, label: Option<&str>, mag_filter: SamplerType, min_filter: SamplerType) -> Self {
        let wgpu = engine.get_wgpu();
        let diffuse_rgba = img.to_rgba8();
        let (width, height) = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = wgpu.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            view_formats: &[],
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label,
        });

        wgpu.queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group_layout = layouts::create_texture_layout(&wgpu.device);

        let texture_sampler = wgpu.device.create_sampler(&wgpu::SamplerDescriptor {
            // what to do when given cordinates outside the textures height/width
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // what do when give less or more than 1 pixel to sample
            // linear interprelates between all of them nearest gives the closet colour
            mag_filter: mag_filter.into(),
            min_filter: min_filter.into(),
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let size = Vec2 {
            x: width as f32,
            y: height as f32,
        };

        Self {
            _view: view,
            bind_group,
            size,
        }
    }
}

/// Loading a texture can fail in two senarios. Either the file cant be opened, or the
/// file loaded is not a supported image file type.
#[derive(Debug)]
pub(crate) enum TextureError {
    IoError(Error),
    ImageError(ImageError),
}

impl From<Error> for TextureError {
    fn from(value: Error) -> TextureError {
        Self::IoError(value)
    }
}

impl From<ImageError> for TextureError {
    fn from(value: ImageError) -> Self {
        Self::ImageError(value)
    }
}

impl std::error::Error for TextureError {}

impl Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "{}", e),
            Self::ImageError(e) => write!(f, "{}", e),
        }
    }
}

/// The diffrent types of sampling modes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamplerType {
    /// Nearest Neighbor sampling
    /// 
    /// This creates a pixelated look when used in upscaling best
    /// for pixel art games
    NearestNeighbor,
    /// Linear Interpolation sampling
    /// 
    /// Creates a smoother blury look when used in upscaling
    LinearInterpolation,
}

impl From<SamplerType> for wgpu::FilterMode {
    fn from(value: SamplerType) -> Self {
        match value {
            SamplerType::LinearInterpolation => wgpu::FilterMode::Linear,
            SamplerType::NearestNeighbor => wgpu::FilterMode::Nearest,
        }
    }
}