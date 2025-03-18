#![allow(missing_docs)]

use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{traits::Aggregate, traits::Coalesce};

use super::shared::{Counter, CountingSet, MinMax, Sampler};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StringContext {
    pub count: Counter,
    pub samples: Sampler<String>,
    /// Keeps track of any occurrences of strings that are known to be fishy.
    #[serde(default, skip_serializing_if = "SuspiciousStrings::is_empty")]
    pub suspicious_strings: SuspiciousStrings,
    /// Runs regexes on the strings to check whether they have interesting values.
    #[serde(default, skip_serializing_if = "SemanticExtractor::is_empty")]
    pub semantic_extractor: SemanticExtractor,
    pub min_max_length: MinMax<usize>,
}
impl Aggregate<str> for StringContext {
    fn aggregate(&mut self, value: &'_ str) {
        self.count.aggregate(value);
        self.samples.aggregate(value);
        self.suspicious_strings.aggregate(value);
        self.semantic_extractor.aggregate(value);
        self.min_max_length.aggregate(&value.len());
    }
}
impl Coalesce for StringContext {
    fn coalesce(&mut self, other: Self) {
        self.count.coalesce(other.count);
        self.samples.coalesce(other.samples);
        self.suspicious_strings.coalesce(other.suspicious_strings);
        self.semantic_extractor.coalesce(other.semantic_extractor);
        self.min_max_length.coalesce(other.min_max_length);
    }
}
impl PartialEq for StringContext {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
            && self.samples == other.samples
            && self.suspicious_strings == other.suspicious_strings
            && self.semantic_extractor == other.semantic_extractor
            && self.min_max_length == other.min_max_length
    }
}

//
// SuspiciousString
//

const NORMALIZED_SUSPICIOUS_STRINGS: &[&str] = &[
    "n/a", "na", "nan", "null", "none", "nil", "?", "-", "/", "", " ", "  ",
];
/// Keeps track of any occurrences of strings that are known to be fishy,
/// open a PR if you have more!
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SuspiciousStrings(pub CountingSet<String>);
impl SuspiciousStrings {
    /// Returns `true` if no suspicious strings have been found.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl Aggregate<str> for SuspiciousStrings {
    fn aggregate(&mut self, value: &'_ str) {
        if NORMALIZED_SUSPICIOUS_STRINGS.contains(&value.to_lowercase().as_str()) {
            self.0.insert(value);
        }
    }
}
impl Coalesce for SuspiciousStrings {
    fn coalesce(&mut self, other: Self) {
        self.0.coalesce(other.0);
    }
}

//
// SemanticExtractor
// This is a POC, more targets should be later added if it works well.
//

const RAW_SEMANTIC_TARGETS: [(&str, &str); 5] = [
    ("Integer", r"[-+]?\d+"),
    ("Simple Float", r"\d+[.,]\d+"),
    ("Date 31-12-2001", r"\d{2}-\d{2}-\d{4}"),
    ("Date 2001-12-31", r"\d{4}-\d{2}-\d{2}"),
    // `(?i)` sets and `(?-i)` clears the case-insensitive flag.
    ("Boolean", r"(?i)(true|yes|false|no)(?-i)"),
];

static SEMANTIC_TARGETS: Lazy<BTreeMap<String, Regex>> = Lazy::new(|| {
    fn from_pattern(p: &str) -> Regex {
        Regex::new(&format!(r"^\s*{}\s*$", p)).unwrap()
    }
    RAW_SEMANTIC_TARGETS
        .iter()
        .map(|(n, p)| (n.to_string(), from_pattern(p)))
        .collect()
});
/// Runs regexes on the strings to check whether they have interesting values.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SemanticExtractor(CountingSet<String>);
impl SemanticExtractor {
    /// Returns `true` if no interesting strings have been found.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl Aggregate<str> for SemanticExtractor {
    fn aggregate(&mut self, value: &'_ str) {
        for (target, regex) in SEMANTIC_TARGETS.iter() {
            if regex.is_match(value) {
                self.0.insert(target);
            }
        }
    }
}
impl Coalesce for SemanticExtractor {
    fn coalesce(&mut self, other: Self) {
        self.0.coalesce(other.0);
    }
}
