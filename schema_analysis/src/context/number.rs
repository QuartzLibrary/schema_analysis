#![allow(missing_docs)]

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{traits::Coalesce, Aggregate, Aggregators};

use super::{
    shared::{MinMax, Sampler},
    Counter,
};

/// The context for numeric values.
///
/// Uses non-generic implementations and Orderly, a helper trait,
/// to allow floats and integer to share the code.
/// Might not be worth it, but oh well.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NumberContext<T: Orderly> {
    pub count: Counter,
    pub samples: Sampler<T::Ordered>,
    #[serde(flatten)]
    pub min_max: MinMax<T>,
    #[serde(skip)]
    pub other_aggregators: Aggregators<T>,
}
impl Aggregate<i128> for NumberContext<i128> {
    fn aggregate(&mut self, value: &i128) {
        self.count.aggregate(value);
        self.samples.aggregate(value);
        self.min_max.aggregate(value);
        self.other_aggregators.aggregate(value);
    }
}
impl Aggregate<f64> for NumberContext<f64> {
    fn aggregate(&mut self, value: &'_ f64) {
        self.count.aggregate(value);
        self.samples.aggregate(value.into()); // ordered_float
        if !value.is_nan() {
            self.min_max.aggregate(value);
        }
        self.other_aggregators.aggregate(value);
    }
}
impl<T: Clone + PartialOrd + Orderly + 'static> Coalesce for NumberContext<T> {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.count.coalesce(other.count);
        self.samples.coalesce(other.samples);
        self.min_max.coalesce(other.min_max);
        self.other_aggregators.coalesce(other.other_aggregators);
    }
}
impl<T: PartialEq + Orderly> PartialEq for NumberContext<T> {
    /// NOTE: [NumberContext]'s [PartialEq] implementation ignores the `other_aggregators`
    /// provided by the user of the library.
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count && self.min_max == other.min_max
    }
}

//
// Marker trait
//

/// A marker trait that
pub trait Orderly: Sized {
    type Ordered: Ord + Clone + Serialize + DeserializeOwned;
}
impl Orderly for i128 {
    type Ordered = i128;
}
impl Orderly for usize {
    type Ordered = usize;
}
impl Orderly for f64 {
    type Ordered = ordered_float::OrderedFloat<f64>;
}
impl Orderly for f32 {
    type Ordered = ordered_float::OrderedFloat<f32>;
}
