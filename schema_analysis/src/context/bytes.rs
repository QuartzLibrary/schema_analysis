#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{traits::Aggregate, traits::Coalesce};

use super::{shared::Counter, shared::MinMax};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BytesContext {
    pub count: Counter,
    pub min_max_length: MinMax<usize>,
}
impl Aggregate<[u8]> for BytesContext {
    fn aggregate(&mut self, value: &'_ [u8]) {
        self.count.aggregate(value);
        self.min_max_length.aggregate(&value.len());
    }
}
impl Coalesce for BytesContext {
    fn coalesce(&mut self, other: Self) {
        self.count.coalesce(other.count);
        self.min_max_length.coalesce(other.min_max_length);
    }
}
impl PartialEq for BytesContext {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count && self.min_max_length == other.min_max_length
    }
}
