use bson::{bson, Bson as Value};

use schema_analysis::InferredSchema;

mod shared;
use shared::FormatTests;

struct Bson;

test_format!(Bson);

impl FormatTests<Value> for Bson {
    fn convert_to_inferred_schema(value: Value) -> InferredSchema {
        let document = bson::to_document(&value).unwrap();
        let mut raw_data = vec![];
        let () = document.to_writer(&mut raw_data).unwrap();
        let processed_schema: InferredSchema = rawbson::de::from_bytes(&raw_data).unwrap();
        processed_schema
    }

    // Bson doesn't allow top-level primitives
    fn null() -> Option<Value> {
        None
    }
    fn boolean() -> Option<Value> {
        None
    }
    fn integer() -> Option<Value> {
        None
    }
    fn float() -> Option<Value> {
        None
    }
    fn string() -> Option<Value> {
        None
    }

    // Bson doesn't allow top-level arrays
    fn empty_sequence() -> Option<Value> {
        None
    }
    fn string_sequence() -> Option<Value> {
        None
    }
    fn integer_sequence() -> Option<Value> {
        None
    }
    fn mixed_sequence() -> Option<Value> {
        None
    }
    fn optional_mixed_sequence() -> Option<Value> {
        None
    }

    fn empty_map_struct() -> Option<Value> {
        Some(bson!({}))
    }
    fn map_struct_single() -> Option<Value> {
        Some(bson!({
            "hello": 1
        }))
    }
    fn map_struct_double() -> Option<Value> {
        Some(bson!({
            "hello": 1,
            "world": "!"
        }))
    }
    fn sequence_map_struct_mixed() -> Option<Value> {
        None // Bson doesn't allow top-level arrays
    }
    fn test_sequence_map_struct_mixed() {}
    fn sequence_map_struct_optional_or_missing() -> Option<Value> {
        None // Bson doesn't allow top-level arrays
    }
    fn test_sequence_map_struct_optional_or_missing() {}
    fn map_struct_mixed_sequence() -> Option<Value> {
        Some(bson!({
            "hello": 1,
            "world": "!",
            "sequence": ["one", "two", "three"]
        }))
    }
    fn map_struct_mixed_sequence_optional() -> Option<Value> {
        Some(bson!({
            "hello": 1,
            "world": "!",
            "optional": null,
            "sequence": ["one", "two", "three", null]
        }))
    }
}
