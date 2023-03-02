use image::GenericImageView;
use crc32fast::Hasher;

use crate::cache::{TextureCache, TextureIndex};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) id: u32, //checksum used for hashing
}

impl Texture {
    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, label: Option<&str>, bytes: &[u8]) -> Result<Self, image::ImageError> {
        let mut hasher = Hasher::new();
        hasher.update(bytes);
        let checksum = hasher.finalize();
        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_image(device, queue, img, label, checksum))
    }

    pub fn from_path(device: &wgpu::Device, queue: &wgpu::Queue, label: Option<&str>, path: &str) -> Result<Self, image::ImageError> {
        let bytes = std::fs::read(path).unwrap();
        let out = Self::from_bytes(device, queue, label, &bytes)?;
        Ok(out)
    }

    fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, img: image::DynamicImage, label: Option<&str>, id: u32) -> Self {
        let diffuse_rgba = img.to_rgba8();
        let (width, height) = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
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

        queue.write_texture(
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
            texture_size
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            // what to do when given cordinates outside the textures height/width
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // what do when give less or more than 1 pixel to sample
            // linear interprelates between all of them nearest gives the closet colour
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = Self::make_bind_group_layout(device);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        Self {
            texture, 
            view, 
            sampler, 
            bind_group, 
            bind_group_layout,
            id,
        }
    }

    pub fn make_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {filterable: true},
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
            label: Some("texture_bind_group_layout"),
        })
    }
}

pub(crate) fn create_texture(texture_cache: &mut TextureCache, device: &wgpu::Device, queue: &wgpu::Queue, path: &str) -> TextureIndex {
    let texture = Texture::from_path(device, queue, None, path).unwrap();
    texture_cache.add_texture(texture)
}

pub(crate) fn create_texture_from_bytes(texture_cache: &mut TextureCache, device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) -> TextureIndex {
    let texture = Texture::from_bytes(device, queue, None, bytes).unwrap();
    texture_cache.add_texture(texture)
}