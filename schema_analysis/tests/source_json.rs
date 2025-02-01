use serde_json::{json, Value};

use schema_analysis::InferredSchema;

mod shared;
use shared::FormatTests;

struct Json;

test_format!(Json);

impl FormatTests for Json {
    type Value = Value;

    fn infer_schema(value: Self::Value) -> InferredSchema {
        let processed_schema: InferredSchema = serde_json::from_str(&value.to_string()).unwrap();
        processed_schema
    }

    fn null() -> Self::Value {
        json!(null)
    }
    fn boolean() -> Self::Value {
        json!(true)
    }
    fn integer() -> Self::Value {
        json!(123)
    }
    fn float() -> Self::Value {
        json!(123.123)
    }
    fn string() -> Self::Value {
        json!("hello there!")
    }

    fn empty_sequence() -> Self::Value {
        json!([])
    }
    fn string_sequence() -> Self::Value {
        json!(["one", "two", "three"])
    }
    fn integer_sequence() -> Self::Value {
        json!([1, 2, 3])
    }
    fn mixed_sequence() -> Self::Value {
        json!([1, "two", 3])
    }
    fn optional_mixed_sequence() -> Self::Value {
        json!([1, "two", 3, null])
    }

    fn empty_map_struct() -> Self::Value {
        json!({})
    }
    fn map_struct_single() -> Self::Value {
        json!({
            "hello": 1
        })
    }
    fn map_struct_double() -> Self::Value {
        json!({
            "hello": 1,
            "world": "!"
        })
    }
    fn sequence_map_struct_mixed() -> Self::Value {
        json!([
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
        ])
    }
    fn sequence_map_struct_optional_or_missing() -> Self::Value {
        json!([
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
        ])
    }
    fn map_struct_mixed_sequence() -> Self::Value {
        json!({
            "hello": 1,
            "world": "!",
            "sequence": ["one", "two", "three"]
        })
    }
    fn map_struct_mixed_sequence_optional() -> Self::Value {
        json!({
            "hello": 1,
            "world": "!",
            "optional": null,
            "sequence": ["one", "two", "three", null]
        })
    }
}
