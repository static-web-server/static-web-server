use std::path::{Path, PathBuf};

use super::Result;

/// Validate and return a directory path.
pub fn get_valid_dirpath<P: AsRef<Path>>(path: P) -> Result<PathBuf>
where
    PathBuf: From<P>,
{
    match PathBuf::from(path) {
        v if !v.exists() => bail!("path \"{:?}\" was not found", &v),
        v if !v.is_dir() => bail!("path \"{:?}\" is not a directory", &v),
        v => Ok(v),
    }
}

/// Get the directory name of a valid directory path.
pub fn get_dirname<P: AsRef<Path>>(path: P) -> Result<String>
where
    PathBuf: From<P>,
{
    let path = get_valid_dirpath(path)?;
    match path.iter().last() {
        Some(v) => Ok(v.to_str().unwrap().to_string()),
        _ => bail!("directory name for path \"{:?}\" was not determined", path),
    }
}
