use serde_yaml::{Mapping, Value};

use schema_analysis::InferredSchema;

mod shared;
use shared::FormatTests;

struct Yaml;

test_format!(Yaml);

impl FormatTests<Value> for Yaml {
    fn convert_to_inferred_schema(value: Value) -> InferredSchema {
        let string_value: String = serde_yaml::to_string(&value).unwrap();
        let processed_schema: InferredSchema = serde_yaml::from_str(&string_value).unwrap();
        processed_schema
    }

    fn null() -> Option<Value> {
        Some(Value::Null)
    }
    fn boolean() -> Option<Value> {
        Some(Value::Bool(true))
    }
    fn integer() -> Option<Value> {
        Some(Value::Number(123.into()))
    }
    fn float() -> Option<Value> {
        Some(Value::Number(123.123.into()))
    }
    fn string() -> Option<Value> {
        Some(Value::String("hello".into()))
    }

    fn empty_sequence() -> Option<Value> {
        Some(Value::Sequence(vec![]))
    }
    fn string_sequence() -> Option<Value> {
        Some(Value::Sequence(vec![
            "one".into(),
            "two".into(),
            "three".into(),
        ]))
    }
    fn integer_sequence() -> Option<Value> {
        Some(Value::Sequence(vec![1.into(), 2.into(), 3.into()]))
    }
    fn mixed_sequence() -> Option<Value> {
        Some(Value::Sequence(vec![1.into(), "two".into(), 3.into()]))
    }
    fn optional_mixed_sequence() -> Option<Value> {
        Some(Value::Sequence(vec![
            1.into(),
            "two".into(),
            3.into(),
            Value::Null,
        ]))
    }

    fn empty_map_struct() -> Option<Value> {
        Some(Value::Mapping(Mapping::new()))
    }
    fn map_struct_single() -> Option<Value> {
        let mut mapping = Mapping::new();
        mapping.insert("hello".into(), 1.into());
        Some(Value::Mapping(mapping))
    }
    fn map_struct_double() -> Option<Value> {
        let mut mapping = Mapping::new();
        mapping.insert("hello".into(), 1.into());
        mapping.insert("world".into(), "!".into());
        Some(Value::Mapping(mapping))
    }
    fn sequence_map_struct_mixed() -> Option<Value> {
        let mut mapping_1 = Mapping::new();
        mapping_1.insert("hello".into(), 1.into());
        mapping_1.insert("world".into(), "!".into());
        mapping_1.insert("mixed".into(), 1.1.into());

        let mut mapping_2 = Mapping::new();
        mapping_2.insert("hello".into(), 1.into());
        mapping_2.insert("world".into(), "!".into());
        mapping_2.insert("mixed".into(), "1.1".into());

        Some(Value::Sequence(vec![
            Value::Mapping(mapping_1),
            Value::Mapping(mapping_2),
        ]))
    }
    fn sequence_map_struct_optional_or_missing() -> Option<Value> {
        let mut mapping_1 = Mapping::new();
        mapping_1.insert("hello".into(), 1.into());
        mapping_1.insert("possibly_null".into(), "!".into());
        mapping_1.insert("possibly_missing".into(), 1.1.into());
        mapping_1.insert("null_or_missing".into(), Value::Null);

        let mut mapping_2 = Mapping::new();
        mapping_2.insert("hello".into(), 2.into());
        mapping_2.insert("possibly_null".into(), Value::Null);

        Some(Value::Sequence(vec![
            Value::Mapping(mapping_1),
            Value::Mapping(mapping_2),
        ]))
    }
    fn map_struct_mixed_sequence() -> Option<Value> {
        let mut mapping = Mapping::new();
        mapping.insert("hello".into(), 1.into());
        mapping.insert("world".into(), "!".into());
        mapping.insert(
            "sequence".into(),
            Value::Sequence(vec!["one".into(), "two".into(), "three".into()]),
        );
        Some(Value::Mapping(mapping))
    }
    fn map_struct_mixed_sequence_optional() -> Option<Value> {
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
        Some(Value::Mapping(mapping))
    }
}
