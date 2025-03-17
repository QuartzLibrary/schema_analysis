//! A module holding the crate's public traits.

/**
This trait defines a way to merge two instances of the same type.

```
# use schema_analysis::{Schema, context::{BooleanContext, DefaultContext}, traits::{Coalesce, Aggregate}};
#
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut context_1: BooleanContext = Default::default();
context_1.aggregate(&true);
context_1.aggregate(&true);
let mut schema_1 = Schema::<DefaultContext>::Boolean(context_1);

let mut context_2: BooleanContext = Default::default();
context_2.aggregate(&false);
let mut schema_2 = Schema::<DefaultContext>::Boolean(context_2);

schema_1.coalesce(schema_2); // schema_2 is gone.

let mut context_merged: BooleanContext = Default::default();
context_merged.aggregate(&true);
context_merged.aggregate(&true);
context_merged.aggregate(&false);
let schema_merged = Schema::<DefaultContext>::Boolean(context_merged);

assert_eq!(schema_1, schema_merged);
#
# Ok(())
# }
```
*/
pub trait Coalesce: Sized {
    /// Merge `other` into `self`.
    fn coalesce(&mut self, other: Self);
}
impl Coalesce for () {
    fn coalesce(&mut self, _other: Self) {}
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
impl<T: ?Sized> Aggregate<T> for () {
    fn aggregate(&mut self, _value: &'_ T) {}
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
