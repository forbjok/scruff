use std::path::Path;

use anyhow::Context;

use scruff::{ui::UiHandler, Cleaner};

pub fn clean(rules_path: &Path, path: &Path, dry_run: bool, ui: &mut dyn UiHandler) -> Result<(), anyhow::Error> {
    let cleaner = Cleaner::load_from_file(rules_path, ui)?;

    cleaner.clean(path, dry_run, ui).with_context(|| "Cleaning")?;

    Ok(())
}
