pub mod cleaner;
mod dir_traverser;
pub mod ruleset;
pub mod ui;
mod util;

pub use cleaner::Cleaner;

pub const RULES_FILENAME: &str = ".scruff";
