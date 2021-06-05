//! A module for any useful helper functions.

pub mod xml {
    //! A module for xml cleaning helper functions.
    //! Check individual functions for details.

    use crate::{Field, Schema};

    /// A wrapper function that applies all XML cleaning transformations.
    ///
    /// [clean_solitary_nested_values]
    /// + [turn_duplicates_into_sequence_field]
    /// + [clean_empty_structs_in_field]
    pub fn cleanup_xml_schema(schema: &mut Schema) {
        clean_solitary_nested_values(schema);
        turn_duplicates_into_sequence_field(schema);
        clean_empty_structs_in_field(schema);
    }

    /// XML documents often result in uselessly nested values because the 'content' of a tag
    /// that isn't itself a tag is put into the `$value` field.
    ///
    /// This function simply finds [Schema::Struct]s with a single field named `$value` and
    /// replaces them with the schema inside the `$value` field.
    pub fn clean_solitary_nested_values(schema: &mut Schema) {
        use Schema::*;
        match schema {
            Null(_) | Boolean(_) | Integer(_) | Float(_) | String(_) | Bytes(_) => {}
            Sequence { field, .. } => match &mut field.schema {
                Some(schema) => clean_solitary_nested_values(schema),
                None => {}
            },
            Struct { fields, .. } => {
                // If the only field is $value, then we 'bring it up'.
                if fields.len() == 1 && fields.contains_key("$value") {
                    if let Some(Field {
                        schema: Some(inner_schema),
                        ..
                    }) = fields.remove("$value")
                    {
                        *schema = inner_schema;
                    }
                } else {
                    for (_, field) in fields.iter_mut() {
                        match &mut field.schema {
                            Some(schema) => clean_solitary_nested_values(schema),
                            None => {}
                        }
                    }
                }
            }
            Union { variants } => {
                for value in variants {
                    clean_solitary_nested_values(value);
                }
            }
        }
    }

    /// XML documents do not have proper sequences, and an 'array' or 'list' is simply
    /// represented as a tag appearing multiple times.
    ///
    /// To help with this the inference software annotates duplicate fields, and this function
    /// takes the schema in that field and places it into a [Schema::Sequence].
    pub fn turn_duplicates_into_sequence_field(schema: &mut Schema) {
        clean_field_recursively(schema, _inner_field_cleaning);

        fn _inner_field_cleaning(field: &mut Field) {
            match &mut field.schema {
                Some(schema) => clean_field_recursively(schema, _inner_field_cleaning),
                None => {}
            }
            // In xml, sequences are simply registered as a field appearing more than once,
            // the parser records this but now we need to move the duplicate field into its own sequence.
            if field.status.may_be_duplicate {
                *field = Field {
                    status: field.status.clone(),
                    schema: Some(Schema::Sequence {
                        field: Box::new(field.clone()),
                        context: Default::default(),
                    }),
                };
                field.status.may_be_duplicate = false;
            }
        }
    }

    /// When a tag is empty, the parser interprets it as as an empty [Schema::Struct].
    ///
    /// This function replaces those fields with empty [Schema::Struct] with fields of
    /// unknown schema.
    pub fn clean_empty_structs_in_field(schema: &mut Schema) {
        clean_field_recursively(schema, _inner_field_cleaning);

        fn _inner_field_cleaning(field: &mut Field) {
            match &mut field.schema {
                Some(Schema::Struct { fields, .. }) if fields.is_empty() => {
                    field.schema = None;
                }
                Some(schema) => clean_field_recursively(schema, _inner_field_cleaning),
                None => {}
            }
        }
    }

    fn clean_field_recursively(schema: &mut Schema, clean_field: fn(&mut Field)) {
        use Schema::*;
        match schema {
            Null(_) | Boolean(_) | Integer(_) | Float(_) | String(_) | Bytes(_) => {}
            Schema::Sequence { field, .. } => clean_field(field),
            Schema::Struct { fields, .. } => {
                for (_, field) in fields.iter_mut() {
                    clean_field(field);
                }
            }
            Schema::Union { variants } => {
                for value in variants {
                    clean_field_recursively(value, clean_field);
                }
            }
        }
    }
}
