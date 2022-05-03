use std::fs;
use std::path::{Path, PathBuf};

use crate::{Context, Result};

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

/// Read the entire contents of a file into a bytes vector.
pub fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).with_context(|| format!("failed to read `{}`", path.display()))
}

/// Read an UTF-8 file from a specific path.
pub fn read_file(path: &Path) -> Result<String> {
    match String::from_utf8(read_bytes(path)?) {
        Ok(s) => Ok(s),
        Err(_) => bail!("path at `{}` was not valid utf-8", path.display()),
    }
}

pub fn stringify(dst: &mut String, path: &serde_ignored::Path<'_>) {
    use serde_ignored::Path;

    match *path {
        Path::Root => {}
        Path::Seq { parent, index } => {
            stringify(dst, parent);
            if !dst.is_empty() {
                dst.push('.');
            }
            dst.push_str(&index.to_string());
        }
        Path::Map { parent, ref key } => {
            stringify(dst, parent);
            if !dst.is_empty() {
                dst.push('.');
            }
            dst.push_str(key);
        }
        Path::Some { parent }
        | Path::NewtypeVariant { parent }
        | Path::NewtypeStruct { parent } => stringify(dst, parent),
    }
}
