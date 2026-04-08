use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use utile::task::Task;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

/// Read a blob as an `ArrayBuffer` asynchronously.
pub async fn read_blob_as_arraybuffer(
    blob: &web_sys::Blob,
) -> Result<js_sys::ArrayBuffer, wasm_bindgen::JsValue> {
    let result = JsFuture::from(blob.array_buffer()).await?;
    Ok(result.unchecked_into())
}

/// Read a file as bytes asynchronously.
pub async fn read_file_as_bytes(file: &web_sys::File) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    let ab = read_blob_as_arraybuffer(file).await?;
    Ok(js_sys::Uint8Array::new(&ab).to_vec())
}

/// Fetch the example file from the server.
pub async fn fetch_example() -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    let window = web_sys::window().unwrap();
    let response = JsFuture::from(window.fetch_with_str("/assets/test.json")).await?;
    let response: web_sys::Response = response.unchecked_into();
    let array_buffer = JsFuture::from(response.array_buffer()?).await?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    Ok(uint8_array.to_vec())
}

pub fn download_blob(content: &str, filename: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Create blob
    let array = js_sys::Array::new();
    array.push(&content.into());
    let options = BlobPropertyBag::new();
    options.set_type("text/plain");
    let blob = Blob::new_with_str_sequence_and_options(&array, &options).unwrap();

    // Create object URL
    let url = Url::create_object_url_with_blob(&blob).unwrap();

    // Create and click anchor
    let anchor: HtmlAnchorElement = document.create_element("a").unwrap().dyn_into().unwrap();
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    // Cleanup
    Url::revoke_object_url(&url).unwrap();
}

pub fn send_sync_future_handle<T>(
    fut: impl Future<Output = T> + 'static,
) -> impl Future<Output = T> + Send + Sync + 'static
where
    T: Send + Sync + 'static,
{
    let (tx, rx) = futures::channel::oneshot::channel();
    let task = Task::new_local(async move {
        let result = fut.await;
        tx.send(result)
            .unwrap_or_else(|_| panic!("failed to send result"));
    });
    async move {
        let result = rx.await.unwrap();
        drop(task);
        result
    }
}

pub fn leak_future<T>(
    fut: impl Future<Output = T> + 'static,
) -> impl Future<Output = T> + Send + Sync + 'static
where
    T: Send + Sync + 'static,
{
    let (tx, rx) = futures::channel::oneshot::channel();
    // TODO: Task::leak
    wasm_bindgen_futures::spawn_local(async move {
        let result = fut.await;
        tx.send(result)
            .unwrap_or_else(|_| panic!("failed to send result"));
    });
    async move { rx.await.unwrap() }
}

// Horrible hack, forces serialization to JSON.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AsJson<T>(pub T);
impl<T> AsJson<T> {
    pub fn inner(self) -> T {
        self.0
    }
}
impl<T> Deref for AsJson<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for AsJson<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T> Serialize for AsJson<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_json::to_string(&self.0)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
impl<'de, T> Deserialize<'de> for AsJson<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        let v = serde_json::from_str(s).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}
