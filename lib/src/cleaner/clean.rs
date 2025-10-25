use std::path::{Path, PathBuf};

use tracing::{error, info};

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
        let root_path = util::normalize_path(root_path);

        info!("Clean directory: {}", root_path.display());

        ui.begin_clean(root_path.to_string_lossy().as_ref());

        let mut rule_sets: Vec<RuleSet> = self.global_rule_sets.to_vec();

        let mut search_path = Some(root_path.as_path());
        let mut search_paths: Vec<&Path> = Vec::with_capacity(root_path.components().count());

        while let Some(path) = search_path {
            search_paths.push(path);
            search_path = path.parent();
        }

        search_paths.reverse();

        let mut init_should_delete: Option<bool> = None;

        // Load rulesets outside root path
        for path in search_paths {
            let rules_file_path = path.join(RULES_FILENAME);
            if rules_file_path.exists() {
                rule_sets.push(RuleSet::load_from_file(&rules_file_path, ui)?);
            }

            init_should_delete = should_delete(path, &rule_sets)?.or(init_should_delete);
        }

        let init_should_delete = init_should_delete.unwrap_or(false);

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
                    let prev_should_delete = if let Some(prev_dirstate) = current_dirstate.take() {
                        let prev_should_delete = prev_dirstate.should_delete;

                        // Store await previous dirstate
                        dirstates.push(prev_dirstate);

                        prev_should_delete
                    } else {
                        init_should_delete
                    };

                    let should_delete = should_delete(&path, &rule_sets)?.unwrap_or(prev_should_delete);

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
                            ui.delete_dir(path.to_string_lossy().as_ref());

                            if !dry_run
                                && util::is_dir_empty(&path)?
                                && let Err(err) = std::fs::remove_dir(path)
                            {
                                error!("Error deleting directory: {err}");
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

                    ui.delete_file(path.to_string_lossy().as_ref());

                    if !dry_run && let Err(err) = std::fs::remove_file(path) {
                        error!("Error deleting file: {err}");
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
