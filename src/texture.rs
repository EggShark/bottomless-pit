//! Cointains the interface into the texture cache and by
//! extension accsss the texture interface

use crate::engine_handle::WgpuClump;
use crate::layouts;
use crate::resource_cache::ResourceCache;
use crate::vectors::Vec2;
use crc32fast::Hasher;
use image::{GenericImageView, ImageError};
use std::fmt::Display;
use std::io::Error;

pub(crate) struct Texture {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) id: u32, //checksum used for hashing
    pub(crate) size: Vec2<f32>,
}

impl Texture {
    pub fn from_bytes(
        wgpu_things: &WgpuClump,
        label: Option<&str>,
        bytes: &[u8],
    ) -> Result<Self, TextureError> {
        let mut hasher = Hasher::new();
        hasher.update(bytes);
        let checksum = hasher.finalize();
        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_image(wgpu_things, img, label, checksum))
    }

    pub fn from_path(
        wgpu_things: &WgpuClump,
        label: Option<&str>,
        path: &str,
    ) -> Result<Self, TextureError> {
        let bytes = std::fs::read(path)?;
        let out = Self::from_bytes(wgpu_things, label, &bytes)?;
        Ok(out)
    }

    fn from_image(
        wgpu_things: &WgpuClump,
        img: image::DynamicImage,
        label: Option<&str>,
        id: u32,
    ) -> Self {
        let diffuse_rgba = img.to_rgba8();
        let (width, height) = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = wgpu_things.device.create_texture(&wgpu::TextureDescriptor {
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

        wgpu_things.queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = wgpu_things.device.create_sampler(&wgpu::SamplerDescriptor {
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

        let bind_group_layout = layouts::create_texture_layout(&wgpu_things.device);

        let bind_group = wgpu_things
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            });

        let size = Vec2 {
            x: width as f32,
            y: height as f32,
        };

        Self {
            view,
            sampler,
            bind_group,
            id,
            size,
        }
    }
}

#[derive(Debug)]
pub enum TextureError {
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

pub(crate) fn create_texture(
    texture_cache: &mut ResourceCache<wgpu::BindGroup>,
    wgpu_things: &WgpuClump,
    path: &str,
) -> Result<TextureIndex, TextureError> {
    let texture = Texture::from_path(wgpu_things, None, path)?;
    texture_cache.add_item(texture.bind_group, texture.id);
    Ok(TextureIndex {
        view: texture.view,
        sampler: texture.sampler,
        id: texture.id,
        size: texture.size,
    })
}

/// A texture, but more specifically a index into a cahce that stores all the textures
pub struct TextureIndex {
    // the info needed to recrate the texture when necciscarry
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    pub(crate) id: u32, //crc32 checksum
    pub size: Vec2<f32>,
}

impl TextureIndex {
    pub(crate) fn get_view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub(crate) fn get_sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}