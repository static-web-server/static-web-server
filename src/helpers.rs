// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

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

/// Read the entire contents of a file into a bytes vector.
pub fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).with_context(|| format!("failed to read file `{}`", path.display()))
}

/// Read the entire contents of a file into a bytes vector or default to empty.
pub fn read_bytes_default(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_default()
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

#[cfg(windows)]
/// In Windows systems it adjusts the `PathBuf` stripping its `\\?\` prefix.
pub fn adjust_canonicalization(p: &Path) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.to_str().unwrap_or_default();
    let p = if p.starts_with(VERBATIM_PREFIX) {
        p.strip_prefix(VERBATIM_PREFIX).unwrap_or_default()
    } else {
        p
    };
    p.to_owned()
}
