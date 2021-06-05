//! The [Context] provides a way to store information about the types found during analysis.

mod aggregators;
mod boolean;
mod bytes;
mod map_struct;
mod null;
mod number;
mod sequence;
mod shared;
mod string;

pub use aggregators::Aggregators;
pub use boolean::BooleanContext;
pub use bytes::BytesContext;
pub use map_struct::MapStructContext;
pub use null::NullContext;
pub use number::NumberContext;
pub use sequence::SequenceContext;
pub use shared::{Counter, CountingSet};
pub use string::{SemanticExtractor, StringContext, SuspiciousStrings};

use serde::{Deserialize, Serialize};

/// The Context holds a fresh copy of the context that each [Schema](crate::Schema)
/// copies when it's first created and then fills as the analysis proceeds.
///
/// All default context should respect a constant memory bound on each node.
/// This will allow analysis of arbitraryly large amounts of data as long as the schema does not
/// grow out of proportion.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Context {
    /// The context for null values.
    pub null: NullContext,
    /// The context for boolean values.
    pub boolean: BooleanContext,
    /// The context for integer values.
    pub integer: NumberContext<i128>,
    /// The context for floating point values.
    pub float: NumberContext<f64>,
    /// The context for string values.
    pub string: StringContext,
    /// The context for bytes values.
    pub bytes: BytesContext,
    /// The context for sequence values.
    pub sequence: SequenceContext,
    /// The context for struct values.
    pub map_struct: MapStructContext,
}

impl Context {
    /// Returns a fresh context for null schemas.
    pub fn for_null(&self) -> NullContext {
        self.null.clone()
    }
    /// Returns a fresh context for boolean schemas.
    pub fn for_boolean(&self) -> BooleanContext {
        self.boolean.clone()
    }
    /// Returns a fresh context for integer schemas.
    pub fn for_integer(&self) -> NumberContext<i128> {
        self.integer.clone()
    }
    /// Returns a fresh context for floating point schemas.
    pub fn for_float(&self) -> NumberContext<f64> {
        self.float.clone()
    }
    /// Returns a fresh context for string schemas.
    pub fn for_string(&self) -> StringContext {
        self.string.clone()
    }
    /// Returns a fresh context for bytes schemas.
    pub fn for_bytes(&self) -> BytesContext {
        self.bytes.clone()
    }
    /// Returns a fresh context for sequence schemas.
    pub fn for_sequence(&self) -> SequenceContext {
        self.sequence.clone()
    }
    /// Returns a fresh context for struct schemas.
    pub fn for_map_struct(&self) -> MapStructContext {
        self.map_struct.clone()
    }
}
