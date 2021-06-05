use std::collections::BTreeMap;

use maplit::btreemap;

use schema_analysis::{helpers, Field, FieldStatus, InferredSchema, Schema};

mod shared;
use shared::FormatTests;

struct Xml;

test_format!(Xml);

impl FormatTests<String> for Xml {
    fn convert_to_inferred_schema(value: String) -> InferredSchema {
        let mut processed_schema: InferredSchema = quick_xml::de::from_str(&value).unwrap();
        helpers::xml::cleanup_xml_schema(&mut processed_schema.schema);
        processed_schema
    }

    // Xml doesn't allow top-level primitives
    fn null() -> Option<String> {
        None
    }
    fn boolean() -> Option<String> {
        None
    }
    fn integer() -> Option<String> {
        None
    }
    fn float() -> Option<String> {
        None
    }
    fn string() -> Option<String> {
        None
    }

    // Xml doesn't allow top-level arrays (quick_xml ignores later elements anyway)
    fn empty_sequence() -> Option<String> {
        None
    }
    fn string_sequence() -> Option<String> {
        None
    }
    fn integer_sequence() -> Option<String> {
        None
    }
    fn mixed_sequence() -> Option<String> {
        None
    }
    fn optional_mixed_sequence() -> Option<String> {
        None
    }

    // Note: root name is discarded
    fn empty_map_struct() -> Option<String> {
        Some(r#"<wrapper></wrapper>"#.into())
    }

    fn map_struct_single() -> Option<String> {
        Some(r#"<wrapper><hello>1</hello></wrapper>"#.into())
    }
    fn test_map_struct_single() {
        // Xml doesn't have integer values
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
        Self::_compare_map_struct(Self::map_struct_single(), fields);
    }

    fn map_struct_double() -> Option<String> {
        Some(r#"<wrapper><hello>1</hello><world>!</world></wrapper>"#.into())
    }
    fn test_map_struct_double() {
        // Xml doesn't have integer values
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
        Self::_compare_map_struct(Self::map_struct_double(), fields);
    }

    // Xml only has strings, so there is no meaning to 'mixed'.
    fn sequence_map_struct_mixed() -> Option<String> {
        None
    }

    fn sequence_map_struct_optional_or_missing() -> Option<String> {
        Some(
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
                .into(),
        )
    }
    fn test_sequence_map_struct_optional_or_missing() {
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

        Self::_compare_map_struct(
            Self::sequence_map_struct_optional_or_missing(),
            btreemap! {
                "element".into() => sequence_field,
            },
        );
    }

    fn map_struct_mixed_sequence() -> Option<String> {
        Some(
            "
            <wrapper>
                <hello>1</hello>
                <world>!</world>
                <sequence>one</sequence><sequence>two</sequence><sequence>three</sequence>
            </wrapper>"
                .into(),
        )
    }
    fn test_map_struct_mixed_sequence() {
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
        Self::_compare_map_struct(Self::map_struct_mixed_sequence(), fields);
    }
    // No built-in null makes this equivalent to the above.
    fn map_struct_mixed_sequence_optional() -> Option<String> {
        None
    }
}
