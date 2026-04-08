use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::pin;
use std::rc::Rc;

use futures::StreamExt;
use futures::channel::mpsc;

use leptos::attr::Attribute;
use leptos::ev::{self, EventDescriptor};
use leptos::html::{ElementType, HtmlElement};
use leptos::prelude::*;
use leptos::tachys::renderer::dom::Event;
use leptos_ext::signal::{ReadSignalExt, WriteSignalExt};
use utile::task::Task;
use wasm_bindgen::prelude::*;
use web_sys::File;

// Double Ext to distinguish from the one in leptos_ext.
pub trait HtmlElementExtExt<E, At, Ch>: Sized
where
    E: ElementType + 'static,
    At: Attribute + 'static,
    Ch: RenderHtml + 'static,
{
    /// Runs an event handler asynchronously.
    ///
    /// Semantics: only one handler is active at a time,
    /// if the event fires again the previous handler stops if incomplete.
    fn on_async_singleton<Ev, Fut>(
        self,
        event: Ev,
        handler: impl FnMut(Ev::EventType) -> Fut + 'static,
    ) -> HtmlElement<E, impl Attribute + 'static, Ch>
    where
        Ev: EventDescriptor + Send + 'static,
        Ev::EventType: 'static,
        Ev::EventType: From<Event>,
        Fut: Future<Output = ()> + 'static;
    /// Runs an event handler asynchronously.
    ///
    /// Semantics: handlers are executed in strict arrival order,
    /// if the event fires again then the new handler is queued.
    ///
    /// NOTE: the initial blocking part of the handler is executed immediately upon event firing.
    fn on_async_ordered<Ev, Fut>(
        self,
        event: Ev,
        handler: impl FnMut(Ev::EventType) -> Fut + 'static,
    ) -> HtmlElement<E, impl Attribute + 'static, Ch>
    where
        Ev: EventDescriptor + Send + 'static,
        Ev::EventType: 'static,
        Ev::EventType: From<Event>,
        Fut: Future<Output = ()> + 'static;
    /// Runs an event handler asynchronously.
    ///
    /// Semantics: handlers are executed in parallel,
    /// if the event fires again then the new handler is started immediately.
    fn on_async_unordered<Ev, Fut>(
        self,
        event: Ev,
        handler: impl FnMut(Ev::EventType) -> Fut + 'static,
    ) -> HtmlElement<E, impl Attribute + 'static, Ch>
    where
        Ev: EventDescriptor + Send + 'static,
        Ev::EventType: 'static,
        Ev::EventType: From<Event>,
        Fut: Future<Output = ()> + 'static;
    /// Runs an event handler asynchronously.
    ///
    /// Semantics: handlers are leaked, and will keep running
    /// even if the element and context are dropped.
    fn on_async_leak<Ev, Fut>(
        self,
        event: Ev,
        handler: impl FnMut(Ev::EventType) -> Fut + 'static,
    ) -> HtmlElement<E, impl Attribute + 'static, Ch>
    where
        Ev: EventDescriptor + Send + 'static,
        Ev::EventType: 'static,
        Ev::EventType: From<Event>,
        Fut: Future<Output = ()> + 'static;

    /// Handle file drop events.
    fn on_drag_and_drop(
        self,
        dragging: RwSignal<bool>,
        handler: impl Fn(Vec<File>) + Clone + 'static,
    ) -> HtmlElement<E, impl Attribute + 'static, Ch>;

    /// Store the value in the element, so that they are dropped together.
    ///
    /// WARNING: this actually just stores it in the reactive context.
    /// TODO: actually store it in the dom value.
    fn hold_value<T: 'static>(self, value: T) -> Self {
        StoredValue::new_local(value); // TODO store in dom value.
        self
    }
}

impl<E, At, Ch> HtmlElementExtExt<E, At, Ch> for HtmlElement<E, At, Ch>
where
    E: ElementType + 'static,
    At: Attribute + 'static,
    Ch: RenderHtml + 'static,
{
    fn on_async_singleton<Ev, Fut>(
        self,
        event: Ev,
        mut handler: impl FnMut(Ev::EventType) -> Fut + 'static,
    ) -> HtmlElement<E, impl Attribute + 'static, Ch>
    where
        Ev: EventDescriptor + Send + 'static,
        Ev::EventType: 'static,
        Ev::EventType: From<Event>,
        Fut: Future<Output = ()> + 'static,
    {
        let mut _task = None;

        self.on(event, move |e| {
            drop(_task.take());
            _task = Some(Task::new_local(handler(e)))
        })
    }
    fn on_async_ordered<Ev, Fut>(
        self,
        event: Ev,
        mut handler: impl FnMut(Ev::EventType) -> Fut + 'static,
    ) -> HtmlElement<E, impl Attribute + 'static, Ch>
    where
        Ev: EventDescriptor + Send + 'static,
        Ev::EventType: 'static,
        Ev::EventType: From<Event>,
        Fut: Future<Output = ()> + 'static,
    {
        let (tx, mut rx) = mpsc::unbounded();
        let _task = Task::new_local(async move {
            while let Some(fut) = rx.next().await {
                fut.await;
            }
        });

        self.on(event, move |e| {
            let _ = tx.unbounded_send(handler(e));
            let _capture = &_task;
        })
    }
    fn on_async_unordered<Ev, Fut>(
        self,
        event: Ev,
        mut handler: impl FnMut(Ev::EventType) -> Fut + 'static,
    ) -> HtmlElement<E, impl Attribute + 'static, Ch>
    where
        Ev: EventDescriptor + Send + 'static,
        Ev::EventType: 'static,
        Ev::EventType: From<Event>,
        Fut: Future<Output = ()> + 'static,
    {
        let tasks = Rc::new(RefCell::new(HashMap::new()));

        let mut counter = 0;

        self.on(event, move |e| {
            let id = counter;
            counter += 1;
            tasks.borrow_mut().insert(
                id,
                Task::new_local({
                    let tasks = tasks.clone();
                    let fut = handler(e);
                    async move {
                        fut.await;
                        tasks.borrow_mut().remove(&id);
                    }
                }),
            );
            let _capture = &tasks;
        })
    }
    fn on_async_leak<Ev, Fut>(
        self,
        event: Ev,
        mut handler: impl FnMut(Ev::EventType) -> Fut + 'static,
    ) -> HtmlElement<E, impl Attribute + 'static, Ch>
    where
        Ev: EventDescriptor + Send + 'static,
        Ev::EventType: 'static,
        Ev::EventType: From<Event>,
        Fut: Future<Output = ()> + 'static,
    {
        self.on(event, move |e| {
            // TODO: Task::leak
            wasm_bindgen_futures::spawn_local(handler(e));
        })
    }

    fn on_drag_and_drop(
        self,
        dragging: RwSignal<bool>,
        handler: impl FnMut(Vec<File>) + Clone + 'static,
    ) -> HtmlElement<E, impl Attribute + 'static, Ch> {
        let active = RwSignal::new(false);

        active.for_each_immediate(move |active| dragging.set(*active));

        self.on(ev::dragenter, move |e: web_sys::DragEvent| {
            e.prevent_default();
            if drag_related_is_outside(&e) {
                active.set_if_changed(true);
            }
        })
        .on(ev::dragover, move |e: web_sys::DragEvent| {
            e.prevent_default();
            if let Some(dt) = e.data_transfer() {
                dt.set_drop_effect("copy");
            }
        })
        .on(ev::dragleave, move |e: web_sys::DragEvent| {
            if drag_related_is_outside(&e) {
                active.set_if_changed(false);
            }
        })
        .on(ev::drop, move |e: web_sys::DragEvent| {
            e.stop_propagation();
            e.prevent_default();

            active.set(false);

            let Some(data_transfer) = e.data_transfer() else {
                return;
            };

            // The DataTransfer is only valid synchronously inside the drop
            // handler, so we must extract every entry now and resolve them
            // (which is async) afterwards.
            let entries = collect_dropped_entries(&data_transfer);

            let mut handler = handler.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let files = resolve_dropped_entries(entries).await;
                handler(files);
            });
        })
    }
}

/// A drop entry. Folders end up as `Directory` so they can be walked
/// recursively after the drop event finishes.
enum DroppedEntry {
    File(File),
    Directory(web_sys::FileSystemDirectoryEntry),
}

/// Pull entries out of a `DataTransfer` synchronously.
///
/// Prefers `webkitGetAsEntry()` so that dropped folders can be walked.
/// Falls back to `DataTransfer::files()` when items aren't available
/// (e.g. some programmatic drops).
fn collect_dropped_entries(data_transfer: &web_sys::DataTransfer) -> Vec<DroppedEntry> {
    let mut out = vec![];

    let items = data_transfer.items();
    if items.length() > 0 {
        for i in 0..items.length() {
            let Some(item) = items.get(i) else { continue };
            if item.kind() != "file" {
                continue;
            }
            match item.webkit_get_as_entry() {
                Ok(Some(entry)) if entry.is_directory() => {
                    let dir: web_sys::FileSystemDirectoryEntry = entry.unchecked_into();
                    out.push(DroppedEntry::Directory(dir));
                }
                Ok(Some(_entry)) => {
                    // `getAsFile` is synchronous and avoids an extra
                    // round-trip through `FileSystemFileEntry::file`.
                    if let Ok(Some(file)) = item.get_as_file() {
                        out.push(DroppedEntry::File(file));
                    }
                }
                Ok(None) => {
                    log::error!("failed to get entry {}: none", item.type_());
                }
                Err(e) => {
                    log::error!("failed to get entry {}: {e:?}", item.type_());
                    if let Ok(Some(file)) = item.get_as_file() {
                        out.push(DroppedEntry::File(file));
                    }
                }
            }
        }
        return out;
    }

    if let Some(files) = data_transfer.files() {
        for i in 0..files.length() {
            if let Some(file) = files.get(i) {
                out.push(DroppedEntry::File(file));
            }
        }
    }

    out
}

async fn resolve_dropped_entries(entries: Vec<DroppedEntry>) -> Vec<File> {
    let mut files = vec![];
    for entry in entries {
        match entry {
            DroppedEntry::File(file) => files.push(file),
            DroppedEntry::Directory(dir) => {
                if let Err(e) = walk_directory(&dir, &mut files).await {
                    log::error!("failed to walk dropped directory {}: {e:?}", dir.name());
                }
            }
        }
    }
    files
}

fn walk_directory<'a>(
    dir: &'a web_sys::FileSystemDirectoryEntry,
    out: &'a mut Vec<File>,
) -> std::pin::Pin<Box<dyn Future<Output = Result<(), JsValue>> + 'a>> {
    Box::pin(async move {
        let reader = dir.create_reader();
        // `readEntries` may return entries in batches; an empty result means we're done.
        while let entries = read_directory_entries(&reader).await?
            && !entries.is_empty()
        {
            for entry in entries {
                if entry.is_file() {
                    let file_entry: web_sys::FileSystemFileEntry = entry.unchecked_into();
                    match file_from_entry(&file_entry).await {
                        Ok(file) => out.push(file),
                        Err(e) => {
                            log::error!("failed to read dropped file {}: {e:?}", file_entry.name());
                        }
                    }
                } else if entry.is_directory() {
                    let dir_entry: web_sys::FileSystemDirectoryEntry = entry.unchecked_into();
                    walk_directory(&dir_entry, out).await?;
                }
            }
        }

        Ok(())
    })
}

async fn read_directory_entries(
    reader: &web_sys::FileSystemDirectoryReader,
) -> Result<Vec<web_sys::FileSystemEntry>, JsValue> {
    let (tx_success, rx_success) = futures::channel::oneshot::channel();
    let (tx_error, rx_error) = futures::channel::oneshot::channel();

    let success = Closure::once(move |entries: JsValue| {
        let array: js_sys::Array = entries.unchecked_into();
        let mut out = Vec::with_capacity(array.length() as usize);
        for v in array.iter() {
            out.push(v.unchecked_into::<web_sys::FileSystemEntry>());
        }
        let _ = tx_success.send(out);
    });
    let error = Closure::once(move |err: JsValue| {
        let _ = tx_error.send(err);
    });

    let success: &JsValue = success.as_ref();
    let error: &JsValue = error.as_ref();

    reader.read_entries_with_callback_and_callback(
        success.dyn_ref().unwrap(),
        error.dyn_ref().unwrap(),
    )?;

    match futures::future::select(
        pin!(async move { rx_success.await.unwrap() }),
        pin!(async move { rx_error.await.unwrap() }),
    )
    .await
    {
        futures::future::Either::Left((success, _)) => Ok(success),
        futures::future::Either::Right((error, _)) => Err(error),
    }
}

async fn file_from_entry(entry: &web_sys::FileSystemFileEntry) -> Result<File, JsValue> {
    let (tx_success, rx_success) = futures::channel::oneshot::channel();
    let (tx_error, rx_error) = futures::channel::oneshot::channel();

    let success = {
        Closure::once(move |file: JsValue| {
            let _ = tx_success.send(file.unchecked_into::<File>());
        })
    };
    let error = Closure::once(move |err: JsValue| {
        let _ = tx_error.send(err);
    });

    let success: &JsValue = success.as_ref();
    let error: &JsValue = error.as_ref();

    entry.file_with_callback_and_callback(success.dyn_ref().unwrap(), error.dyn_ref().unwrap());

    match futures::future::select(
        pin!(async move { rx_success.await.unwrap() }),
        pin!(async move { rx_error.await.unwrap() }),
    )
    .await
    {
        futures::future::Either::Left((success, _)) => Ok(success),
        futures::future::Either::Right((error, _)) => Err(error),
    }
}

/// Check if a drag event's `relatedTarget` is outside the `currentTarget`.
/// Child-to-child transitions within the zone have `relatedTarget` inside, so they're ignored.
/// Real entry/exit has `relatedTarget` outside (or null), so it's detected.
fn drag_related_is_outside(e: &web_sys::DragEvent) -> bool {
    let Some(current) = e.current_target() else {
        return true;
    };
    let current: web_sys::Node = current.unchecked_into();
    match e.related_target() {
        None => true,
        Some(related) => {
            let related: web_sys::Node = related.unchecked_into();
            !current.contains(Some(&related))
        }
    }
}
