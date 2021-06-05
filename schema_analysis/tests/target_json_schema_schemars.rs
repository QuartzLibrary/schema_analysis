#![cfg(feature = "schemars_integration")]

use serde_json::{json, Value};

use schema_analysis::{InferredSchema, Schema};

mod shared;
use shared::FormatTests;

struct JSchema;

test_format!(JSchema);

const SCHEMA_TYPE: &str = "https://json-schema.org/draft/2019-09/schema";

impl FormatTests<Value> for JSchema {
    fn convert_to_inferred_schema(_value: Value) -> InferredSchema {
        // Not needed for testing the target.
        unreachable!()
    }

    // Note: here we are actually switching the source and target.
    // The target schema from the tests before is now being serialized to a json schema and then
    // parsed and compared to the json values below.
    fn compare(target_json_schema: Value, tested_schema: Schema) {
        let serialized_json_schema = tested_schema.to_json_schema_with_schemars().unwrap();
        let deserialized_json_schema: Value =
            serde_json::from_str(&serialized_json_schema).unwrap();
        assert_eq!(deserialized_json_schema, target_json_schema);
    }

    fn null() -> Option<Value> {
        Some(json! ({
            "$schema": SCHEMA_TYPE,
            "type": "null",
        }))
    }

    fn boolean() -> Option<Value> {
        Some(json! ({
            "$schema": SCHEMA_TYPE,
            "type": "boolean",
        }))
    }

    fn integer() -> Option<Value> {
        Some(json! ({
            "$schema": SCHEMA_TYPE,
            "type": "integer",
        }))
    }

    fn float() -> Option<Value> {
        Some(json! ({
            "$schema": SCHEMA_TYPE,
            "type": "number",
        }))
    }

    fn string() -> Option<Value> {
        Some(json! ({
            "$schema": SCHEMA_TYPE,
            "type": "string",
        }))
    }

    fn empty_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "array",
            "items": true
        }))
    }

    fn string_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "array",
            "items": {
                "type": "string"
            }
        }))
    }

    fn integer_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "array",
            "items": {
                "type": "integer"
            }
        }))
    }

    fn mixed_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "array",
            "items": {
                "anyOf": [
                    // Note: order is important here because the representation is a vec
                    {
                        "type": "integer"
                    },
                    {
                        "type": "string"
                    },
                ]
            }
        }))
    }

    fn optional_mixed_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "array",
            "items": {
                "anyOf": [
                    // Note: order is important here because the representation is a vec
                    {
                        "anyOf": [
                            { "type": "integer" },
                            { "type": "string" },
                        ]
                    },
                    { "type": "null" }
                ]
            }
        }))
    }

    fn empty_map_struct() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "object",
        }))
    }

    fn map_struct_single() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "object",
            "properties": {
                "hello": { "type": "integer" }
            },
            "required": [ "hello" ]
        }))
    }

    fn map_struct_double() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "object",
            "properties": {
                "hello": { "type": "integer" },
                "world": { "type": "string" },
            },
            "required": [ "hello", "world" ]
        }))
    }

    fn sequence_map_struct_mixed() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "array",
            "items": {
                "type": "object",
                "required": [ "hello", "mixed", "world" ],
                "properties": {
                    "hello": { "type": "integer" },
                    "mixed": {
                        "anyOf": [
                            { "type": "number" },
                            { "type": "string" }
                        ]
                    },
                    "world": { "type": "string" },
                },
            }
        }))
    }

    fn sequence_map_struct_optional_or_missing() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "array",
            "items": {
                "type": "object",
                "required": [ "hello", "possibly_null" ],
                "properties": {
                    "hello": { "type": "integer" },
                    // We don't know what it is when it's not null, so it might be anything.
                    "null_or_missing": true,
                    "possibly_missing": { "type": "number" },
                    "possibly_null": { "type": ["string", "null"] }
                },
            }
        }))
    }

    fn map_struct_mixed_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "object",
            "required": [ "hello", "sequence", "world" ],
            "properties": {
                "hello": { "type": "integer" },
                "world": { "type": "string" },
                "sequence": {
                    "type": "array",
                    "items": { "type": "string" },
                },
            },
        }))
    }

    fn map_struct_mixed_sequence_optional() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "type": "object",
            "required": [ "hello", "optional", "sequence", "world" ],
            "properties": {
                "hello": { "type": "integer" },
                "optional": true,
                "world": { "type": "string" },
                "sequence": {
                    "type": "array",
                    "items": { "type": [ "string", "null" ] },
                },
            },
        }))
    }
}
