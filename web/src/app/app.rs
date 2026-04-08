use std::collections::BTreeMap;

use leptos::attr;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos_ext::signal::{Load, ReadSignalExt};
use schema_analysis::Schema;

use schema_analysis_web::FileId;
use schema_analysis_web::util::fetch_example;

use crate::components::file_list::file_list;
use crate::components::file_modal::file_modal;
use crate::components::file_upload::file_upload_button;
use crate::components::format_picker::format_picker;
use crate::components::output_picker::output_picker;
use crate::components::output_view::{OutputType, output_view};

use crate::AppState;
use crate::element_ext::HtmlElementExtExt;

pub fn app() -> impl IntoView {
    let state = AppState::new();

    let dragging = RwSignal::new(false);
    let output_type = RwSignal::new(OutputType::Raw);
    let paste_text = RwSignal::new(String::new());

    // (format, file_ids) →* schemas via worker inference (with worker-side caching).
    let schemas: Signal<Load<BTreeMap<FileId, Result<Schema, String>>>> = state.schemas();
    let display_schemas = schemas.skip_if(|s| matches!(s, Load::Loading));
    let is_loading = schemas.map(|s| !s.is_ready());

    html::div()
        .class("sa-app-wrap")
        .class(("sa-app--dragging", dragging))
        .on_drag_and_drop(dragging, {
            let state = state.clone();
            move |files| {
                log::info!("dropped {} file(s)", files.len());
                let state = state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    state.add_files(files).await;
                });
            }
        })
        .child((
            html::div()
                .class("sa-app__drop-overlay")
                .class(("sa-app__drop-overlay--visible", dragging))
                .child("Drop files to add"),
            html::a()
                .class("sa-skip-nav")
                .attr("href", "#main-content")
                .child("Skip to content"),
            html::main()
                .class("sa-app")
                .attr("id", "main-content")
                .child((
                    html::header().class("sa-app__header").child((
                        html::h1().child("Schema Analysis"),
                        html::p()
                            .class("sa-app__hint")
                            .child("Drop files anywhere, paste data, or upload to infer schemas."),
                    )),
                    format_picker(state.clone()),
                    html::div().class("sa-app__input").child((
                        html::textarea()
                            .class("sa-app__textarea")
                            .attr("rows", "4")
                            .attr("placeholder", "Paste data here...")
                            .attr("aria-label", "Paste data here")
                            .bind(attr::Value, paste_text),
                        html::div().class("sa-app__actions").child((
                            html::button()
                                .class("sa-btn")
                                .attr("disabled", move || paste_text.get().is_empty())
                                .on_async_singleton(ev::click, {
                                    let state = state.clone();
                                    move |_| {
                                        let content = paste_text.get_untracked();
                                        paste_text.set(String::new());
                                        let state = state.clone();
                                        async move {
                                            if !content.is_empty() {
                                                log::info!(
                                                    "adding pasted text ({} bytes)",
                                                    content.len()
                                                );
                                                state.add_pasted(content).await;
                                            }
                                        }
                                    }
                                })
                                .child("Add pasted text"),
                            file_upload_button(state.clone()),
                            html::button()
                                .class("sa-btn")
                                .on_async_singleton(ev::click, {
                                    let state = state.clone();
                                    move |_| {
                                        let state = state.clone();
                                        async move {
                                            match fetch_example().await {
                                                Ok(bytes) => {
                                                    log::info!(
                                                        "loaded example ({} bytes)",
                                                        bytes.len()
                                                    );
                                                    let uint8 = js_sys::Uint8Array::from(
                                                        bytes.as_slice(),
                                                    );
                                                    let parts = js_sys::Array::of1(&uint8);
                                                    let options =
                                                        web_sys::FilePropertyBag::new();
                                                    options.set_type("application/json");
                                                    let file = web_sys::File::new_with_u8_array_sequence_and_options(
                                                        &parts,
                                                        "example.json",
                                                        &options,
                                                    )
                                                    .unwrap();
                                                    state.add_files(vec![file]).await;
                                                }
                                                Err(e) => {
                                                    log::error!(
                                                        "failed to fetch example: {:?}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                    }
                                })
                                .child("Try example"),
                        )),
                    )),
                    {
                        let state = state.clone();
                        move || {
                            state
                                .files
                                .with(|f| !f.is_empty())
                                .then(|| file_list(state.clone(), display_schemas))
                        }
                    },
                    output_section(state.clone(), display_schemas, is_loading, output_type),
                    html::footer().class("sa-app__footer").child((
                        "Don't see the format you were looking for? It might be just a ",
                        html::a()
                            .attr("href", "https://github.com/QuartzLibrary/schema_analysis")
                            .child("PR"),
                        " away.",
                    )),
                )),
            file_modal(state.clone(), display_schemas),
        ))
}

fn output_section(
    state: AppState,
    display_schemas: Signal<Load<BTreeMap<FileId, Result<Schema, String>>>>,
    is_loading: Signal<bool>,
    output_type: RwSignal<OutputType>,
) -> impl IntoView {
    move || {
        if state.files.with(|f| f.is_empty()) {
            return ().into_any();
        }

        let Load::Ready(schemas) = display_schemas.get() else {
            return html::div()
                .class("sa-app__output")
                .child((
                    output_picker(output_type),
                    html::div().class("sa-app__loading").child("Analyzing..."),
                ))
                .into_any();
        };

        let errors: Vec<(FileId, String, String)> = state.files.with(|files| {
            schemas
                .iter()
                .filter_map(|(&id, result)| match result {
                    Err(msg) => {
                        let name = files
                            .get(&id)
                            .map(|f| f.name())
                            .unwrap_or_else(|| format!("{id}"));
                        Some((id, name, msg.clone()))
                    }
                    Ok(_) => None,
                })
                .collect()
        });

        let merged = merged_schema(&schemas);

        let content = match (merged, errors.is_empty()) {
            (Some(schema), true) => output_view(schema, output_type.into()).into_any(),
            (Some(schema), false) => html::div()
                .child((error_list(errors), output_view(schema, output_type.into())))
                .into_any(),
            (None, false) => error_list(errors).into_any(),
            (None, true) => html::div()
                .class("sa-app__empty")
                .child("No schemas available")
                .into_any(),
        };

        let loading = is_loading.get();

        html::div()
            .class("sa-app__output")
            .class(("sa-app__output--loading", loading))
            .child((output_picker(output_type), content))
            .into_any()
    }
}

fn merged_schema(schemas: &BTreeMap<FileId, Result<Schema, String>>) -> Option<Schema> {
    use schema_analysis::traits::Coalesce;

    let mut merged: Option<Schema> = None;
    for result in schemas.values() {
        let Ok(schema) = result else { continue };
        match &mut merged {
            None => merged = Some(schema.clone()),
            Some(m) => m.coalesce(schema.clone()),
        }
    }
    merged
}

fn error_list(errors: Vec<(FileId, String, String)>) -> impl IntoView {
    html::div().class("sa-errors").child(
        errors
            .into_iter()
            .map(|(id, name, msg)| {
                html::div().class("sa-errors__entry").child((
                    html::strong().child(format!("#{} {}: ", id, name)),
                    html::pre().class("sa-errors__detail").child(msg),
                ))
            })
            .collect::<Vec<_>>(),
    )
}
