mod clean;

use std::path::PathBuf;

use crate::ruleset::RuleSet;

pub use self::clean::*;

#[derive(Debug)]
pub struct Cleaner {
    pub ignore: globset::GlobSet,
    pub global_rule_sets: Vec<RuleSet>,
}
