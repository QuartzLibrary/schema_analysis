#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{traits::Coalesce, Aggregate};

use super::{shared::Counter, shared::MinMax, Aggregators};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BytesContext {
    pub count: Counter,
    pub min_max_length: MinMax<usize>,
    #[serde(skip)]
    pub other_aggregators: Aggregators<[u8]>,
}
impl Aggregate<[u8]> for BytesContext {
    fn aggregate(&mut self, value: &'_ [u8]) {
        self.count.aggregate(value);
        self.min_max_length.aggregate(&value.len());
        self.other_aggregators.aggregate(value);
    }
}
impl Coalesce for BytesContext {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.count.coalesce(other.count);
        self.min_max_length.coalesce(other.min_max_length);
        self.other_aggregators.coalesce(other.other_aggregators);
    }
}
impl PartialEq for BytesContext {
    /// NOTE: [BytesContext]'s [PartialEq] implementation ignores the `other_aggregators`
    /// provided by the user of the library.
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count && self.min_max_length == other.min_max_length
    }
}
