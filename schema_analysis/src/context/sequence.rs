#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{traits::Aggregate, traits::Coalesce};

use super::{shared::MinMax, Counter};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SequenceContext {
    pub count: Counter,
    pub length: MinMax<usize>,
}
impl Aggregate<usize> for SequenceContext {
    fn aggregate(&mut self, value: &usize) {
        self.count.aggregate(value);
        self.length.aggregate(value);
    }
}
impl Coalesce for SequenceContext {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.count.coalesce(other.count);
        self.length.coalesce(other.length);
    }
}
impl PartialEq for SequenceContext {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count && self.length == other.length
    }
}
