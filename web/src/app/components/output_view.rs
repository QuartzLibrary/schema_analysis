use leptos::prelude::*;
use leptos::{ev, html};
use schema_analysis::Schema;
use schema_analysis::targets::json_typegen::OutputMode;
use std::time::Duration;

use wasm_bindgen::prelude::*;

use schema_analysis_web::util::download_blob;

use crate::element_ext::HtmlElementExtExt;
use crate::web_sys_events_ext::GlobalEventTarget;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    Raw,
    JsonSchema,
    Rust,
    KotlinJackson,
    KotlinKotlinx,
    Typescript,
    TypescriptTypeAlias,
}

pub const OUTPUT_TYPES: &[OutputType] = &[
    OutputType::Raw,
    OutputType::JsonSchema,
    OutputType::Rust,
    OutputType::KotlinJackson,
    OutputType::KotlinKotlinx,
    OutputType::Typescript,
    OutputType::TypescriptTypeAlias,
];

impl OutputType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Raw => "Raw Schema",
            Self::JsonSchema => "JSON Schema",
            Self::Rust => "Rust",
            Self::KotlinJackson => "Kotlin Jackson",
            Self::KotlinKotlinx => "Kotlin Kotlinx",
            Self::Typescript => "TypeScript",
            Self::TypescriptTypeAlias => "TS Type Alias",
        }
    }

    fn language(&self) -> &'static str {
        match self {
            Self::Raw | Self::JsonSchema => "json",
            Self::Rust => "rust",
            Self::KotlinJackson | Self::KotlinKotlinx => "kotlin",
            Self::Typescript | Self::TypescriptTypeAlias => "typescript",
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            Self::Raw | Self::JsonSchema => "json",
            Self::Rust => "rs",
            Self::KotlinJackson | Self::KotlinKotlinx => "kt",
            Self::Typescript | Self::TypescriptTypeAlias => "ts",
        }
    }

    fn file_name(&self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::JsonSchema => "jsonschema",
            Self::Rust => "rust",
            Self::KotlinJackson => "kotlinjackson",
            Self::KotlinKotlinx => "kotlinkotlinx",
            Self::Typescript => "typescript",
            Self::TypescriptTypeAlias => "typescripttypealias",
        }
    }

    fn generate(&self, schema: &Schema) -> Option<String> {
        match self {
            Self::Raw => serde_json::to_string_pretty(schema).ok(),
            Self::JsonSchema => schema.to_json_schema_with_schemars().ok(),
            Self::Rust => schema.process_with_json_typegen(OutputMode::Rust).ok(),
            Self::KotlinJackson => schema
                .process_with_json_typegen(OutputMode::KotlinJackson)
                .ok(),
            Self::KotlinKotlinx => schema
                .process_with_json_typegen(OutputMode::KotlinKotlinx)
                .ok(),
            Self::Typescript => schema
                .process_with_json_typegen(OutputMode::Typescript)
                .ok(),
            Self::TypescriptTypeAlias => schema
                .process_with_json_typegen(OutputMode::TypescriptTypeAlias)
                .ok(),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = hljs)]
    fn highlight(code: &str, options: &JsValue) -> JsValue;
}

fn highlight_code(code: &str, language: &str) -> String {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"language".into(), &language.into()).unwrap();
    let result = highlight(code, &options);
    js_sys::Reflect::get(&result, &"value".into())
        .unwrap()
        .as_string()
        .unwrap_or_else(|| code.to_string())
}

pub fn output_view(schema: Schema, output_type: Signal<OutputType>) -> impl IntoView {
    let schema = StoredValue::new(schema);

    // Save on Ctrl+S (shared global listener, removed when the reactive owner is cleaned up).
    StoredValue::new_local(web_sys::window().unwrap().add_global_event_listener(
        "keydown",
        move |event: web_sys::KeyboardEvent| {
            if event.ctrl_key() && event.key() == "s" {
                event.prevent_default();
                let ot = output_type.get_untracked();
                if let Some(text) = ot.generate(&schema.get_value()) {
                    log::info!("saving {}.{}", ot.file_name(), ot.extension());
                    download_blob(&text, &format!("{}.{}", ot.file_name(), ot.extension()));
                }
            }
        },
    ));

    html::div().class("sa-output-view").child(move || {
        let ot = output_type.get();
        if let Some(text) = ot.generate(&schema.get_value()) {
            html::div()
                .child((code_block(text, ot, schema), credits(ot)))
                .into_any()
        } else {
            html::div()
                .class("sa-output-view__unavailable")
                .child("Output not available for this format")
                .into_any()
        }
    })
}

const ICON_COPY: &str = include_str!("../../../assets/icons/copy.svg");
const ICON_CHECK: &str = include_str!("../../../assets/icons/check.svg");
const ICON_DOWNLOAD: &str = include_str!("../../../assets/icons/download.svg");

fn code_block(text: String, output_type: OutputType, schema: StoredValue<Schema>) -> impl IntoView {
    let highlighted = highlight_code(&text, output_type.language());
    let text = StoredValue::new(text);
    let copied = RwSignal::new(false);

    html::div().class("sa-output-view__code-wrap").child((
        html::div().class("sa-output-view__code-actions").child((
            html::button()
                .class("sa-btn")
                .class("sa-output-view__action-btn")
                .class(("sa-output-view__action-btn--copied", copied))
                .attr(
                    "title",
                    move || if copied.get() { "Copied!" } else { "Copy" },
                )
                .attr("aria-label", move || {
                    if copied.get() {
                        "Copied!"
                    } else {
                        "Copy to clipboard"
                    }
                })
                .on_async_singleton(ev::click, move |_| {
                    let window = web_sys::window().unwrap();
                    let navigator = window.navigator();
                    let clipboard = navigator.clipboard();
                    let _ = clipboard.write_text(&text.get_value());
                    copied.set(true);
                    async move {
                        utile::time::sleep(Duration::from_secs(2)).await;
                        copied.set(false);
                    }
                })
                .child(move || {
                    if copied.get() {
                        html::span()
                            .attr("aria-hidden", "true")
                            .inner_html(ICON_CHECK)
                            .into_any()
                    } else {
                        html::span()
                            .attr("aria-hidden", "true")
                            .inner_html(ICON_COPY)
                            .into_any()
                    }
                }),
            html::button()
                .class("sa-btn")
                .class("sa-output-view__action-btn")
                .attr("title", "Save")
                .attr("aria-label", "Download file")
                .on(ev::click, move |_| {
                    if let Some(text) = output_type.generate(&schema.get_value()) {
                        log::info!(
                            "saving {}.{}",
                            output_type.file_name(),
                            output_type.extension()
                        );
                        download_blob(
                            &text,
                            &format!("{}.{}", output_type.file_name(), output_type.extension()),
                        );
                    }
                })
                .child(
                    html::span()
                        .attr("aria-hidden", "true")
                        .inner_html(ICON_DOWNLOAD),
                ),
        )),
        html::pre()
            .class("sa-output-view__code")
            .attr("role", "region")
            .attr("aria-label", format!("{} output", output_type.name()))
            .child(html::code().inner_html(highlighted)),
    ))
}

fn credits(output_type: OutputType) -> impl IntoView {
    match output_type {
        OutputType::Raw => html::div()
            .class("sa-output-view__credits")
            .child((
                "Powered by ",
                html::a()
                    .attr("href", "https://github.com/serde-rs/serde")
                    .child("Serde"),
                " and its integrations.",
            ))
            .into_any(),
        OutputType::JsonSchema => html::div()
            .class("sa-output-view__credits")
            .child((
                "Generated via ",
                html::a()
                    .attr("href", "https://github.com/GREsau/schemars")
                    .child("Schemars"),
                ".",
            ))
            .into_any(),
        _ => html::div()
            .class("sa-output-view__credits")
            .child((
                "Generated via ",
                html::a()
                    .attr("href", "https://github.com/evestera/json_typegen")
                    .child("json_typegen"),
                ".",
            ))
            .into_any(),
    }
}
