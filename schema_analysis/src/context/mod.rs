//! A [Context] provides a way to store information about the types found during analysis.
//!
//! [DefaultContext] is the one used by default. `()` can be used to skip any additional analysis.

mod boolean;
mod bytes;
mod map_struct;
mod null;
mod number;
mod sequence;
mod shared;
mod string;

pub use boolean::BooleanContext;
pub use bytes::BytesContext;
pub use map_struct::MapStructContext;
pub use null::NullContext;
pub use number::NumberContext;
pub use sequence::SequenceContext;
pub use shared::{Counter, CountingSet};
pub use string::{SemanticExtractor, StringContext, SuspiciousStrings};

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{traits::Aggregate, Coalesce};

/// Interface describing the custom analysis that will be run on each type
/// alongside the schema shape.
///
/// For no analysis, you can use `()`. [DefaultContext] is the one used by default.
pub trait Context {
    /// The state for the analysis run on null values.
    type Null: Aggregate<()> + Coalesce + Clone;
    /// The state for the analysis run on boolean values.
    type Boolean: Aggregate<bool> + Coalesce + Clone;
    /// The state for the analysis run on integer values.
    type Integer: Aggregate<i128> + Coalesce + Clone;
    /// The state for the analysis run on floating point values.
    type Float: Aggregate<f64> + Coalesce + Clone;
    /// The state for the analysis run on strings.
    type String: Aggregate<str> + Coalesce + Clone;
    /// The state for the analysis run on binary data.
    type Bytes: Aggregate<[u8]> + Coalesce + Clone;
    /// The state for the analysis run on sequence values.
    type Sequence: Aggregate<usize> + Coalesce + Clone;
    /// The state for the analysis run on struct values.
    type Struct: Aggregate<[String]> + Coalesce + Clone;

    /// A fresh copy of the context for null values.
    fn new_null(&self) -> Self::Null;
    /// A fresh copy of the context for boolean values.
    fn new_boolean(&self) -> Self::Boolean;
    /// A fresh copy of the context for integer values.
    fn new_integer(&self) -> Self::Integer;
    /// A fresh copy of the context for floating point values.
    fn new_float(&self) -> Self::Float;
    /// A fresh copy of the context for string values.
    fn new_string(&self) -> Self::String;
    /// A fresh copy of the context for binary data.
    fn new_bytes(&self) -> Self::Bytes;
    /// A fresh copy of the context for sequence values.
    fn new_sequence(&self) -> Self::Sequence;
    /// A fresh copy of the context for struct values.
    fn new_map_struct(&self) -> Self::Struct;
}

impl Context for () {
    type Null = ();
    type Boolean = ();
    type Integer = ();
    type Float = ();
    type String = ();
    type Bytes = ();
    type Sequence = ();
    type Struct = ();

    fn new_null(&self) -> Self::Null {}
    fn new_boolean(&self) -> Self::Boolean {}
    fn new_integer(&self) -> Self::Integer {}
    fn new_float(&self) -> Self::Float {}
    fn new_string(&self) -> Self::String {}
    fn new_bytes(&self) -> Self::Bytes {}
    fn new_sequence(&self) -> Self::Sequence {}
    fn new_map_struct(&self) -> Self::Struct {}
}

/// This is the default [Context].
/// It performs some basic analysis like counting and sampling.
///
/// This context has a memory bound for each node.
/// This allows the analysis of arbitraryly large amounts of data as long
/// as the schema itself does not grow out of proportion.
/// (Do note that sampling might still be very large if individual leaves are large.)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DefaultContext {
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
impl Context for DefaultContext {
    type Null = NullContext;
    type Boolean = BooleanContext;
    type Integer = NumberContext<i128>;
    type Float = NumberContext<f64>;
    type String = StringContext;
    type Bytes = BytesContext;
    type Sequence = SequenceContext;
    type Struct = MapStructContext;

    fn new_null(&self) -> Self::Null {
        self.null.clone()
    }
    fn new_boolean(&self) -> Self::Boolean {
        self.boolean.clone()
    }
    fn new_integer(&self) -> Self::Integer {
        self.integer.clone()
    }
    fn new_float(&self) -> Self::Float {
        self.float.clone()
    }
    fn new_string(&self) -> Self::String {
        self.string.clone()
    }
    fn new_bytes(&self) -> Self::Bytes {
        self.bytes.clone()
    }
    fn new_sequence(&self) -> Self::Sequence {
        self.sequence.clone()
    }
    fn new_map_struct(&self) -> Self::Struct {
        self.map_struct.clone()
    }
}
