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

use crate::engine_handle::{BpEvent, Engine};
use crate::io::{self, ReadError};
use crate::shader::{Shader, ShaderOptions};
use crate::text::Font;
use crate::texture::{SamplerType, Texture};

#[derive(Debug)]
pub(crate) struct Resource {
    path: PathBuf,
    pub(crate) data: Vec<u8>,
    pub(crate) id: NonZeroU64,
    pub(crate) resource_type: ResourceType,
}

#[derive(Debug)]
pub(crate) struct ResourceError {
    pub(crate) error: ReadError,
    path: PathBuf,
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
                path,
                id,
                resource_type,
            }),
        }
    }
}

pub(crate) fn compare_resources(
    left: &InProgressResource,
    right: &Result<Resource, ResourceError>,
) -> bool {
    match right {
        Ok(right) => {
            left.id == right.id
                && left.resource_type == right.resource_type
                && left.path == right.path
        }
        Err(right) => {
            left.id == right.id
                && left.resource_type == right.resource_type
                && left.path == right.path
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
    Shader(ShaderOptions),
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

pub(crate) fn start_load<P: AsRef<Path>>(
    engine: &Engine,
    path: P,
    ip_resource: InProgressResource,
) {
    let path = path.as_ref().to_owned();
    let id = ip_resource.id;
    let resource_type = ip_resource.resource_type;
    let event_loop_proxy = engine.get_proxy();
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen_futures::spawn_local;
        spawn_local(async move {
            let result = io::read(&path).await;
            let resource = Resource::from_result(result, path, id, resource_type);
            event_loop_proxy
                .send_event(BpEvent::ResourceLoaded(resource))
                .unwrap();
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let pool = engine.thread_pool();
        pool.spawn_ok(async move {
            let result = io::read(&path).await;
            let resource = Resource::from_result(result, path, id, resource_type);
            event_loop_proxy
                .send_event(BpEvent::ResourceLoaded(resource))
                .unwrap();
        });
    }
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
