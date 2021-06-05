use std::collections::BTreeMap;

use serde_cbor::Value;

use schema_analysis::InferredSchema;

mod shared;
use shared::FormatTests;

struct Cbor;

test_format!(Cbor);

impl FormatTests<Value> for Cbor {
    fn convert_to_inferred_schema(value: Value) -> InferredSchema {
        let vec_value = serde_cbor::to_vec(&value).unwrap();
        let processed_schema: InferredSchema = serde_cbor::from_slice(&vec_value).unwrap();
        processed_schema
    }

    fn null() -> Option<Value> {
        Some(Value::Null)
    }
    fn boolean() -> Option<Value> {
        Some(Value::Bool(true))
    }
    fn integer() -> Option<Value> {
        Some(Value::Integer(123))
    }
    fn float() -> Option<Value> {
        Some(Value::Float(123.123))
    }
    fn string() -> Option<Value> {
        Some(Value::Text("hello".into()))
    }

    fn empty_sequence() -> Option<Value> {
        Some(Value::Array(vec![]))
    }
    fn string_sequence() -> Option<Value> {
        Some(Value::Array(vec![
            Value::Text("one".into()),
            Value::Text("two".into()),
            Value::Text("three".into()),
        ]))
    }
    fn integer_sequence() -> Option<Value> {
        Some(Value::Array(vec![1.into(), 2.into(), 3.into()]))
    }
    fn mixed_sequence() -> Option<Value> {
        Some(Value::Array(vec![
            1.into(),
            Value::Text("two".into()),
            3.into(),
        ]))
    }
    fn optional_mixed_sequence() -> Option<Value> {
        Some(Value::Array(vec![
            1.into(),
            Value::Text("two".into()),
            3.into(),
            Value::Null,
        ]))
    }

    fn empty_map_struct() -> Option<Value> {
        Some(Value::Map(BTreeMap::new()))
    }
    fn map_struct_single() -> Option<Value> {
        let mut mapping = BTreeMap::new();
        mapping.insert(Value::Text("hello".into()), 1.into());
        Some(Value::Map(mapping))
    }
    fn map_struct_double() -> Option<Value> {
        let mut mapping = BTreeMap::new();
        mapping.insert(Value::Text("hello".into()), 1.into());
        mapping.insert(Value::Text("world".into()), Value::Text("!".into()));
        Some(Value::Map(mapping))
    }
    fn sequence_map_struct_mixed() -> Option<Value> {
        let mut mapping_1 = BTreeMap::new();
        mapping_1.insert(Value::Text("hello".into()), 1.into());
        mapping_1.insert(Value::Text("world".into()), Value::Text("!".into()));
        mapping_1.insert(Value::Text("mixed".into()), 1.1.into());

        let mut mapping_2 = BTreeMap::new();
        mapping_2.insert(Value::Text("hello".into()), 1.into());
        mapping_2.insert(Value::Text("world".into()), Value::Text("!".into()));
        mapping_2.insert(Value::Text("mixed".into()), Value::Text("1.1".into()));

        Some(Value::Array(vec![
            Value::Map(mapping_1),
            Value::Map(mapping_2),
        ]))
    }
    fn sequence_map_struct_optional_or_missing() -> Option<Value> {
        let mut mapping_1 = BTreeMap::new();
        mapping_1.insert(Value::Text("hello".into()), 1.into());
        mapping_1.insert(Value::Text("possibly_null".into()), Value::Text("!".into()));
        mapping_1.insert(Value::Text("possibly_missing".into()), 1.1.into());
        mapping_1.insert(Value::Text("null_or_missing".into()), Value::Null);

        let mut mapping_2 = BTreeMap::new();
        mapping_2.insert(Value::Text("hello".into()), 2.into());
        mapping_2.insert(Value::Text("possibly_null".into()), Value::Null);

        Some(Value::Array(vec![
            Value::Map(mapping_1),
            Value::Map(mapping_2),
        ]))
    }
    fn map_struct_mixed_sequence() -> Option<Value> {
        let mut mapping = BTreeMap::new();
        mapping.insert(Value::Text("hello".into()), 1.into());
        mapping.insert(Value::Text("world".into()), Value::Text("!".into()));
        mapping.insert(
            Value::Text("sequence".into()),
            Value::Array(vec![
                Value::Text("one".into()),
                Value::Text("two".into()),
                Value::Text("three".into()),
            ]),
        );
        Some(Value::Map(mapping))
    }
    fn map_struct_mixed_sequence_optional() -> Option<Value> {
        let mut mapping = BTreeMap::new();
        mapping.insert(Value::Text("hello".into()), 1.into());
        mapping.insert(Value::Text("world".into()), Value::Text("!".into()));
        mapping.insert(Value::Text("optional".into()), Value::Null);
        mapping.insert(
            Value::Text("sequence".into()),
            Value::Array(vec![
                Value::Text("one".into()),
                Value::Text("two".into()),
                Value::Text("three".into()),
                Value::Null,
            ]),
        );
        Some(Value::Map(mapping))
    }
}
