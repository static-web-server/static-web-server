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

/// Read a file into a trimmed `String`, returning an empty string on any error.
/// Non-UTF-8 bytes are replaced with the Unicode replacement character.
pub fn read_text_default(path: &Path) -> String {
    String::from_utf8_lossy(&read_bytes_default(path))
        .trim()
        .to_owned()
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn temp_file(name: &str, content: &[u8]) -> PathBuf {
        let path = std::env::temp_dir().join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn valid_dirpath_returns_pathbuf() {
        let dir = std::env::temp_dir();
        let result = get_valid_dirpath(&dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dir);
    }

    #[test]
    fn valid_dirpath_rejects_nonexistent() {
        let result = get_valid_dirpath("/this/path/does/not/exist/sws");
        assert!(result.is_err());
    }

    #[test]
    fn valid_dirpath_rejects_file() {
        let path = temp_file("sws_helpers_test_valid_dirpath.txt", b"hello");
        let result = get_valid_dirpath(&path);
        fs::remove_file(&path).ok();
        assert!(result.is_err());
    }

    #[test]
    fn read_bytes_returns_content() {
        let path = temp_file("sws_helpers_read_bytes.bin", b"\x00\x01\x02");
        let bytes = read_bytes(&path);
        fs::remove_file(&path).ok();
        assert_eq!(bytes.unwrap(), vec![0u8, 1, 2]);
    }

    #[test]
    fn read_bytes_errors_on_missing_file() {
        let result = read_bytes(std::path::Path::new("/no/such/sws_file.bin"));
        assert!(result.is_err());
    }

    #[test]
    fn read_bytes_default_returns_content() {
        let path = temp_file("sws_helpers_read_bytes_default.txt", b"world");
        let result = read_bytes_default(&path);
        fs::remove_file(&path).ok();
        assert_eq!(result, b"world");
    }

    #[test]
    fn read_bytes_default_returns_empty_on_missing() {
        assert_eq!(
            read_bytes_default(std::path::Path::new("/no/such/sws_file.txt")),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn read_text_default_returns_trimmed_string() {
        let path = temp_file("sws_helpers_read_text_default.txt", b"  hello world  \n");
        let result = read_text_default(&path);
        fs::remove_file(&path).ok();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn read_text_default_returns_empty_on_missing() {
        assert_eq!(
            read_text_default(std::path::Path::new("/no/such/sws_file.txt")),
            ""
        );
    }

    #[test]
    fn read_file_returns_utf8_string() {
        let path = temp_file("sws_helpers_read_file_utf8.txt", b"static-web-server");
        let result = read_file(&path);
        fs::remove_file(&path).ok();
        assert_eq!(result.unwrap(), "static-web-server");
    }

    #[test]
    fn read_file_errors_on_non_utf8() {
        let path = temp_file("sws_helpers_read_file_bad.bin", b"\xff\xfe");
        let result = read_file(&path);
        fs::remove_file(&path).ok();
        assert!(result.is_err());
    }

    #[test]
    fn read_file_errors_on_missing() {
        assert!(read_file(std::path::Path::new("/no/such/sws_file.txt")).is_err());
    }
}
