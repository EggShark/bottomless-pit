use std::path::Path;

pub(crate) async fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, ReadError> {

    #[cfg(target_arch="wasm32")] 
    {
        use wasm_bindgen_futures::JsFuture;
        use wasm_bindgen::JsCast;
        use js_sys::Uint8Array;

        let path = path.as_ref()
            .as_os_str()
            .to_str()
            .unwrap();

        match web_sys::window() {
            Some(window) => {
                let response_value = 
                    JsFuture::from(window.fetch_with_str(path)).await.unwrap();

                let response: web_sys::Response = response_value.dyn_into().unwrap();

                if !response.ok() {
                    Err(ReadError::ResponseError(response.status(), response.status_text()))?;
                }

                let data = JsFuture::from(response.array_buffer().unwrap()).await.unwrap();
                let bytes = Uint8Array::new(&data).to_vec();
                Ok(bytes)

            }
            None => {
                Err(ReadError::WindowError)
            }
        }
    }

    #[cfg(not(target_arch="wasm32"))]
    {
        Ok(std::fs::read(path)?)
    }
}

#[derive(Debug)]
pub(crate) enum ReadError {
    IoError(std::io::Error),
    #[cfg(target_arch="wasm32")]
    ResponseError(u16, String),
    #[cfg(target_arch="wasm32")]
    WindowError,
}

impl From<std::io::Error> for ReadError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}