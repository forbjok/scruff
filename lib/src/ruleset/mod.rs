mod builder;
mod load;
mod operation;

use std::path::PathBuf;

pub use self::builder::RuleSetBuilder;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    Delete,
    Keep,
}

#[derive(Debug)]
pub struct Rule {
    pub operation: Operation,
    pub pattern: String,
}

#[derive(Clone, Debug)]
pub struct RuleSet {
    pub base_path: PathBuf,
    rules: Vec<BakedRule>,
}

#[derive(Clone, Debug)]
struct BakedRule {
    pub operation: Operation,
    pub matcher: globset::GlobMatcher,
}
