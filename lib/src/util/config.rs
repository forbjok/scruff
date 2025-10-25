use std::path::{Path, PathBuf};

use crate::util;

/// Traverse the filesystem outwards from the specified start path,
/// checking if the specified filename exists in each directory.
/// Returns the path of the first file found, or None is no file is found.
pub fn locate_config_file(
    start_path: impl AsRef<Path>,
    filename: &str,
) -> Result<Option<(PathBuf, PathBuf)>, anyhow::Error> {
    find_outwards(start_path, move |path| {
        let config_file = path.join(filename);

        if config_file.exists() {
            Ok(Some(config_file))
        } else {
            Ok(None)
        }
    })
}

/// Traverse the filesystem outwards from the specified start path,
/// executing the predicate function in each directory.
/// Returns the path and value of the first predicate that returns Some.
pub fn find_outwards<T, E, P>(start_path: impl AsRef<Path>, predicate: P) -> Result<Option<(PathBuf, T)>, E>
where
    P: Fn(&Path) -> Result<Option<T>, E>,
{
    let start_path = util::normalize_path(start_path.as_ref());
    let mut current_path: Option<&Path> = Some(start_path.as_ref());

    while let Some(path) = current_path {
        if let Some(rv) = predicate(path)? {
            return Ok(Some((path.to_path_buf(), rv)));
        }

        current_path = path.parent();
    }

    Ok(None)
}
