// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module that provides file path-related facilities.

use anyhow::Context;
use hyper::StatusCode;
use percent_encoding::percent_decode_str;
use std::path::{Component, Path, PathBuf};

use crate::Result;

/// `Path` extensions trait.
pub(crate) trait PathExt {
    /// If file path is hidden.
    fn is_hidden(&self) -> bool;
    /// If the path is a symlink or contains intermediate symlink components.
    fn contains_symlink(&self, base: &Path) -> Result<bool>;
}

impl PathExt for Path {
    /// Checks if the current path is hidden (dot file).
    fn is_hidden(&self) -> bool {
        self.components()
            .filter_map(|cmp| match cmp {
                Component::Normal(s) => s.to_str(),
                _ => None,
            })
            .any(|s| s.starts_with('.'))
    }

    /// Checks if the path is a symlink or contains intermediate symlink components.
    /// Second check could be an expensive operation as it requires filesystem access for each path component.
    fn contains_symlink(&self, base: &Path) -> Result<bool> {
        let mut current = base.to_path_buf();
        current.reserve(self.as_os_str().len());
        for component in self.components() {
            match component {
                Component::Normal(c) => {
                    current.push(c);
                    let meta = std::fs::symlink_metadata(&current).with_context(|| {
                        format!("unable to get metadata for path '{}'", current.display())
                    })?;
                    if meta.file_type().is_symlink() {
                        return Ok(true);
                    }
                }
                Component::CurDir => {}
                Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                    tracing::debug!(
                        "dir: skipping segment containing invalid prefix, dots or backslashes"
                    );
                }
            }
        }
        Ok(false)
    }
}

#[cfg(unix)]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    OsStr::from_bytes(bytes).into()
}

#[cfg(windows)]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    // This should really be OsStr::from_encoded_bytes_unchecked() but it’s
    // unsafe. With this fallback non-Unicode file names will result in 404.
    String::from_utf8_lossy(bytes).into_owned().into()
}

fn decode_tail_path(tail: &str) -> PathBuf {
    let bytes = percent_decode_str(tail.trim_start_matches('/')).collect::<Vec<_>>();
    path_from_bytes(&bytes)
}

/// Sanitizes a base/tail path and then it returns a unified one.
pub(crate) fn sanitize_path(base: &Path, tail: &str) -> Result<PathBuf, StatusCode> {
    let path_decoded = decode_tail_path(tail);
    let mut full_path = base.to_path_buf();
    tracing::trace!("dir: base={:?}, route={:?}", full_path, path_decoded);

    for component in path_decoded.components() {
        match component {
            Component::Normal(comp) => {
                // Protect against paths like `/foo/c:/bar/baz`
                // https://github.com/seanmonstar/warp/issues/937
                if Path::new(&comp)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
                {
                    full_path.push(comp)
                } else {
                    tracing::debug!("dir: skipping segment with invalid prefix");
                }
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                tracing::debug!(
                    "dir: skipping segment containing invalid prefix, dots or backslashes"
                );
            }
        }
    }
    Ok(full_path)
}

#[cfg(test)]
mod tests {
    use super::{PathExt, sanitize_path};
    use std::path::PathBuf;

    fn root_dir() -> PathBuf {
        PathBuf::from("docker/public/")
    }

    #[test]
    fn test_sanitize_path() {
        let base_dir = &PathBuf::from("docker/public");

        assert_eq!(
            sanitize_path(base_dir, "/index.html").unwrap(),
            root_dir().join("index.html")
        );

        // bad paths
        assert_eq!(
            sanitize_path(base_dir, "/../foo.html").unwrap(),
            root_dir().join("foo.html"),
        );
        assert_eq!(
            sanitize_path(base_dir, "/../W�foo.html").unwrap(),
            root_dir().join("W�foo.html"),
        );
        assert_eq!(
            sanitize_path(base_dir, "/%EF%BF%BD/../bar.html").unwrap(),
            root_dir().join("�/bar.html"),
        );
        assert_eq!(
            sanitize_path(base_dir, "àí/é%20/öüñ").unwrap(),
            root_dir().join("àí/é /öüñ"),
        );

        #[cfg(unix)]
        let expected_path = root_dir().join("C:\\/foo.html");
        #[cfg(windows)]
        let expected_path = PathBuf::from("docker/public/\\foo.html");
        assert_eq!(
            sanitize_path(base_dir, "/C:\\/foo.html").unwrap(),
            expected_path
        );
    }

    #[test]
    fn test_contains_symlink_returns_false() {
        let base = PathBuf::from("tests/fixtures/public");
        let user_path = PathBuf::from("./index.htm");

        match user_path.contains_symlink(&base) {
            Ok(contains) => assert!(!contains),
            Err(err) => panic!("unexpected error when checking for symlinks: {err}"),
        }
    }

    #[test]
    fn test_contains_symlink_returns_true() {
        let base = PathBuf::from("tests/fixtures/public");
        let user_path = PathBuf::from("./readme.md");

        match user_path.contains_symlink(&base) {
            Ok(contains) => assert!(contains),
            Err(err) => panic!("unexpected error when checking for symlinks: {err}"),
        }
    }

    #[test]
    fn test_contains_symlink_returns_error() {
        let base = PathBuf::from("tests/fixtures/public");
        let user_path = PathBuf::from("./unknown_file.txt");

        match user_path.contains_symlink(&base) {
            Ok(_) => panic!("expected error when checking for symlinks on non-existent path"),
            Err(err) => assert!(
                err.to_string().contains("unable to get metadata for path"),
                "unexpected error message: {err}"
            ),
        }
    }
}
