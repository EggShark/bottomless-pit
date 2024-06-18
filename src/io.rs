use crate::resource::ResourceId;
use crate::shader::Shader;
use crate::texture::Texture;

use std::path::Path;

use futures::executor::ThreadPool;

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
    #[cfg(not(target_arch="wasm32"))]
    pool: ThreadPool,
    #[cfg(target_arch = "wasm32")]
    blocked: bool,
}

impl Loader {
    // just fs read that stuff man
    #[cfg(not(target_arch = "wasm32"))]
    pub fn blocking_load() {

    }

    // request but flip flag :3
    #[cfg(target_arch = "wasm32")]
    pub fn blocking_load() {

    }

    // threadpool / aysnc
    pub fn background_load() {

    }
}

pub(crate) struct DefualtResources {
    pub(crate) defualt_texture_id: ResourceId<Texture>,
    pub(crate) default_pipeline_id: ResourceId<Shader>,
    pub(crate) line_pipeline_id: ResourceId<Shader>,
}