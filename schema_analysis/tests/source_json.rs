use serde_json::{json, Value};

use schema_analysis::InferredSchema;

mod shared;
use shared::FormatTests;

struct Json;

test_format!(Json);

impl FormatTests<Value> for Json {
    fn convert_to_inferred_schema(value: Value) -> InferredSchema {
        let processed_schema: InferredSchema = serde_json::from_str(&value.to_string()).unwrap();
        processed_schema
    }

    fn null() -> Option<Value> {
        Some(json!(null))
    }
    fn boolean() -> Option<Value> {
        Some(json!(true))
    }
    fn integer() -> Option<Value> {
        Some(json!(123))
    }
    fn float() -> Option<Value> {
        Some(json!(123.123))
    }
    fn string() -> Option<Value> {
        Some(json!("hello there!"))
    }

    fn empty_sequence() -> Option<Value> {
        Some(json!([]))
    }
    fn string_sequence() -> Option<Value> {
        Some(json!(["one", "two", "three"]))
    }
    fn integer_sequence() -> Option<Value> {
        Some(json!([1, 2, 3]))
    }
    fn mixed_sequence() -> Option<Value> {
        Some(json!([1, "two", 3]))
    }
    fn optional_mixed_sequence() -> Option<Value> {
        Some(json!([1, "two", 3, null]))
    }

    fn empty_map_struct() -> Option<Value> {
        Some(json!({}))
    }
    fn map_struct_single() -> Option<Value> {
        Some(json!({
            "hello": 1
        }))
    }
    fn map_struct_double() -> Option<Value> {
        Some(json!({
            "hello": 1,
            "world": "!"
        }))
    }
    fn sequence_map_struct_mixed() -> Option<Value> {
        Some(json!([
            {
                "hello": 1,
                "world": "!",
                "mixed": 1.1,
            },
            {
                "hello": 1,
                "world": "!",
                "mixed": "1.1",
            }
        ]))
    }
    fn sequence_map_struct_optional_or_missing() -> Option<Value> {
        Some(json!([
            {
                "hello": 1,
                "possibly_null": "!",
                "possibly_missing": 1.1,
                "null_or_missing": null,
            },
            {
                "hello": 2,
                "possibly_null": null,
            }
        ]))
    }
    fn map_struct_mixed_sequence() -> Option<Value> {
        Some(json!({
            "hello": 1,
            "world": "!",
            "sequence": ["one", "two", "three"]
        }))
    }
    fn map_struct_mixed_sequence_optional() -> Option<Value> {
        Some(json!({
            "hello": 1,
            "world": "!",
            "optional": null,
            "sequence": ["one", "two", "three", null]
        }))
    }
}
