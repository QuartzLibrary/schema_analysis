use schema_analysis::{InferredSchema, Schema, StructuralEq};

/// This provides a way for formats to quickly implement some basic tests.
///
/// Each format should implement the `infer_schema` function which will run the analysis.
///
/// Tests may be skipped by reimplementing `test_*` functions to be no-ops.
pub trait FormatTests {
    type Value;

    /// This is how the format runs inference on the value.
    fn infer_schema(value: Self::Value) -> InferredSchema;

    fn compare(value: Self::Value, target_schema: Schema) {
        let InferredSchema { schema } = Self::infer_schema(value);
        let success = schema.structural_eq(&target_schema);
        if !success {
            println!("INFERRED: {:#?}\n", schema);
            println!("TARGET  : {:#?}\n", target_schema);
        }
        assert!(success);
    }

    fn null() -> Self::Value;
    fn test_null() {
        Self::compare(Self::null(), targets::null())
    }
    fn boolean() -> Self::Value;
    fn test_boolean() {
        Self::compare(Self::boolean(), targets::boolean())
    }
    fn integer() -> Self::Value;
    fn test_integer() {
        Self::compare(Self::integer(), targets::integer())
    }
    fn float() -> Self::Value;
    fn test_float() {
        Self::compare(Self::float(), targets::float())
    }
    fn string() -> Self::Value;
    fn test_string() {
        Self::compare(Self::string(), targets::string())
    }

    fn empty_sequence() -> Self::Value;
    fn test_empty_sequence() {
        Self::compare(Self::empty_sequence(), targets::empty_sequence())
    }
    fn string_sequence() -> Self::Value;
    fn test_string_sequence() {
        Self::compare(Self::string_sequence(), targets::string_sequence())
    }
    fn integer_sequence() -> Self::Value;
    fn test_integer_sequence() {
        Self::compare(Self::integer_sequence(), targets::integer_sequence())
    }
    fn mixed_sequence() -> Self::Value;
    fn test_mixed_sequence() {
        Self::compare(Self::mixed_sequence(), targets::mixed_sequence())
    }
    fn optional_mixed_sequence() -> Self::Value;
    fn test_optional_mixed_sequence() {
        Self::compare(
            Self::optional_mixed_sequence(),
            targets::optional_mixed_sequence(),
        )
    }

    fn empty_map_struct() -> Self::Value;
    fn test_empty_map_struct() {
        Self::compare(Self::empty_map_struct(), targets::empty_map_struct());
    }
    fn map_struct_single() -> Self::Value;
    fn test_map_struct_single() {
        Self::compare(Self::map_struct_single(), targets::map_struct_single());
    }
    fn map_struct_double() -> Self::Value;
    fn test_map_struct_double() {
        Self::compare(Self::map_struct_double(), targets::map_struct_double());
    }
    fn sequence_map_struct_mixed() -> Self::Value;
    fn test_sequence_map_struct_mixed() {
        Self::compare(
            Self::sequence_map_struct_mixed(),
            targets::sequence_map_struct_mixed(),
        );
    }
    fn sequence_map_struct_optional_or_missing() -> Self::Value;
    fn test_sequence_map_struct_optional_or_missing() {
        Self::compare(
            Self::sequence_map_struct_optional_or_missing(),
            targets::sequence_map_struct_optional_or_missing(),
        );
    }
    fn map_struct_mixed_sequence() -> Self::Value;
    fn test_map_struct_mixed_sequence() {
        Self::compare(
            Self::map_struct_mixed_sequence(),
            targets::map_struct_mixed_sequence(),
        );
    }
    fn map_struct_mixed_sequence_optional() -> Self::Value;
    fn test_map_struct_mixed_sequence_optional() {
        Self::compare(
            Self::map_struct_mixed_sequence_optional(),
            targets::map_struct_mixed_sequence_optional(),
        );
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

mod targets {
    use std::collections::BTreeMap;

    use maplit::btreemap;

    use schema_analysis::{Field, FieldStatus, Schema};

    pub fn null() -> Schema {
        Schema::Null(Default::default())
    }
    pub fn boolean() -> Schema {
        Schema::Boolean(Default::default())
    }
    pub fn integer() -> Schema {
        Schema::Integer(Default::default())
    }
    pub fn float() -> Schema {
        Schema::Float(Default::default())
    }
    pub fn string() -> Schema {
        Schema::String(Default::default())
    }

    pub fn empty_sequence() -> Schema {
        let mut field = Field::default();
        field.status.may_be_missing = true;
        Schema::Sequence {
            field: Box::new(field),
            context: Default::default(),
        }
    }
    pub fn string_sequence() -> Schema {
        let mut field = Field {
            status: FieldStatus::default(),
            schema: Some(Schema::String(Default::default())),
        };
        field.status.may_be_normal = true;
        Schema::Sequence {
            field: Box::new(field),
            context: Default::default(),
        }
    }
    pub fn integer_sequence() -> Schema {
        let mut field = Field {
            status: FieldStatus::default(),
            schema: Some(Schema::Integer(Default::default())),
        };
        field.status.may_be_normal = true;
        Schema::Sequence {
            field: Box::new(field),
            context: Default::default(),
        }
    }
    pub fn mixed_sequence() -> Schema {
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
        Schema::Sequence {
            field: Box::new(field),
            context: Default::default(),
        }
    }
    pub fn optional_mixed_sequence() -> Schema {
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
        Schema::Sequence {
            field: Box::new(field),
            context: Default::default(),
        }
    }

    pub fn empty_map_struct() -> Schema {
        let field_schemas: BTreeMap<String, Field> = BTreeMap::new();
        Schema::Struct {
            fields: field_schemas,
            context: Default::default(),
        }
    }
    pub fn map_struct_single() -> Schema {
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
        Schema::Struct {
            fields,
            context: Default::default(),
        }
    }
    pub fn map_struct_double() -> Schema {
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
        Schema::Struct {
            fields,
            context: Default::default(),
        }
    }
    pub fn sequence_map_struct_mixed() -> Schema {
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

        Schema::Sequence {
            field: Box::new(element_field),
            context: Default::default(),
        }
    }
    pub fn sequence_map_struct_optional_or_missing() -> Schema {
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

        Schema::Sequence {
            field: Box::new(element_field),
            context: Default::default(),
        }
    }
    pub fn map_struct_mixed_sequence() -> Schema {
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
        Schema::Struct {
            fields,
            context: Default::default(),
        }
    }
    pub fn map_struct_mixed_sequence_optional() -> Schema {
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
        Schema::Struct {
            fields,
            context: Default::default(),
        }
    }
}
