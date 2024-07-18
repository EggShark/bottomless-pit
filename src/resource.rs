//! Contains the code for the loading of all assets and [ResourceId]
//!
//! In order for bottomless-pit to run on both desktop and web enviorments file reading
//! has to be done asynchronously. When you call [Texture::new](crate::texture::Texture::new),
//! [Shader::new](crate::shader::Shader::new), [Font::new](crate::text::Font::new),
//! or [Engine::create_resource](crate::engine_handle::Engine::create_resource) you are given a
//! handle to the resource and not the resoruce directly. Since resources are loaded asynchronously it will not
//! be imeaditly availble. In order to have the resource be available to use as soon as possible the engine halts
//! while waiting for the resource. [Game::update](crate::Game::update) will not be called and
//! [Game::render](crate::Game::render) will not be called either.
//! ```rust
//! fn main() {
//!     let engine = mut EngineBuilder::new().build();
//!     
//!     let texture: ResourceId<Texture> = Texture::new(&mut engine, "path.png");
//!     // anything loaded in main will be ready on the first frame of the game
//!     let material = MaterialBuilder::new().add_texture(texture).build();
//!
//!     let game = YourGame {
//!         textured_material: material,
//!     }
//! }
//!
//! struct YourGame {
//!     textured_material: Material,
//! }
//!
//! impl Game for YourGame {
//!     fn update(&mut self, engine: &mut Engine) {
//!         if engine.is_key_pressed(Key::A) {
//!             let texutre = Texture::new(engine, "path2.png");
//!             // render and update wont be called until after the texture finishes loading
//!         }
//!     }
//!
//!     fn render<'pass, 'others>(&'others mut self, renderer: RenderInformation<'pass, 'others>) where 'others: 'pass {
//!         // do stuff
//!     }
//! }
//! ```
//! Because of this stalling behavior it is recomended you do all your loading of assests in as large of chunks as possible.
use std::collections::HashMap;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use winit::event_loop::EventLoopProxy;

use crate::engine_handle::{BpEvent, Engine};
use crate::shader::{IntermediateOptions, Shader};
use crate::text::Font;
use crate::texture::{SamplerType, Texture};

use futures::executor::ThreadPool;

#[cfg(target_arch = "wasm32")]
async fn web_read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, ReadError> {
    use env_logger::builder;
    use js_sys::Uint8Array;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use crate::engine_handle::BuildError;

    let path = path.as_ref().as_os_str().to_str().unwrap();

    match web_sys::window() {
        Some(window) => {
            let response_value = JsFuture::from(window.fetch_with_str(path)).await.unwrap();

            let response: web_sys::Response = response_value.dyn_into().unwrap();

            if !response.ok() {
                Err(ReadError::ResponseError(
                    response.status(),
                    response.status_text(),
                ))?;
            }

            let data = JsFuture::from(response.array_buffer().unwrap())
                .await
                .unwrap();
            let bytes = Uint8Array::new(&data).to_vec();
            Ok(bytes)
        }
        None => Err(ReadError::WindowError),
    }
}

pub(crate) async fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, ReadError> {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Uint8Array;
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;

        let path = path.as_ref().as_os_str().to_str().unwrap();

        match web_sys::window() {
            Some(window) => {
                let response_value = JsFuture::from(window.fetch_with_str(path)).await.unwrap();

                let response: web_sys::Response = response_value.dyn_into().unwrap();

                if !response.ok() {
                    Err(ReadError::ResponseError(
                        response.status(),
                        response.status_text(),
                    ))?;
                }

                let data = JsFuture::from(response.array_buffer().unwrap())
                    .await
                    .unwrap();
                let bytes = Uint8Array::new(&data).to_vec();
                Ok(bytes)
            }
            None => Err(ReadError::WindowError),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(std::fs::read(path)?)
    }
}

#[derive(Debug)]
pub(crate) enum ReadError {
    IoError(std::io::Error),
    #[cfg(target_arch = "wasm32")]
    ResponseError(u16, String),
    #[cfg(target_arch = "wasm32")]
    WindowError,
}

impl From<std::io::Error> for ReadError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

pub(crate) struct Loader {
    items_loading: usize,
    blocked_loading: usize,
    background_loading: usize,
    // for items that were laoded before engine.run()
    preload_queue: Vec<InProgressResource>,
    #[cfg(not(target_arch="wasm32"))]
    pool: ThreadPool,
    #[cfg(target_arch = "wasm32")]
    blocked: bool,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            items_loading: 0,
            background_loading: 0,
            blocked_loading: 0,
            preload_queue: Vec::new(),
            #[cfg(not(target_arch="wasm32"))]
            pool: ThreadPool::new().unwrap(),
            #[cfg(target_arch = "wasm32")]
            blocked: false,
        }
    }

    pub fn remove_item_loading(&mut self) {
        self.items_loading -= 1;
    }

    pub fn get_loading_resources(&self) -> usize {
        self.items_loading
    }

    #[cfg(target_arch="wasm32")]
    pub fn is_blocked(&self) -> bool {
        self.blocked
    }

    // just fs read that stuff man
    // becuase this is all happening on the main thread stuff will be read in before
    // render() is called
    #[cfg(not(target_arch = "wasm32"))]
    pub fn blocking_load(&mut self, ip_resource: InProgressResource, proxy: EventLoopProxy<BpEvent>) {
        let data: Result<Vec<u8>, ReadError> = match std::fs::read(&ip_resource.path) {
            Ok(d) => Ok(d),
            Err(e) => Err(e.into())
        };

        let resource = Resource::from_result(data, ip_resource.path, ip_resource.id, ip_resource.resource_type);
        self.items_loading += 1;
        proxy.send_event(BpEvent::ResourceLoaded(resource)).unwrap();
    }

    // request but flip flag :3
    #[cfg(target_arch = "wasm32")]
    pub fn blocking_load(&mut self, ip_resource: InProgressResource, proxy: EventLoopProxy<BpEvent>) {
        use wasm_bindgen_futures::spawn_local;
        self.items_loading += 1;
        self.blocked = true;
        spawn_local(async move {
            let result = web_read(&ip_resource.path).await;
            let resource = Resource::from_result(result, ip_resource.path, ip_resource.id, ip_resource.resource_type);
            proxy
                .send_event(BpEvent::ResourceLoaded(resource))
                .unwrap();
        });
    }

    // threadpool / aysnc
    pub fn background_load(&mut self, ip_resource: InProgressResource, proxy: EventLoopProxy<BpEvent>) {
        self.items_loading += 1;
        #[cfg(not(target_arch="wasm32"))]
        {
            self.pool.spawn_ok(async move {
                let result = read(&ip_resource.path).await;
                let resource = Resource::from_result(result, ip_resource.path, ip_resource.id, ip_resource.resource_type);
                proxy
                    .send_event(BpEvent::ResourceLoaded(resource))
                    .unwrap();
            });
        }

        #[cfg(target_arch="wasm32")]
        {
            use wasm_bindgen_futures::spawn_local;
            spawn_local(async move {
                let result = web_read(&ip_resource.path).await;
                let resource = Resource::from_result(result, ip_resource.path, ip_resource.id, ip_resource.resource_type);
                proxy
                    .send_event(BpEvent::ResourceLoaded(resource))
                    .unwrap();
            });
        }
    }

    pub fn preload(&mut self, ip_resource: InProgressResource) {
        self.preload_queue.push(ip_resource);
    }

    pub fn execute_preload_queue(&mut self, proxy: EventLoopProxy<BpEvent>) {
        let mut e_vec = Vec::new();
        std::mem::swap(&mut self.preload_queue, &mut e_vec);

        for item in e_vec {
            self.blocking_load(item, proxy.clone());
        }
    }
}

#[derive(Debug)]
pub(crate) struct Resource {
    pub(crate) path: PathBuf,
    pub(crate) data: Vec<u8>,
    pub(crate) id: NonZeroU64,
    pub(crate) resource_type: ResourceType,
}

#[derive(Debug)]
pub(crate) struct ResourceError {
    pub(crate) error: ReadError,
    _path: PathBuf,
    pub(crate) id: NonZeroU64,
    pub(crate) resource_type: ResourceType,
}

impl Resource {
    pub fn from_result(
        result: Result<Vec<u8>, ReadError>,
        path: PathBuf,
        id: NonZeroU64,
        resource_type: ResourceType,
    ) -> Result<Self, ResourceError> {
        match result {
            Ok(data) => Ok(Self {
                path,
                data,
                id,
                resource_type,
            }),
            Err(e) => Err(ResourceError {
                error: e,
                _path: path,
                id,
                resource_type,
            }),
        }
    }
}

#[derive(Debug)]
pub(crate) struct InProgressResource {
    path: PathBuf,
    id: NonZeroU64,
    resource_type: ResourceType,
}

impl InProgressResource {
    pub fn new(path: &Path, id: NonZeroU64, resource_type: ResourceType) -> Self {
        Self {
            path: path.to_owned(),
            id,
            resource_type,
        }
    }
}

#[derive(Debug)]
pub(crate) enum ResourceType {
    Image(SamplerType, SamplerType),
    Shader(IntermediateOptions),
    Bytes,
    Font,
}

impl PartialEq for ResourceType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bytes, Self::Bytes) => true,
            (Self::Font, Self::Font) => true,
            (Self::Shader(option_1), Self::Shader(option_2)) => {
                option_1.check_has() == option_2.check_has()
            } 
            (Self::Image(s1, s2), Self::Image(s3, s4)) => {
                s1 == s3 && s2 == s4
            }
            _ => false,
        }
    }
}

pub(crate) fn generate_id<T>() -> ResourceId<T> {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    ResourceId(NonZeroU64::new(id).unwrap(), PhantomData::<T>)
}

/// An Id for a specific type of resource used interally in the engine
#[derive(PartialOrd, Ord)]
pub struct ResourceId<T>(NonZeroU64, std::marker::PhantomData<T>);

impl<T> ResourceId<T> {
    pub(crate) fn from_number(number: NonZeroU64) -> Self {
        Self(number, PhantomData)
    }

    pub(crate) fn get_id(&self) -> NonZeroU64 {
        self.0
    }
}

impl<T> Clone for ResourceId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ResourceId<T> {}

impl<T> std::fmt::Debug for ResourceId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ID").field(&self.0).finish()
    }
}

impl<T> PartialEq for ResourceId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for ResourceId<T> {}

impl<T> std::hash::Hash for ResourceId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

type ResourceMap<T> = HashMap<ResourceId<T>, T>;

pub(crate) struct ResourceManager {
    btye_resources: ResourceMap<Vec<u8>>,
    bindgroup_resources: ResourceMap<Texture>,
    pipeline_resource: ResourceMap<Shader>,
    fonts: ResourceMap<Font>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            btye_resources: HashMap::new(),
            bindgroup_resources: HashMap::new(),
            pipeline_resource: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn insert_bytes(&mut self, key: ResourceId<Vec<u8>>, data: Vec<u8>) {
        self.btye_resources.insert(key, data);
    }

    pub fn insert_texture(&mut self, key: ResourceId<Texture>, data: Texture) {
        self.bindgroup_resources.insert(key, data);
    }

    pub fn insert_pipeline(&mut self, key: ResourceId<Shader>, data: Shader) {
        self.pipeline_resource.insert(key, data);
    }

    pub fn insert_font(&mut self, key: ResourceId<Font>, data: Font) {
        self.fonts.insert(key, data);
    }

    pub fn get_byte_resource(&self, key: &ResourceId<Vec<u8>>) -> Option<&Vec<u8>> {
        self.btye_resources.get(key)
    }

    pub fn get_texture(&self, key: &ResourceId<Texture>) -> Option<&Texture> {
        self.bindgroup_resources.get(key)
    }

    pub fn get_pipeline(&self, key: &ResourceId<Shader>) -> Option<&Shader> {
        self.pipeline_resource.get(key)
    }

    pub fn get_font(&self, key: &ResourceId<Font>) -> Option<&Font> {
        self.fonts.get(key)
    }

    pub fn get_mut_shader(&mut self, key: &ResourceId<Shader>) -> Option<&mut Shader> {
        self.pipeline_resource.get_mut(key)
    }
}