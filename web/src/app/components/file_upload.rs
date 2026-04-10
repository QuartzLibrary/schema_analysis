use leptos::prelude::*;
use leptos::{ev, html};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use crate::AppState;

pub fn file_upload_button(state: AppState) -> impl IntoView {
    let input_ref = NodeRef::<html::Input>::new();

    let on_click = move |_| {
        if let Some(input) = input_ref.get() {
            input.click();
        }
    };

    let on_change = move |ev: web_sys::Event| {
        let target: HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(file_list) = target.files() {
            let count = file_list.length();
            log::info!("selected {} file(s) via upload", count);
            let mut files = Vec::with_capacity(count as usize);
            for i in 0..count {
                if let Some(file) = file_list.get(i) {
                    files.push(file);
                }
            }
            if !files.is_empty() {
                let state = state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    state.add_files(files).await;
                });
            }
        }
        target.set_value("");
    };

    html::div().class("sa-file-upload").child((
        html::input()
            .attr("type", "file")
            .attr("multiple", true)
            .attr("tabindex", "-1")
            .attr("aria-hidden", "true")
            .style("position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap")
            .node_ref(input_ref)
            .on(ev::change, on_change),
        html::button()
            .class("sa-btn")
            .on(ev::click, on_click)
            .child("Upload files"),
    ))
}
