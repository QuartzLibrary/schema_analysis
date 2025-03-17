use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
};

use serde::{Deserialize, Serialize};

use crate::{traits::Coalesce, Aggregate};

//
// Counter
//

/// As simple as an aggregato can be, counts the aggregated values.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Counter(pub usize);
impl<T: ?Sized> Aggregate<T> for Counter {
    fn aggregate(&mut self, _value: &'_ T) {
        self.0 += 1;
    }
}
impl Coalesce for Counter {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.0 += other.0;
    }
}

//
// CountingSet
//

/// Keeps track of the inserted values and how many times they have occurred.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CountingSet<T: Ord>(pub BTreeMap<T, usize>);
impl<T: Clone + Ord> CountingSet<T> {
    /// adds a value to the set or increases its counter if it already exists.
    pub fn insert<Q>(&mut self, key: &Q)
    where
        T: Borrow<Q>,
        Q: Ord + ToOwned<Owned = T> + ?Sized,
    {
        match self.0.get_mut(key) {
            Some(v) => *v += 1,
            None => {
                self.0.insert(ToOwned::to_owned(key), 1);
            }
        };
    }
    /// Checks if a specific value is present inside.
    pub fn contains_key<Q>(&mut self, key: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.0.contains_key(key)
    }
    /// Returns `true` if no value has been inserted yet.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Returns the number of distinct values inside.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T: Ord> Coalesce for CountingSet<T> {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        for (k, v) in other.0 {
            let s: &mut usize = self.0.entry(k).or_insert(0);
            *s += v;
        }
    }
}
impl<T: Ord> Default for CountingSet<T> {
    // To avoid imposing T: Default
    fn default() -> Self {
        Self(Default::default())
    }
}

//
// MinMax
//

/// Keeps track of the highest and lowest values.
///
/// It should not be fed NaN values, as it won't work if they are the first value presented.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct MinMax<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}
impl<T: Clone + PartialOrd> Aggregate<T> for MinMax<T> {
    fn aggregate(&mut self, value: &'_ T) {
        match &self.min {
            Some(old_min) if value < old_min => self.min = Some(value.clone()),
            Some(_) => {}
            None => self.min = Some(value.clone()),
        };
        match &self.max {
            Some(old_max) if value > old_max => self.max = Some(value.clone()),
            Some(_) => {}
            None => self.max = Some(value.clone()),
        };
    }
}
impl<T: Clone + PartialOrd> Coalesce for MinMax<T> {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        if let Some(other_min) = other.min {
            self.aggregate(&other_min);
        }
        if let Some(other_max) = other.max {
            self.aggregate(&other_max);
        }
    }
}

//
// Sampler
//

/// Keeps track of the first [MAX_SAMPLE_COUNT] distinct samples.
/// If more are passed it'll flip the is_exaustive flag.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sampler<T: Ord> {
    values: BTreeSet<T>,
    is_exaustive: bool,
}
const MAX_SAMPLE_COUNT: usize = 5;
impl<T, Q> Aggregate<Q> for Sampler<T>
where
    T: Ord + Borrow<Q>,
    Q: Ord + ToOwned<Owned = T> + ?Sized,
{
    fn aggregate(&mut self, value: &'_ Q) {
        if self.values.len() <= MAX_SAMPLE_COUNT {
            self.values.insert(value.to_owned());
        } else if self.is_exaustive && !self.values.contains(value) {
            self.is_exaustive = false;
        }
    }
}
impl<T: Ord> Coalesce for Sampler<T> {
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.values.extend(other.values);
        if self.values.len() > MAX_SAMPLE_COUNT {
            self.is_exaustive = false;
        }
        self.values = std::mem::take(&mut self.values)
            .into_iter()
            .take(MAX_SAMPLE_COUNT)
            .collect();
    }
}
impl<T: Ord> Default for Sampler<T> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            is_exaustive: true,
        }
    }
}
