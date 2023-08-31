use crate::ui::UiHandler;

use super::{BakedRule, Cleaner, Rule};

#[derive(Debug, Default)]
pub struct CleanerBuilder {
    rules: Vec<Rule>,
}

impl CleanerBuilder {
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn build(self, _ui: &mut dyn UiHandler) -> Result<Cleaner, anyhow::Error> {
        let rules = self
            .rules
            .into_iter()
            .map(|r| BakedRule {
                operation: r.operation,
                matcher: globset::Glob::new(&r.pattern).unwrap().compile_matcher(),
            })
            .collect();

        Ok(Cleaner { rules })
    }
}
