use toml::{value::Table, Value};

use schema_analysis::InferredSchema;

mod shared;
use shared::FormatTests;

struct Toml;

test_format!(Toml);

impl FormatTests<Value> for Toml {
    fn convert_to_inferred_schema(value: Value) -> InferredSchema {
        let string_value: String = toml::to_string(&value).unwrap();
        let processed_schema: InferredSchema = toml::from_str(&string_value).unwrap();
        processed_schema
    }

    // Toml doesn't have null values
    fn null() -> Option<Value> {
        None
    }
    // Toml doesn't allow top-level primitives
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

    // Toml doesn't allow top-level arrays
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
        Some(Value::Table(Table::new()))
    }
    fn map_struct_single() -> Option<Value> {
        let mut mapping = Table::new();
        mapping.insert("hello".into(), 1.into());
        Some(Value::Table(mapping))
    }
    fn map_struct_double() -> Option<Value> {
        let mut mapping = Table::new();
        mapping.insert("hello".into(), 1.into());
        mapping.insert("world".into(), "!".into());
        Some(Value::Table(mapping))
    }
    fn sequence_map_struct_mixed() -> Option<Value> {
        None // Toml doesn't allow top-level arrays
    }
    fn sequence_map_struct_optional_or_missing() -> Option<Value> {
        None // Toml doesn't allow top-level arrays
    }
    fn map_struct_mixed_sequence() -> Option<Value> {
        let mut mapping = Table::new();
        mapping.insert("hello".into(), 1.into());
        mapping.insert("world".into(), "!".into());
        mapping.insert(
            "sequence".into(),
            Value::Array(vec!["one".into(), "two".into(), "three".into()]),
        );
        Some(Value::Table(mapping))
    }
    fn map_struct_mixed_sequence_optional() -> Option<Value> {
        None // Toml doesn't have null values
    }
}
