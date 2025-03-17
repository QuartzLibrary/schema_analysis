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
    type Null: Aggregate<()> + Coalesce + Default;
    /// The state for the analysis run on boolean values.
    type Boolean: Aggregate<bool> + Coalesce + Default;
    /// The state for the analysis run on integer values.
    type Integer: Aggregate<i128> + Coalesce + Default;
    /// The state for the analysis run on floating point values.
    type Float: Aggregate<f64> + Coalesce + Default;
    /// The state for the analysis run on strings.
    type String: Aggregate<str> + Coalesce + Default;
    /// The state for the analysis run on binary data.
    type Bytes: Aggregate<[u8]> + Coalesce + Default;
    /// The state for the analysis run on sequence values.
    type Sequence: Aggregate<usize> + Coalesce + Default;
    /// The state for the analysis run on struct values.
    type Struct: Aggregate<[String]> + Coalesce + Default;
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
}

/// This is the default [Context].
/// It performs some basic analysis like counting and sampling.
///
/// This context has a memory bound for each node.
/// This allows the analysis of arbitraryly large amounts of data as long
/// as the schema itself does not grow out of proportion.
/// (Do note that sampling might still be very large if individual leaves are large.)
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct DefaultContext;
impl Context for DefaultContext {
    type Null = NullContext;
    type Boolean = BooleanContext;
    type Integer = NumberContext<i128>;
    type Float = NumberContext<f64>;
    type String = StringContext;
    type Bytes = BytesContext;
    type Sequence = SequenceContext;
    type Struct = MapStructContext;
}
