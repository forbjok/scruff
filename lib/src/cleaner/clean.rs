use std::path::{Path, PathBuf};

use tracing::info;

use crate::{
    RULES_FILENAME,
    dir_traverser::{DirTraverser, DirTraverserEvent},
    ruleset::{Operation, RuleSet},
    ui::UiHandler,
    util,
};

use super::Cleaner;

impl Cleaner {
    pub fn new() -> anyhow::Result<Self> {
        let ignore = {
            let mut builder = globset::GlobSetBuilder::new();
            builder.add(globset::Glob::new("**/.git")?);
            builder.add(globset::Glob::new(&format!("**/{RULES_FILENAME}"))?);

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

        // Load rulesets outside root path
        let mut rule_sets = {
            let mut rule_sets: Vec<RuleSet> = self.global_rule_sets.to_vec();

            let mut search_path = root_path.to_path_buf();

            while let Some((dir_path, config_file_path)) = util::locate_config_file(&search_path, RULES_FILENAME)? {
                rule_sets.push(RuleSet::load_from_file(&config_file_path, ui)?);

                search_path = dir_path;
                search_path.pop();
            }

            rule_sets
        };

        #[derive(Debug, Default)]
        struct DirState {
            path: PathBuf,
            should_delete: bool,
            has_ruleset: bool,
        }

        let mut current_dirstate: Option<DirState> = None;
        let mut dirstates: Vec<DirState> = Default::default();

        let traverser = DirTraverser::new(root_path.to_path_buf()).ignore(self.ignore.clone());

        for event in traverser.into_iter() {
            match event? {
                DirTraverserEvent::EnterDir(path) => {
                    let ignored = self.ignore.is_match(&path);

                    let prev_should_delete = if let Some(prev_dirstate) = current_dirstate.take() {
                        let prev_should_delete = prev_dirstate.should_delete;

                        // Store await previous dirstate
                        dirstates.push(prev_dirstate);

                        prev_should_delete
                    } else {
                        false
                    };

                    let should_delete = if ignored {
                        false
                    } else {
                        should_delete(&path, &rule_sets)?.unwrap_or(prev_should_delete)
                    };

                    let rules_file_path = path.join(RULES_FILENAME);
                    let has_ruleset = if rules_file_path.is_file() {
                        let rule_set = RuleSet::load_from_file(&rules_file_path, ui)?;
                        rule_sets.push(rule_set);

                        true
                    } else {
                        false
                    };

                    // Create new current dirstate
                    current_dirstate = Some(DirState {
                        path,
                        should_delete,
                        has_ruleset,
                    });
                }
                DirTraverserEvent::ExitDir(path) => {
                    if let Some(current_dirstate) = current_dirstate {
                        if current_dirstate.has_ruleset {
                            rule_sets.pop();
                        }

                        if current_dirstate.should_delete {
                            let rel_path = path.strip_prefix(root_path)?;
                            ui.delete_dir(rel_path.to_string_lossy().as_ref());

                            if !dry_run {
                                //std::fs::remove_dir(path)?;
                            }
                        }
                    }

                    current_dirstate = dirstates.pop();
                }
                DirTraverserEvent::File(filename) => {
                    let Some(current_dirstate) = current_dirstate.as_ref() else {
                        continue;
                    };

                    let path = current_dirstate.path.join(filename);

                    if self.ignore.is_match(&path) {
                        continue;
                    }

                    let should_delete = should_delete(&path, &rule_sets)?.unwrap_or(current_dirstate.should_delete);

                    if !should_delete {
                        continue;
                    }

                    let rel_path = path.strip_prefix(root_path)?;
                    ui.delete_file(rel_path.to_string_lossy().as_ref());

                    if !dry_run {
                        //std::fs::remove_file(path)?;
                    }
                }
            }
        }

        info!("Clean complete");
        ui.end_clean();

        Ok(())
    }
}

fn should_delete(path: &Path, rule_sets: &[RuleSet]) -> anyhow::Result<Option<bool>> {
    for rs in rule_sets.iter().rev() {
        let rel_path = path.strip_prefix(&rs.base_path)?;

        let Some(op) = rs.operation(rel_path) else {
            continue;
        };

        return Ok(Some(op == Operation::Delete));
    }

    Ok(None)
}
