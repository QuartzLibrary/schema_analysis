#![cfg(feature = "json_typegen")]

use json_typegen_shared::Shape;

use linked_hash_map::LinkedHashMap;
use schema_analysis::{InferredSchema, Schema};

mod shared;
use shared::FormatTests;

struct JsonTypegen;

test_format!(JsonTypegen);

impl FormatTests<Shape> for JsonTypegen {
    fn convert_to_inferred_schema(_shape: Shape) -> InferredSchema {
        // Not needed for testing the target.
        unreachable!()
    }

    // Note: here we are actually switching the source and target.
    // The target schema from the tests before is now being serialized to a json schema and then
    // parsed and compared to the json Shapes below.
    fn compare(target_shape: Shape, tested_schema: Schema) {
        let tested_schema_shape: Shape = tested_schema.to_json_typegen_shape();
        assert_eq!(tested_schema_shape, target_shape);
    }

    fn null() -> Option<Shape> {
        Some(Shape::Null)
    }
    fn boolean() -> Option<Shape> {
        Some(Shape::Bool)
    }
    fn integer() -> Option<Shape> {
        Some(Shape::Integer)
    }
    fn float() -> Option<Shape> {
        Some(Shape::Floating)
    }
    fn string() -> Option<Shape> {
        Some(Shape::StringT)
    }

    fn empty_sequence() -> Option<Shape> {
        Some(Shape::VecT {
            // `Null` represents optionality with no further information.
            // [Equivalent to `Optional(Bottom)`]
            elem_type: Box::new(Shape::Null),
        })
    }
    fn string_sequence() -> Option<Shape> {
        Some(Shape::VecT {
            elem_type: Box::new(Shape::StringT),
        })
    }
    fn integer_sequence() -> Option<Shape> {
        Some(Shape::VecT {
            elem_type: Box::new(Shape::Integer),
        })
    }
    fn mixed_sequence() -> Option<Shape> {
        Some(Shape::VecT {
            elem_type: Box::new(Shape::Any),
        })
    }
    fn optional_mixed_sequence() -> Option<Shape> {
        Some(Shape::VecT {
            elem_type: Box::new(Shape::Optional(Box::new(Shape::Any))),
        })
    }

    fn empty_map_struct() -> Option<Shape> {
        Some(Shape::Struct {
            fields: Default::default(),
        })
    }
    fn map_struct_single() -> Option<Shape> {
        // Note that the LinkedHashMap preserves order.
        let mut fields = LinkedHashMap::new();

        fields.insert("hello".to_string(), Shape::Integer);

        Some(Shape::Struct { fields })
    }
    fn map_struct_double() -> Option<Shape> {
        // Note that the LinkedHashMap preserves order.
        let mut fields = LinkedHashMap::new();

        fields.insert("hello".to_string(), Shape::Integer);
        fields.insert("world".to_string(), Shape::StringT);

        Some(Shape::Struct { fields })
    }
    fn sequence_map_struct_mixed() -> Option<Shape> {
        // Note that the LinkedHashMap preserves order.
        let mut fields = LinkedHashMap::new();

        fields.insert("hello".to_string(), Shape::Integer);
        fields.insert("mixed".to_string(), Shape::Any);
        fields.insert("world".to_string(), Shape::StringT);

        Some(Shape::VecT {
            elem_type: Box::new(Shape::Struct { fields }),
        })
    }
    fn sequence_map_struct_optional_or_missing() -> Option<Shape> {
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

        Some(Shape::VecT {
            elem_type: Box::new(Shape::Struct { fields }),
        })
    }
    fn map_struct_mixed_sequence() -> Option<Shape> {
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

        Some(Shape::Struct { fields })
    }
    fn map_struct_mixed_sequence_optional() -> Option<Shape> {
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

        Some(Shape::Struct { fields })
    }
}
