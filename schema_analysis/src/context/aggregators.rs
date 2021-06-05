use std::{any::Any, fmt::Debug};

use crate::{Aggregate, Coalesce, CoalescingAggregator};

/// A collection of aggregators that should allow the user of the library to run arbitrary
/// aggregation code on the data as it is being analyzed.
///
/// This is an experimental feature.
#[derive(Debug)]
pub struct Aggregators<V: ?Sized>(pub Vec<Box<dyn CoalescingAggregator<V>>>);

impl<V: ?Sized> Aggregate<V> for Aggregators<V> {
    fn aggregate(&mut self, value: &'_ V) {
        for a in &mut self.0 {
            a.aggregate(value)
        }
    }
}
impl<T: ?Sized + 'static> Coalesce for Aggregators<T> {
    fn coalesce(&mut self, other: Aggregators<T>)
    where
        Self: Sized,
    {
        'outer: for o in other.0 {
            let mut o: Box<dyn Any> = o.into_any();
            for s in &mut self.0 {
                // coalesce_any returns the value if it doesn't manage to coalesce it.
                o = match s.coalesce_any(o) {
                    Some(o) => o,
                    None => continue 'outer,
                }
            }
            let o = *o.downcast::<Box<dyn CoalescingAggregator<T>>>().unwrap();
            self.0.push(o);
        }
    }
}
impl<T: ?Sized> Clone for Aggregators<T> {
    fn clone(&self) -> Self {
        Aggregators(self.0.clone())
    }
}
impl<T: ?Sized> Default for Aggregators<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<V: ?Sized> From<Vec<Box<dyn CoalescingAggregator<V>>>> for Aggregators<V> {
    fn from(value: Vec<Box<dyn CoalescingAggregator<V>>>) -> Self {
        Self(value)
    }
}
