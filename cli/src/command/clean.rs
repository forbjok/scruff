use std::path::Path;

use anyhow::Context;

use scruff::{Cleaner, ui::UiHandler};

pub fn clean(path: &Path, dry_run: bool, ui: &mut dyn UiHandler) -> Result<(), anyhow::Error> {
    let cleaner = Cleaner::new()?;

    cleaner.clean(path, dry_run, ui).with_context(|| "Cleaning")?;

    Ok(())
}
