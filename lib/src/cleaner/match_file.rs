use std::path::Path;

use super::{Cleaner, Operation};

impl Cleaner {
    pub fn would_delete(&self, path: &Path) -> bool {
        let mut delete = false;

        for r in self.rules.iter() {
            if r.matcher.is_match(path) {
                match r.operation {
                    Operation::Delete => delete = true,
                    Operation::Keep => delete = false,
                };
            }
        }

        delete
    }
}
