//! A module holding the crate's public traits.

use std::{any::Any, collections::BTreeMap, fmt::Debug};

use downcast_rs::Downcast;

/**
This trait defines a way to merge two instances of the same type.

```
# use schema_analysis::{Schema, Coalesce, Aggregate, context::BooleanContext};
#
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut context_1: BooleanContext = Default::default();
context_1.aggregate(&true);
context_1.aggregate(&true);
let mut schema_1 = Schema::Boolean(context_1);

let mut context_2: BooleanContext = Default::default();
context_2.aggregate(&false);
let mut schema_2 = Schema::Boolean(context_2);

schema_1.coalesce(schema_2); // schema_2 is gone.

let mut context_merged: BooleanContext = Default::default();
context_merged.aggregate(&true);
context_merged.aggregate(&true);
context_merged.aggregate(&false);
let schema_merged = Schema::Boolean(context_merged);

assert_eq!(schema_1, schema_merged);
#
# Ok(())
# }
```
*/
pub trait Coalesce {
    /// Merge `other` into `self`.
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized;
}
/// This trait allows the merging of a type with an arbitrary trait object.
///
/// If the merger is unsuccessful (they are not of the same type) the trait object is returned.
///
/// This trait has a blanket implementation on any [Sized] type implementing [Coalesce].
pub trait CoalesceAny: Coalesce {
    /// Merge `other` into `self`. Trait object is returned if merging was unsuccessful.
    fn coalesce_any(&mut self, other: Box<dyn Any>) -> Option<Box<dyn Any>>;
}
impl<T: Coalesce + 'static> CoalesceAny for T {
    fn coalesce_any(&mut self, other: Box<dyn Any>) -> Option<Box<dyn Any>> {
        let other: Self = match other.downcast() {
            Ok(downcasted) => *downcasted,
            Err(not_downcasted) => return Some(not_downcasted),
        };
        self.coalesce(other);
        None
    }
}

/// This trait defines an interface used for types that need to receive values one at a time to
/// record something about them.
///
/// V is ?[Sized] to allow for `Aggregator<str>`.
/// In the future a better borrowing API might be implemented.
pub trait Aggregate<V: ?Sized> {
    /// Run the internal logic on value
    fn aggregate(&mut self, value: &'_ V);
}
/// A trait used by [crate::context::Aggregators].
/// It's an experimental feature meant to allow library users to run arbitrary aggregation logic on
/// the input data.
#[dyn_clonable::clonable]
pub trait CoalescingAggregator<V: ?Sized>:
    Aggregate<V> + CoalesceAny + Downcast + Debug + Clone + Send + Sync
{
}

/// This trait checks whether the shape of two objects is the same.
/// The goal is to determine whether two representations are equivalent.
///
/// Example: two schemas of the same type might be structurally equivalent even if some of the
///  internal metadata is different because they have been visited by different sets of examples.
/// Example: two schemas of different types might be structurally equivalent if they represent the
///  same shape of data.
///
/// Notes:
///  - sample-dependent metadata should be ignored.
///  - semantic information (like a regex pattern) should match.
///  - unreliable information (like an *inferred* regex) might be ignored.
///
/// This trait closely mirrors [PartialEq].
pub trait StructuralEq<Rhs: ?Sized = Self> {
    /// Returns `true` if `self` and `other` share the same structure.
    fn structural_eq(&self, other: &Rhs) -> bool;

    /// Returns `true` if `self` and `other` DO NOT share the same structure.
    fn structural_ne(&self, other: &Rhs) -> bool {
        !self.structural_eq(other)
    }
}
impl StructuralEq for String {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl<T: StructuralEq> StructuralEq for Vec<T> {
    fn structural_eq(&self, other: &Self) -> bool {
        self.iter().zip(other).all(|(s, o)| s.structural_eq(o))
    }
}
impl<T: StructuralEq> StructuralEq for Option<T> {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(s), Some(o)) => s.structural_eq(o),
            (Some(_), None) | (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}
impl<K: StructuralEq, V: StructuralEq> StructuralEq for BTreeMap<K, V> {
    fn structural_eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other)
                .all(|((sk, sv), (ok, ov))| sk.structural_eq(ok) && sv.structural_eq(ov))
    }
}
