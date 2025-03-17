#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{traits::Aggregate, traits::Coalesce};

use super::Counter;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapStructContext {
    pub count: Counter,
}
impl Aggregate<[String]> for MapStructContext {
    fn aggregate(&mut self, value: &[String]) {
        self.count.aggregate(value);
    }
}
impl Coalesce for MapStructContext {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.count.coalesce(other.count);
    }
}
impl PartialEq for MapStructContext {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}
