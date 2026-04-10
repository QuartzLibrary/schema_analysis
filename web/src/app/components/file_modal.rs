use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos::{ev, html};
use leptos_ext::signal::{Load, ReadSignalExt};
use schema_analysis::Schema;

use schema_analysis_web::FileId;
use schema_analysis_web::util::send_sync_future_handle;

use crate::components::file_list::format_size;
use crate::web_sys_events_ext::GlobalEventTarget;
use crate::{AppState, head};

const MAX_DISPLAY_BYTES: usize = 100_000; // 100 kB

pub fn file_modal(
    state: AppState,
    schemas: Signal<Load<BTreeMap<FileId, Result<Schema, String>>>>,
) -> impl IntoView {
    let files: RwSignal<_> = state.files.into();
    let selected_file: RwSignal<_> = state.selected_file.into();

    // Close on Escape (shared global listener, removed when the reactive owner is cleaned up).
    StoredValue::new_local(web_sys::window().unwrap().add_global_event_listener(
        "keydown",
        move |event: web_sys::KeyboardEvent| {
            if event.key() == "Escape" {
                selected_file.set(None);
            }
        },
    ));

    let modal_content = Signal::derive(move || {
        let selected_file = selected_file.get()?;
        let info = files.with(|files| files.get(&selected_file).cloned())?;
        Some(info)
    })
    .map_async(|info| {
        let info = info.clone();
        send_sync_future_handle(async move {
            let info = info?;
            let (mut head, truncated) = head(&info, MAX_DISPLAY_BYTES).await;
            if truncated {
                head.push_str("\n\n... (truncated, showing first 100 kB)");
            }
            Some(head)
        })
    })
    .map(|content| match content {
        Load::Loading => None,
        Load::Ready(content) => content.clone(),
    });

    let on_close = move |_: web_sys::MouseEvent| selected_file.set(None);

    move || {
        let id = selected_file.get()?;
        let files = files.get();
        let file = files.get(&id)?;

        let name = file.name();
        let size = format_size(file.size() as usize);
        let type_name = file.type_();

        let error = schemas.with(|s| match s {
            Load::Ready(map) => match map.get(&id) {
                Some(Err(msg)) => Some(msg.clone()),
                _ => None,
            },
            Load::Loading => None,
        });

        let header_text = format!("#{id}  {name}  {size}  {type_name}");

        let content = modal_content.get();

        Some(
            html::div()
                .class("sa-modal-backdrop")
                .on(ev::click, on_close)
                .child(
                    html::div()
                        .class("sa-modal")
                        .attr("role", "dialog")
                        .attr("aria-modal", "true")
                        .attr("aria-label", name.clone())
                        .on(ev::click, |e: web_sys::MouseEvent| e.stop_propagation())
                        .child((
                            html::div().class("sa-modal__header").child((
                                html::span().class("sa-modal__title").child(header_text),
                                html::button()
                                    .class("sa-modal__close")
                                    .attr("aria-label", "Close")
                                    .on(ev::click, on_close)
                                    .child("x"),
                            )),
                            error.map(|msg| html::pre().class("sa-modal__error").child(msg)),
                            match content {
                                Some(text) => html::pre()
                                    .class("sa-modal__content")
                                    .child(text)
                                    .into_any(),
                                None => html::div()
                                    .class("sa-modal__loading")
                                    .child("Loading...")
                                    .into_any(),
                            },
                        )),
                ),
        )
    }
}
