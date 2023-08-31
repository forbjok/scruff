mod builder;
mod clean;
mod load;
mod match_file;

pub use self::builder::*;
pub use self::clean::*;
pub use self::load::*;

#[derive(Debug)]
pub enum Operation {
    Delete,
    Keep,
}

#[derive(Debug)]
pub struct Rule {
    pub operation: Operation,
    pub pattern: String,
}

#[derive(Debug)]
pub struct Cleaner {
    rules: Vec<BakedRule>,
}

#[derive(Debug)]
struct BakedRule {
    pub operation: Operation,
    pub matcher: globset::GlobMatcher,
}
