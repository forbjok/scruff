use std::path::Path;

use tracing::info;

use crate::ui::UiHandler;

use super::Cleaner;

impl Cleaner {
    pub fn clean(&self, root_path: &Path, dry_run: bool, ui: &mut dyn UiHandler) -> Result<(), anyhow::Error> {
        info!("Clean directory: {}", root_path.display());

        ui.begin_clean(root_path.to_string_lossy().as_ref());

        let files_to_delete: Vec<_> = walkdir::WalkDir::new(root_path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                if entry.file_type().is_dir() {
                    return None;
                }

                let path = entry.into_path();

                // Make path relative, as we only want to match on the path
                // relative to the root.
                let rel_path = path.strip_prefix(root_path).unwrap();

                if self.would_delete(rel_path) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        for path in files_to_delete {
            let rel_path = path.strip_prefix(root_path).unwrap();

            ui.delete_file(rel_path.to_string_lossy().as_ref());

            if !dry_run {
                std::fs::remove_file(path)?;
            }
        }

        info!("Clean complete");
        ui.end_clean();

        Ok(())
    }
}
