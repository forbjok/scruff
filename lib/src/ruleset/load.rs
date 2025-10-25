use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::Context;
use tracing::info;

use crate::ui::UiHandler;

use super::{Operation, Rule, RuleSet, builder::RuleSetBuilder};

impl RuleSet {
    pub fn load_from_file(path: &Path, ui: &mut dyn UiHandler) -> anyhow::Result<Self> {
        let dir_path = path.parent().context("Get parent directory of ruleset")?;
        let filename = path.file_name().context("Get filename of ruleset")?.to_string_lossy();

        info!("Load ruleset: {}", path.display());
        ui.begin_load(&filename);

        let mut builder = RuleSetBuilder::new(dir_path.to_path_buf());

        let file = fs::File::open(path)?;
        let file = BufReader::new(file);

        let lines = file.lines().map_while(Result::ok);
        for line in lines {
            // Skip comments
            if line.starts_with('#') {
                continue;
            }

            let Some((op, pattern)) = line.split_once(' ') else {
                return Err(anyhow::anyhow!("Invalid rule: {line}"));
            };

            let pattern = pattern.trim();

            let operation = match op {
                "-" => Operation::Keep,
                "+" => Operation::Delete,
                _ => {
                    return Err(anyhow::anyhow!("Invalid rule: {line}"));
                }
            };

            let pattern = pattern.to_owned();

            builder.add_rule(Rule { operation, pattern });
        }

        info!("Ruleset loaded");
        ui.end_load();

        builder.build(ui)
    }
}
