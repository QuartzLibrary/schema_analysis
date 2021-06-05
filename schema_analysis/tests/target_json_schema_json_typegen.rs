#![cfg(feature = "json_typegen")]

use json_typegen_shared::OutputMode;
use serde_json::{json, Value};

use schema_analysis::{InferredSchema, Schema};

mod shared;
use shared::FormatTests;

struct JSchema;

test_format!(JSchema);

const SCHEMA_TYPE: &str = "http://json-schema.org/draft-07/schema#";
const SCHEMA_TITLE: &str = "Generated schema for Root";

impl FormatTests<Value> for JSchema {
    fn convert_to_inferred_schema(_value: Value) -> InferredSchema {
        // Not needed for testing the target.
        unreachable!()
    }

    // Note: here we are actually switching the source and target.
    // The target schema from the tests before is now being serialized to a json schema and then
    // parsed and compared to the json values below.
    fn compare(target_json_schema: Value, tested_schema: Schema) {
        let serialized_json_schema = tested_schema
            .process_with_json_typegen(OutputMode::JsonSchema)
            .unwrap();
        let deserialized_json_schema: Value =
            serde_json::from_str(&serialized_json_schema).unwrap();
        assert_eq!(
            deserialized_json_schema.to_string(),
            target_json_schema.to_string()
        );
    }

    fn null() -> Option<Value> {
        Some(json! ({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,

            // In json_typegen `Null` means that the value is null/missing and there are no
            //  further information, but it's assumed an actual schema actually does exist
            //  underneath, so we don't restrict the type to "null" here.
            // "type": "null",
        }))
    }
    fn boolean() -> Option<Value> {
        Some(json! ({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "boolean",
        }))
    }
    fn integer() -> Option<Value> {
        Some(json! ({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            // json_typegen always uses "number"
            "type": "number",
        }))
    }
    fn float() -> Option<Value> {
        Some(json! ({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "number",
        }))
    }
    fn string() -> Option<Value> {
        Some(json! ({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "string",
        }))
    }

    fn empty_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            "items": {}
        }))
    }
    fn string_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            "items": {
                "type": "string"
            }
        }))
    }
    fn integer_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            "items": {
                // json_typegen always uses "number"
                "type": "number"
            }
        }))
    }
    fn mixed_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            // json_typegen does not have a concept of union types.
            "items": {},
        }))
    }
    fn optional_mixed_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            // json_typegen does not have a concept of union types.
            "items": {},
        }))
    }

    fn empty_map_struct() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "object",

            // json_typegen always inserts these fields.
            "properties": {},
            "required": [],
        }))
    }
    fn map_struct_single() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "object",
            "properties": {
                // json_typegen always uses "number"
                "hello": { "type": "number" }
            },
            "required": [ "hello" ]
        }))
    }
    fn map_struct_double() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "object",
            "properties": {
                // json_typegen always uses "number"
                "hello": { "type": "number" },
                "world": { "type": "string" },
            },
            "required": [ "hello", "world" ]
        }))
    }
    fn sequence_map_struct_mixed() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    // json_typegen always uses "number"
                    "hello": { "type": "number" },
                    // json_typegen does not have a concept of union types.
                    "mixed": {},
                    "world": { "type": "string" },
                },
                "required": [ "hello", "mixed", "world" ],
            }
        }))
    }
    fn sequence_map_struct_optional_or_missing() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    // json_typegen always uses "number"
                    "hello": { "type": "number" },
                    // We don't know what it is when it's not null, so it might be anything.
                    "null_or_missing": {},
                    "possibly_missing": { "type": "number" },
                    // json_typegen considers being of type "null" and missing equivalent,
                    // so it's simply not required instead of required and both "string" and "null".
                    "possibly_null": { "type": "string" }
                },
                // FIXME: "null_or_missing" is included because json_typegen collapses
                // null/missing + no inference info into it, but checks only Shape::Optional
                // for the "required" field.
                "required": [ "hello", "null_or_missing" ],
            }
        }))
    }
    fn map_struct_mixed_sequence() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "object",
            "properties": {
                // json_typegen always uses "number"
                "hello": { "type": "number" },
                "sequence": {
                    "type": "array",
                    "items": { "type": "string" },
                },
                "world": { "type": "string" },
            },
            "required": [ "hello", "sequence", "world" ],
        }))
    }
    fn map_struct_mixed_sequence_optional() -> Option<Value> {
        Some(json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "object",
            "properties": {
                // json_typegen always uses "number"
                "hello": { "type": "number" },
                "optional": {},
                "sequence": {
                    "type": "array",

                    // FIXME:
                    // json_typegen seems to discard optional info when generating a
                    // json schema for a sequence type with optional values.
                    // "items": { "type": [ "string", "null" ] },
                    "items": { "type": "string" },
                },
                "world": { "type": "string" },
            },
            "required": [ "hello", "optional", "sequence", "world" ],
        }))
    }
}
