#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{traits::Coalesce, Aggregate};

use super::{Aggregators, Counter};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapStructContext {
    pub count: Counter,
    #[serde(skip)]
    pub other_aggregators: Aggregators<[String]>,
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
    /// NOTE: [MapStructContext]'s [PartialEq] implementation ignores the `other_aggregators`
    /// provided by the user of the library.
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}
