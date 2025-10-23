use std::path::{Path, PathBuf};

use tracing::info;

use crate::{
    RULES_FILENAME,
    ruleset::{Operation, RuleSet},
    ui::UiHandler,
    util,
};

use super::Cleaner;

impl Cleaner {
    pub fn new() -> anyhow::Result<Self> {
        let ignore = {
            let mut builder = globset::GlobSetBuilder::new();
            builder.add(globset::Glob::new("**/.git/")?);

            builder.build()?
        };

        Ok(Self {
            ignore,
            global_rule_sets: Default::default(),
        })
    }

    pub fn clean(&self, root_path: &Path, dry_run: bool, ui: &mut dyn UiHandler) -> Result<(), anyhow::Error> {
        info!("Clean directory: {}", root_path.display());

        ui.begin_clean(root_path.to_string_lossy().as_ref());

        let path_entries: Vec<_> = walkdir::WalkDir::new(root_path)
            .into_iter()
            .filter_entry(|entry| !self.ignore.is_match(entry.path()))
            .filter_map(|entry| entry.ok())
            .collect();

        // Load rulesets outside root path
        let mut rule_sets = {
            let mut rule_sets: Vec<(usize, RuleSet)> =
                self.global_rule_sets.iter().cloned().map(|rs| (0, rs)).collect();

            let mut search_path = root_path.to_path_buf();

            while let Some((dir_path, config_file_path)) = util::locate_config_file(&search_path, RULES_FILENAME)? {
                rule_sets.push((
                    dir_path.components().count(),
                    RuleSet::load_from_file(&config_file_path, ui)?,
                ));

                search_path = dir_path;
                search_path.pop();
            }

            rule_sets
        };

        let mut prev_path_depth = root_path.components().count();
        let mut deleted_file_count: usize = 0;
        let mut deleted_file_count_per_directory: Vec<(PathBuf, usize)> = Default::default();

        for path_entry in path_entries {
            let path = path_entry.path();

            if path.is_dir() {
                let path_depth = path.components().count();

                // Remove rule sets that are not relevant to the new path depth
                rule_sets.retain(|(rs_depth, _)| *rs_depth < path_depth);

                let rules_file_path = path.join(RULES_FILENAME);
                if rules_file_path.is_file() {
                    let rule_set = RuleSet::load_from_file(&rules_file_path, ui)?;
                    rule_sets.push((path_depth, rule_set));
                }

                if path_depth <= prev_path_depth {

                }

                prev_path_depth = path_depth;
                deleted_file_count_per_directory.push((path.to_path_buf(), deleted_file_count));
                deleted_file_count = 0;

                continue;
            }

            let mut should_delete = false;

            for (_, rs) in rule_sets.iter().rev() {
                let rel_path = path.strip_prefix(&rs.base_path)?;

                let Some(op) = rs.operation(rel_path) else {
                    continue;
                };

                should_delete = op == Operation::Delete;
                break;
            }

            if !should_delete {
                continue;
            }

            let rel_path = path.strip_prefix(root_path)?;
            ui.delete_file(rel_path.to_string_lossy().as_ref());

            if !dry_run {
                //std::fs::remove_file(path)?;
            }
        }

        info!("Clean complete");
        ui.end_clean();

        Ok(())
    }
}
