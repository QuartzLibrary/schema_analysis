use toml::{value::Table, Value};

use schema_analysis::InferredSchema;

mod shared;
use shared::FormatTests;

struct Toml;

test_format!(Toml);

impl FormatTests for Toml {
    type Value = Value;

    fn infer_schema(value: Self::Value) -> InferredSchema {
        let string_value: String = toml::to_string(&value).unwrap();
        let processed_schema: InferredSchema = toml::from_str(&string_value).unwrap();
        processed_schema
    }

    fn test_null() {}
    fn null() -> Self::Value {
        unreachable!() // Toml doesn't have null values
    }

    // Toml doesn't allow top-level primitives
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

    // Toml doesn't allow top-level arrays
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
        Value::Table(Table::new())
    }
    fn map_struct_single() -> Self::Value {
        let mut mapping = Table::new();
        mapping.insert("hello".into(), 1.into());
        Value::Table(mapping)
    }
    fn map_struct_double() -> Self::Value {
        let mut mapping = Table::new();
        mapping.insert("hello".into(), 1.into());
        mapping.insert("world".into(), "!".into());
        Value::Table(mapping)
    }
    fn test_sequence_map_struct_mixed() {}
    fn sequence_map_struct_mixed() -> Self::Value {
        unreachable!() // Toml doesn't allow top-level arrays
    }
    fn test_sequence_map_struct_optional_or_missing() {}
    fn sequence_map_struct_optional_or_missing() -> Self::Value {
        unreachable!() // Toml doesn't allow top-level arrays
    }
    fn map_struct_mixed_sequence() -> Self::Value {
        let mut mapping = Table::new();
        mapping.insert("hello".into(), 1.into());
        mapping.insert("world".into(), "!".into());
        mapping.insert(
            "sequence".into(),
            Value::Array(vec!["one".into(), "two".into(), "three".into()]),
        );
        Value::Table(mapping)
    }
    fn test_map_struct_mixed_sequence_optional() {}
    fn map_struct_mixed_sequence_optional() -> Self::Value {
        unreachable!() // Toml doesn't have null values
    }
}
