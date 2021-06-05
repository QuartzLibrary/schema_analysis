use std::collections::BTreeMap;

use maplit::btreemap;

use schema_analysis::{Field, FieldStatus, InferredSchema, Schema, StructuralEq};

/// This provides a way for formats to quickly implement some basic tests.
///
/// Each format should implement the 'compare' function which normally will first serialize
/// a Rust representation of a value that should yield the required schema, then use the crate to
/// run the analysis on it and compare to a provided schema.
/// For each test the format might return `None` to skip it.
pub trait FormatTests<T> {
    fn convert_to_inferred_schema(value: T) -> InferredSchema;
    fn compare(value: T, target_schema: Schema) {
        let InferredSchema { schema } = Self::convert_to_inferred_schema(value);
        let success = schema.structural_eq(&target_schema);
        if !success {
            println!("INFERRED: {:#?}\n", schema);
            println!("TARGET  : {:#?}\n", target_schema);
        }
        assert!(success);
    }
    fn _compare_option(value: Option<T>, target_schema: Schema) {
        if let Some(inner) = value {
            Self::compare(inner, target_schema)
        }
    }
    fn _compare_sequence(value: Option<T>, field: Field) {
        Self::_compare_option(
            value,
            Schema::Sequence {
                field: Box::new(field),
                context: Default::default(),
            },
        )
    }
    fn _compare_map_struct(value: Option<T>, fields: BTreeMap<String, Field>) {
        Self::_compare_option(
            value,
            Schema::Struct {
                fields,
                context: Default::default(),
            },
        )
    }

    fn null() -> Option<T>;
    fn test_null() {
        Self::_compare_option(Self::null(), Schema::Null(Default::default()))
    }
    fn boolean() -> Option<T>;
    fn test_boolean() {
        Self::_compare_option(Self::boolean(), Schema::Boolean(Default::default()))
    }
    fn integer() -> Option<T>;
    fn test_integer() {
        Self::_compare_option(Self::integer(), Schema::Integer(Default::default()))
    }
    fn float() -> Option<T>;
    fn test_float() {
        Self::_compare_option(Self::float(), Schema::Float(Default::default()))
    }
    fn string() -> Option<T>;
    fn test_string() {
        Self::_compare_option(Self::string(), Schema::String(Default::default()))
    }

    fn empty_sequence() -> Option<T>;
    fn test_empty_sequence() {
        let mut field = Field::default();
        field.status.may_be_missing = true;
        Self::_compare_sequence(Self::empty_sequence(), field)
    }
    fn string_sequence() -> Option<T>;
    fn test_string_sequence() {
        let mut field = Field {
            status: FieldStatus::default(),
            schema: Some(Schema::String(Default::default())),
        };
        field.status.may_be_normal = true;
        Self::_compare_sequence(Self::string_sequence(), field);
    }
    fn integer_sequence() -> Option<T>;
    fn test_integer_sequence() {
        let mut field = Field {
            status: FieldStatus::default(),
            schema: Some(Schema::Integer(Default::default())),
        };
        field.status.may_be_normal = true;
        Self::_compare_sequence(Self::integer_sequence(), field);
    }
    fn mixed_sequence() -> Option<T>;
    fn test_mixed_sequence() {
        let mut field = Field {
            status: FieldStatus::default(),
            schema: Some(Schema::Union {
                variants: vec![
                    Schema::Integer(Default::default()),
                    Schema::String(Default::default()),
                ],
            }),
        };
        field.status.may_be_normal = true;
        Self::_compare_sequence(Self::mixed_sequence(), field);
    }
    fn optional_mixed_sequence() -> Option<T>;
    fn test_optional_mixed_sequence() {
        let mut field = Field {
            status: FieldStatus::default(),
            schema: Some(Schema::Union {
                variants: vec![
                    Schema::Integer(Default::default()),
                    Schema::String(Default::default()),
                ],
            }),
        };
        field.status.may_be_normal = true;
        field.status.may_be_null = true;
        Self::_compare_sequence(Self::optional_mixed_sequence(), field);
    }

    fn empty_map_struct() -> Option<T>;
    fn test_empty_map_struct() {
        let field_schemas: BTreeMap<String, Field> = BTreeMap::new();
        Self::_compare_map_struct(Self::empty_map_struct(), field_schemas);
    }
    fn map_struct_single() -> Option<T>;
    fn test_map_struct_single() {
        let fields = {
            let mut hello_field = Field {
                status: FieldStatus::default(),
                schema: Some(Schema::Integer(Default::default())),
            };
            hello_field.status.may_be_normal = true;
            btreemap! {
                "hello".into() => hello_field
            }
        };
        Self::_compare_map_struct(Self::map_struct_single(), fields);
    }
    fn map_struct_double() -> Option<T>;
    fn test_map_struct_double() {
        let fields = {
            let mut hello_field = Field {
                status: FieldStatus::default(),
                schema: Some(Schema::Integer(Default::default())),
            };
            hello_field.status.may_be_normal = true;
            let mut world_field = Field {
                status: FieldStatus::default(),
                schema: Some(Schema::String(Default::default())),
            };
            world_field.status.may_be_normal = true;
            btreemap! {
                "hello".into() => hello_field,
                "world".into() => world_field,
            }
        };
        Self::_compare_map_struct(Self::map_struct_double(), fields);
    }
    fn sequence_map_struct_mixed() -> Option<T>;
    fn test_sequence_map_struct_mixed() {
        let inner_fields = {
            let mut hello_field = Field::with_schema(Schema::Integer(Default::default()));
            hello_field.status.may_be_normal = true;

            let mut world_field = Field::with_schema(Schema::String(Default::default()));
            world_field.status.may_be_normal = true;

            let mut mixed_field = Field::with_schema(Schema::Union {
                variants: vec![
                    Schema::Float(Default::default()),
                    Schema::String(Default::default()),
                ],
            });
            mixed_field.status.may_be_normal = true;

            btreemap! {
                "hello".into() => hello_field,
                "world".into() => world_field,
                "mixed".into() => mixed_field,
            }
        };

        let mut element_field = Field::with_schema(Schema::Struct {
            fields: inner_fields,
            context: Default::default(),
        });
        element_field.status.may_be_normal = true;

        Self::_compare_sequence(Self::sequence_map_struct_mixed(), element_field);
    }
    fn sequence_map_struct_optional_or_missing() -> Option<T>;
    fn test_sequence_map_struct_optional_or_missing() {
        let inner_fields = {
            let mut hello_field = Field::with_schema(Schema::Integer(Default::default()));
            hello_field.status.may_be_normal = true;

            let mut possibly_null_field = Field::with_schema(Schema::String(Default::default()));
            possibly_null_field.status.may_be_normal = true;
            possibly_null_field.status.may_be_null = true;

            let mut possibly_missing_field = Field::with_schema(Schema::Float(Default::default()));
            possibly_missing_field.status.may_be_normal = true;
            possibly_missing_field.status.may_be_missing = true;

            let mut null_or_missing_field = Field::default();
            null_or_missing_field.status.may_be_null = true;
            null_or_missing_field.status.may_be_missing = true;

            btreemap! {
                "hello".into() => hello_field,
                "possibly_null".into() => possibly_null_field,
                "possibly_missing".into() => possibly_missing_field,
                "null_or_missing".into() => null_or_missing_field,
            }
        };

        let mut element_field = Field::with_schema(Schema::Struct {
            fields: inner_fields,
            context: Default::default(),
        });
        element_field.status.may_be_normal = true;

        Self::_compare_sequence(
            Self::sequence_map_struct_optional_or_missing(),
            element_field,
        );
    }
    fn map_struct_mixed_sequence() -> Option<T>;
    fn test_map_struct_mixed_sequence() {
        let fields = {
            let mut hello_field = Field::with_schema(Schema::Integer(Default::default()));
            hello_field.status.may_be_normal = true;

            let mut world_field = Field::with_schema(Schema::String(Default::default()));
            world_field.status.may_be_normal = true;

            let mut sequence_field = {
                let mut sequence_element_field =
                    Field::with_schema(Schema::String(Default::default()));
                sequence_element_field.status.may_be_normal = true;

                Field::with_schema(Schema::Sequence {
                    field: Box::new(sequence_element_field),
                    context: Default::default(),
                })
            };
            sequence_field.status.may_be_normal = true;

            btreemap! {
                "hello".into() => hello_field,
                "world".into() => world_field,
                "sequence".into() => sequence_field,
            }
        };
        Self::_compare_map_struct(Self::map_struct_mixed_sequence(), fields);
    }
    fn map_struct_mixed_sequence_optional() -> Option<T>;
    fn test_map_struct_mixed_sequence_optional() {
        let fields = {
            let mut hello_field = Field::with_schema(Schema::Integer(Default::default()));
            hello_field.status.may_be_normal = true;

            let mut world_field = Field::with_schema(Schema::String(Default::default()));
            world_field.status.may_be_normal = true;

            let mut optional_field = Field::default();
            optional_field.status.may_be_null = true;

            let mut sequence_field = {
                let mut sequence_element_field =
                    Field::with_schema(Schema::String(Default::default()));
                sequence_element_field.status.may_be_normal = true;
                sequence_element_field.status.may_be_null = true;

                Field::with_schema(Schema::Sequence {
                    field: Box::new(sequence_element_field),
                    context: Default::default(),
                })
            };
            sequence_field.status.may_be_normal = true;

            btreemap! {
                "hello".into() => hello_field,
                "world".into() => world_field,
                "optional".into() => optional_field,
                "sequence".into() => sequence_field,
            }
        };
        Self::_compare_map_struct(Self::map_struct_mixed_sequence_optional(), fields);
    }
}

#[macro_export]
macro_rules! test_format {
    ($F:ty) => {
        #[test]
        fn null() {
            <$F>::test_null();
        }
        #[test]
        fn boolean() {
            <$F>::test_boolean();
        }
        #[test]
        fn integer() {
            <$F>::test_integer();
        }
        #[test]
        fn float() {
            <$F>::test_float();
        }
        #[test]
        fn string() {
            <$F>::test_string();
        }

        #[test]
        fn empty_sequence() {
            <$F>::test_empty_sequence();
        }
        #[test]
        fn string_sequence() {
            <$F>::test_string_sequence();
        }
        #[test]
        fn integer_sequence() {
            <$F>::test_integer_sequence();
        }
        #[test]
        fn mixed_sequence() {
            <$F>::test_mixed_sequence();
        }
        #[test]
        fn optional_mixed_sequence() {
            <$F>::test_optional_mixed_sequence();
        }

        #[test]
        fn empty_map_struct() {
            <$F>::test_empty_map_struct();
        }
        #[test]
        fn map_struct_single() {
            <$F>::test_map_struct_single();
        }
        #[test]
        fn map_struct_double() {
            <$F>::test_map_struct_double();
        }
        #[test]
        fn sequence_map_struct_mixed() {
            <$F>::test_sequence_map_struct_mixed();
        }
        #[test]
        fn sequence_map_struct_optional_or_missing() {
            <$F>::test_sequence_map_struct_optional_or_missing();
        }
        #[test]
        fn map_struct_mixed_sequence() {
            <$F>::test_map_struct_mixed_sequence();
        }
        #[test]
        fn map_struct_mixed_sequence_optional() {
            <$F>::test_map_struct_mixed_sequence_optional();
        }
    };
}
