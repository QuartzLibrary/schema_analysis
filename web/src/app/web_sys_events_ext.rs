use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Weak};

use utile::drop::ExecuteOnDrop;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::EventTarget;

/// Adds an event listener to the target.
/// Returns a guard that removes the listener when dropped.
pub fn add_event_listener<E: JsCast + 'static>(
    target: &EventTarget,
    event: &str,
    listener: impl Fn(E) + 'static,
) -> ExecuteOnDrop {
    let closure = Closure::<dyn Fn(JsValue)>::new(move |ev: JsValue| {
        listener(ev.dyn_into::<E>().unwrap());
    });
    let js_fn: js_sys::Function = closure
        .as_ref()
        .dyn_ref::<js_sys::Function>()
        .unwrap()
        .clone();

    target
        .add_event_listener_with_callback(event, &js_fn)
        .unwrap();

    let target = target.clone();
    let event = event.to_string();
    ExecuteOnDrop::new_dyn(move || {
        let _ = target.remove_event_listener_with_callback(&event, &js_fn);
        drop(closure);
    })
}

thread_local! {
    static SHARED_LISTENERS: RefCell<SharedListeners> = RefCell::new(SharedListeners::default());
}

pub trait GlobalEventTarget {
    /// Adds a global listener for the given event.
    ///
    /// Multiple listeners on the same event share a single underlying JS
    /// event listener. The JS listener is created when the first
    /// callback is registered and removed when the last guard is dropped.
    ///
    /// Returns a guard that removes the listener when dropped.
    fn add_global_event_listener<E: JsCast + 'static>(
        &self,
        event: &'static str,
        listener: impl Fn(E) + 'static,
    ) -> ExecuteOnDrop;
}
impl GlobalEventTarget for web_sys::Window {
    fn add_global_event_listener<E: JsCast + 'static>(
        &self,
        event: &'static str,
        listener: impl Fn(E) + 'static,
    ) -> ExecuteOnDrop {
        add_shared_event_listener(SharedTarget::Window, event, listener)
    }
}
impl GlobalEventTarget for web_sys::Document {
    fn add_global_event_listener<E: JsCast + 'static>(
        &self,
        event: &'static str,
        listener: impl Fn(E) + 'static,
    ) -> ExecuteOnDrop {
        add_shared_event_listener(SharedTarget::Document, event, listener)
    }
}
impl GlobalEventTarget for web_sys::HtmlBodyElement {
    fn add_global_event_listener<E: JsCast + 'static>(
        &self,
        event: &'static str,
        listener: impl Fn(E) + 'static,
    ) -> ExecuteOnDrop {
        add_shared_event_listener(SharedTarget::Body, event, listener)
    }
}

#[derive(Default)]
struct SharedListeners {
    window: HashMap<&'static str, Weak<SharedInner>>,
    document: HashMap<&'static str, Weak<SharedInner>>,
    body: HashMap<&'static str, Weak<SharedInner>>,
}
struct SharedInner {
    target: EventTarget,
    event: &'static str,
    js_fn: js_sys::Function,
    _closure: Closure<dyn Fn(JsValue)>,
    callbacks: RefCell<HashMap<u64, Callback>>,
    next_id: Cell<u64>,
}
type Callback = Rc<dyn Fn(JsValue)>;
impl Drop for SharedInner {
    fn drop(&mut self) {
        let _ = self
            .target
            .remove_event_listener_with_callback(self.event, &self.js_fn);
    }
}

impl SharedListeners {
    fn map_for(&mut self, target: SharedTarget) -> &mut HashMap<&'static str, Weak<SharedInner>> {
        match target {
            SharedTarget::Window => &mut self.window,
            SharedTarget::Document => &mut self.document,
            SharedTarget::Body => &mut self.body,
        }
    }
}

#[derive(Clone, Copy)]
enum SharedTarget {
    Window,
    Document,
    Body,
}
impl SharedTarget {
    fn event_target(self) -> EventTarget {
        let window = web_sys::window().unwrap();
        match self {
            SharedTarget::Window => window.into(),
            SharedTarget::Document => window.document().unwrap().into(),
            SharedTarget::Body => window.document().unwrap().body().unwrap().into(),
        }
    }
}

/// Adds a shared event listener to a global target (window, document, or body).
///
/// Multiple listeners on the same `(target, event)` pair share a single
/// underlying JS event listener. The JS listener is created when the first
/// callback is registered and removed when the last guard is dropped.
fn add_shared_event_listener<E: JsCast + 'static>(
    target: SharedTarget,
    event: &'static str,
    listener: impl Fn(E) + 'static,
) -> ExecuteOnDrop<Box<dyn FnOnce()>> {
    SHARED_LISTENERS.with(|shared| {
        let mut shared = shared.borrow_mut();
        let map = shared.map_for(target);

        // Try to reuse an existing shared listener.
        let inner = map.get(event).and_then(|w| w.upgrade());

        let inner = match inner {
            Some(inner) => inner,
            None => {
                let event_target = target.event_target();

                let inner = Arc::new_cyclic(|weak: &Weak<SharedInner>| {
                    let weak = weak.clone();
                    let closure = Closure::<dyn Fn(JsValue)>::new(move |ev: JsValue| {
                        let Some(inner) = weak.upgrade() else { return };
                        // Clone Rc's so callbacks can modify the listener set re-entrantly.
                        let cbs: Vec<_> = inner.callbacks.borrow().values().cloned().collect();
                        for cb in &cbs {
                            cb(ev.clone());
                        }
                        drop(cbs);
                        // If we're the last Arc holder, defer the drop to avoid
                        // freeing this Closure from within its own invocation.
                        if Arc::strong_count(&inner) == 1 {
                            wasm_bindgen_futures::spawn_local(async move { drop(inner) });
                        }
                    });
                    let js_fn: js_sys::Function = closure
                        .as_ref()
                        .dyn_ref::<js_sys::Function>()
                        .unwrap()
                        .clone();

                    event_target
                        .add_event_listener_with_callback(event, &js_fn)
                        .unwrap();

                    SharedInner {
                        target: event_target,
                        event,
                        js_fn,
                        _closure: closure,
                        callbacks: RefCell::new(HashMap::new()),
                        next_id: Cell::new(0),
                    }
                });

                map.insert(event, Arc::downgrade(&inner));

                inner
            }
        };

        let id = inner.next_id.get();
        inner.next_id.set(id + 1);
        inner.callbacks.borrow_mut().insert(
            id,
            Rc::new(move |ev: JsValue| listener(ev.dyn_into::<E>().unwrap())),
        );

        ExecuteOnDrop::new_dyn(move || {
            inner.callbacks.borrow_mut().remove(&id);
        })
    })
}
