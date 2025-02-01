#![cfg(feature = "json_typegen")]

use json_typegen_shared::OutputMode;
use serde_json::{json, Value};

use schema_analysis::{InferredSchema, Schema};

mod shared;
use shared::FormatTests;

const INTEGER: &str = "number"; // json_typegen always uses "number"

struct JSchema;

test_format!(JSchema);

const SCHEMA_TYPE: &str = "http://json-schema.org/draft-07/schema#";
const SCHEMA_TITLE: &str = "Generated schema for Root";

impl FormatTests for JSchema {
    type Value = Value;

    fn infer_schema(_value: Self::Value) -> InferredSchema {
        // Not needed for testing the target.
        unreachable!()
    }

    // Note: here we are actually switching the source and target.
    // The target schema from the tests before is now being serialized to a json schema and then
    // parsed and compared to the json values below.
    fn compare(target_json_schema: Self::Value, tested_schema: Schema) {
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

    fn null() -> Self::Value {
        json! ({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,

            // In json_typegen `Null` means that the value is null/missing and there are no
            //  further information, but it's assumed an actual schema actually does exist
            //  underneath, so we don't restrict the type to "null" here.
            // "type": "null",
        })
    }
    fn boolean() -> Self::Value {
        json! ({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "boolean",
        })
    }
    fn integer() -> Self::Value {
        json! ({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": INTEGER,
        })
    }
    fn float() -> Self::Value {
        json! ({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "number",
        })
    }
    fn string() -> Self::Value {
        json! ({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "string",
        })
    }

    fn empty_sequence() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            "items": {}
        })
    }
    fn string_sequence() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            "items": {
                "type": "string"
            }
        })
    }
    fn integer_sequence() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            "items": {
                "type": INTEGER
            }
        })
    }
    fn mixed_sequence() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            // json_typegen does not have a concept of union types.
            "items": {},
        })
    }
    fn optional_mixed_sequence() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            // json_typegen does not have a concept of union types.
            "items": {},
        })
    }

    fn empty_map_struct() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "object",

            // json_typegen always inserts these fields.
            "properties": {},
            "required": [],
        })
    }
    fn map_struct_single() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "object",
            "properties": {
                "hello": { "type": INTEGER }
            },
            "required": [ "hello" ]
        })
    }
    fn map_struct_double() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "object",
            "properties": {
                "hello": { "type": INTEGER },
                "world": { "type": "string" },
            },
            "required": [ "hello", "world" ]
        })
    }
    fn sequence_map_struct_mixed() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "hello": { "type": INTEGER },
                    "world": { "type": "string" },
                    // json_typegen does not have a concept of union types.
                    "mixed": {},
                },
                "required": [ "hello", "world", "mixed" ],
            }
        })
    }
    fn sequence_map_struct_optional_or_missing() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "hello": { "type": INTEGER },
                    // json_typegen considers being of type "null" and missing equivalent,
                    // so it's simply not required instead of required and both "string" and "null".
                    "possibly_null": { "type": "string" },
                    "possibly_missing": { "type": "number" },
                    // We don't know what it is when it's not null, so it might be anything.
                    "null_or_missing": {},
                },
                // FIXME: "null_or_missing" is included because json_typegen collapses
                // null/missing + no inference info into it, but checks only Shape::Optional
                // for the "required" field.
                "required": [ "hello", "null_or_missing" ],
            }
        })
    }
    fn map_struct_mixed_sequence() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "object",
            "properties": {
                "hello": { "type": INTEGER },
                "world": { "type": "string" },
                "sequence": {
                    "type": "array",
                    "items": { "type": "string" },
                },
            },
            "required": [ "hello", "world", "sequence" ],
        })
    }
    fn map_struct_mixed_sequence_optional() -> Self::Value {
        json!({
            "$schema": SCHEMA_TYPE,
            "title": SCHEMA_TITLE,
            "type": "object",
            "properties": {
                "hello": { "type": INTEGER },
                "world": { "type": "string" },
                "optional": {},
                "sequence": {
                    "type": "array",

                    // FIXME:
                    // json_typegen seems to discard optional info when generating a
                    // json schema for a sequence type with optional values.
                    // "items": { "type": [ "string", "null" ] },
                    "items": { "type": "string" },
                },
            },
            "required": [ "hello", "world", "optional", "sequence" ],
        })
    }
}
