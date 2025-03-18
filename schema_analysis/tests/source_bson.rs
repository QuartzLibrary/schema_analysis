use bson::bson;

use schema_analysis::InferredSchema;

mod shared;
use shared::FormatTests;

struct Bson;

test_format!(Bson);

impl FormatTests for Bson {
    type Value = bson::Bson;

    fn infer_schema(value: Self::Value) -> InferredSchema {
        let bytes = bson::to_vec(&value).unwrap();
        let processed_schema: InferredSchema = bson::from_slice(&bytes).unwrap();
        processed_schema
    }

    // Bson doesn't allow top-level primitives
    fn test_null() {}
    fn null() -> Self::Value {
        unreachable!()
    }
    fn test_boolean() {}
    fn boolean() -> Self::Value {
        unreachable!()
    }
    fn test_integer() {}
    fn integer() -> Self::Value {
        unreachable!()
    }
    fn test_float() {}
    fn float() -> Self::Value {
        unreachable!()
    }
    fn test_string() {}
    fn string() -> Self::Value {
        unreachable!()
    }

    // Bson doesn't allow top-level arrays
    fn test_empty_sequence() {}
    fn empty_sequence() -> Self::Value {
        unreachable!()
    }
    fn test_string_sequence() {}
    fn string_sequence() -> Self::Value {
        unreachable!()
    }
    fn test_integer_sequence() {}
    fn integer_sequence() -> Self::Value {
        unreachable!()
    }
    fn test_mixed_sequence() {}
    fn mixed_sequence() -> Self::Value {
        unreachable!()
    }
    fn test_optional_mixed_sequence() {}
    fn optional_mixed_sequence() -> Self::Value {
        unreachable!()
    }

    fn empty_map_struct() -> Self::Value {
        bson!({})
    }
    fn map_struct_single() -> Self::Value {
        bson!({
            "hello": 1
        })
    }
    fn map_struct_double() -> Self::Value {
        bson!({
            "hello": 1,
            "world": "!"
        })
    }
    fn test_sequence_map_struct_mixed() {}
    fn sequence_map_struct_mixed() -> Self::Value {
        unreachable!() // Bson doesn't allow top-level arrays
    }
    fn test_sequence_map_struct_optional_or_missing() {}
    fn sequence_map_struct_optional_or_missing() -> Self::Value {
        unreachable!() // Bson doesn't allow top-level arrays
    }
    fn map_struct_mixed_sequence() -> Self::Value {
        bson!({
            "hello": 1,
            "world": "!",
            "sequence": ["one", "two", "three"]
        })
    }
    fn map_struct_mixed_sequence_optional() -> Self::Value {
        bson!({
            "hello": 1,
            "world": "!",
            "optional": null,
            "sequence": ["one", "two", "three", null]
        })
    }
}
