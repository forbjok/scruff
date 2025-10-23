pub mod cleaner;
pub mod ruleset;
pub mod ui;
mod util;

pub use cleaner::Cleaner;

pub const RULES_FILENAME: &str = ".scruff";
