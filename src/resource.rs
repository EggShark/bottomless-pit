use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{PathBuf, Path};
use std::num::NonZeroU64;
use std::sync::atomic::AtomicU64;

use crate::engine_handle::{Engine, BpEvent};
use crate::io::{self, ReadError};

#[derive(Debug)]
pub(crate) enum Resource {
    Ok {
        path: PathBuf,
        data: Vec<u8>,
        id: NonZeroU64,
        resource_type: ResourceType,
    },
    Error {
        error: ReadError,
        path: PathBuf,
        id: NonZeroU64,
        resource_type: ResourceType,
    }
}

impl Resource {
    pub fn from_result(result: Result<Vec<u8>, ReadError>, path: PathBuf, id: NonZeroU64, resource_type: ResourceType) -> Self {
        match result {
            Ok(data) => {
                Resource::Ok {
                    path,
                    data,
                    id,
                    resource_type,
                }
            },
            Err(e) => {
                Resource::Error {
                    error: e,
                    path,
                    id,
                    resource_type,
                }
            }
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


#[derive(Clone, Copy, Debug)]
pub(crate) enum ResourceType {
    Image,
    Shader,
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
    #[cfg(target_arch="wasm32")]
    {
        use wasm_bindgen_futures::spawn_local;
        let event_loop_proxy = engine.get_proxy();
        spawn_local(async move {
            let result = io::read(&path).await;
            let resource = Resource::from_result(result, path, id, resource_type);
            event_loop_proxy.send_event(BpEvent::ResourceLoaded(resource)).unwrap();
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceId<T>(NonZeroU64, std::marker::PhantomData<T>);

impl<T> ResourceId<T> {
    pub(crate) fn get_id(&self) -> NonZeroU64 {
        self.0
    }
}

type ResourceMap<T> = HashMap<ResourceId<T>, T>;

pub(crate) struct ResourceManager {
    btye_resources: ResourceMap<Vec<u8>>,
    bindgroup_resources: ResourceMap<wgpu::BindGroup>,
    pipeline_resource: ResourceMap<wgpu::RenderPipeline>,
}