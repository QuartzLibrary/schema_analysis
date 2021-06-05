use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    context::{
        BooleanContext, BytesContext, MapStructContext, NullContext, NumberContext,
        SequenceContext, StringContext,
    },
    Coalesce, StructuralEq,
};

/// This enum is the core output of the analysis, it describes the structure of a document.
///
/// Each variant also contains [context](crate::context) data that allows it to store information
/// about the values it has encountered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Schema {
    /// The Null variant is a special one that is only ever found when a document has a single
    /// null value at the root of the document.
    /// Null values in [Struct](Schema::Struct)s or [Sequence](Schema::Sequence)s are instead
    /// handled at the [Field] level, where it is more ergonomic.
    Null(NullContext),
    /// Represents a boolean value.
    Boolean(BooleanContext),
    /// Represents an integer value.
    Integer(NumberContext<i128>),
    /// Represents a floating point value.
    Float(NumberContext<f64>),
    /// Represents a textual value.
    String(StringContext),
    /// Represents a value of raw bytes.
    Bytes(BytesContext),
    /// Represents a sequence of values described by a [Field].
    /// It assumes all values share the same schema.
    Sequence {
        /// The field is the structure shared by all the elements of the sequence.
        field: Box<Field>,
        /// The context aggregates information about the sequence.
        /// It is passed the length of the sequence.
        context: SequenceContext,
    },
    /// Represents a [String]->[Field] mapping.
    ///
    /// Note: currently there is not a true map and only strings may be used as keys.
    Struct {
        /// Each [String] key gets assigned a [Field].
        /// Currently we are using a [BTreeMap], but that might change in the future.
        fields: BTreeMap<String, Field>,
        /// The context aggregates information about the struct.
        /// It is passed a vector of the key names.
        context: MapStructContext,
    },
    /// Simply a vector of [Schema]s, it should never contain an Union or multiple instances of the
    /// same variant inside.
    ///
    /// Note: content needs to be a struct variant to work with `#[serde(tag = "type")]`.
    Union {
        /// A list of the possible schemas that were found.
        variants: Vec<Schema>,
    },
    // Tuple(..),
    // Map(..),
}

/// A [Field] is a useful abstraction to record metadata that does not belong or would be unyieldy
/// to place into the [Schema] and to account for cases in which the existence of a [Field] might be
/// known, but nothing is known about its shape.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Field {
    /// The status holds information on the the field, like whether it might be null or
    /// missing altogether. Duplicate fields are also recorded.
    #[serde(flatten)]
    pub status: FieldStatus,
    /// The inner Schema is optional because we might have no information on the shape of the field
    /// (like for an empty array).
    #[serde(flatten)]
    pub schema: Option<Schema>,
}

/// The FieldStatus keeps track of what kind of values a [Field] has been found to have.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct FieldStatus {
    /// The [Field] has been found to be [None] or of the unit type `()`.
    pub may_be_null: bool,
    /// The [Field] has been found to be a normal value, where normal means
    /// any valid value for the [Schema] associated with the [Field].
    pub may_be_normal: bool,
    /// The [Field] was found only on some [Struct](Schema::Struct)s or the
    /// [Sequence](Schema::Sequence) to which it belongs might be empty (if only empty sequences
    /// are found, then the [Schema] in the [Field] will also be [None]).
    pub may_be_missing: bool,
    /// Sometimes a field might appear more than once in the same [Struct](Schema::Struct).
    /// In that case all instances are considered, but this flag is also enabled.
    /// This is useful to spot suspicious data, but also to detect sequences in xml files.
    /// See [here](crate::helpers::xml) for more info.
    pub may_be_duplicate: bool,
}

//
// Schema implementations
//
impl StructuralEq for Schema {
    fn structural_eq(&self, other: &Self) -> bool {
        use Schema::*;
        match (self, other) {
            (Null(_), Null(_)) => true,
            (Boolean(_), Boolean(_)) => true,
            (Integer(_), Integer(_)) => true,
            (Float(_), Float(_)) => true,
            (String(_), String(_)) => true,
            (Bytes(_), Bytes(_)) => true,

            (Sequence { field: field_1, .. }, Sequence { field: field_2, .. }) => {
                field_1.structural_eq(field_2)
            }

            (
                Struct {
                    fields: fields_1, ..
                },
                Struct {
                    fields: fields_2, ..
                },
            ) => fields_1.structural_eq(fields_2),

            (Union { variants: s }, Union { variants: o }) => {
                let mut s = s.clone();
                let mut o = o.clone();
                s.sort_by(schema_cmp);
                o.sort_by(schema_cmp);
                s.structural_eq(&o)
            }

            // Listing these out makes sure it fails if new variants are added.
            (Null(_), _)
            | (Boolean(_), _)
            | (Integer(_), _)
            | (Float(_), _)
            | (String(_), _)
            | (Bytes(_), _)
            | (Sequence { .. }, _)
            | (Struct { .. }, _)
            | (Union { .. }, _) => false,
        }
    }
}
impl Coalesce for Schema {
    fn coalesce(&mut self, other: Self) {
        use Schema::*;
        match (self, other) {
            (Boolean(s), Boolean(o)) => s.coalesce(o),
            (Integer(s), Integer(o)) => s.coalesce(o),
            (Float(s), Float(o)) => s.coalesce(o),
            (String(s), String(o)) => s.coalesce(o),
            (Bytes(s), Bytes(o)) => s.coalesce(o),

            (
                Sequence {
                    field: self_boxed,
                    context: self_agg,
                },
                Sequence {
                    field: other_boxed,
                    context: other_agg,
                },
            ) => {
                self_agg.coalesce(other_agg);
                self_boxed.coalesce(*other_boxed);
            }

            (
                Struct {
                    fields: self_fields,
                    context: self_agg,
                },
                Struct {
                    fields: other_fields,
                    context: other_agg,
                },
            ) => {
                self_agg.coalesce(other_agg);
                for (name, other_schema) in other_fields {
                    self_fields
                        .entry(name)
                        .and_modify(|schema| schema.coalesce(other_schema.clone()))
                        .or_insert_with(|| other_schema);
                }
            }
            (
                Union {
                    variants: self_alternatives,
                },
                Union {
                    variants: other_alternatives,
                },
            ) => coalesce_unions(self_alternatives, other_alternatives),
            (
                Union {
                    variants: self_alternatives,
                },
                any_other,
            ) => coalesce_to_alternatives(self_alternatives, any_other),
            (
                any_self,
                Union {
                    variants: mut other_alternatives,
                },
            ) => {
                let self_original = std::mem::replace(any_self, Schema::Null(Default::default()));
                coalesce_to_alternatives(&mut other_alternatives, self_original);
                *any_self = Schema::Union {
                    variants: other_alternatives,
                };
            }

            (any_self, any_other) => {
                let self_original = std::mem::replace(any_self, Schema::Null(Default::default()));
                *any_self = Union {
                    variants: vec![self_original, any_other],
                };
            }
        };
        return;

        fn coalesce_unions(selfs: &mut Vec<Schema>, others: Vec<Schema>) {
            for o in others {
                coalesce_to_alternatives(selfs, o);
            }
        }

        /// This function attempts to match the incomming schema against all the
        /// alternatives already present, and if it fails it pushes it to the vector as a
        /// new alternative.
        fn coalesce_to_alternatives(alternatives: &mut Vec<Schema>, mut other: Schema) {
            use Schema::*;
            for s in alternatives.iter_mut() {
                match (s, other) {
                    // Nested unions should never happen.
                    // It is the job of the root impl of Coalesce for Schema to guarantee this.
                    (Union { .. }, _) | (_, Union { .. }) => {
                        unreachable!("nested union")
                    }

                    // If they are the same, go ahead and coalesce!
                    (Boolean(s), Boolean(o)) => {
                        s.coalesce(o);
                        return;
                    }
                    (Integer(s), Integer(o)) => {
                        s.coalesce(o);
                        return;
                    }
                    (Float(s), Float(o)) => {
                        s.coalesce(o);
                        return;
                    }
                    (String(s), String(o)) => {
                        s.coalesce(o);
                        return;
                    }
                    (Bytes(s), Bytes(o)) => {
                        s.coalesce(o);
                        return;
                    }

                    (
                        Sequence {
                            field: self_boxed,
                            context: self_agg,
                        },
                        Sequence {
                            field: other_boxed,
                            context: other_agg,
                        },
                    ) => {
                        self_agg.coalesce(other_agg);
                        self_boxed.coalesce(*other_boxed);
                        return;
                    }

                    (
                        Struct {
                            fields: self_fields,
                            context: self_agg,
                        },
                        Struct {
                            fields: other_fields,
                            context: other_agg,
                        },
                    ) => {
                        self_agg.coalesce(other_agg);
                        for (name, other_schema) in other_fields {
                            self_fields
                                .entry(name)
                                .and_modify(|schema| schema.coalesce(other_schema.clone()))
                                .or_insert_with(|| other_schema);
                        }
                        return;
                    }

                    // If they don't match just continue ahead to the next one.
                    (_, caught_other) => {
                        other = caught_other;
                    }
                }
            }

            // If we were unable to find a match, push the schema to the alternatives:
            alternatives.push(other);
        }
    }
}
impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        use Schema::*;
        match (self, other) {
            (Null(s), Null(o)) => s == o,
            (Boolean(s), Boolean(o)) => s == o,
            (Integer(s), Integer(o)) => s == o,
            (Float(s), Float(o)) => s == o,
            (String(s), String(o)) => s == o,
            (Bytes(s), Bytes(o)) => s == o,

            (
                Sequence {
                    field: field_1,
                    context: context_1,
                },
                Sequence {
                    field: field_2,
                    context: context_2,
                },
            ) => field_1 == field_2 && context_1 == context_2,

            (
                Struct {
                    fields: fields_1,
                    context: context_1,
                },
                Struct {
                    fields: fields_2,
                    context: context_2,
                },
            ) => fields_1 == fields_2 && context_1 == context_2,

            (Union { variants: s }, Union { variants: o }) => {
                let mut s = s.clone();
                let mut o = o.clone();
                s.sort_by(schema_cmp);
                o.sort_by(schema_cmp);
                s == o
            }

            // Listing these out makes sure it fails if new variants are added.
            (Null(_), _)
            | (Boolean(_), _)
            | (Integer(_), _)
            | (Float(_), _)
            | (String(_), _)
            | (Bytes(_), _)
            | (Sequence { .. }, _)
            | (Struct { .. }, _)
            | (Union { .. }, _) => false,
        }
    }
}

//
// Field implementations
//
impl Field {
    /// Returns a [Field] with the given [Schema] and default [FieldStatus].
    pub fn with_schema(schema: Schema) -> Self {
        Self {
            status: FieldStatus::default(),
            schema: Some(schema),
        }
    }
}
impl Coalesce for Field {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.status.coalesce(other.status);
        self.schema = match (self.schema.take(), other.schema) {
            (Some(mut s), Some(o)) => {
                s.coalesce(o);
                Some(s)
            }
            (Some(s), None) => Some(s),
            (None, Some(o)) => Some(o),
            (None, None) => None,
        }
    }
}
impl StructuralEq for Field {
    fn structural_eq(&self, other: &Self) -> bool {
        self.status == other.status && self.schema.structural_eq(&other.schema)
    }
}

//
// FieldStatus implementations
//
impl FieldStatus {
    /// If the value passed is true, then the status will allow duplicates.
    /// Otherwise no changes are made.
    pub fn allow_duplicates(&mut self, is_duplicate: bool) {
        self.may_be_duplicate |= is_duplicate;
    }
    /// `true` if the status allows for null or missing values.
    pub fn is_option(&self) -> bool {
        self.may_be_null || self.may_be_missing
    }
}
impl Coalesce for FieldStatus {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.may_be_null |= other.may_be_null;
        self.may_be_normal |= other.may_be_normal;
        self.may_be_missing |= other.may_be_missing;
        self.may_be_duplicate |= other.may_be_duplicate;
    }
}

//
// Helper functions
//

/// A helper function that returns the [Ordering](std::cmp::Ordering) of two [Schema]s
/// to help in comparing two [Schema::Union].
/// Since a [Schema::Union] should never hold two schemas of the same type, it is enough to
/// just compare the top level without recursion.
fn schema_cmp(first: &Schema, second: &Schema) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    use Schema::*;
    match first {
        Null(_) => match second {
            Null(_) => Equal,
            _ => Less,
        },
        Boolean(_) => match second {
            Null(_) | Boolean(_) => Equal,
            _ => Less,
        },
        Integer(_) => match second {
            Null(_) | Boolean(_) => Greater,
            Integer(_) => Equal,
            _ => Less,
        },
        Float(_) => match second {
            Null(_) | Boolean(_) | Integer(_) => Greater,
            Float(_) => Equal,
            _ => Less,
        },
        String(_) => match second {
            Null(_) | Boolean(_) | Integer(_) | Float(_) => Greater,
            String(_) => Equal,
            _ => Less,
        },
        Bytes(_) => match second {
            Null(_) | Boolean(_) | Integer(_) | Float(_) | String(_) => Greater,
            Bytes(_) => Equal,
            _ => Less,
        },
        Sequence { .. } => match second {
            Null(_) | Boolean(_) | Integer(_) | Float(_) | String(_) | Bytes(_) => Greater,
            Sequence { .. } => Equal,
            _ => Less,
        },
        Struct { .. } => match second {
            Null(_)
            | Boolean(_)
            | Integer(_)
            | Float(_)
            | String(_)
            | Bytes(_)
            | Sequence { .. } => Greater,
            Struct { .. } => Equal,
            _ => Less,
        },
        Union { .. } => match second {
            Null(_)
            | Boolean(_)
            | Integer(_)
            | Float(_)
            | String(_)
            | Bytes(_)
            | Sequence { .. }
            | Struct { .. } => Greater,
            Union { .. } => Equal,
        },
    }
}
