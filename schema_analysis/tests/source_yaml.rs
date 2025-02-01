use serde_yaml::{Mapping, Value};

use schema_analysis::InferredSchema;

mod shared;
use shared::FormatTests;

struct Yaml;

test_format!(Yaml);

impl FormatTests for Yaml {
    type Value = Value;

    fn infer_schema(value: Self::Value) -> InferredSchema {
        let string_value: String = serde_yaml::to_string(&value).unwrap();
        let processed_schema: InferredSchema = serde_yaml::from_str(&string_value).unwrap();
        processed_schema
    }

    fn null() -> Self::Value {
        Value::Null
    }
    fn boolean() -> Self::Value {
        Value::Bool(true)
    }
    fn integer() -> Self::Value {
        Value::Number(123.into())
    }
    fn float() -> Self::Value {
        Value::Number(123.123.into())
    }
    fn string() -> Self::Value {
        Value::String("hello".into())
    }

    fn empty_sequence() -> Self::Value {
        Value::Sequence(vec![])
    }
    fn string_sequence() -> Self::Value {
        Value::Sequence(vec!["one".into(), "two".into(), "three".into()])
    }
    fn integer_sequence() -> Self::Value {
        Value::Sequence(vec![1.into(), 2.into(), 3.into()])
    }
    fn mixed_sequence() -> Self::Value {
        Value::Sequence(vec![1.into(), "two".into(), 3.into()])
    }
    fn optional_mixed_sequence() -> Self::Value {
        Value::Sequence(vec![1.into(), "two".into(), 3.into(), Value::Null])
    }

    fn empty_map_struct() -> Self::Value {
        Value::Mapping(Mapping::new())
    }
    fn map_struct_single() -> Self::Value {
        let mut mapping = Mapping::new();
        mapping.insert("hello".into(), 1.into());
        Value::Mapping(mapping)
    }
    fn map_struct_double() -> Self::Value {
        let mut mapping = Mapping::new();
        mapping.insert("hello".into(), 1.into());
        mapping.insert("world".into(), "!".into());
        Value::Mapping(mapping)
    }
    fn sequence_map_struct_mixed() -> Self::Value {
        let mut mapping_1 = Mapping::new();
        mapping_1.insert("hello".into(), 1.into());
        mapping_1.insert("world".into(), "!".into());
        mapping_1.insert("mixed".into(), 1.1.into());

        let mut mapping_2 = Mapping::new();
        mapping_2.insert("hello".into(), 1.into());
        mapping_2.insert("world".into(), "!".into());
        mapping_2.insert("mixed".into(), "1.1".into());

        Value::Sequence(vec![Value::Mapping(mapping_1), Value::Mapping(mapping_2)])
    }
    fn sequence_map_struct_optional_or_missing() -> Self::Value {
        let mut mapping_1 = Mapping::new();
        mapping_1.insert("hello".into(), 1.into());
        mapping_1.insert("possibly_null".into(), "!".into());
        mapping_1.insert("possibly_missing".into(), 1.1.into());
        mapping_1.insert("null_or_missing".into(), Value::Null);

        let mut mapping_2 = Mapping::new();
        mapping_2.insert("hello".into(), 2.into());
        mapping_2.insert("possibly_null".into(), Value::Null);

        Value::Sequence(vec![Value::Mapping(mapping_1), Value::Mapping(mapping_2)])
    }
    fn map_struct_mixed_sequence() -> Self::Value {
        let mut mapping = Mapping::new();
        mapping.insert("hello".into(), 1.into());
        mapping.insert("world".into(), "!".into());
        mapping.insert(
            "sequence".into(),
            Value::Sequence(vec!["one".into(), "two".into(), "three".into()]),
        );
        Value::Mapping(mapping)
    }
    fn map_struct_mixed_sequence_optional() -> Self::Value {
        let mut mapping = Mapping::new();
        mapping.insert("hello".into(), 1.into());
        mapping.insert("world".into(), "!".into());
        mapping.insert("optional".into(), Value::Null);
        mapping.insert(
            "sequence".into(),
            Value::Sequence(vec![
                "one".into(),
                "two".into(),
                "three".into(),
                Value::Null,
            ]),
        );
        Value::Mapping(mapping)
    }
}
