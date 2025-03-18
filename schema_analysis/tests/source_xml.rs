use schema_analysis::{helpers, InferredSchema};

mod shared;
use shared::FormatTests;

struct Xml;

test_format!(Xml);

impl FormatTests for Xml {
    type Value = String;

    fn infer_schema(value: Self::Value) -> InferredSchema {
        let mut processed_schema: InferredSchema = quick_xml::de::from_str(&value).unwrap();
        helpers::xml::cleanup_xml_schema(&mut processed_schema.schema);
        processed_schema
    }

    // Xml doesn't allow top-level primitives
    fn test_null() {}
    fn null() -> Self::Value {
        unreachable!()
    }
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

    // Xml doesn't allow top-level arrays (quick_xml ignores later elements anyway)
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

    // Note: root name is discarded
    fn empty_map_struct() -> Self::Value {
        r#"<wrapper></wrapper>"#.into()
    }

    fn map_struct_single() -> Self::Value {
        r#"<wrapper><hello>1</hello></wrapper>"#.into()
    }
    fn test_map_struct_single() {
        Self::compare(Self::map_struct_single(), targets::map_struct_single());
    }

    fn map_struct_double() -> Self::Value {
        r#"<wrapper><hello>1</hello><world>!</world></wrapper>"#.into()
    }
    fn test_map_struct_double() {
        Self::compare(Self::map_struct_double(), targets::map_struct_double());
    }

    // Xml only has strings, so there is no meaning to 'mixed'.
    fn test_sequence_map_struct_mixed() {}
    fn sequence_map_struct_mixed() -> Self::Value {
        unreachable!()
    }

    fn sequence_map_struct_optional_or_missing() -> Self::Value {
        "
        <wrapper>
            <element>
                <hello>1</hello>
                <possibly_null></possibly_null>
                <possibly_missing>1.1</possibly_missing>
                <null_or_missing></null_or_missing>
            </element>
            <element>
                <hello>1</hello>
                <possibly_null>!</possibly_null>
            </element>
        </wrapper>"
            .into()
    }
    fn test_sequence_map_struct_optional_or_missing() {
        Self::compare(
            Self::sequence_map_struct_optional_or_missing(),
            targets::sequence_map_struct_optional_or_missing(),
        );
    }

    fn map_struct_mixed_sequence() -> Self::Value {
        "
        <wrapper>
            <hello>1</hello>
            <world>!</world>
            <sequence>one</sequence><sequence>two</sequence><sequence>three</sequence>
        </wrapper>"
            .into()
    }
    fn test_map_struct_mixed_sequence() {
        Self::compare(
            Self::map_struct_mixed_sequence(),
            targets::map_struct_mixed_sequence(),
        );
    }
    // No built-in null makes this equivalent to the above.
    fn test_map_struct_mixed_sequence_optional() {}
    fn map_struct_mixed_sequence_optional() -> Self::Value {
        unreachable!()
    }
}

/// We need to redefine some targets because, for example, xml doesn't have integer values.
mod targets {
    use std::collections::BTreeMap;

    use maplit::btreemap;

    use schema_analysis::{Field, FieldStatus, Schema};

    pub fn map_struct_single() -> Schema {
        let fields: BTreeMap<String, Field> = {
            let mut hello_field = Field {
                status: FieldStatus::default(),
                schema: Some(Schema::String(Default::default())),
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
        let fields: BTreeMap<String, Field> = {
            let mut hello_field = Field {
                status: FieldStatus::default(),
                schema: Some(Schema::String(Default::default())),
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

    pub fn sequence_map_struct_optional_or_missing() -> Schema {
        // NOTE: in xml sequences are detected as the same key appearing multiple times, so
        // the inner schema is correctly computed over all instances but it is not detected
        // as a sequence.
        // The cleanup_xml_schema fixes this by modifying the fields marked as duplicates
        // by the parser.
        let inner_fields = {
            let mut hello_field = Field::with_schema(Schema::String(Default::default()));
            hello_field.status.may_be_normal = true;

            let mut possibly_null_field = Field::with_schema(Schema::String(Default::default()));
            possibly_null_field.status.may_be_normal = true;
            // possibly_null_field.status.may_be_null = true;  // No built-in null

            let mut possibly_missing_field = Field::with_schema(Schema::String(Default::default()));
            possibly_missing_field.status.may_be_normal = true;
            possibly_missing_field.status.may_be_missing = true;

            let mut null_or_missing_field = Field::default();
            // null_or_missing_field.status.may_be_null = true; //  No built-in null
            null_or_missing_field.status.may_be_normal = true; //  No built-in null
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
        element_field.status.allow_duplicates(true);

        // Note: xml files always have a root struct, so we need to wrap it for the
        // comparison to make sense.
        let mut sequence_field = Field::with_schema(Schema::Sequence {
            field: Box::new(element_field),
            context: Default::default(),
        });
        sequence_field.status.may_be_normal = true;

        Schema::Struct {
            fields: btreemap! {
                "element".into() => sequence_field,
            },
            context: Default::default(),
        }
    }

    pub fn map_struct_mixed_sequence() -> Schema {
        let fields: BTreeMap<String, Field> = {
            let mut hello_field = Field::with_schema(Schema::String(Default::default())); //
            hello_field.status.may_be_normal = true;

            let mut world_field = Field::with_schema(Schema::String(Default::default()));
            world_field.status.may_be_normal = true;

            let mut sequence_field = {
                let mut sequence_element_field =
                    Field::with_schema(Schema::String(Default::default()));
                sequence_element_field.status.may_be_normal = true;
                sequence_element_field.status.allow_duplicates(true); //

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
}
