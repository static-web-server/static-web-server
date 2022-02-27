use std::fs;
use std::path::{Path, PathBuf};

use crate::Result;

/// Validate and return a directory path.
pub fn get_valid_dirpath<P: AsRef<Path>>(path: P) -> Result<PathBuf>
where
    PathBuf: From<P>,
{
    match PathBuf::from(path) {
        v if !v.exists() => bail!("path {} was not found or inaccessible", v.display()),
        v if !v.is_dir() => bail!("path {} is not a valid directory", v.display()),
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
        Some(v) => Ok(v.to_str().unwrap().to_owned()),
        _ => bail!(
            "directory name for path {} was not determined",
            path.display()
        ),
    }
}

/// Read the entire contents of a file into a string if valid or returns empty otherwise.
pub fn read_file_content(p: &str) -> String {
    if !p.is_empty() && Path::new(p).exists() {
        return fs::read_to_string(p).unwrap_or_default();
    }
    String::new()
}
