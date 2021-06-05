#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{traits::Coalesce, Aggregate};

use super::Counter;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct NullContext {
    pub count: Counter,
}
impl Aggregate<()> for NullContext {
    fn aggregate(&mut self, value: &'_ ()) {
        self.count.aggregate(value);
    }
}
impl Coalesce for NullContext {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.count.coalesce(other.count);
    }
}
