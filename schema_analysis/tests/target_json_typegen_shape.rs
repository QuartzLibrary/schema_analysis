#![cfg(feature = "json_typegen")]

use json_typegen_shared::Shape;

use linked_hash_map::LinkedHashMap;
use schema_analysis::{InferredSchema, Schema};

mod shared;
use shared::FormatTests;

struct JsonTypegen;

test_format!(JsonTypegen);

impl FormatTests for JsonTypegen {
    type Value = Shape;

    fn infer_schema(_value: Self::Value) -> InferredSchema {
        // Not needed for testing the target.
        unreachable!()
    }

    // Note: here we are actually switching the source and target.
    // The target schema from the tests before is now being serialized to a json schema and then
    // parsed and compared to the json Shapes below.
    fn compare(target_shape: Self::Value, tested_schema: Schema) {
        let tested_schema_shape: Shape = tested_schema.to_json_typegen_shape();
        assert_eq!(tested_schema_shape, target_shape);
    }

    fn null() -> Self::Value {
        Shape::Null
    }
    fn boolean() -> Self::Value {
        Shape::Bool
    }
    fn integer() -> Self::Value {
        Shape::Integer
    }
    fn float() -> Self::Value {
        Shape::Floating
    }
    fn string() -> Self::Value {
        Shape::StringT
    }

    fn empty_sequence() -> Self::Value {
        Shape::VecT {
            elem_type: Box::new(Shape::Bottom),
        }
    }
    fn string_sequence() -> Self::Value {
        Shape::VecT {
            elem_type: Box::new(Shape::StringT),
        }
    }
    fn integer_sequence() -> Self::Value {
        Shape::VecT {
            elem_type: Box::new(Shape::Integer),
        }
    }
    fn mixed_sequence() -> Self::Value {
        Shape::VecT {
            elem_type: Box::new(Shape::Any),
        }
    }
    fn optional_mixed_sequence() -> Self::Value {
        Shape::VecT {
            elem_type: Box::new(Shape::Optional(Box::new(Shape::Any))),
        }
    }

    fn empty_map_struct() -> Self::Value {
        Shape::Struct {
            fields: Default::default(),
        }
    }
    fn map_struct_single() -> Self::Value {
        // Note that the LinkedHashMap preserves order.
        let mut fields = LinkedHashMap::new();

        fields.insert("hello".to_string(), Shape::Integer);

        Shape::Struct { fields }
    }
    fn map_struct_double() -> Self::Value {
        // Note that the LinkedHashMap preserves order.
        let mut fields = LinkedHashMap::new();

        fields.insert("hello".to_string(), Shape::Integer);
        fields.insert("world".to_string(), Shape::StringT);

        Shape::Struct { fields }
    }
    fn sequence_map_struct_mixed() -> Self::Value {
        // Note that the LinkedHashMap preserves order.
        let mut fields = LinkedHashMap::new();

        fields.insert("hello".to_string(), Shape::Integer);
        fields.insert("mixed".to_string(), Shape::Any);
        fields.insert("world".to_string(), Shape::StringT);

        Shape::VecT {
            elem_type: Box::new(Shape::Struct { fields }),
        }
    }
    fn sequence_map_struct_optional_or_missing() -> Self::Value {
        // Note that the LinkedHashMap preserves order.
        let mut fields = LinkedHashMap::new();

        fields.insert("hello".to_string(), Shape::Integer);
        fields.insert("null_or_missing".to_string(), Shape::Null);
        fields.insert(
            "possibly_missing".to_string(),
            Shape::Optional(Box::new(Shape::Floating)),
        );
        fields.insert(
            "possibly_null".to_string(),
            Shape::Optional(Box::new(Shape::StringT)),
        );

        Shape::VecT {
            elem_type: Box::new(Shape::Struct { fields }),
        }
    }
    fn map_struct_mixed_sequence() -> Self::Value {
        // Note that the LinkedHashMap preserves order.
        let mut fields = LinkedHashMap::new();

        fields.insert("hello".to_string(), Shape::Integer);
        fields.insert(
            "sequence".to_string(),
            Shape::VecT {
                elem_type: Box::new(Shape::StringT),
            },
        );
        fields.insert("world".to_string(), Shape::StringT);

        Shape::Struct { fields }
    }
    fn map_struct_mixed_sequence_optional() -> Self::Value {
        // Note that the LinkedHashMap preserves order.
        let mut fields = LinkedHashMap::new();

        fields.insert("hello".to_string(), Shape::Integer);
        fields.insert("optional".to_string(), Shape::Null);
        fields.insert(
            "sequence".to_string(),
            Shape::VecT {
                elem_type: Box::new(Shape::Optional(Box::new(Shape::StringT))),
            },
        );
        fields.insert("world".to_string(), Shape::StringT);

        Shape::Struct { fields }
    }
}
