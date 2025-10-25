use std::path::Path;

use super::{Operation, RuleSet};

impl RuleSet {
    pub fn operation(&self, path: &Path) -> Option<Operation> {
        for r in self.rules.iter() {
            if r.matcher.is_match(path) {
                return Some(r.operation);
            }
        }

        None
    }
}
