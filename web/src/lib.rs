pub mod util;

use std::collections::BTreeMap;

use schema_analysis::Schema;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Unique identifier for file entries.
pub type FileId = u64;

/// The supported data formats for input.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[repr(u8)]
pub enum Format {
    #[default]
    Json = 1,
    Yaml = 2,
    Cbor = 3,
    Toml = 4,
    Bson = 5,
    Xml = 6,
}

/// Messages sent from the app to the worker.
#[derive(Clone)]
pub struct ToWorker {
    pub format: Format,
    pub files: BTreeMap<FileId, web_sys::File>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromWorker(pub BTreeMap<FileId, Result<Schema, String>>);

// ---------------------------------------------------------------------------
// JS conversion
// ---------------------------------------------------------------------------

fn js_set(obj: &JsValue, key: &str, val: &JsValue) {
    js_sys::Reflect::set(obj, &JsValue::from_str(key), val).unwrap();
}

fn js_get(obj: &JsValue, key: &str) -> Option<JsValue> {
    let v = js_sys::Reflect::get(obj, &JsValue::from_str(key)).ok()?;
    if v.is_undefined() { None } else { Some(v) }
}

impl ToWorker {
    /// Serialize to JS.
    ///
    /// The `files` map is built manually so the `web_sys::File` handles are
    /// kept as live JS values (which structured-clone cheaply across the
    /// worker boundary) instead of being round-tripped through serde.
    pub fn into_js(self) -> JsValue {
        let outer = js_sys::Object::new();
        js_set(
            &outer,
            "format",
            &serde_wasm_bindgen::to_value(&self.format).unwrap(),
        );
        let arr = js_sys::Array::new();
        for (id, file) in self.files {
            let entry = js_sys::Object::new();
            js_set(&entry, "id", &JsValue::from(id as f64));
            js_set(&entry, "file", &file);
            arr.push(&entry);
        }
        js_set(&outer, "files", &arr);
        outer.into()
    }

    pub fn from_js(msg: JsValue) -> Self {
        let format: Format =
            serde_wasm_bindgen::from_value(js_get(&msg, "format").expect("missing format"))
                .expect("bad format");
        let arr: js_sys::Array = js_get(&msg, "files")
            .expect("missing files")
            .unchecked_into();
        let files = arr
            .iter()
            .map(|entry| {
                let id = js_get(&entry, "id")
                    .and_then(|v| v.as_f64())
                    .expect("missing id") as FileId;
                let file: web_sys::File = js_get(&entry, "file")
                    .expect("missing file")
                    .unchecked_into();
                (id, file)
            })
            .collect();
        Self { format, files }
    }
}

impl FromWorker {
    pub fn to_js(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self).unwrap()
    }

    pub fn from_js(msg: JsValue) -> Self {
        serde_wasm_bindgen::from_value(msg).unwrap()
    }
}

// ---------------------------------------------------------------------------
// Boilerplate
// ---------------------------------------------------------------------------

mod boilerplate {
    use std::{fmt, str::FromStr};

    use super::*;

    impl Format {
        pub const ALL: &[Format] = &[
            Format::Json,
            Format::Yaml,
            Format::Cbor,
            Format::Toml,
            Format::Bson,
            Format::Xml,
        ];

        pub fn name(&self) -> &'static str {
            match self {
                Self::Json => "json",
                Self::Yaml => "yaml",
                Self::Cbor => "cbor",
                Self::Toml => "toml",
                Self::Bson => "bson",
                Self::Xml => "xml",
            }
        }

        /// Infer format from a file extension (without the leading dot).
        pub fn from_extension(ext: &str) -> Option<Format> {
            match ext {
                "json" => Some(Format::Json),
                "yaml" | "yml" => Some(Format::Yaml),
                "cbor" => Some(Format::Cbor),
                "toml" => Some(Format::Toml),
                "bson" => Some(Format::Bson),
                "xml" => Some(Format::Xml),
                _ => None,
            }
        }

        /// Infer format from a filename by extracting its extension.
        pub fn from_filename(name: &str) -> Option<Format> {
            let ext = name.rsplit('.').next()?;
            Self::from_extension(ext)
        }
    }
    impl fmt::Display for Format {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name())
        }
    }
    impl FromStr for Format {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match Self::Json {
                Format::Json
                | Format::Yaml
                | Format::Cbor
                | Format::Toml
                | Format::Bson
                | Format::Xml => {} // Exhaustive match
            };

            match s {
                "json" => Ok(Format::Json),
                "yaml" => Ok(Format::Yaml),
                "cbor" => Ok(Format::Cbor),
                "toml" => Ok(Format::Toml),
                "bson" => Ok(Format::Bson),
                "xml" => Ok(Format::Xml),
                _ => Err(format!("Invalid data type: {}", s)),
            }
        }
    }
}
