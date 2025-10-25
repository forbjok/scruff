use std::{fs, path::Path};

pub fn is_dir_empty(path: &Path) -> anyhow::Result<bool> {
    match fs::read_dir(path)?.next() {
        Some(_) => Ok(false),
        None => Ok(true),
    }
}
