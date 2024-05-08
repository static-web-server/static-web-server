// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module that provides file path-related facilities.

use hyper::StatusCode;
use percent_encoding::percent_decode_str;
use std::path::{Component, Path, PathBuf};

/// `Path` extensions trait.
pub(crate) trait PathExt {
    /// If file path is hidden.
    fn is_hidden(&self) -> bool;
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
    use super::sanitize_path;
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
}
