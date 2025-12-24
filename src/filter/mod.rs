//! Filter module for nrs.
//!
//! Provides fuzzy matching and filtering for script names and descriptions.

mod fuzzy;

pub use fuzzy::{
    filter_scripts, filter_scripts_with_matcher, get_match_indices, match_score, matches,
    FuzzyMatcher,
};
