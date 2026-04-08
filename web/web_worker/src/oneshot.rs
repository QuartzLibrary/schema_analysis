//! Request–response layer on top of [`crate`].
//!
//! - Use [`OneshotHandle`] on the main thread.
//! - Use [`OneshotScope`] on the worker thread.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use futures::StreamExt;
use futures::channel::{mpsc, oneshot};
use futures::stream::{AbortHandle, AbortRegistration, Aborted};
use js_sys::Array;
use utile::drop::ExecuteOnDrop;
use utile::task::Task;
use wasm_bindgen::prelude::*;
use web_sys::MessageEvent;

use crate::{WorkerHandle, WorkerScope};

// ---------------------------------------------------------------------------
// OneshotHandle (main thread)
// ---------------------------------------------------------------------------

/// Async request–response handle backed by a [`WorkerHandle`].
pub struct OneshotHandle {
    inner: WorkerHandle,
    pending: Rc<RefCell<HashMap<u64, Callback>>>,
    next_nonce: Rc<Cell<u64>>,
}

enum ToWorker {
    Call { nonce: u64, msg: JsValue },
    Cancel { nonce: u64 },
}

struct FromWorker {
    nonce: u64,
    msg: JsValue,
}

type Callback = Box<dyn FnOnce(JsValue)>;

// TO bypass lack of precise capture rules if we use an inner function.
// https://github.com/rust-lang/rust/issues/130043
macro_rules! run_internal {
    ($self:ident, $message:expr, $post:expr) => {{
        let loc = std::panic::Location::caller();
        let (tx, rx) = oneshot::channel();

        let cleanup_on_drop = $self.run_internal_sync(
            $message,
            move |data: JsValue| match tx.send(data) {
                Ok(()) => {}
                Err(_data) => {
                    log::error!("[Oneshot] Receiver dropped at {loc}");
                }
            },
            $post,
        );

        async move {
            let result = rx.await.expect("worker dropped without responding");
            drop(cleanup_on_drop);
            result
        }
    }};
}

impl OneshotHandle {
    /// Spawn a new worker and set up nonce-based response routing.
    pub fn new(script_url: &str) -> Result<Self, JsValue> {
        let pending: Rc<RefCell<HashMap<u64, Callback>>> = Default::default();

        let worker = WorkerHandle::new(script_url, {
            let pending = pending.clone();
            move |_worker, event| {
                let FromWorker { nonce, msg } =
                    FromWorker::from_js(&event.data()).expect("worker response missing envelope");

                let callback = pending.borrow_mut().remove(&nonce);
                if let Some(callback) = callback {
                    callback(msg);
                } else {
                    log::debug!("[Oneshot] received response for unknown nonce {nonce}.");
                }
            }
        })?;

        Ok(Self {
            inner: worker,
            pending,
            next_nonce: Rc::new(Cell::new(0)),
        })
    }

    /// Structured-clone the entire message (no transfers).
    ///
    /// Note: the message will be dispatched synchronously.
    #[track_caller]
    pub fn run_copy(&self, message: &JsValue) -> impl Future<Output = JsValue> + use<> {
        run_internal!(self, message, |inner, msg| inner.post_copy(msg))
    }

    /// Automatically transfer every transferable object reachable from the
    /// message.
    ///
    /// Note: the message will be dispatched synchronously.
    #[track_caller]
    pub fn run_owned(&self, message: &JsValue) -> impl Future<Output = JsValue> + use<> {
        run_internal!(self, message, |inner, msg| inner.post_owned(msg))
    }

    /// Transfer the listed objects explicitly.
    ///
    /// Note: the message will be dispatched synchronously.
    #[track_caller]
    pub fn run_with(
        &self,
        message: &JsValue,
        transfer: &[&JsValue],
    ) -> impl Future<Output = JsValue> + use<> {
        run_internal!(self, message, move |inner, msg| inner
            .post_with(msg, transfer))
    }

    /// Direct `postMessage` with an optional transfer list.
    ///
    /// Note: the message will be dispatched synchronously.
    #[track_caller]
    pub fn run_message_raw(
        &self,
        message: &JsValue,
        transfer: Option<&Array>,
    ) -> impl Future<Output = JsValue> + use<> {
        run_internal!(self, message, move |inner, msg| {
            inner.post_message_raw(msg, transfer)
        })
    }

    fn run_internal_sync(
        &self,
        message: &JsValue,
        callback: impl FnOnce(JsValue) + 'static,
        post: impl FnOnce(&WorkerHandle, &JsValue) -> Result<(), JsValue>,
    ) -> ExecuteOnDrop {
        let nonce = self.next_nonce();

        self.pending.borrow_mut().insert(nonce, Box::new(callback));

        let cleanup = ExecuteOnDrop::new_dyn({
            let inner = self.inner.clone();
            let pending = self.pending.clone();

            move || {
                if pending.borrow_mut().remove(&nonce).is_some() {
                    log::debug!("[Oneshot] cancelling request for nonce {nonce}.");
                    inner
                        .post_copy(&ToWorker::Cancel { nonce }.to_js())
                        .expect("failed to post cancel message to worker");
                }
            }
        });

        post(
            &self.inner,
            &ToWorker::Call {
                nonce,
                msg: message.clone(),
            }
            .to_js(),
        )
        .expect("failed to post message to worker");

        cleanup
    }

    fn next_nonce(&self) -> u64 {
        let nonce = self.next_nonce.get();
        self.next_nonce.set(nonce + 1);
        nonce
    }
}

// ---------------------------------------------------------------------------
// OneshotScope (worker thread)
// ---------------------------------------------------------------------------

/// Worker-side counterpart to [`OneshotHandle`].
///
/// Calls the handler for each incoming message and posts the return value back.
pub struct OneshotScope {
    _scope: WorkerScope,
}

impl OneshotScope {
    /// Install a synchronous request→response handler.
    pub fn new(handler: impl Fn(JsValue) -> JsValue + 'static) -> Self {
        let scope = WorkerScope::new(move |scope, event| {
            match ToWorker::from_js(&event.data()).expect("request missing envelope") {
                ToWorker::Call { nonce, msg } => {
                    scope
                        .post_copy(
                            &FromWorker {
                                nonce,
                                msg: handler(msg),
                            }
                            .to_js(),
                        )
                        .expect("failed to post response");
                }
                ToWorker::Cancel { nonce } => {
                    log::warn!(
                        "[Oneshot] received cancellation request for nonce {nonce}, but worker is blocking."
                    );
                }
            }
        });

        Self { _scope: scope }
    }

    /// Install an async handler; concurrent requests are processed in
    /// parallel (completion order may differ from arrival order).
    pub fn new_async_unordered<F>(handler: impl Fn(JsValue) -> F + 'static) -> Self
    where
        F: Future<Output = JsValue> + 'static,
    {
        let cancellations = Rc::new(RefCell::new(HashMap::new()));

        let scope = WorkerScope::new_async_unordered(move |scope, event| {
            let event = ToWorker::from_js(&event.data()).expect("request missing envelope");

            let job = match event {
                ToWorker::Call { nonce, msg } => {
                    let cancellations = cancellations.clone();

                    // Start the task.
                    let task = handler(msg);

                    // Make it cancellable.
                    let (abort_handle, abort_registration) = AbortHandle::new_pair();
                    let task = futures::future::Abortable::new(task, abort_registration);
                    cancellations.borrow_mut().insert(nonce, abort_handle);

                    Some(async move {
                        // Wait for it to be done.
                        let response = task.await;

                        // Once done, clean up the cancellation handle.
                        let _abort_handle = cancellations.borrow_mut().remove(&nonce);

                        // Reply if the task was successful, or log if it was aborted.
                        let response = match response {
                            Ok(response) => response,
                            Err(Aborted) => {
                                log::debug!("[Oneshot] task for nonce {nonce} was aborted.");
                                return;
                            }
                        };

                        scope
                            .post_copy(
                                &FromWorker {
                                    nonce,
                                    msg: response,
                                }
                                .to_js(),
                            )
                            .expect("failed to post response");
                    })
                }
                ToWorker::Cancel { nonce } => {
                    // If the task is still running, abort it.
                    if let Some(abort_handle) = cancellations.borrow_mut().remove(&nonce) {
                        abort_handle.abort();
                    }

                    None
                }
            };
            async move {
                if let Some(future) = job {
                    future.await;
                }
            }
        });

        Self { _scope: scope }
    }

    /// Install an async handler; requests are processed strictly in arrival
    /// order (each must complete before the next starts).
    pub fn new_async_ordered<F>(handler: impl Fn(JsValue) -> F + 'static) -> Self
    where
        F: Future<Output = JsValue> + 'static,
    {
        // We do not use [WorkerScope::new_async_ordered] because we want to
        // allow cancellation requests to be processed immediately.

        struct OrderedJob {
            worker: WorkerScope,
            nonce: u64,
            msg: JsValue,
            abort_registration: AbortRegistration,
        }

        let cancellations = Rc::new(RefCell::new(HashMap::new()));

        let (tx, mut rx) = mpsc::unbounded::<OrderedJob>();
        let _task = Task::new_local({
            let cancellations = cancellations.clone();

            async move {
                while let Some(job) = rx.next().await {
                    let OrderedJob {
                        worker,
                        nonce,
                        msg,
                        abort_registration,
                    } = job;

                    if !cancellations.borrow_mut().contains_key(&nonce) {
                        // Already cancelled.
                        // Note: this prevents the initial sync part of the handler from executing.
                        log::debug!("[Oneshot] task for nonce {nonce} was aborted.");
                        continue;
                    }

                    // Start the task.
                    let task = handler(msg);

                    let task = futures::future::Abortable::new(task, abort_registration);
                    let response = task.await;

                    // Once done, clean up the cancellation handle.
                    let _abort_handle = cancellations.borrow_mut().remove(&nonce);

                    let response = match response {
                        Ok(response) => response,
                        Err(Aborted) => {
                            log::debug!("[Oneshot] task for nonce {nonce} was aborted.");
                            continue;
                        }
                    };

                    worker
                        .post_copy(
                            &FromWorker {
                                nonce,
                                msg: response,
                            }
                            .to_js(),
                        )
                        .expect("failed to post response");
                }
                log::error!("[Oneshot] Executing task has finished. This should not happen.");
            }
        });

        let scope = WorkerScope::new(move |worker, event: MessageEvent| {
            let _capture = &_task;

            let event = ToWorker::from_js(&event.data()).expect("request missing envelope");

            match event {
                ToWorker::Call { nonce, msg } => {
                    // If a new task, queue it along with the cancellation handle.

                    let (abort_handle, abort_registration) = AbortHandle::new_pair();
                    cancellations.borrow_mut().insert(nonce, abort_handle);

                    let _ = tx.unbounded_send(OrderedJob {
                        worker,
                        nonce,
                        msg,
                        abort_registration,
                    });
                }
                ToWorker::Cancel { nonce } => {
                    // If the task is still running, abort it.
                    if let Some(abort_handle) = cancellations.borrow_mut().remove(&nonce) {
                        abort_handle.abort();
                    }
                }
            }
        });

        Self { _scope: scope }
    }
}

// ---------------------------------------------------------------------------
// Envelope helpers
// ---------------------------------------------------------------------------

const KIND_KEY: &str = "kind";
const NONCE_KEY: &str = "nonce";
const MSG_KEY: &str = "msg";

impl ToWorker {
    fn to_js(&self) -> JsValue {
        let envelope = js_sys::Object::new();
        match self {
            ToWorker::Call { nonce, msg } => {
                set(&envelope, KIND_KEY, &JsValue::from_str("call"));
                set(&envelope, NONCE_KEY, &JsValue::from(*nonce as f64));
                set(&envelope, MSG_KEY, msg);
            }
            ToWorker::Cancel { nonce } => {
                set(&envelope, KIND_KEY, &JsValue::from_str("cancel"));
                set(&envelope, NONCE_KEY, &JsValue::from(*nonce as f64));
            }
        }
        envelope.into()
    }

    fn from_js(envelope: &JsValue) -> Option<Self> {
        let kind = get(envelope, KIND_KEY)?.as_string()?;
        let nonce = get(envelope, NONCE_KEY)?.as_f64()? as u64;
        match kind.as_str() {
            "call" => {
                let msg = get(envelope, MSG_KEY)?;
                Some(ToWorker::Call { nonce, msg })
            }
            "cancel" => Some(ToWorker::Cancel { nonce }),
            _ => None,
        }
    }
}

impl FromWorker {
    fn to_js(&self) -> JsValue {
        let envelope = js_sys::Object::new();
        set(&envelope, NONCE_KEY, &JsValue::from(self.nonce as f64));
        set(&envelope, MSG_KEY, &self.msg);
        envelope.into()
    }

    fn from_js(envelope: &JsValue) -> Option<Self> {
        let nonce = get(envelope, NONCE_KEY)?.as_f64()? as u64;
        let msg = get(envelope, MSG_KEY)?;
        Some(FromWorker { nonce, msg })
    }
}

fn set(envelope: &js_sys::Object, key: &str, value: &JsValue) {
    js_sys::Reflect::set(envelope, &JsValue::from_str(key), value)
        .unwrap_or_else(|_| panic!("failed to set {key} on envelope"));
}

fn get(envelope: &JsValue, key: &str) -> Option<JsValue> {
    js_sys::Reflect::get(envelope, &JsValue::from_str(key)).ok()
}
