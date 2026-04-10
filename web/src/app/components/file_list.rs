use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos::{ev, html};
use leptos_ext::signal::{Load, ReadSignalExt};
use schema_analysis::Schema;

use schema_analysis_web::FileId;

use crate::AppState;
use crate::element_ext::HtmlElementExtExt;

pub fn file_list(
    state: AppState,
    schemas: Signal<Load<BTreeMap<FileId, Result<Schema, String>>>>,
) -> impl IntoView {
    html::div().class("sa-files").child((
        html::div().class("sa-files__label").child("Files"),
        html::ul().class("sa-files__list").child(move || {
            state.files.with(|files| {
                files
                    .iter()
                    .map(|(&id, _)| {
                        file_entry(
                            state.clone(),
                            id,
                            schemas.map(move |s| match s {
                                Load::Loading => None,
                                Load::Ready(map) => map.get(&id).cloned(),
                            }),
                        )
                    })
                    .collect::<Vec<_>>()
            })
        }),
    ))
}

fn file_entry(
    state: AppState,
    file_id: FileId,
    schemas: Signal<Option<Result<Schema, String>>>,
) -> impl IntoView {
    let file = state.files.map(move |files| files.get(&file_id).cloned());

    let on_click = {
        let state = state.clone();
        move |_: web_sys::MouseEvent| {
            state.selected_file.set(Some(file_id));
        }
    };

    let on_remove = {
        let state = state.clone();
        move |ev: web_sys::MouseEvent| {
            ev.stop_propagation();
            let state = state.clone();
            async move {
                state.remove_file(file_id).await;
            }
        }
    };

    move || {
        file.get().map(|f| {
            let name = f.name();
            let size = format_size(f.size() as usize);
            html::li()
                .class("sa-files__entry")
                .attr("aria-current", {
                    let state = state.clone();
                    move || (state.selected_file.get() == Some(file_id)).then_some("true")
                })
                .child((
                    html::button()
                        .class("sa-files__entry-btn")
                        .on(ev::click, on_click.clone())
                        .child((
                            html::span().class("sa-files__bullet").child("•"),
                            html::span().class("sa-files__name").child(name.clone()),
                            html::span().class("sa-files__size").child(size),
                            move || status_indicator(schemas.get()),
                        )),
                    html::button()
                        .class("sa-files__remove")
                        .attr("aria-label", format!("Remove {}", name))
                        .on_async_leak(ev::click, on_remove.clone())
                        .child("x"),
                ))
        })
    }
}

fn status_indicator(schemas: Option<Result<Schema, String>>) -> impl IntoView {
    let (modifier, text) = match schemas {
        None => ("sa-files__status--loading", "..."),
        Some(Ok(_)) => ("sa-files__status--ready", "ok"),
        Some(Err(_)) => ("sa-files__status--error", "err"),
    };

    html::span()
        .class(format!("sa-files__status {modifier}"))
        .child(text)
}

pub(crate) fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
