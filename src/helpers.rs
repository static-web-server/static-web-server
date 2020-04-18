use std::path::{Path, PathBuf};

pub enum PathError {
    PathNotFound,
    NotDirectory,
}

/// Validate a path if exist and is a directory.
pub fn validate_dirpath<P: AsRef<Path>>(path: P) -> Result<PathBuf, PathError>
where
    PathBuf: From<P>,
{
    match PathBuf::from(path) {
        p if !p.exists() => Result::Err(PathError::PathNotFound),
        p if !p.is_dir() => Result::Err(PathError::NotDirectory),
        p => Result::Ok(p),
    }
}

/// Format a `PathError` description
pub fn path_error_fmt(err: PathError, dirname: &str, dirpath: &str) -> String {
    match err {
        PathError::PathNotFound => format!("{} path \"{}\" was not found", dirname, dirpath),
        PathError::NotDirectory => format!("{} path \"{}\" is not a directory", dirname, dirpath),
    }
}
