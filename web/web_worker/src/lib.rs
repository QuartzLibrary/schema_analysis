//! Thin wrapper around the Web Worker API.
//!
//! Provides [`WorkerHandle`] (main thread) and [`WorkerScope`] (worker thread)
//! with closure-based message handling and support for transferable objects.
//!
//! # Posting variants
//!
//! | Method              | Semantics                                                  |
//! |---------------------|------------------------------------------------------------|
//! | `post_message_raw`  | Direct `postMessage` — caller controls the transfer list   |
//! | `post_copy`         | Structured clone only — everything is copied               |
//! | `post_with`         | Caller provides the list of transferable objects explicitly |
//! | `post_owned`        | Walks the value, transfers every transferable it finds     |

pub mod oneshot;

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

use futures::StreamExt;
use futures::channel::mpsc;
use js_sys::Array;
use utile::task::Task;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use web_sys::{
    DedicatedWorkerGlobalScope, ErrorEvent, ImageBitmap, MessageEvent, MessagePort,
    OffscreenCanvas, ReadableStream, TransformStream, VideoFrame, Worker, WorkerOptions,
    WorkerType, WritableStream,
};

// ---------------------------------------------------------------------------
// WorkerHandle (main thread)
// ---------------------------------------------------------------------------

/// Main-thread handle to a Web Worker.
#[derive(Clone)]
pub struct WorkerHandle {
    inner: Rc<WorkerHandleInner>,
}
struct WorkerHandleInner {
    inner: Worker,
    /// Prevent the closures from being GC'd on the JS side.
    _onmessage: Closure<dyn Fn(MessageEvent)>,
    _onerror: Closure<dyn Fn(ErrorEvent)>,
}

impl Drop for WorkerHandleInner {
    fn drop(&mut self) {
        self.inner.terminate();
    }
}

impl WorkerHandle {
    /// Spawn a new worker from `script_url` and install `on_message`
    /// as the handler for messages posted back by the worker.
    pub fn new(
        script_url: &str,
        on_message: impl Fn(WorkerHandle, MessageEvent) + 'static,
    ) -> Result<Self, JsValue> {
        let inner = with_blob_url(&absolute_url(script_url), |blob_url| {
            Worker::new_with_options(blob_url, &{
                let opts = WorkerOptions::new();
                opts.set_type(WorkerType::Module);
                opts
            })
        })??;

        Ok(Self {
            inner: Rc::new_cyclic(move |weak: &Weak<_>| {
                WorkerHandleInner::new(weak, inner, on_message)
            }),
        })
    }
    pub fn new_async_unordered<F>(
        script_url: &str,
        on_message: impl Fn(Self, MessageEvent) -> F + 'static,
    ) -> Result<Self, JsValue>
    where
        F: Future<Output = ()> + 'static,
    {
        let on_message = {
            let tasks = Rc::new(RefCell::new(HashMap::new()));
            let counter = Cell::new(0);
            move |worker: Self, event: MessageEvent| {
                let id = counter.get();
                counter.set(id + 1);
                let future = on_message(worker, event);
                let task = Task::new_local({
                    let tasks = tasks.clone();
                    async move {
                        future.await;
                        tasks.borrow_mut().remove(&id);
                    }
                });
                tasks.borrow_mut().insert(id, task);
            }
        };

        Self::new(script_url, on_message)
    }

    pub fn new_async_ordered<F>(
        script_url: &str,
        on_message: impl Fn(Self, MessageEvent) -> F + 'static,
    ) -> Result<Self, JsValue>
    where
        F: Future<Output = ()> + 'static,
    {
        let on_message = {
            let (tx, mut rx) = mpsc::unbounded();
            let _task = Task::new_local(async move {
                while let Some((worker, event)) = rx.next().await {
                    on_message(worker, event).await;
                    log::debug!("[Worker Parent] ordered task completed");
                }
                log::error!("[Worker Parent] Executing task has finished. This should not happen.");
            });
            move |worker: Self, event: MessageEvent| {
                let _ = tx.unbounded_send((worker, event));
                let _capture = &_task;
            }
        };

        Self::new(script_url, on_message)
    }

    /// Direct `postMessage` with an optional transfer list.
    pub fn post_message_raw(
        &self,
        message: &JsValue,
        transfer: Option<&Array>,
    ) -> Result<(), JsValue> {
        log::debug!("[Worker Parent] post_message_raw");
        match transfer {
            Some(t) => self.inner.inner.post_message_with_transfer(message, t),
            None => self.inner.inner.post_message(message),
        }
    }

    /// Structured-clone the entire message (no transfers).
    ///
    /// Transferable objects reachable from `message` are *copied*, not moved.
    pub fn post_copy(&self, message: &JsValue) -> Result<(), JsValue> {
        log::debug!("[Worker Parent] post_copy");
        self.inner.inner.post_message(message)
    }

    /// Post `message`, transferring the listed objects instead of copying them.
    ///
    /// Each entry in `transfer` must be a transferable object reachable from
    /// `message`. After the call, transferred objects are detached in the
    /// sending context.
    pub fn post_with(&self, message: &JsValue, transfer: &[&JsValue]) -> Result<(), JsValue> {
        log::debug!("[Worker Parent] post_with");
        let arr = Array::from_iter(transfer.iter().map(|obj| (*obj).clone()));
        self.inner.inner.post_message_with_transfer(message, &arr)
    }

    /// Post `message`, automatically transferring every transferable object
    /// reachable from it.
    ///
    /// Walks the value tree and collects all transferable objects into the
    /// transfer list. After the call, those objects are detached in the
    /// sending context.
    pub fn post_owned(&self, message: &JsValue) -> Result<(), JsValue> {
        log::debug!("[Worker Parent] post_owned");
        let transfer = collect_transferables(message);
        if transfer.length() == 0 {
            self.inner.inner.post_message(message)
        } else {
            self.inner
                .inner
                .post_message_with_transfer(message, &transfer)
        }
    }
}

impl WorkerHandleInner {
    fn new(
        weak: &Weak<Self>,
        inner: Worker,
        on_message: impl Fn(WorkerHandle, MessageEvent) + 'static,
    ) -> Self {
        log::debug!("[Worker Parent] new");

        // Install an onerror handler that panics so failures are loud
        // instead of silently hanging.
        let onerror: Closure<dyn Fn(ErrorEvent)> = Closure::new(|e: ErrorEvent| {
            panic!(
                "Worker error: {} ({}:{}:{})",
                e.message(),
                e.filename(),
                e.lineno(),
                e.colno(),
            );
        });
        inner.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        let on_message = {
            let weak = weak.clone();
            move |event: MessageEvent| {
                log::debug!("[Worker Parent] onmessage");

                if let Some(worker) = weak.upgrade() {
                    on_message(WorkerHandle { inner: worker }, event);
                } else {
                    log::error!("[Worker Parent] worker has been dropped");
                }
            }
        };

        let on_message = Closure::wrap(Box::new(on_message) as Box<dyn Fn(MessageEvent)>);
        inner.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        Self {
            inner,
            _onmessage: on_message,
            _onerror: onerror,
        }
    }
}

// ---------------------------------------------------------------------------
// WorkerScope (worker thread)
// ---------------------------------------------------------------------------

/// Worker-side handle for communicating back to the main thread.
#[derive(Clone)]
pub struct WorkerScope {
    inner: Rc<WorkerScopeInner>,
}

struct WorkerScopeInner {
    inner: DedicatedWorkerGlobalScope,
    /// Prevent the closure from being GC'd on the JS side.
    _onmessage: Closure<dyn Fn(MessageEvent)>,
}

impl WorkerScope {
    /// Capture the current worker's global scope and install `on_message`
    /// as the handler for messages posted by the main thread.
    pub fn new(on_message: impl Fn(Self, MessageEvent) + 'static) -> Self {
        let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();

        if scope.onmessage().is_some() {
            log::error!("[Worker] onmessage is already set");
        }

        Self {
            inner: Rc::new_cyclic(move |weak: &Weak<_>| {
                WorkerScopeInner::new(weak, scope, on_message)
            }),
        }
    }
    pub fn new_async_unordered<F>(on_message: impl Fn(Self, MessageEvent) -> F + 'static) -> Self
    where
        F: Future<Output = ()> + 'static,
    {
        let on_message = {
            let tasks = Rc::new(RefCell::new(HashMap::new()));
            let counter = Cell::new(0);
            move |worker: Self, event: MessageEvent| {
                let id = counter.get();
                counter.set(id + 1);
                let future = on_message(worker, event);
                let task = Task::new_local({
                    let tasks = tasks.clone();
                    async move {
                        future.await;
                        log::debug!("[Worker] unordered task completed {id}");
                        tasks.borrow_mut().remove(&id);
                    }
                });
                tasks.borrow_mut().insert(id, task);
            }
        };

        Self::new(on_message)
    }

    pub fn new_async_ordered<F>(on_message: impl Fn(Self, MessageEvent) -> F + 'static) -> Self
    where
        F: Future<Output = ()> + 'static,
    {
        let on_message = {
            let (tx, mut rx) = mpsc::unbounded();
            let _task = Task::new_local(async move {
                while let Some((worker, event)) = rx.next().await {
                    on_message(worker, event).await;
                    log::debug!("[Worker] ordered task completed");
                }
                log::error!("[Worker] Executing task has finished. This should not happen.");
            });
            move |worker: Self, event: MessageEvent| {
                let _ = tx.unbounded_send((worker, event));
                let _capture = &_task;
            }
        };

        Self::new(on_message)
    }

    /// Direct `postMessage` with an optional transfer list.
    pub fn post_message_raw(
        &self,
        message: &JsValue,
        transfer: Option<&Array>,
    ) -> Result<(), JsValue> {
        log::debug!("[Worker] post_message_raw");
        match transfer {
            Some(t) => self.inner.inner.post_message_with_transfer(message, t),
            None => self.inner.inner.post_message(message),
        }
    }

    /// Structured-clone the entire message (no transfers).
    pub fn post_copy(&self, message: &JsValue) -> Result<(), JsValue> {
        log::debug!("[Worker] post_copy");
        self.inner.inner.post_message(message)
    }

    /// Post `message`, transferring the listed objects instead of copying them.
    pub fn post_with(&self, message: &JsValue, transfer: &[&JsValue]) -> Result<(), JsValue> {
        log::debug!("[Worker] post_with");
        let arr = Array::from_iter(transfer.iter().map(|obj| (*obj).clone()));
        self.inner.inner.post_message_with_transfer(message, &arr)
    }

    /// Post `message`, automatically transferring every transferable object
    /// reachable from it.
    pub fn post_owned(&self, message: &JsValue) -> Result<(), JsValue> {
        log::debug!("[Worker] post_owned");
        let transfer = collect_transferables(message);
        if transfer.length() == 0 {
            self.inner.inner.post_message(message)
        } else {
            self.inner
                .inner
                .post_message_with_transfer(message, &transfer)
        }
    }
}

impl WorkerScopeInner {
    /// Capture the current worker's global scope and install `on_message`
    /// as the handler for messages posted by the main thread.
    pub fn new(
        weak: &Weak<Self>,
        scope: DedicatedWorkerGlobalScope,
        on_message: impl Fn(WorkerScope, MessageEvent) + 'static,
    ) -> Self {
        log::debug!("[Worker] new");

        // We pass a mock closure to avoid recursion.
        let on_message = {
            let weak = weak.clone();
            move |event: MessageEvent| {
                log::debug!("[Worker] onmessage");
                if let Some(worker) = weak.upgrade() {
                    on_message(WorkerScope { inner: worker }, event);
                } else {
                    log::error!("[Worker] worker has been dropped");
                }
            }
        };

        let on_message = Closure::wrap(Box::new(on_message) as Box<dyn Fn(MessageEvent)>);
        scope.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        Self {
            inner: scope,
            _onmessage: on_message,
        }
    }
}

// ---------------------------------------------------------------------------
// Transferable collection
// ---------------------------------------------------------------------------

/// Walk a JS value tree, calling `visitor` on every leaf and nested value.
///
/// Traverses own enumerable properties of objects and elements of arrays.
/// The visitor is called *before* descending into children, so it sees
/// containers as well as leaves.  Values for which [`is_opaque`] returns
/// `true` are visited but never descended into.
pub fn walk_js(value: &JsValue, visitor: &mut impl FnMut(&JsValue)) {
    visitor(value);

    if is_opaque(value) {
        return;
    }

    if Array::is_array(value) {
        let arr: &Array = value.unchecked_ref();
        for item in arr.iter() {
            walk_js(&item, visitor);
        }
    } else if value.is_object() {
        let obj: &js_sys::Object = value.unchecked_ref();
        let keys = js_sys::Object::keys(obj);
        for key in keys.iter() {
            if let Ok(val) = js_sys::Reflect::get(value, &key) {
                walk_js(&val, visitor);
            }
        }
    }
}

/// Returns `true` if `walk_js` should **not** descend into the value's
/// properties.  Transferable objects are opaque host objects whose internal
/// slots are not visible via `Object.keys`, and some (e.g. `TransformStream`)
/// expose nested transferables that should not be collected separately.
fn is_opaque(v: &JsValue) -> bool {
    is_transferable(v) // TODO: check
}

/// Returns `true` if the value is a
/// [transferable object](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects).
fn is_transferable(v: &JsValue) -> bool {
    v.is_instance_of::<js_sys::ArrayBuffer>()
        || v.is_instance_of::<MessagePort>()
        || v.is_instance_of::<ReadableStream>()
        || v.is_instance_of::<WritableStream>()
        || v.is_instance_of::<TransformStream>()
        || v.is_instance_of::<OffscreenCanvas>()
        || v.is_instance_of::<ImageBitmap>()
        || v.is_instance_of::<VideoFrame>()
}

/// Collect every transferable object reachable from `value`.
fn collect_transferables(value: &JsValue) -> Array {
    let out = Array::new();
    walk_js(value, &mut |v| {
        if is_transferable(v) {
            out.push(v);
        }
    });
    out
}

fn with_blob_url<O>(absolute_url: &str, f: impl FnOnce(&str) -> O) -> Result<O, JsValue> {
    let blob_url = web_sys::Url::create_object_url_with_blob(
        &web_sys::Blob::new_with_str_sequence_and_options(
            &Array::of1(&JsValue::from_str(&{
                // Initialising a wasm module is async, so we cache the messages
                // and replay them after the module is initialised.
                format!(
                    "import init from '{absolute_url}';
                     const _buffer = [];
                     const _handler = (e) => _buffer.push(e);
                     self.addEventListener('message', _handler);
                     await init();
                     self.removeEventListener('message', _handler);
                     for (const e of _buffer) self.onmessage(e);",
                )
            })),
            &{
                let blob_opts = web_sys::BlobPropertyBag::new();
                blob_opts.set_type("application/javascript");
                blob_opts
            },
        )?,
    )?;

    let result = f(&blob_url);

    // Revoke the blob URL since the worker has already started loading.
    let _ = web_sys::Url::revoke_object_url(&blob_url);

    Ok(result)
}

fn absolute_url(script_url: &str) -> String {
    let base = web_sys::window()
        .expect("no window")
        .location()
        .href()
        .expect("no href");
    web_sys::Url::new_with_base(script_url, &base)
        .expect("invalid script_url")
        .href()
}
