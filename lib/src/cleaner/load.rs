use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
};

use tracing::info;

use crate::ui::UiHandler;

use super::{Cleaner, CleanerBuilder, Operation, Rule};

impl Cleaner {
    pub fn load_from_file(path: &Path, ui: &mut dyn UiHandler) -> Result<Self, anyhow::Error> {
        let filename = path.file_name().unwrap().to_string_lossy();

        info!("Load ruleset: {}", path.display());
        ui.begin_load(&filename);

        let mut builder = CleanerBuilder::default();

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
                "+" => Operation::Keep,
                "-" => Operation::Delete,
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
