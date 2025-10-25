mod clean;

use crate::ruleset::RuleSet;

#[derive(Debug)]
pub struct Cleaner {
    pub ignore: globset::GlobSet,
    pub global_rule_sets: Vec<RuleSet>,
}
