//! Cointains the interface into the texture cache and by
//! extension accsss the texture interface

use crate::engine_handle::Engine;
use crate::layouts;
use crate::vectors::Vec2;
use cfg_if::cfg_if;
use image::{GenericImageView, ImageError};
use std::fmt::Display;
use std::io::Error;
use std::path::Path;
use std::pin::Pin;
use std::task::Poll;

/// Contains all the information need to render an image/texture to the screen.
/// In order to be used it must be put inside a [Material](../material/struct.Material.html)
pub struct Texture {
    pub(crate) _view: wgpu::TextureView,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) size: Vec2<f32>,
}

impl Texture {
    /// Attempts to load an image from a byte array
    pub fn from_bytes(
        engine: &Engine,
        label: Option<&str>,
        bytes: &[u8],
    ) -> Result<Self, TextureError> {
        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_image(engine, img, label))
    }

    /// Attempts to both read a file at the specified path and turn it into an iamge
    pub fn from_path<P>(
        engine: &Engine,
        label: Option<&str>,
        path: P,
    ) -> Result<(), TextureError> where P: AsRef<Path> {
        let bytes = read(path, engine)?;
        // let out = Self::from_bytes(engine, label, &bytes)?;
        // Ok(out)
        Ok(())
    }

    fn from_image(
        engine: &Engine,
        img: image::DynamicImage,
        label: Option<&str>,
    ) -> Self {
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

        let bind_group = wgpu
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
                        resource: wgpu::BindingResource::Sampler(engine.get_texture_sampler()),
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

    /// Adds a texture to the internal cache this step must be done
    /// to add a texture to a [Material](../material/struct.Material.html)
    /// ```rust
    /// let texture = Texture::from_path("assets/image.png")?
    ///     .register(&mut engine);
    /// 
    /// let material = MaterialBuilder::new()
    ///     .add_texture(texutre)
    ///     .build();
    /// ```
    pub fn register(self, engine: &mut Engine) -> RegisteredTexture {
        let id = self.bind_group.global_id();
        let size = self.size;

        engine.add_to_bind_group_cache(self.bind_group, id);

        RegisteredTexture {
            bindgroup_id: id,
            texture_size: size,
        }
    }
}


#[cfg(target_arch="wasm32")]
async fn make_request<U: AsRef<Path>>(path: U, engine: &Engine) -> Result<Vec<u8>, TextureError> {
    use wasm_bindgen_futures::JsFuture;

    let path = path.as_ref()
        .as_os_str()
        .to_str()
        .unwrap();
    let req = web_sys::RequestInit::new();

    log::warn!("fetch!");
    let x = engine.web_window.fetch_with_str_and_init(&path, &req);

    let awaited = JsFuture::from(x).await;

    log::warn!("{:?}", awaited);


    todo!("finish the functions 4head")
}


cfg_if!{
    if #[cfg(target_arch="wasm32")] {
        fn read<U: AsRef<Path>>(path: U, engine: &Engine) -> Result<Vec<u8>, TextureError> {
            let x = make_request(path, engine);

            Ok(Vec::new())
        }
    } else {
        fn read<P: AsRef<Path>>(path: P, _engine: &Engine) -> Result<Vec<u8>, TextureError> {
            Ok(std::fs::read(path)?)
        }
    }
}

/// A struct that contains an Id and the size of a texture stored interally. This
/// can only be obtained after registering a texture and its only purpose is to 
/// be added to a [Material](../material/struct.Material.html).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegisteredTexture {
    pub(crate) bindgroup_id: wgpu::Id<wgpu::BindGroup>,
    pub(crate) texture_size: Vec2<f32>,
}

/// Loading a texture can fail in two senarios. Either the file cant be opened, or the
/// file loaded is not a supported image file type.
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