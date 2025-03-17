use std::{error::Error, fmt, sync::Mutex};

use once_cell::sync::Lazy;
use wasm_bindgen::prelude::wasm_bindgen;

use schema_analysis::{targets::json_typegen::OutputMode, InferredSchema, Schema};

// For some reason Rust Analyzer (clippy) errors with missing-unsafe if log! (or similar)
// is not in a nested function. Adding an unsafe block results in a rustc lint (unused_unsafe).
// So for now I'm gonna leave everything in nested functions to avoid all that.
macro_rules! log { ( $( $t:tt )* ) => { web_sys::console::log_1(&format!( $( $t )* ).into()); } }

static INFERRED_SCHEMA: Lazy<Mutex<Option<InferredSchema>>> = Lazy::new(|| Mutex::new(None));

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    // Avoid 0 value that can lead to problems given Typescript's handling of enums and
    // true/false casting in Javascript.
    Json = 1,
    Yaml = 2,
    Cbor = 3,
    Toml = 4,
    Bson = 5,
    Xml = 6,
}
impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DataType::Json => "json",
            DataType::Yaml => "yaml",
            DataType::Cbor => "cbor",
            DataType::Toml => "toml",
            DataType::Bson => "bson",
            DataType::Xml => "xml",
        };
        f.write_str(s)
    }
}

#[wasm_bindgen]
pub fn infer(data: Vec<u8>, file_type: DataType) -> Result<(), wasm_bindgen::JsValue> {
    log_size_and_type(&data, &file_type);

    match file_type {
        DataType::Json => infer::from_json(&data).map_err(to_js_string)?,
        DataType::Yaml => infer::from_yaml(&data).map_err(to_js_string)?,
        DataType::Cbor => infer::from_cbor(&data).map_err(to_js_string)?,
        DataType::Toml => infer::from_toml(&data).map_err(to_js_string)?,
        DataType::Bson => infer::from_bson(&data).map_err(to_js_string)?,
        DataType::Xml => infer::from_xml(&data).map_err(to_js_string)?,
    };

    return Ok(());

    // See `log!` for why logging goes in nested functions.
    fn log_size_and_type(data: &[u8], file_type: &DataType) {
        log!(
            "[WASM] Received {} bytes of {}",
            data.len(),
            file_type.to_string()
        );
    }
    fn to_js_string(e: impl ToString) -> wasm_bindgen::JsValue {
        wasm_bindgen::JsValue::from_str(&e.to_string())
    }
}

mod infer {

    use serde::{de::DeserializeSeed, Deserializer};

    use schema_analysis::InferredSchema;

    pub fn from_json(v: &[u8]) -> Result<(), serde_json::Error> {
        process(
            v,
            serde_json::from_slice,
            &mut serde_json::Deserializer::from_slice(v),
        )
    }

    pub fn from_yaml(v: &[u8]) -> Result<(), serde_yaml::Error> {
        process(
            v,
            serde_yaml::from_slice,
            serde_yaml::Deserializer::from_slice(v),
        )
    }

    pub fn from_cbor(v: &[u8]) -> Result<(), serde_cbor::Error> {
        process(
            v,
            serde_cbor::from_slice,
            &mut serde_cbor::Deserializer::from_slice(v),
        )
    }

    pub fn from_toml(v: &[u8]) -> Result<(), toml::de::Error> {
        use serde::de::Error;
        let s = std::str::from_utf8(v).map_err(|e| toml::de::Error::custom(e.to_string()))?;
        process(v, toml::from_slice, &mut toml::Deserializer::new(s))
    }

    pub fn from_bson(v: &[u8]) -> Result<(), bson::de::Error> {
        // Here we do double work on the first deserialisation because the deserializer
        // from raw bytes is hidden in `bson::de::raw`.
        let bson: bson::Bson = bson::from_slice(v)?;
        process(v, bson::from_slice, bson::de::Deserializer::new(bson))
    }

    pub fn from_xml(v: &[u8]) -> Result<(), quick_xml::DeError> {
        process(
            v,
            quick_xml::de::from_reader,
            &mut quick_xml::de::Deserializer::from_reader(v),
        )
    }

    pub fn process<'de, D>(
        v: &'de [u8],
        from_slice: fn(&'de [u8]) -> Result<InferredSchema, D::Error>,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut schema = super::INFERRED_SCHEMA.lock().unwrap();

        start();
        match schema.take() {
            Some(mut inferred) => {
                inferred.deserialize(deserializer)?;
                schema.replace(inferred);
            }
            None => {
                let inferred: InferredSchema = from_slice(v)?;
                schema.replace(inferred);
            }
        };
        end();

        return Ok(());

        const TIMER_LABEL: &str = "[WASM] Inference";
        // See `log!` for why logging goes in nested functions.
        fn start() {
            web_sys::console::time_with_label(TIMER_LABEL);
        }
        fn end() {
            web_sys::console::time_end_with_label(TIMER_LABEL);
        }
    }
}

#[wasm_bindgen]
pub fn get_json_schema() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    get_target(Schema::to_json_schema_with_schemars)
}

#[wasm_bindgen]
pub fn get_rust_types() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    get_target(|s| Schema::process_with_json_typegen(s, OutputMode::Rust))
}

#[wasm_bindgen]
pub fn get_kotlin_jackson_types() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    get_target(|s| Schema::process_with_json_typegen(s, OutputMode::KotlinJackson))
}

#[wasm_bindgen]
pub fn get_kotlin_kotlinx_types() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    get_target(|s| Schema::process_with_json_typegen(s, OutputMode::KotlinKotlinx))
}

#[wasm_bindgen]
pub fn get_typescript_types() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    get_target(|s| Schema::process_with_json_typegen(s, OutputMode::Typescript))
}

#[wasm_bindgen]
pub fn get_typescript_type_alias_types() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    get_target(|s| Schema::process_with_json_typegen(s, OutputMode::TypescriptTypeAlias))
}

#[wasm_bindgen]
pub fn get_raw() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    get_target(serde_json::to_string_pretty::<Schema>)
}

fn get_target<E: Error>(
    func: fn(&Schema) -> Result<String, E>,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    let inferred_schema = INFERRED_SCHEMA.lock().unwrap();

    let result = match inferred_schema.as_ref() {
        Some(s) => func(&s.schema).map_err(|e| e.to_string()),
        None => Err("There is no inferred schema yet.".into()),
    };

    match result {
        Ok(s) => Ok(wasm_bindgen::JsValue::from_str(&s)),
        Err(s) => Err(wasm_bindgen::JsValue::from_str(&s)),
    }
}

#[wasm_bindgen]
pub fn clear_schema() {
    let _: Option<InferredSchema> = INFERRED_SCHEMA.lock().unwrap().take();

    // See `log!` for why logging goes in nested functions.
    fn log_inner() {
        log!("[WASM] Schema cleared");
    }
    log_inner();
}
