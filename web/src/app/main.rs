mod app;
mod components;

pub mod element_ext;
pub mod web_sys_events_ext;

use std::cell::{LazyCell, RefCell};
use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos_ext::signal::{Load, ReadSignalExt};
use schema_analysis::Schema;
use schema_analysis_web::util::read_blob_as_arraybuffer;
use schema_analysis_web::util::send_sync_future_handle;
use web_sys::File;

use web_worker::oneshot::OneshotHandle;

use schema_analysis_web::FileId;
use schema_analysis_web::Format;
use schema_analysis_web::FromWorker;
use schema_analysis_web::ToWorker;

pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    })
    .unwrap();
    log::info!("app init");
    leptos::mount::mount_to_body(app::app);

    // Remove the static loading indicator now that the app has mounted.
    if let Some(el) = leptos::prelude::document().get_element_by_id("loading") {
        el.remove();
    }
}

fn send(msg: ToWorker) -> impl Future<Output = FromWorker> + Send + Sync {
    type Cache = BTreeMap<Format, BTreeMap<FileId, Result<Schema, String>>>;
    thread_local! {
        static HANDLE: LazyCell<OneshotHandle> =
            LazyCell::new(|| OneshotHandle::new("/worker.js").unwrap());
        static CACHE: RefCell<Cache> = const { RefCell::new(Cache::new()) };
    }

    // Split incoming files into already-cached results and ones we still need
    // to ask the worker to infer.
    let format = msg.format;
    let mut cached: BTreeMap<FileId, Result<Schema, String>> = BTreeMap::new();
    let mut uncached: BTreeMap<FileId, web_sys::File> = BTreeMap::new();
    CACHE.with(|c| {
        let c = c.borrow();
        let entries = c.get(&format);
        for (id, file) in msg.files {
            match entries.and_then(|e| e.get(&id)) {
                Some(hit) => {
                    cached.insert(id, hit.clone());
                }
                None => {
                    uncached.insert(id, file);
                }
            }
        }
    });

    // Only round-trip to the worker when there is at least one cache miss.
    let response = (!uncached.is_empty()).then(|| {
        let request = ToWorker {
            format,
            files: uncached,
        }
        .into_js();
        HANDLE.with(|h| h.run_owned(&request)) // Send synchronously
    });

    send_sync_future_handle(async move {
        if let Some(response) = response {
            let new = FromWorker::from_js(response.await).0;
            CACHE.with(|c| {
                let mut c = c.borrow_mut();
                let entry = c.entry(format).or_default();
                for (id, result) in &new {
                    entry.insert(*id, result.clone());
                }
            });
            cached.extend(new);
        }
        FromWorker(cached)
    })
}

#[derive(Clone)]
struct AppState {
    pub user_format: ArcRwSignal<Option<Format>>,
    pub files: ArcRwSignal<BTreeMap<FileId, web_sys::File>>,
    pub selected_file: ArcRwSignal<Option<FileId>>,
    next_id: ArcRwSignal<FileId>,
}

impl AppState {
    fn new() -> Self {
        Self {
            user_format: Default::default(),
            files: Default::default(),
            selected_file: Default::default(),
            next_id: Default::default(),
        }
    }

    // files -> inferred_format
    // (user_format, inferred_format) → format
    pub fn format(&self) -> Signal<Format> {
        let user_format = self.user_format.clone();
        let files = self.files.clone();
        Signal::derive(move || {
            if let Some(fmt) = user_format.get() {
                return fmt;
            }
            // Infer from first file with a recognizable extension.
            if let Some(fmt) = files.with(|files| {
                files
                    .values()
                    .find_map(|f| Format::from_filename(&f.name()))
            }) {
                return fmt;
            }

            Format::Json
        })
    }

    // (format, file_ids) →* schemas via worker inference (with worker-side caching).
    pub fn schemas(&self) -> Signal<Load<BTreeMap<FileId, Result<Schema, String>>>> {
        let files = self.files.clone();
        let format = self.format();

        Signal::derive(move || {
            let format = format.get();
            let files = files.with(|f| f.iter().map(|(id, file)| (*id, file.clone())).collect());
            ToWorker { format, files }
        })
        .map_async(move |msg| {
            let response = send(msg.clone());
            async move { response.await.0 }
        })
    }

    fn next_id(&self) -> FileId {
        let id = self.next_id.get_untracked();
        self.next_id.update(|n| *n += 1);
        id
    }

    pub async fn add_files(&self, files: Vec<File>) {
        if files.is_empty() {
            return;
        }

        let file_infos: Vec<_> = files
            .into_iter()
            .map(|file| (self.next_id(), file))
            .collect();

        // Now update files signal — triggers reactive chain → Infer.
        self.files.update(|files| {
            for (id, info) in file_infos {
                files.insert(id, info);
            }
        });
    }

    pub async fn add_pasted(&self, content: String) {
        let uint8 = js_sys::Uint8Array::from(content.as_bytes());
        let parts = js_sys::Array::of1(&uint8);
        let options = web_sys::FilePropertyBag::new();
        let file = web_sys::File::new_with_u8_array_sequence_and_options(
            &parts,
            &format!("pasted {}", self.next_id.get_untracked()),
            &options,
        )
        .expect("failed to construct File from pasted text");
        self.add_files(vec![file]).await;
    }

    pub async fn remove_file(&self, id: FileId) {
        log::info!("removing file {}", id);

        // Then update state.
        self.files.update(|files| {
            files.remove(&id);
        });
        if self.selected_file.get_untracked() == Some(id) {
            self.selected_file.set(None);
        }
    }
}

pub async fn head(file: &web_sys::File, size: usize) -> (String, bool) {
    (_head(file, size).await, file.size() as usize > size)
}
async fn _head(file: &web_sys::File, size: usize) -> String {
    if file.size() as usize <= size {
        return content(file).await;
    }

    let blob = file.slice_with_i32_and_i32(0, size as i32).unwrap();
    let array_buffer = match read_blob_as_arraybuffer(&blob).await {
        Ok(ab) => ab,
        Err(e) => {
            log::error!("failed to read file {}: {e:?}", file.name());
            return format!("(failed to read file: {}): {e:?}", file.name());
        }
    };

    let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();
    String::from_utf8(bytes).unwrap_or_else(|_| "(binary content)".into())
}
pub async fn content(file: &web_sys::File) -> String {
    let array_buffer = match read_blob_as_arraybuffer(file).await {
        Ok(ab) => ab,
        Err(e) => {
            log::error!("failed to read file {}: {e:?}", file.name());
            return format!("(failed to read file: {}): {e:?}", file.name());
        }
    };

    let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();
    String::from_utf8(bytes).unwrap_or_else(|_| "(binary content)".into())
}
