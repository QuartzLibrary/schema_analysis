#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{traits::Coalesce, Aggregate};

use super::Counter;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct BooleanContext {
    pub count: Counter,
    pub trues: Counter,
    pub falses: Counter,
}
impl Aggregate<bool> for BooleanContext {
    fn aggregate(&mut self, value: &'_ bool) {
        self.count.aggregate(value);
        match value {
            true => self.trues.aggregate(&()),
            false => self.falses.aggregate(&()),
        }
    }
}
impl Coalesce for BooleanContext {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.count.coalesce(other.count);
        self.trues.coalesce(other.trues);
        self.falses.coalesce(other.falses);
    }
}
