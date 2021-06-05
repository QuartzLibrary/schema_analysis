#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{traits::Coalesce, Aggregate, Aggregators};

use super::{shared::MinMax, Counter};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SequenceContext {
    pub count: Counter,
    pub length: MinMax<usize>,
    #[serde(skip)]
    pub other_aggregators: Aggregators<usize>,
}
impl Aggregate<usize> for SequenceContext {
    fn aggregate(&mut self, value: &usize) {
        self.count.aggregate(value);
        self.length.aggregate(value);
        self.other_aggregators.aggregate(value);
    }
}
impl Coalesce for SequenceContext {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.count.coalesce(other.count);
        self.length.coalesce(other.length);
        self.other_aggregators.coalesce(other.other_aggregators);
    }
}
impl PartialEq for SequenceContext {
    /// NOTE: [SequenceContext]'s [PartialEq] implementation ignores the `other_aggregators`
    /// provided by the user of the library.
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count && self.length == other.length
    }
}
