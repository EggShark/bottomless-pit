use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{PathBuf, Path};
use std::num::NonZeroU64;
use std::sync::atomic::AtomicU64;

use crate::engine_handle::{Engine, BpEvent};
use crate::io::{self, ReadError};
use crate::shader::Shader;
use crate::texture::Texture;

#[derive(Debug)]
pub(crate) struct Resource {
    path: PathBuf,
    pub(crate) data: Vec<u8>,
    pub(crate) id: NonZeroU64,
    pub(crate) resource_type: ResourceType,
}

#[derive(Debug)]
pub(crate) struct ResourceError {
    error: ReadError,
    path: PathBuf,
    id: NonZeroU64,
    resource_type: ResourceType,
}

impl Resource {
    pub fn from_result(result: Result<Vec<u8>, ReadError>, path: PathBuf, id: NonZeroU64, resource_type: ResourceType) -> Result<Self, ResourceError> {
        match result {
            Ok(data) => {
                Ok(Self {
                    path,
                    data,
                    id,
                    resource_type,
                })
            },
            Err(e) => {
                Err(ResourceError {
                    error: e,
                    path,
                    id,
                    resource_type,
                })
            },
        }
    }
}

pub(crate) fn compare_resources(left: &InProgressResource, right: &Result<Resource, ResourceError>)  -> bool {
    match right {
        Ok(right) => {
            left.id == right.id && left.resource_type == right.resource_type && left.path == right.path
        },
        Err(right) => {
            left.id == right.id && left.resource_type == right.resource_type && left.path == right.path
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


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResourceType {
    Image,
    Shader(bool),
    Bytes,
}

pub fn generate_id<T>() -> ResourceId<T> {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    ResourceId(NonZeroU64::new(id).unwrap(), PhantomData::<T>)
}

pub(crate) fn start_load<P: AsRef<Path>>(engine: &Engine, path: P, ip_resource: &InProgressResource) {
    let path = path.as_ref().to_owned();
    let id = ip_resource.id;
    let resource_type = ip_resource.resource_type;
    let event_loop_proxy = engine.get_proxy();
    #[cfg(target_arch="wasm32")]
    {
        use wasm_bindgen_futures::spawn_local;
        spawn_local(async move {
            let result = io::read(&path).await;
            let resource = Resource::from_result(result, path, id, resource_type);
            event_loop_proxy.send_event(BpEvent::ResourceLoaded(resource)).unwrap();
        });
    }

    #[cfg(not(target_arch="wasm32"))]
    {
        let pool = engine.thread_pool();
        pool.spawn_ok(async move {
            let result = io::read(&path).await;
            let resource = Resource::from_result(result, path, id, resource_type);
            event_loop_proxy.send_event(BpEvent::ResourceLoaded(resource)).unwrap();
        });
    }
}

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
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            btye_resources: HashMap::new(),
            bindgroup_resources: HashMap::new(),
            pipeline_resource: HashMap::new(),
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

    pub fn get_byte_resource(&self, key: &ResourceId<Vec<u8>>) -> Option<&Vec<u8>> {
        self.btye_resources.get(key)
    }

    pub fn get_texture(&self, key: &ResourceId<Texture>) -> Option<&Texture> {
        self.bindgroup_resources.get(key)
    }

    pub fn get_pipeline(&self, key: &ResourceId<Shader>) -> Option<&Shader> {
        self.pipeline_resource.get(key)
    } 
}