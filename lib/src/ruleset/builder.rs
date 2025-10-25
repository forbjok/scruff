use std::path::PathBuf;

use crate::ui::UiHandler;

use super::{BakedRule, Rule, RuleSet};

#[derive(Debug, Default)]
pub struct RuleSetBuilder {
    base_path: PathBuf,
    rules: Vec<Rule>,
}

impl RuleSetBuilder {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            rules: Default::default(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn build(self, _ui: &mut dyn UiHandler) -> Result<RuleSet, anyhow::Error> {
        let base_path = self.base_path;

        let rules = self
            .rules
            .into_iter()
            .map(|r| BakedRule {
                operation: r.operation,
                matcher: globset::Glob::new(&r.pattern).unwrap().compile_matcher(),
            })
            .rev()
            .collect();

        Ok(RuleSet { base_path, rules })
    }
}
