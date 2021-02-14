use std::error;
use std::path::{Path, PathBuf};

/// Validate and return a directory path.
pub fn get_valid_dirpath<P: AsRef<Path>>(p: P) -> Result<PathBuf, Box<dyn error::Error>>
where
    PathBuf: From<P>,
{
    let p: PathBuf = p.into();
    match p {
        v if !v.exists() => Err(From::from(format!("path \"{:?}\" was not found", &v))),
        v if !v.is_dir() => Err(From::from(format!("path \"{:?}\" is not a directory", &v))),
        v => Ok(v),
    }
}

/// Get the directory name of a valid directory path.
pub fn get_dirname<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn error::Error>>
where
    PathBuf: From<P>,
{
    let path = match get_valid_dirpath(path) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    match path.iter().last() {
        Some(v) => Ok(v.to_str().unwrap().to_string()),
        None => Err(From::from(format!(
            "directory name for path \"{:?}\" was not determined",
            path,
        ))),
    }
}

#[cfg(not(windows))]
pub fn adjust_canonicalization<P: AsRef<std::path::Path>>(p: P) -> String {
    p.as_ref().display().to_string()
}

#[cfg(windows)]
pub fn adjust_canonicalization<P: AsRef<std::path::Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.as_ref().display().to_string();
    if p.starts_with(VERBATIM_PREFIX) {
        p[VERBATIM_PREFIX.len()..].to_string()
    } else {
        p
    }
}
